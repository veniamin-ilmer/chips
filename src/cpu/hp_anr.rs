//! The HP 1820-0848, known as the Arithmetic and Register (A&R) chip was used in 1972 in the HP-35, the first handheld calculator.
//!
//! Each instruction was one 10 bits long. The task of decoding and executing the instruction was divided between this A&R chip and the C&T chip.
//! Each clock cycle ends up taking 280 microseconds. (3.671 kHz)
//! The A&R is the arithmetic unit and its main tasks are:
//!
//! * Decoding and executing instructions on registers
//! * Handling the 7 register stack
//! * Controlling the display
//!
//! Useful links
//! * <https://archived.hpcalc.org/laporte/HP%2035%20Saga.htm>
//! * <https://patentimages.storage.googleapis.com/44/5c/ab/197897f4ecaacb/US4001569.pdf>
use log::{info,trace};

use crate::shifter;
pub type Register = shifter::Shifter<u64, 56>;

use arbitrary_int::{
  u4,   //Register nibbles
  u5,   //Type 2 Operation code
  u6,   //Type 5 Instruction
  u10,  //Opcode
  u14,  //WordSelect
  u56,  //Register
};

const ZERO: u4 = u4::new(0);
const ONE: u4 = u4::new(1);

/// HP 1820-0848 Arithmetic and Register (A&R) chip
#[allow(non_camel_case_types)]
pub struct HP_AnR {
  /// General Register
  pub a: Register,
  /// General register, usually holding the display mask
  pub b: Register,
  /// Display register
  pub c: Register,
  /// Stack
  pub d: Register,
  /// Stack
  pub e: Register,
  /// Top of stack
  pub f: Register,
  /// Memory Register
  pub m: Register,
  /// Read by C&T
  pub next_carry: bool,
  /// Read by display
  pub display_on: bool,
}

struct WordSelect {
  data: u14,
}

impl WordSelect {
  fn shift(&mut self, direction: shifter::Direction) -> bool {
    match direction {
      shifter::Direction::Left => {
        let carry = self.data & u14::new(0b10000000000000) == u14::new(0b10000000000000); //Check 14th bit
        self.data <<= 1;
        carry
      },
      shifter::Direction::Right => {
        let carry = self.data & u14::new(1) == u14::new(1);
        self.data >>= 1;
        carry
      }
    }
  }
}


// BCD add
fn add(num1: u4, num2: u4, carry: bool) -> (u4, bool) {
  let mut result = num1.value() + num2.value();
  if carry {
    result += 1;
  }
  if result >= 10 {
    (u4::new(result - 10), true)
  } else {
    (u4::new(result), false)
  }
}

//BCD subtract
fn sub(num1: u4, num2: u4, borrow: bool) -> (u4, bool) {
  let mut result = num1.value() as isize - num2.value() as isize;
  if borrow {
    result -= 1;
  }
  if result < 0 {
    (u4::new((result + 10) as u8), true)
  } else {
    (u4::new(result as u8), false)
  }
}

impl HP_AnR {
  /// Initialize and reset registers
  pub fn new() -> Self {
    HP_AnR {
      a: Default::default(),
      b: Default::default(),
      c: Default::default(),
      d: Default::default(),
      e: Default::default(),
      f: Default::default(),
      m: Default::default(),
      next_carry: true,
      display_on: false,
    }
  }

  /// Print debug data of all registers
  pub fn print(&self) {
    trace!("A:{:014X} B:{:014X} C:{:014X} D:{:014X} E:{:014X} F:{:014X} M:{:014X} Carry: {}",
      self.a.read_parallel(), self.b.read_parallel(), self.c.read_parallel(), self.d.read_parallel(), self.e.read_parallel(), self.f.read_parallel(), self.m.read_parallel(), self.next_carry
    );
  }

  /// Handles Type 2 and Type 5 opcodes
  pub fn run_cycle(&mut self, opcode: u10, word_select_data: u14, ram_data: Register) {
    self.next_carry = true; //Default, until proven otherwise.
    match opcode.value() & 0b11 {
      0b10 => self.type_2(u5::new((opcode.value() >> 5) as u8), WordSelect { data: word_select_data }),
      0b00 => if opcode.value() & 0b1111 == 0b1000 {
        self.type_5(u6::new((opcode.value() >> 4) as u8), WordSelect { data: word_select_data }, ram_data);
      },
      _ => {},  //C&T stuff
    }
  }
  
  /// Type 2 - Arithmetic and Register. Returns carry
  /// See page 42, sections 19 and 20 of patent.
  fn type_2(&mut self, operation_code: u5, mut word_select: WordSelect) {
    trace!("Word Select: {:014b}", word_select.data.value());
    //First do the calculations
    let mut carry = false;
    let mut first_digit = true;
    let mut prev_nibble = u4::new(0);
    let direction = if matches!(operation_code.value(), 0b10010 | 0b10100 | 0b10110) {
      shifter::Direction::Left
    } else {
      shifter::Direction::Right
    };
    for i in 0..14 {
      let mut a = self.a.read_nibble(direction);
      let mut b = self.b.read_nibble(direction);
      let mut c = self.c.read_nibble(direction);
      if word_select.shift(direction) {
        //This is meant for math calculations, where we only need the first digit to be one.
        let one = if first_digit {
          first_digit = false;
          ONE
        } else {
          ZERO
        };
        match operation_code.value() {
          0b00000 => { trace!("? 0 - B[{i}]"); (_, carry) = sub(ZERO, b, carry) },
          0b00001 => { trace!("B[{i}] = 0"); b = ZERO; },
          0b00010 => { trace!("? A[{i}] - C[{i}]"); (_, carry) = sub(a, c, carry) },
          0b00011 => { trace!("? C[{i}] - 1"); (_, carry) = sub(c, one, carry); },
          
          0b00100 => { trace!("C[{i}] = B[{i}]"); c = b; },
          0b00101 => { trace!("C = 0 - C[{i}]"); (c, carry) = sub(ZERO, c, carry) },
          0b00110 => { trace!("C[{i}] = 0"); c = ZERO; },
          0b00111 => { trace!("C[{i}] = 0 - C[{i}] - 1");
            let (c1, carry1) = sub(ZERO, c, carry);
            let (c2, carry2) = sub(c1, one, false);
            c = c2;
            carry = carry1 | carry2;
          },
          
          0b01000 => { trace!("A[{i}] << 4"); (a, prev_nibble) = (prev_nibble, a); },
          0b01001 => { trace!("B[{i}] = A[{i}]"); b = a; },
          0b01010 => { trace!("C[{i}] = A[{i}] - C[{i}]"); (c, carry) = sub(a, c, carry) },
          0b01011 => { trace!("C[{i}]--"); (c, carry) = sub(c, one, carry) },
          
          0b01100 => { trace!("A[{i}] = C[{i}]"); a = c; },
          0b01101 => { trace!("? 0 - C[{i}]"); (_, carry) = sub(ZERO, c, carry) },
          0b01110 => { trace!("C[{i}] = A[{i}] + C[{i}]"); (c, carry) = add(a, c, carry) },
          0b01111 => { trace!("C[{i}]++"); (c, carry) = add(c, one, carry) },
          
          0b10000 => { trace!("_ = A[{i}] - B[{i}]"); (_, carry) = sub(a, b, carry) },
          0b10001 => { trace!("XHG B[{i}], C[{i}]"); (b, c) = (c, b); },
          0b10010 => { trace!("C[{i}] >> 4"); (c, prev_nibble) = (prev_nibble, c); },
          0b10011 => { trace!("? A[{i}] - 1"); (_, carry) = sub(a, one, carry) },
          
          0b10100 => { trace!("B[{i}] >> 4"); (b, prev_nibble) = (prev_nibble, b); },
          0b10101 => { trace!("C[{i}] = C[{i}] + C[{i}]"); (c, carry) = add(c, c, carry) },
          0b10110 => { trace!("A[{i}] >> 4"); (a, prev_nibble) = (prev_nibble, a); },
          0b10111 => { trace!("A[{i}] = 0"); a = ZERO; },
          
          0b11000 => { trace!("A[{i}] = A[{i}] - B[{i}]"); (a, carry) = sub(a, b, carry) },
          0b11001 => { trace!("XHG B[{i}], A[{i}]"); (b, a) = (a, b); },
          0b11010 => { trace!("A[{i}] = A[{i}] - C[{i}]"); (a, carry) = sub(a, c, carry) },
          0b11011 => { trace!("A[{i}]--"); (a, carry) = sub(a, one, carry) },

          0b11100 => { trace!("A[{i}] = A[{i}] + B[{i}]"); (a, carry) = add(a, b, carry) },
          0b11101 => { trace!("XHG A[{i}], C[{i}]"); (c, a) = (a, c); },
          0b11110 => { trace!("A[{i}] = A[{i}] + C[{i}]"); (a, carry) = add(a, c, carry) },
          _ => { trace!("A[{i}]++"); (a, carry) = add(a, one, carry) }, //0b11111
        }
      }
      self.a.shift_with_nibble(direction, a);
      self.b.shift_with_nibble(direction, b);
      self.c.shift_with_nibble(direction, c);
    }
    self.next_carry = !carry; //Note carries are always recorded as opposites.
  }
  
  // Type 5 - Data Entry and Display
  fn type_5(&mut self, instruction: u6, mut word_select: WordSelect, ram_data: Register) {
    match instruction.value() & 0b11 {
      0b00 => {}, //Reserved
      0b01 => { trace!("LOAD CONSTANT");
        for i in 0..14 {
          let mut c = self.c.read_nibble(shifter::Direction::Right);
          if word_select.shift(shifter::Direction::Right) {
            c = u4::new(instruction.value() >> 2);
            trace!("C[{}] = {}", i, c);
          }
          self.c.shift_with_nibble(shifter::Direction::Right, c);
        }
      },
      0b10 | 0b11 => {
        match instruction.value() >> 2 {
          0b0000 => { info!("Display Toggle"); self.display_on = !self.display_on; },
          0b0010 => { trace!("XHG C, M"); (self.c, self.m) = (self.m, self.c) },      //Exchange Memory
          0b0100 => { trace!("PUSH C"); (self.d, self.e, self.f) = (self.c, self.d, self.e); }  //Up Stack
          0b0110 => { trace!("POP A"); (self.e, self.d, self.a) = (self.f, self.e, self.d); }   //Down Stack
          0b1000 => { info!("Display off"); self.display_on = false; },
          0b1010 => { trace!("MOV C, M"); self.c = self.m; }  //Recall memory
          0b1011 => { info!("C = Data Storage ({:014X})", ram_data.read_parallel()); self.c = ram_data; }, //Send Data from Auxiliary Data Storage Circuit into C Register
          0b1100 => { trace!("Rotate C"); (self.f, self.e, self.d, self.c) = (self.c, self.f, self.e, self.d); }  //Rotate Down
          0b1110 => { trace!("CLEAR REGS"); *self = HP_AnR::new(); },
          _ => unimplemented!(),
        }
      },
      _ => todo!(),
    }
  }
}
