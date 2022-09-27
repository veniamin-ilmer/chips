//! Generic Programmable Read Only chip
//!
//! Initialize the ROM by writing data to it. Afterward, data could only be read from it.
//!
//! ### Example
//!
//! ```
//! use chips::ROM;
//!
//! let mut rom: ROM<3> = ROM::new([10,20,30]);
//! assert_eq!(rom.read_byte(1), 20);
//! ```

/// Generic Programmable Read Only chip
///
/// Initialize the ROM by writing data to it. Afterward, data could only be read from it.
///
/// ### Example
///
/// ```
/// use chips::ROM;
///
/// let mut rom: ROM<3> = ROM::new([10,20,30]);
/// assert_eq!(rom.read_byte(1), 20);
/// ```
pub struct ROM<const LENGTH: usize> {
  /// Data in the ROM chip
  data: [u8; LENGTH],
}

impl<const LENGTH: usize> ROM<LENGTH> {
  /// Create a new ROM chip
  /// 
  /// `let rom: ROM<3> = ROM::new([10,20,30]);`
  ///
  #[inline]
  pub fn new(data: [u8; LENGTH]) -> Self {
    Self {
      data,
    }
  }
  
  /// Read byte at address
  #[inline]
  pub fn read_byte(&self, addr: usize) -> u8 {
    self.data[addr]
  }
}