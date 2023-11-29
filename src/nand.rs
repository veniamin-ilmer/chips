//! Logical NAND chips

use arbitrary_int::u4;
use core::marker::PhantomData;
use core::ops;

/// 4 bit NAND chip
pub type S74X00 = NAND<u4>;

/// ### Example
/// ```
/// use arbitrary_int::u4;
/// use chips::nand::S74X00;
///
/// assert_eq!(S74X00::nand(u4::new(0b1011),u4::new(0b1101)), u4::new(0b0110));
/// ```
pub struct NAND<T> {
  phantom: PhantomData<T>,
}

impl <T> NAND<T> where T: ops::BitAnd<Output=T> + ops::Not<Output=T> {
  /// NOT AND function
  #[inline]
  pub fn nand(num1: T, num2: T) -> T {
    !(num1 & num2)
  }
}
