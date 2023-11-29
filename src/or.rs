//! Logical OR chips

use arbitrary_int::u4;
use core::marker::PhantomData;
use core::ops;

/// 4 bit OR chip
pub type S74X32 = OR<u4>;

/// ### Example
/// ```
/// use arbitrary_int::u4;
/// use chips::or::S74X32;
///
/// assert_eq!(S74X32::or(u4::new(0b1011),u4::new(0b1101)), u4::new(0b1111));
/// ```
pub struct OR<T> {
  phantom: PhantomData<T>,
}

impl <T> OR<T> where T: ops::BitOr<Output=T> {
  /// OR function
  #[inline]
  pub fn or(num1: T, num2: T) -> T {
    num1 | num2
  }
}
