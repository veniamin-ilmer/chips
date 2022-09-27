use crate::ram::RAM;

use log::warn;
use arbitrary_int::{u2, u4};

pub trait IO {
  fn port0(&mut self, value: bool);
  fn port1(&mut self, value: bool);
  fn port2(&mut self, value: bool);
  fn port3(&mut self, value: bool);
}

#[derive(Default)]
struct Register {
  characters: RAM<u4, 16>,
  status: RAM<u4, 4>,
}

#[derive(Default)]
pub struct I4002 {
  registers: [Register; 4],
  ports: u4,
}
impl I4002 {
  pub fn new() -> Self {
    Default::default()
  }

  #[inline]
  pub fn read_character(&self, reg_index: u2, character_index: u4) -> u4 {
    if let Some(register) = self.registers.get(reg_index.value() as usize) {
      register.characters.read(character_index.value() as usize)
    } else {
      warn!("Register index too big: {}", reg_index);
      Default::default()
    }
  }

  #[inline]
  pub fn write_character(&mut self, reg_index: u2, character_index: u4, val: u4) {
    if let Some(register) = self.registers.get_mut(reg_index.value() as usize) {
      register.characters.write(character_index.value() as usize, val);
    } else {
      warn!("Register index too big: {}", reg_index);
    }
  }

  #[inline]
  pub fn read_status(&self, reg_index: u2, status_index: u2) -> u4 {
    if let Some(register) = self.registers.get(reg_index.value() as usize) {
      register.status.read(status_index.value() as usize)
    } else {
      warn!("Register index too big: {}", reg_index);
      Default::default()
    }
  }
  
  #[inline]
  pub fn write_status(&mut self, reg_index: u2, status_index: u2, val: u4) {
    if let Some(register) = self.registers.get_mut(reg_index.value() as usize) {
      register.status.write(status_index.value() as usize, val);
    } else {
      warn!("Register index too big: {}", reg_index);
    }
  }
  
  #[inline]
  pub fn read_ports(&self) -> u4 {
    self.ports
  }
  #[inline]
  pub fn write_ports(&mut self, val: u4) {
    self.ports = val;
  }
  
  #[inline]
  pub fn read_character_array(&self, reg_index: u2) -> [u4; 16] {
    if let Some(register) = self.registers.get(reg_index.value() as usize) {
      register.characters.data.clone()
    } else {
      warn!("Register index too big: {}", reg_index);
      Default::default()
    }
  }
  
  #[inline]
  pub fn read_status_array(&self, reg_index: u2) -> [u4; 4] {
    if let Some(register) = self.registers.get(reg_index.value() as usize) {
      register.status.data.clone()
    } else {
      warn!("Register index too big: {}", reg_index);
      Default::default()
    }
  }

}