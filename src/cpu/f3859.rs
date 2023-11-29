//! The 3859 was a combination of the 3850 CPU and a 3851 PSU (ROM).
//!
//! Each clock cycle was 0.5 microseconds (2 MHz).

use crate::{rom,cpu};

/// Fairchild 3859 CPU + PSU
pub struct F3859 {
  cpu: cpu::F3850,
  psu: rom::F3851,
}

impl F3859 {
  /// Creates a CPU and PSU
  pub fn new(data: [u8; 1024]) -> Self {
    Self {
      cpu: cpu::F3850::new(),
      psu: rom::F3851::new(data),
    }
  }

  /// Runs the CPU and has it interact with the PSU
  pub fn run_cycle(&mut self) {
    let mut io = F3850IO {
      rom: &mut self.psu,
    };
    self.cpu.run_cycle(&mut io);
  }
}

struct F3850IO<'a> {
  rom: &'a mut rom::F3851,
}

/// Outputs (upper, lower)
fn u16_to_u8(source: u16) -> (u8, u8) {
  let bytes = source.to_be_bytes();
  (bytes[0], bytes[1])
}

fn u8_to_u16(upper: u8, lower: u8) -> u16 {
  u16::from_be_bytes([upper, lower])
}


impl cpu::f3850::IO for F3850IO<'_> {
  fn output(&mut self, _port: u8, _value: u8) {
    todo!();
  }
  /// Read from IO port
  fn input(&mut self, _port: u8) -> u8 {
    todo!();
  }
  
  /// Read next code byte
  fn next_code(&mut self) -> u8 {
    if let Some(byte) = self.rom.next_code() {
      byte
    } else {
      0
    }
  }
  /// Read code byte without updating read pointer
  fn peak_code(&mut self) -> u8 {
    if let Some(byte) = self.rom.peak_code() {
      byte
    } else {
      0
    }
  }
  
  /// Read next data byte
  fn next_data(&mut self) -> u8 {
    if let Some(byte) = self.rom.next_data() {
      byte
    } else {
      0
    }
  }
  /// Write next data byte
  fn write_data(&mut self, _data: u8) {
    todo!();
  }

  /// Jump to direct address. push_pc will back up the current position, so you can return to it later. (Call vs Jump)
  fn jump(&mut self, upper: u8, lower: u8, push_pc: bool) {
    self.rom.jump(u8_to_u16(upper, lower), push_pc);
  }
  /// Jump to relative address.
  fn jump_relative(&mut self, relative_addr: i8) {
    self.rom.jump_relative(relative_addr);
  }
  /// Return from address.
  fn ret_pc(&mut self) {
    self.rom.ret_pc();
  }
  
  /// Used by ADC instruction
  fn add_dc0(&mut self, a: u8) {
    self.rom.add_dc0(a);
  }
  /// Get dc0 pointer, returns upper, lower
  fn get_dc0(&mut self) -> (u8, u8) {
    u16_to_u8(self.rom.dc0)
  }
  /// Set dc0 pointer
  fn set_dc0(&mut self, upper: u8, lower: u8) {
    self.rom.dc0 = u8_to_u16(upper, lower);
  }
  /// Swap DC pointers
  fn swap_dc(&mut self) {
    self.rom.swap_dc();
  }
  
  /// Get pc1 pointer, returns upper, lower
  fn get_pc1(&mut self) -> (u8, u8) {
    u16_to_u8(self.rom.pc1)
  }
  /// Set pc1 pointer
  fn set_pc1(&mut self, upper: u8, lower: u8) {
    self.rom.pc1 = u8_to_u16(upper, lower);
  }
  
}
