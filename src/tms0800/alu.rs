//! Handling all of the arthimetic of the TMS0800 series

use log::{debug, trace};
use arbitrary_int::{u4,u5};
use crate::shifter;

/// 11 nibbles of BCD digits
pub type Register = shifter::Shifter64<44>;

/// WordSelect, Mapped from the "mask".
type WordSelect = shifter::Shifter16<11>;

#[derive(Debug, Clone, Copy)]
enum Operation {
  Add,
  Shr,
  Shl,
  Subtract,
  ExchangeAB,
  WaitForKey,
}

#[derive(Debug, Clone, Copy)]
enum Arg {
  A,
  B,
  C,
  K,
  None
}

#[derive(Clone, Copy)]
struct Instruction {
  operation: Operation,
  arg1: Arg,
  arg2: Arg,
  dest: Arg,
  hex: bool,
}

use core::fmt;

impl fmt::Debug for Instruction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.hex {
      write!(f, "{:?} = {:?} {:?}, {:?} (hex)", self.dest, self.operation, self.arg1, self.arg2)
    } else {
      write!(f, "{:?} = {:?} {:?}, {:?}", self.dest, self.operation, self.arg1, self.arg2)
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
  instructions: [Instruction; 32],
  constants: [u4; 16],
}

impl ALU {
  pub fn new(alu_map: [u32; 13], constants: [u4; 16]) -> Self {

    Self {
      a: Register::new(0),
      b: Register::new(0),
      c: Register::new(0),
      instructions: decode_instructions(alu_map),
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
        let instruction = self.instructions[instruction.value() as usize];
        trace!("{:?}", instruction);
        match instruction.operation {
          Operation::Add | Operation::WaitForKey | Operation::Subtract => {
            let arg1 = match instruction.arg1 {
              Arg::A => a,
              Arg::B => b,
              Arg::C => c,
              Arg::K => k,
              Arg::None => u4::new(0),
            };
            let arg2 = match instruction.arg2 {
              Arg::A => a,
              Arg::B => b,
              Arg::C => c,
              Arg::K => k,
              Arg::None => u4::new(0),
            };
            let result = if matches!(instruction.operation, Operation::Subtract) {
              sub(arg1, arg2, carry, instruction.hex)
            } else {
              add(arg1, arg2, carry, instruction.hex)
            };
            carry = result.1;
            match instruction.dest {
              Arg::A => a = result.0,
              Arg::B => b = result.0,
              Arg::C => c = result.0,
              Arg::K | Arg::None => {},
            }
          },
          Operation::ExchangeAB => (a, b) = (b, a),
          Operation::Shl | Operation::Shr => {
            match instruction.dest {
              Arg::A => (a, prev_nibble) = (prev_nibble, a),
              Arg::B => (b, prev_nibble) = (prev_nibble, b),
              Arg::C => (c, prev_nibble) = (prev_nibble, c),
              Arg::K | Arg::None => {},
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

/// Decodes the programmable arithmetic control
fn decode_instructions(alu_map: [u32; 13]) -> [Instruction; 32] {
  let mut instructions = [Instruction {
    operation: Operation::Add,
    arg1: Arg::None,
    arg2: Arg::None,
    dest: Arg::None,
    hex: false,
  }; 32];

  for i in 0..32 {
    if (alu_map[0x0] >> i) & 1 == 1 { instructions[i].operation = Operation::Shr; }
    if (alu_map[0x1] >> i) & 1 == 1 { instructions[i].operation = Operation::Shl; }
    if (alu_map[0x2] >> i) & 1 == 1 { instructions[i].operation = Operation::Subtract; }
    if (alu_map[0x3] >> i) & 1 == 1 { instructions[i].hex = true; }
    if (alu_map[0x4] >> i) & 1 == 1 { instructions[i].arg1 = Arg::C; }
    if (alu_map[0x5] >> i) & 1 == 1 { instructions[i].arg1 = Arg::A; }
    if (alu_map[0x6] >> i) & 1 == 1 { instructions[i].arg2 = Arg::B; }
    if (alu_map[0x7] >> i) & 1 == 1 { instructions[i].arg2 = Arg::K; }
    if (alu_map[0x8] >> i) & 1 == 1 { instructions[i].dest = Arg::A; }
    if (alu_map[0x9] >> i) & 1 == 1 { instructions[i].dest = Arg::C; }
    if (alu_map[0xA] >> i) & 1 == 1 { instructions[i].dest = Arg::B; }
    if (alu_map[0xB] >> i) & 1 == 1 { instructions[i].operation = Operation::ExchangeAB; }
    if (alu_map[0xC] >> i) & 1 == 1 { instructions[i].operation = Operation::WaitForKey; }
  }
  instructions
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
