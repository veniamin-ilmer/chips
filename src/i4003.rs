//! Intel 4003 is a shift register with 10 bits, capable of both serial and parallel read/write

/// Intel 4003 is a shift register with 10 bits, capable of both serial and parallel read/write
pub type I4003 = crate::Shifter<10>;