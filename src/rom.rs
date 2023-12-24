//! All ROM chips

mod i4001; pub use i4001::I4001;
mod f3851; pub use f3851::F3851;
mod hp_rom; pub use hp_rom::HP_ROM;

/// Intel 8702 is a 256 byte ROM.
pub type I8702 = ROM<0x100>;
/// Intel 3604 is a 512 byte ROM.
pub type I3604 = ROM<0x200>;
/// Intel 8604 is a 512 byte ROM.
pub type I8604 = ROM<0x200>;
/// Intel 8308 is a 1 KB ROM.
pub type I8308 = ROM<0x400>;
/// Intel 8316 is a 2 KB ROM.
pub type I8316 = ROM<0x800>;
/// TMS2716 is a 2 KB ROM.
pub type TMS2716 = ROM<0x800>;

/// ### Generic Programmable Read Only chip
///
/// Initialize the ROM by writing data to it. Afterward, data could only be read from it.
///
/// ### Example
/// ```
/// use chips::ROM;
///
/// let mut rom: ROM<3> = ROM::new([10,20,30]);
/// assert_eq!(rom.read::<u8>(1), 20);
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
  pub fn read<T: crate::ReadArr>(&self, addr: usize) -> T {
    T::read(&self.data[addr..])
  }
  
  /// Look up how many bytes is the generic const LENGTH set to.
  pub const LENGTH: usize = LENGTH;
}
