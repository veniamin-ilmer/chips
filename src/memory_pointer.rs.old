//! This is a general functionality used mostly by CPUs to read from memory and update pointers.

/// The data is being requested from RAM and ROM chips with methods in this trait
pub trait MemoryIO<ADDRESS> {
  /// Read from ROM chip
  fn read_mem<T: crate::ReadArr>(&self, address: ADDRESS) -> T;
  /// Write byte to RAM chip
  fn write_mem<T: crate::WriteArr>(&mut self, address: ADDRESS, value: T);
}

/// Memory register
///
/// Keeps track of current position, reading and incrementing
#[derive(Default)]
pub struct MemoryPointer<ADDRESS> {
  /// Pointer pointing to address
  pub pointer: ADDRESS,
}

impl<ADDRESS: core::default::Default + core::ops::AddAssign<ADDRESS> + core::ops::SubAssign<ADDRESS> + From<u8> + Copy> MemoryPointer<ADDRESS> {
  /// Create a new pointer
  #[inline]
  pub fn new() -> Self {
    Default::default()
  }
  
  /// Read data and move over pointer
  #[inline]
  pub fn pop_u8(&mut self, io: &impl MemoryIO<ADDRESS>) -> u8 {
    let ret = io.read_mem(self.pointer);
    self.pointer += ADDRESS::from(1);
    ret
  }
  
  /// Read data and move over program counter
  #[inline]
  pub fn pop_u16(&mut self, io: &impl MemoryIO<ADDRESS>) -> u16 {
    let ret = io.read_mem(self.pointer);
    self.pointer += ADDRESS::from(2);
    ret
  }
  
  /// Push data and decrement pointer
  #[inline]
  pub fn push_u8(&mut self, io: &mut impl MemoryIO<ADDRESS>, value: u8) {
    self.pointer -= ADDRESS::from(1);
    io.write_mem(self.pointer, value);
  }
  
  /// Push data and decrement pointer
  #[inline]
  pub fn push_u16(&mut self, io: &mut impl MemoryIO<ADDRESS>, value: u16) {
    self.pointer -= ADDRESS::from(2);
    io.write_mem(self.pointer, value);
  }
  
}