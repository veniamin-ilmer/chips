//! All Counter Chips

use arbitrary_int::u4;
use core::{default};
use num_traits::ops::wrapping;

/// 7400 Series 4 bit binary counter
pub type S74X393 = Counter<u4>;

/// ### Generic Counter Chip
///
/// ```
/// use chips::Counter;
///
/// let mut counter: Counter<u8> = Counter::new();
/// counter.increment();
/// let val = counter.increment();
/// assert_eq!(val, 2);
///
/// counter.clear();
/// //loop around
/// for _ in 0..0x100 {
///    counter.increment();
/// }
/// assert_eq!(counter.read(), 0);
/// ```
#[derive(default::Default)]
pub struct Counter<T> {
  /// The counter's "memory"
  count: T,
}

impl<T: default::Default + Copy + wrapping::WrappingAdd + arbitrary_int::Number> Counter<T> {
  /// Create a new Counter chip
  #[inline]
  pub fn new() -> Self {
    Default::default()
  }

  /// Read current count without changing it
  #[inline]
  pub fn clear(&mut self) -> T {
    self.count = Default::default();
    self.count
  }

  /// Increment count
  #[inline]
  pub fn increment(&mut self) -> T {
    self.count = self.count.wrapping_add(&T::new(1.into()));
    self.count
  }
  
  /// Read current count without changing it
  #[inline]
  pub fn read(&self) -> T {
    self.count
  }
}