//! Handling all of the arthimetic of the TMS0800 series

use log::{debug, trace};
use arbitrary_int::{u4,u5};
use crate::shifter;

/// 11 nibbles of BCD digits
pub type Register = shifter::Shifter64<44>;

/// WordSelect, Mapped from the "mask".
type WordSelect = shifter::Shifter16<11>;

#[derive(Debug, Clone, Copy)]
pub enum Oper {
  Plus,
  Shr,
  Shl,
  Minus,
  ExchangeAB,
  Wait,
}

#[derive(Debug, Clone, Copy)]
pub enum Arg1 {
  A,
  C,
  None
}

#[derive(Debug, Clone, Copy)]
pub enum Arg2 {
  B,
  K,
  None
}

#[derive(Debug, Clone, Copy)]
pub enum Dest {
  A,
  B,
  C,
  None
}

#[derive(Clone, Copy)]
pub struct Opcode{ operation: Oper, dest: Dest, arg1: Arg1, arg2: Arg2, hex: bool }

impl Opcode {
  pub const fn new(dest: Dest, arg1: Arg1, operation: Oper, arg2: Arg2, hex: bool) -> Self {
    Opcode {
      operation,
      arg1,
      arg2,
      dest,
      hex
    }
  }
}

use core::fmt;

impl fmt::Debug for Opcode {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.hex {
      write!(f, "{:?} = {:?} {:?} {:?} (hex)", self.dest, self.arg1, self.operation, self.arg2)
    } else {
      write!(f, "{:?} = {:?} {:?} {:?}", self.dest, self.arg1, self.operation, self.arg2)
    }
  }
}

/// TMS0800 ALU chip
pub struct ALU {
  /// Register read by LED display
  pub a: Register,
  /// Register contains decimal position
  pub b: Register,
  /// General purpose register
  pub c: Register,
  opcodes: [Opcode; 32],
  constants: [u4; 16],
}

impl ALU {
  pub fn new(opcodes: [Opcode; 32], constants: [u4; 16]) -> Self {

    Self {
      a: Register::new(0),
      b: Register::new(0),
      c: Register::new(0),
      opcodes,
      constants,
    }
  }
  
  /// Returns carry
  pub fn run_cycle(&mut self, mut word_select: WordSelect, instruction: u5, mask: u4) -> bool {
    let mut carry = false;
    let direction = if matches!(instruction.value(), 0x17..=0x19) {
      shifter::Direction::Left
    } else {
      shifter::Direction::Right
    };
    let mut prev_nibble = u4::new(0);
    let mut first_time = true;
    for _ in 0..11 {
      let mut a = self.a.read_nibble(direction);
      let mut b = self.b.read_nibble(direction);
      let mut c = self.c.read_nibble(direction);
      if word_select.read_and_shift_bit(direction, false) {
        let k = if first_time {
          self.constants[mask.value() as usize]
        } else {
          u4::new(0)
        };
        let opcode = self.opcodes[instruction.value() as usize];
        trace!("{:?}", opcode);
        match opcode.operation {
          Oper::Plus | Oper::Wait | Oper::Minus => {
            let arg1 = match opcode.arg1 {
              Arg1::A => a,
              Arg1::C => c,
              Arg1::None => u4::new(0),
            };
            let arg2 = match opcode.arg2 {
              Arg2::B => b,
              Arg2::K => k,
              Arg2::None => u4::new(0),
            };
            let (result, new_carry) = if matches!(opcode.operation, Oper::Minus) {
              sub(arg1, arg2, carry, opcode.hex)
            } else {
              add(arg1, arg2, carry, opcode.hex)
            };
            carry = new_carry;
            match opcode.dest {
              Dest::A => a = result,
              Dest::B => b = result,
              Dest::C => c = result,
              Dest::None => {},
            }
          },
          Oper::ExchangeAB => (a, b) = (b, a),
          Oper::Shl | Oper::Shr => {
            match opcode.dest {
              Dest::A => (a, prev_nibble) = (prev_nibble, a),
              Dest::B => (b, prev_nibble) = (prev_nibble, b),
              Dest::C => (c, prev_nibble) = (prev_nibble, c),
              Dest::None => {},
            }
          },
        };
        first_time = false;
      }
      self.a.shift_with_nibble(direction, a);
      self.b.shift_with_nibble(direction, b);
      self.c.shift_with_nibble(direction, c);
    }
    debug!("A: {:011X} B: {:011X} C: {:011X}", self.a.read_parallel(), self.b.read_parallel(), self.c.read_parallel());
    carry
  }

}

/// BCD/hex add
fn add(num1: u4, num2: u4, carry: bool, hex: bool) -> (u4, bool) {
  let mut result = num1.value() + num2.value();
  if carry {
    result += 1;
  }
  let ten = if hex { 16 } else { 10 };
  if result >= ten {
    (u4::new(result - ten), true)
  } else {
    (u4::new(result), false)
  }
}

/// BCD/hex subtract
fn sub(num1: u4, num2: u4, borrow: bool, hex: bool) -> (u4, bool) {
  let mut result = num1.value() as isize - num2.value() as isize;
  if borrow {
    result -= 1;
  }
  let ten = if hex { 16 } else { 10 };
  if result < 0 {
    (u4::new((result + ten) as u8), true)
  } else {
    (u4::new(result as u8), false)
  }
}
