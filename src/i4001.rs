use arbitrary_int::u4;
use crate::rom::ROM;

pub trait IO {
  fn out_port(&mut self, value: u4);
}

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