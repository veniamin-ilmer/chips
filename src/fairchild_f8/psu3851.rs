//! The 3851 PSU (Program Storage Unit) had 1024 bytes of ROM. was an unusual ROM as it handled the code and data count registers.
//! * <https://wiki.console5.com/tw/images/5/50/Fairchild_F3851.pdf>

use crate::ROM;
use log::debug;
use arbitrary_int::u6;

const ROM_SIZE: usize = 1024;

/// Fairchild 3851 chip
pub struct F3851 {
  /// ROM data
  rom: ROM<ROM_SIZE>,
  /// This page mask is added to all memory pointers to know which memory chip is being referenced
  page_mask: usize,
  /// Process Counter 0
  pub pc0: u16,
  /// Process Counter 1 (This is just a backup, and so it doesn't need to be a MemoryPointer.)
  pub pc1: u16,
  /// Data Counter 0. Note that the original F3851 PSU does NOT have a DC1 register.
  pub dc0: u16,
  /// DC1 is undocumented, but it is required for some roms to work. It seems it's part of the F3856 (A later version of the F3851)
  dc1: u16,
  /// IO Ports
  pub ports: [u8; 4],
  /// port mask is added to all port addresses to know which memory chips is being referenced
  port_mask: usize,
}

impl F3851 {
  /// Create a new 3851 chip. Pass in the rom data.
  #[inline]
  pub fn new(data: [u8; ROM_SIZE], page: u6, port_select: u6) -> Self {
    Self {
      rom: ROM::new(data),
      page_mask: (page.value() as usize) << 10,
      pc0: 0,
      pc1: 0,
      dc0: 0,
      dc1: 0,
      ports: [0;4],
      port_mask: (port_select.value() as usize) << 2,
    }
  }
  
  /// Print debug data of all registers
  pub fn print(&self) {
    debug!("PC0: 0x{:04X} PC1: 0x{:04X} DC0: 0x{:04X}", self.pc0, self.pc1, self.dc0);
  }
  /// Used by the JMP, PI, and PK instructions.
  pub fn jump(&mut self, address: u16, push_pc: bool) {
    if push_pc {
      self.pc1 = self.pc0;
    }
    self.pc0 = address;
  }

  /// Used by the BT, BP, BC, BZ instructions.
  pub fn jump_relative(&mut self, relative_addr: i8) {
    self.pc0 = self.pc0.wrapping_add(relative_addr as u16);
  }

  /// Used by the POP instruction. I think it is meant as a way to return from an PK / PI / IRQ.
  pub fn ret_pc(&mut self) {
    self.pc0 = self.pc1;
  }

  /// Used by ADC instruction
  pub fn add_dc0(&mut self, a: i8) {
    self.dc0 = self.dc0.wrapping_add(a as u16);
  }


  /// ROMC00 or ROMC03
  /// For reading the next instruction
  pub fn next_code(&mut self) -> u8 {
    let pointer = self.pc0 as usize;
    self.pc0 += 1;  //Incremented even if it doesn't have it.
    if pointer >= self.page_mask && pointer < ROM_SIZE + self.page_mask {
      return self.rom.read(pointer & 0b1111111111) //Take off the high page mask
    }
    0
  }

  /// ROMC01
  /// Read, without updated pc0. Used by relative jump
  pub fn peak_code(&self) -> i8 {
    let pointer = self.pc0 as usize;
    if pointer >= self.page_mask && pointer < ROM_SIZE + self.page_mask {
      return self.rom.read::<u8>(pointer & 0b1111111111) as i8 //Take off the high page mask
    }
    0
  }

  /// ROMC02
  /// Used by commands LM, AM, CM, etc reading data from memory
  pub fn next_data(&mut self) -> u8 {
    let pointer = self.dc0 as usize;
    self.dc0 += 1;  //Incremented even if it doesn't have it.
    if pointer >= self.page_mask && pointer < ROM_SIZE + self.page_mask {
      return self.rom.read(pointer & 0b1111111111) //Take off the high page mask
    }
    0
  }
  
  /// Used by the XDC (Exchange DC) instruction. DC1 is undocumented, but it is required for some roms to work. It seems it's part of the F3856 (A later version of the F3851)
  pub fn swap_dc(&mut self) {
    (self.dc0,self.dc1) = (self.dc1,self.dc0);
  }

  /// ROMC1A
  pub fn write_port(&mut self, port: u8, value: u8) {
    if (port as usize) >= self.port_mask && (port as usize) < (0b11 | self.port_mask) {
      self.ports[port as usize & 0b11] = value;
    }
  }
  
  /// ROMC1B
  pub fn read_port(&self, port: u8) -> u8 {
    if (port as usize) >= self.port_mask && (port as usize) < (0b11 | self.port_mask) {
      return self.ports[port as usize & 0b11]
    }
    0
  }
}
