//! Generic Shift Register Chip
//! Example use:
//!
//! ```
//! use chips::shifter::Shifter;
//!
//! let shifter = Shifter<10>::new();
//! shifter.read_write_serial(false, true);
//! shifter.read_write_serial(true, true);

pub struct Shifter<const NUM_BITS: usize> {
  bits: usize,
  clock: bool,
  out_bit: bool,
}

impl<const NUM_BITS: usize> Default for Shifter<NUM_BITS> {
  #[inline]
  fn default() -> Self {
    Self {
      bits: 0,
      clock: false,
      out_bit: false,
    }
  }
}

impl<const NUM_BITS: usize> Shifter<NUM_BITS> {
  /// Create a new Shifter chip
  #[inline]
  pub fn new() -> Self {
    Default::default()
  }
  
  #[inline]
  fn mask(&self) -> usize {
    (1 << NUM_BITS) - 1
  }
  
  /// Overwrite all of the bits at once.
  #[inline]
  pub fn write_parallel(&mut self, bits: usize) {
    self.bits = bits & self.mask();
  }
  
  /// Write and Read to the shifter at the same time.
  /// Write will only happen if `clock` switches from high to low. Otherwise `in_bit` will be ignored.
  #[inline]
  pub fn read_write_serial(&mut self, clock: bool, in_bit: bool) -> bool {
    if self.clock && !clock {  //Updates happen when switching from a high clock to low clock.
      self.out_bit = (self.bits >> (NUM_BITS - 1)) & 1 == 1;
      self.bits <<= 1;
      self.bits &= self.mask();
      if in_bit {
        self.bits |= 1;
      }
    }
    self.clock = clock;
    self.out_bit
  }
  
  /// Read all bits at once
  #[inline]
  pub fn read_parallel(&self) -> usize {
    self.bits
  }
  #[inline]
  pub fn read_serial_without_shift(&self) -> bool {
    self.out_bit
  }
}