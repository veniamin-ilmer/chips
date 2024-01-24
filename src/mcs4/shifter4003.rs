//! Intel 4003 is a shift register with 10 bits, capable of both serial and parallel read/write

use crate::shifter;

/// 16 bit shifter chip which has a `read_write_serial` method.
#[derive(Default, Clone, Copy, PartialEq)]
pub struct Shifter {
  shifter: shifter::Shifter16<10>,
  /// We only shift data after pulse switches from high to low, so we must keep track of the pulse
  pulse: bool,
}

impl Shifter {

  /// Create a new shifter chip
  #[inline]
  pub fn new() -> Self {
    Default::default()
  }

  /// Write and Read to the shifter at the same time.
  ///
  /// Write will only happen if `pulse` switches from high to low. Otherwise `in_bit` will be ignored.
  #[inline]
  pub fn read_write_serial(&mut self, direction: shifter::Direction, in_bit: bool, pulse: bool) -> bool {
    let out_bit = self.shifter.read_bit(direction);
    if self.pulse && !pulse {  //Updates happen when switching from a high pulse to low pulse.
      self.shifter.shift_with_bit(direction, in_bit);
    }
    self.pulse = pulse;
    out_bit
  }

  pub fn read_parallel(&self) -> u16 {
    self.shifter.read_parallel()
  }

}