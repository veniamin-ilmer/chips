//! The 4002 DRAM chip was quite unusual. Data could be saved in 256 bits of "character data" and an additional 64 bits of "status data", totaling 320 bits of data. This was the equivalent of 80 nibbles, or 40 bytes or data. Extremely small by modern standards. It was also unusual for including 4 io ports for the CPU to write out to peripheral devices.
//! Up to 16 RAM chips could be connected together, allowing for a maximum of 1280 nibbles, or 640 bytes of data.

use arbitrary_int::{u2, u4};
use log::{trace,info};
use crate::{Indexer16, Indexer64};

#[derive(Default)]
pub struct Register {
  pub characters: Indexer64,  //16 nibbles = 8 bytes
  pub status: Indexer16, //4 nibbles = 2 bytes
}

/// Intel 4002 chip
pub struct RAM {
  /// This page mask is added to all memory pointers to know which memory chip is being referenced
  page_mask: u2,
  active: bool,
  register_index: u2,
  character_index: u4,
  /// Each register's memory is divided into character and status
  pub registers: [Register; 4],
  /// 4 bits of io ports
  pub ports: u4,
}

impl RAM {
  /// Create a new 4002 chip. Pass in the rom data.
  #[inline]
  pub fn new(page_mask: u2) -> Self {
    Self {
      page_mask,
      active: page_mask == u2::new(0),
      register_index: Default::default(),
      character_index: Default::default(),
      registers: Default::default(),
      ports: Default::default(),
    }
  }

  /// SRC. Contains the chip select, register index, and character index
  pub fn set_register_control(&mut self, data_in: super::Byte) {
    self.active = (data_in.high().value() >> 2) == self.page_mask.value();
    self.register_index = u2::new(data_in.high().value() & 0b11);
    self.character_index = data_in.low();
    trace!("Active: {}, Reg Index: {} Character Index: {}", self.active, self.register_index, self.character_index);
  }

  /// Clock X2 - I/O instruction. It can send data back to the CPU
  pub fn io_read(&self, modifier: u4) -> u4 {
    let mut data_out = u4::new(0);
    if self.active {
      match modifier.value() {
        0x8 | 0x9 | 0xB => {
          trace!("Read Ram Character"); //Read RAM character
          if let Some(register) = self.registers.get(self.register_index.value() as usize) {
            data_out = register.characters.read_nibble(self.character_index.value())
          }
        },
        0xc..=0xf => {
          let status_index = modifier.value() & 0b11;
          trace!("RD{}", status_index); //Read Status
          if let Some(register) = self.registers.get(self.register_index.value() as usize) {
            data_out = register.status.read_nibble(status_index);
          }
        },
        _ => {},
      }
    }
    data_out
  }

  /// Clock X2 - SRC or I/O instruction. Input: High: modifier. Low: Value
  pub fn io_write(&mut self, data_in: super::Byte) {
    if self.active {
      match data_in.high().value() {  //modifier
        0x0 => { info!("Reg[{}]Char[{}] = {}", self.register_index.value(), self.character_index.value(), data_in.low()); //WRM - Write to RAM character
          if let Some(register) = self.registers.get_mut(self.register_index.value() as usize) {
            register.characters.write_nibble(self.character_index.value(), data_in.low());
          }
        },
        0x1 => { trace!("WMP {}", data_in.low()); //Write to RAM Port
          self.ports = data_in.low();
        },
        0x4..=0x7 => {
          let status_index = data_in.high().value() & 0b11;
          trace!("WR{} {}", status_index, data_in.low()); //Write Acc to RAM status
          if let Some(register) = self.registers.get_mut(self.register_index.value() as usize) {
            register.status.write_nibble(status_index, data_in.low());
          }
        },
        _ => {},
      }
    }
  }

  pub fn read_full_character(&self, reg_index: u2) -> Indexer64 {
    if let Some(register) = self.registers.get(reg_index.value() as usize) {
      register.characters
    } else {
      Default::default()
    }
  }

  pub fn read_full_status(&self, reg_index: u2) -> Indexer16 {
    if let Some(register) = self.registers.get(reg_index.value() as usize) {
      register.status
    } else {
      Default::default()
    }
  }
}
