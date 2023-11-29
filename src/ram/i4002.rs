//! The 4002 DRAM chip was quite unusual. Data could be saved in 256 bits of "character data" and an additional 64 bits of "status data", totaling 320 bits of data. This was the equivalent of 80 nibbles, or 40 bytes or data. Extremely small by modern standards. It was also unusual for including 4 io ports for the CPU to write out to peripheral devices.
//! Up to 16 RAM chips could be connected together, allowing for a maximum of 1280 nibbles, or 640 bytes of data.

use log::warn;
use arbitrary_int::{u2, u4};

#[derive(Default)]
struct Register {
  characters: [u4; 16], //The default RAM implementation is limited to u8, and splitting it up into u4 is painful, so I will just keep this as an array here.
  status: [u4; 4], //4 nibbles = 2 bytes
}

/// Intel 4002 chip
#[derive(Default)]
pub struct I4002 {
  /// Each register's memory is divided into character and status
  registers: [Register; 4],
  /// 4 bits of io ports
  ports: u4,
}
impl I4002 {
  /// Create a new 4002 chip.
  pub fn new() -> Self {
    Default::default()
  }

  /// Read memory for a certain register and character address.
  #[inline]
  pub fn read_character(&self, reg_index: u2, character_index: u4) -> u4 {
    if let Some(register) = self.registers.get(reg_index.value() as usize) {
      register.characters[character_index.value() as usize]
    } else {
      warn!("Register index too big: {}", reg_index);
      Default::default()
    }
  }

  /// Write memory for a certain register and character address.
  #[inline]
  pub fn write_character(&mut self, reg_index: u2, character_index: u4, val: u4) {
    if let Some(register) = self.registers.get_mut(reg_index.value() as usize) {
      register.characters[character_index.value() as usize] = val;
    } else {
      warn!("Register index too big: {}", reg_index);
    }
  }

  /// Read memory for a certain register and status address.
  #[inline]
  pub fn read_status(&self, reg_index: u2, status_index: u2) -> u4 {
    if let Some(register) = self.registers.get(reg_index.value() as usize) {
      register.status[status_index.value() as usize]
    } else {
      warn!("Register index too big: {}", reg_index);
      Default::default()
    }
  }
  
  /// Write memory for a certain register and status address.
  #[inline]
  pub fn write_status(&mut self, reg_index: u2, status_index: u2, val: u4) {
    if let Some(register) = self.registers.get_mut(reg_index.value() as usize) {
      register.status[status_index.value() as usize] = val;
    } else {
      warn!("Register index too big: {}", reg_index);
    }
  }
  
  /// Read all 4 port bits
  #[inline]
  pub fn read_ports(&self) -> u4 {
    self.ports
  }
  
  /// Write all 4 port bits
  #[inline]
  pub fn write_ports(&mut self, val: u4) {
    self.ports = val;
  }
  
  /// Read all character memory in a register
  #[inline]
  pub fn read_character_array(&self, reg_index: u2) -> [u4; 16] {
    if let Some(register) = self.registers.get(reg_index.value() as usize) {
      register.characters
    } else {
      warn!("Register index too big: {}", reg_index);
      Default::default()
    }
  }
  
  /// Read all status memory in a register
  #[inline]
  pub fn read_status_array(&self, reg_index: u2) -> [u4; 4] {
    if let Some(register) = self.registers.get(reg_index.value() as usize) {
      register.status
    } else {
      warn!("Register index too big: {}", reg_index);
      Default::default()
    }
  }

}
