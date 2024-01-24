//! Small Indexable Register Chips

use arbitrary_int::u4;

/// ### Small Indexable Register Chip
///
/// Many chips contain small (128 bit or less) registers.
/// These are not shift registers, because you can access any part of it without shifting.
/// This is separate from large RAM chips, which need to be indexed as an array.
#[derive(Default, Clone, Copy, PartialEq)]
pub struct Indexer64 {
    /// The indexer's "memory"
    pub data: u64,
}

impl Indexer64 {
  /// Create a new Indexer chip
  #[inline]
  pub fn new() -> Self {
    Default::default()
  }

  /// Read bit
  #[inline]
  pub fn read_bit(&self, index: u8) -> bool {
    self.data >> index & 1 == 1
  }

  /// Write bit
  #[inline]
  pub fn write_bit(&mut self, index: u8, bit: bool) {
    // Clear the bit at the specified index
    self.data &= !(1 << index);
    // Set the new value at the specified index
    if bit {
      self.data |= 1 << index;
    }
  }

  /// Read nibble
  #[inline]
  pub fn read_nibble(&self, index: u8) -> u4 {
    u4::new(((self.data >> (index * 4)) & 0xF) as u8)
  }

  /// Write nibble
  #[inline]
  pub fn write_nibble(&mut self, index: u8, nibble: u4) {
    let index = index * 4;
    // Clear the nibble at the specified index
    self.data &= !(0xF << index);
    // Set the new value at the specified index
    self.data |= (nibble.value() as u64) << index;
  }
}

/// 16 bit indexer chip
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Indexer16 {
    /// The indexer's "memory"
    pub data: u16,
}

impl Indexer16 {
  /// Create a new Indexer chip
  #[inline]
  pub fn new() -> Self {
    Default::default()
  }

  /// Read bit
  #[inline]
  pub fn read_bit(&self, index: u8) -> bool {
    self.data >> index & 1 == 1
  }

  /// Write bit
  #[inline]
  pub fn write_bit(&mut self, index: u8, bit: bool) {
    // Clear the bit at the specified index
    self.data &= !(1 << index);
    // Set the new value at the specified index
    if bit {
      self.data |= 1 << index;
    }
  }

  /// Read nibble
  #[inline]
  pub fn read_nibble(&self, index: u8) -> u4 {
    u4::new(((self.data >> (index * 4)) & 0xF) as u8)
  }

  /// Write nibble
  #[inline]
  pub fn write_nibble(&mut self, index: u8, nibble: u4) {
    let index = index * 4;
    // Clear the nibble at the specified index
    self.data &= !(0xF << index);
    // Set the new value at the specified index
    self.data |= (nibble.value() as u16) << index;
  }
}
