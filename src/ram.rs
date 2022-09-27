//! Generic RAM Chip
//! Example use:
//!
//! ```
//! use chips::RAM;
//!
//! let mut ram: RAM<u8, 200> = RAM::new();
//! ram.write(123, 8);
//! assert_eq!(ram.read(123), 8);
//! ```

/// Generic RAM Chip
/// Example use:
///
/// ```
/// use chips::RAM;
///
/// let mut ram: RAM<u8, 200> = RAM::new();
/// ram.write(123, 8);
/// assert_eq!(ram.read(123), 8);
/// ```
pub struct RAM<TYPE: Sized, const LENGTH: usize> {
  /// Data in the ROM chip
  pub data: [TYPE; LENGTH],
}

impl<TYPE: core::marker::Copy + core::default::Default, const LENGTH: usize> Default for RAM<TYPE, LENGTH> {
  #[inline]
  fn default() -> Self {
    Self {
      data: [Default::default(); LENGTH]
    }
  }
}

impl<TYPE: core::marker::Copy + core::default::Default, const LENGTH: usize> RAM<TYPE, LENGTH> {
  /// Create a new RAM chip
  #[inline]
  pub fn new() -> Self {
    Default::default()
  }

  /// Read at address
  #[inline]
  pub fn read(&self, addr: usize) -> TYPE {
    self.data[addr]
  }

  /// Write to address
  #[inline]
  pub fn write(&mut self, addr: usize, what: TYPE) {
    self.data[addr] = what;
  }
}