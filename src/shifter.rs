//! All Shift Register Chips

use arbitrary_int::{ u4 };

/// 7400 Series 8 bit shift register
//pub type S74X166 = Shifter<u8, 8>;

/// ### Generic Shift Register Chip
///
/// Shift registers have been a historical way to store a few bits of data.
///
/// There are two ways to read and write the data. Serial and parallel. Parallel data access is what we are used to with modern processors, where we can load all bits of a byte at the same time.
///
/// Serial is where you only read/write data one bit at a time.
///
/// If for example you have only one wire, then you could only communicate through it by one bit at a time.
///
/// It is called a shift register because in serial mode, bits are literally being shifted over.
///
/// In order to read old bits, you need to push new bits in.
///
/// ### Example
/// ```
/// use chips::Shifter;
///
/// let mut shifter: Shifter<10> = Shifter::new();
/// shifter.read_write_serial(Direction::Left, true, false);
/// shifter.read_write_serial(Direction::Left, true, true);
/// ```
#[derive(Default, Clone, Copy, PartialEq)]
pub struct Shifter64<const NUM_BITS: u32> {
  /// The shifter's "memory"
  data: u64,
}

/// Shifting direction
#[derive(Debug, Clone, Copy)]
pub enum Direction {
  /// Shift left. Also, read from the left, but write from the right.
  Left,
  /// Shift right. Also, read from the left, but write from the left.
  Right,
}

impl<const NUM_BITS: u32> Shifter64<NUM_BITS> {
  const MASK: u64 = (u64::MAX >> (u64::BITS - NUM_BITS));

  /// Create a new shifter
  #[inline]
  pub fn new(data: u64) -> Self {
    Self {
      data
    }
  }

  /// Read all bits at once
  #[inline]
  pub fn read_parallel(&self) -> u64 {
    self.data
  }

  /// If left direction, writes right most bit
  /// If right direction, writes left most bit
  #[inline]
  pub fn shift_with_bit(&mut self, direction: Direction, bit: bool) {
    let shifted = match direction {
      Direction::Left => (self.data << 1) & Self::MASK,
      Direction::Right => self.data >> 1,
    };
    self.data = if bit {
      shifted | match direction {
        Direction::Left => 1,
        Direction::Right => 1 << (NUM_BITS - 1),
      }
    } else {
      shifted
    };
  }

  /// If left direction, writes right most nibble
  /// If right direction, writes left most nibble
  #[inline]
  pub fn shift_with_nibble(&mut self, direction: Direction, nibble: u4) {
    let shifted = match direction {
      Direction::Left => (self.data << 4) & Self::MASK,
      Direction::Right => self.data >> 4,
    };
    self.data = shifted | match direction {
      Direction::Left => nibble.value() as u64,
      Direction::Right => (nibble.value() as u64) << (NUM_BITS - 4),
    };
  }

  
  /// If left direction, reads left most bit
  /// If right direction, reads right most bit
  #[inline]
  pub fn read_bit(self, direction: Direction) -> bool {
    let bit = match direction {
      Direction::Left => self.data >> (NUM_BITS - 1),
      Direction::Right => self.data,
    };
    bit & 1 == 1
  }
  
  /// If left direction, reads left most nibble
  /// If right direction, reads right most nibble
  #[inline]
  pub fn read_nibble(self, direction: Direction) -> u4 {
    let nibble = match direction {
      Direction::Left => self.data >> (NUM_BITS - 4),
      Direction::Right => self.data,
    } as u8;
    u4::new(nibble & 0xF)
  }

}

/// 16 bit shifter chip
#[derive(Default, Clone, Copy, PartialEq)]
pub struct Shifter16<const NUM_BITS: u32> {
  /// The shifter's "memory"
  data: u16,
}


impl<const NUM_BITS: u32> Shifter16<NUM_BITS> {
  const MASK: u16 = (u16::MAX >> (u16::BITS - NUM_BITS));

  /// Create a new shifter chip
  #[inline]
  pub fn new(data: u16) -> Self {
    Self {
      data
    }
  }

  /// Read all bits at once
  #[inline]
  pub fn read_parallel(&self) -> u16 {
    self.data
  }

  /// Shift in new bit, get out the previous bit
  pub fn read_and_shift_bit(&mut self, direction: Direction, in_bit: bool) -> bool {
    let out_bit = self.read_bit(direction);
    self.shift_with_bit(direction, in_bit);
    out_bit
  }

  /// If left direction, writes right most bit
  /// If right direction, writes left most bit
  #[inline]
  pub fn shift_with_bit(&mut self, direction: Direction, bit: bool) {
    let shifted = match direction {
      Direction::Left => (self.data << 1) & Self::MASK,
      Direction::Right => self.data >> 1,
    };
    self.data = if bit {
      shifted | match direction {
        Direction::Left => 1,
        Direction::Right => 1 << (NUM_BITS - 1),
      }
    } else {
      shifted
    };
  }

  /// If left direction, writes right most nibble
  /// If right direction, writes left most nibble
  #[inline]
  pub fn shift_with_nibble(&mut self, direction: Direction, nibble: u4) {
    let shifted = match direction {
      Direction::Left => (self.data << 4) & Self::MASK,
      Direction::Right => self.data >> 4,
    };
    self.data = shifted | match direction {
      Direction::Left => nibble.value() as u16,
      Direction::Right => (nibble.value() as u16) << (NUM_BITS - 4),
    };
  }

  
  /// If left direction, reads left most bit
  /// If right direction, reads right most bit
  #[inline]
  pub fn read_bit(self, direction: Direction) -> bool {
    let bit = match direction {
      Direction::Left => self.data >> (NUM_BITS - 1),
      Direction::Right => self.data,
    };
    bit & 1 == 1
  }
  
  /// If left direction, reads left most nibble
  /// If right direction, reads right most nibble
  #[inline]
  pub fn read_nibble(self, direction: Direction) -> u4 {
    let nibble = match direction {
      Direction::Left => self.data >> (NUM_BITS - 4),
      Direction::Right => self.data,
    } as u8;
    u4::new(nibble & 0xF)
  }

}
