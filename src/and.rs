//! Logical AND chips

use arbitrary_int::u4;
use core::marker::PhantomData;
use core::ops;

/// 4 bit AND chip
pub type S74X08 = AND<u4>;

/// ### Example
/// ```
/// use arbitrary_int::u4;
/// use chips::and::S74X08;
///
/// assert_eq!(S74X08::and(u4::new(0b1011),u4::new(0b1101)), u4::new(0b1001));
/// ```
pub struct AND<T> {
  phantom: PhantomData<T>,
}

impl <T> AND<T> where T: ops::BitAnd<Output=T> {
  /// AND function
  #[inline]
  pub fn and(num1: T, num2: T) -> T {
    num1 & num2
  }

  /// Bit AND
  #[inline]
  pub fn and_bits(bit1: bool, bit2: bool) -> bool {
    bit1 & bit2
  }
}
