//! The Auxilary Data Storage, is the RAM, first used by the HP-45.

use crate::shifter;
pub type Register = shifter::Shifter<u64, 56>;

use arbitrary_int::{
  u4,   //Address
  u10,  //ROM opcode
};
use log::{info,trace};

/// Auxilary Data Storage
#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct HP_RAM {
  addr: u4,
  pub r0: Register,
  pub r1: Register,
  pub r2: Register,
  pub r3: Register,
  pub r4: Register,
  pub r5: Register,
  pub r6: Register,
  pub r7: Register,
  pub r8: Register,
  pub r9: Register,
}

impl HP_RAM {
  /// Initialize and reset registers
  pub fn new() -> Self {
    Default::default()
  }

  pub fn run_cycle(&mut self, opcode: u10, mut c: Register) -> Register {
    //Send Address from C Register to Data Storage Circuit
    if matches!(opcode.value(), 0b1001110000 | 0b1101110000) {
      c.shift_with_nibble(shifter::Direction::Left, u4::new(0));  //Skip mantissa sign
      self.addr = c.read_nibble(shifter::Direction::Left);
      info!("New RAM address: {:X}", self.addr);
    }
    //Send Data from C register to Auxiliary Data Storage Circuit
    else if opcode.value() == 0b1011110000 {
      info!("Save {:014X} in RAM", c.read_parallel());
      match self.addr.value() {
        0 => self.r0 = c,
        1 => self.r1 = c,
        2 => self.r2 = c,
        3 => self.r3 = c,
        4 => self.r4 = c,
        5 => self.r5 = c,
        6 => self.r6 = c,
        7 => self.r7 = c,
        8 => self.r8 = c,
        _ => self.r9 = c,
      }
    }
    //Send Data from Auxiliary Data Storage Circuit into C Register
    else if opcode.value() == 0b1011111000 {
      //handled by A&R
    }
    match self.addr.value() {
      0 => self.r0,
      1 => self.r1,
      2 => self.r2,
      3 => self.r3,
      4 => self.r4,
      5 => self.r5,
      6 => self.r6,
      7 => self.r7,
      8 => self.r8,
      _ => self.r9,
    }
  }
}
