//! All RAM Chips

mod i4002; pub use i4002::I4002;
mod f3852; pub use f3852::F3852;

/// Intel 2107B is a 512 byte RAM.
pub type I2107B = RAM<0x200>;

/// Mostek 4015 is a 512 byte (4096 bit) DRAM with 250ns access time, 380ns cycle
pub type MK4015 = RAM<0x200>;

/// Mostek 4027 is a 512 byte (4096 bit) DRAM with 120-200ns access time, 320-375ns cycle
pub type MK4027 = RAM<0x200>;

/// ### Generic RAM Chip
/// Internally stores all data as bytes, but can read/write in u4,u8,u16,etc.
///
/// ### Example
/// ```
/// use chips::RAM;
///
/// let mut ram: RAM<200> = RAM::new();
/// ram.write(123, 8_u8);
/// assert_eq!(ram.read::<u8>(123), 8);
/// ```
pub struct RAM<const LENGTH: usize> {
  /// Data in the RAM chip
  pub data: [u8; LENGTH],
}

impl<const LENGTH: usize> Default for RAM<LENGTH> {
  #[inline]
  fn default() -> Self {
    Self {
      data: [Default::default(); LENGTH]
    }
  }
}

impl<const LENGTH: usize> RAM<LENGTH> {
  /// Create a new RAM chip
  #[inline]
  pub fn new() -> Self {
    Default::default()
  }

  /// Read at address
  #[inline]
  pub fn read<T: crate::ReadArr>(&self, addr: usize) -> T {
    T::read(&self.data[addr..])
  }

  /// Write to address
  #[inline]
  pub fn write<T: crate::WriteArr>(&mut self, addr: usize, value: T) {
    T::write(&mut self.data[addr..], value);
  }

  /// Read bits
  #[inline]
  pub fn read_bit(&self, bit_addr: usize) -> bool {
    let byte_addr = bit_addr / 8;
    let which_bit = bit_addr % 8;
    let mask = 1 << which_bit;
    (self.data[byte_addr] & mask) != 0
  }

  /// Write bit to bit address
  #[inline]
  pub fn write_bit(&mut self, bit_addr: usize, bit_value: bool) {
    let byte_addr = bit_addr / 8;
    let which_bit = bit_addr % 8;
    let mask = 1 << which_bit;
    if bit_value {
      self.data[byte_addr] |= mask;
    } else {
      self.data[byte_addr] &= !mask;
    }
  }

  /// For initializing the RAM the same way that you would initialize a ROM with all data at once
  #[inline]
  pub fn set_total(&mut self, data: [u8; LENGTH]) {
    self.data = data;
  }

  /// Look up how many bytes is the generic const LENGTH set to.
  pub const LENGTH: usize = LENGTH;
}