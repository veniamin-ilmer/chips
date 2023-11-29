//! Segment Decoders and decimal decoders

extern crate alloc;

use arbitrary_int::u4;
use alloc::{vec, vec::Vec};

/// In hardware, this would be coded with AND, OR, and NOT gates.
/// I found however that a match lookup table is more efficient, using fewer instructions at the cost of a few bytes of memory.
pub struct SegmentDecoder {
  lookup: Vec<u8>,
}

impl SegmentDecoder {
  
  /// 7 segment secoder with 6 and 9 not having a horizontal line
  pub fn new_s74x47() -> Self {
    Self {
      lookup: vec![
        0b0_11_1_11_1, 0b0_00_0_11_0, 0b1_01_1_01_1, 0b1_00_1_11_1,
        0b1_10_0_11_0, 0b1_10_1_10_1, 0b1_11_1_10_0, 0b0_00_0_11_1,
        0b1_11_1_11_1, 0b1_10_0_11_1, 0b1_01_1_00_0, 0b1_00_1_10_0,
        0b1_10_0_01_0, 0b1_10_1_00_1, 0b1_11_1_00_0, 0,
      ]
    }
  }
  
  /// 7 segment secoder with 6 and 9 having a horizontal line
  pub fn new_s74x247() -> Self {
    Self {
      lookup: vec![
        0b0_11_1_11_1, 0b0_00_0_11_0, 0b1_01_1_01_1, 0b1_00_1_11_1,
        0b1_10_0_11_0, 0b1_10_1_10_1, 0b1_11_1_10_1, 0b0_00_0_11_1,
        0b1_11_1_11_1, 0b1_10_1_11_1, 0b1_01_1_00_0, 0b1_00_1_10_0,
        0b1_10_0_01_0, 0b1_10_1_00_1, 0b1_11_1_00_0, 0,
      ]
    }
  }
  
  /// Convert u4 into 7 segment display bits
  #[inline]
  pub fn decode(&self, decimal: u4) -> u8 {
    let mut decimal_usize = decimal.value() as usize;
    if decimal_usize >= self.lookup.len() {
      decimal_usize = self.lookup.len() - 1;
    }
    self.lookup[decimal_usize]
  }
}

