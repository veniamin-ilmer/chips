//! The 4001 ROM was limited to only 255 bytes of data. It was also unusual for having 4 io ports for the CPU to read/write with peripheral devices. Up to 16 ROM could be connected together, allowing for a maximum of 4 KB of binary code to be stored.

use arbitrary_int::u4;
use crate::rom::ROM;

pub struct I4001 {
  rom: ROM<0x100>,
  ports: u4,
}

impl I4001 {
  #[inline]
  pub fn new(data: [u8; 0x100]) -> Self {
    Self {
      rom: ROM::new(data),
      ports: Default::default(),
    }
  }
  #[inline]
  pub fn read_byte(&self, addr: u8) -> u8 {
    self.rom.read_byte(addr as usize)
  }
  #[inline]
  pub fn read_ports(&self) -> u4 {
    self.ports
  }
  #[inline]
  pub fn write_ports(&mut self, val: u4) {
    self.ports = val;
  }
}
