//! The Auxilary Data Storage, is the RAM, first used by the HP-45.

use crate::shifter;
/// Same Register as on the A&R
pub type Register = shifter::Shifter<u64, 56>;

use arbitrary_int::{
  u4,   //Address
  u10,  //ROM opcode
};
use log::trace;

/// Auxilary Data Storage
#[allow(non_camel_case_types)]
pub struct HP_RAM<const REG_COUNT: usize> {
  addr: u8,
  /// All of the Registers
  pub regs: [Register; REG_COUNT],
}

impl<const REG_COUNT: usize> Default for HP_RAM<REG_COUNT> {
  fn default() -> Self {
    HP_RAM {
      addr: Default::default(),
      regs: [Default::default(); REG_COUNT],
    }
  }
}

impl<const REG_COUNT: usize> HP_RAM<REG_COUNT> {
  /// Initialize and reset registers
  pub fn new() -> Self {
    Default::default()
  }

  /// With the right opcode, c could be copied into this memory.
  /// Returns the current register's value.
  pub fn run_cycle(&mut self, opcode: u10, mut c: Register) -> Register {
    if REG_COUNT == 0 {
      return Default::default();
    }
    //Send Address from C Register to Data Storage Circuit
    if matches!(opcode.value(), 0b1001110000 | 0b1101110000) {
      c.shift_with_nibble(shifter::Direction::Left, u4::new(0));  //Skip mantissa sign
      self.addr = c.read_nibble(shifter::Direction::Left).value();
      trace!("New RAM address: {:X}", self.addr);
    }
    //Send Data from C register to Auxiliary Data Storage Circuit
    else if opcode.value() == 0b1011110000 {
      trace!("Save {:014X} in RAM", c.read_parallel());
      self.regs[self.addr as usize] = c;
    }
    //Send Data from Auxiliary Data Storage Circuit into C Register
    else if opcode.value() == 0b1011111000 {
      //handled by A&R
    }
    self.regs[self.addr as usize]
  }
}
