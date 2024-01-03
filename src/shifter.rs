//! All Shift Register Chips

use arbitrary_int::{ u4 };

/// Intel 4003 is a shift register with 10 bits, capable of both serial and parallel read/write
pub type I4003 = PulsedShifter16<10>;

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
        Direction::Right => 1 << NUM_BITS,
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


#[derive(Default, Clone, Copy, PartialEq)]
pub struct PulsedShifter16<const NUM_BITS: u32> {
  /// The shifter's "memory"
  data: u16,
  /// We only shift data after pulse switches from high to low, so we must keep track of the pulse
  pulse: bool,
}


impl<const NUM_BITS: u32> PulsedShifter16<NUM_BITS> {
  const MASK: u16 = (u16::MAX >> (u16::BITS - NUM_BITS));

  #[inline]
  pub fn new() -> Self {
    Default::default()
  }

  /// Write and Read to the shifter at the same time.
  ///
  /// Write will only happen if `pulse` switches from high to low. Otherwise `in_bit` will be ignored.
  #[inline]
  pub fn read_write_serial(&mut self, direction: Direction, in_bit: bool, pulse: bool) -> bool {
    let out_bit = self.read_bit(direction);
    if self.pulse && !pulse {  //Updates happen when switching from a high pulse to low pulse.
      self.shift_with_bit(direction, in_bit);
    }
    self.pulse = pulse;
    out_bit
  }

  /// Read all bits at once
  #[inline]
  pub fn read_parallel(&self) -> u16 {
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
        Direction::Right => 1 << NUM_BITS,
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
/*
#[derive(Default, Clone, Copy, PartialEq)]
pub struct PlusableShifter16<const NUM_BITS: u64> {
  /// The shifter's "memory"
  data: u64,
  /// We only shift data after pulse switches from high to low, so we must keep track of the pulse
  pulse: bool,
}

/// Implement the BitOperations trait for u16
impl BitOperations for u16 {
  /// If left direction, writes right most bit
  /// If right direction, writes left most bit
  #[inline]
  fn shift_with_bit(self, direction: Direction, num_bits: u32, bit: bool) -> Self {
    let shifted = match direction {
      Direction::Left => { let num = self << 1; num.mask(num_bits) },
      Direction::Right => self >> 1,
    };
    let num = match direction {
      Direction::Left => 1,
      Direction::Right => 1 << num_bits,
    };
    if bit {
      shifted | num
    } else {
      shifted & !num
    }
  }

  #[inline]
  fn mask(self, bits: u32) -> Self {
    self & (Self::MAX >> (Self::BITS - bits))
  }

  /// If left direction, writes right most nibble
  /// If right direction, writes left most nibble
  #[inline]
  fn shift_with_nibble(self, direction: Direction, num_bits: u32, nibble: u4) -> Self {
    let shifted = match direction {
      Direction::Left => { let num = self << 4; num.mask(num_bits) },
      Direction::Right => self >> 4,
    };
    shifted | match direction {
      Direction::Left => nibble.value(),
      Direction::Right => nibble.value() << num_bits,
    } as Self
  }
  
  /// If left direction, reads left most bit
  /// If right direction, reads right most bit
  #[inline]
  fn read_bit(self, direction: Direction, num_bits: u32) -> bool {
    let num = match direction {
      Direction::Left => self >> (num_bits - 1),
      Direction::Right => self,
    };
    num & 1 == 1
  }
  
  /// If left direction, reads left most nibble
  /// If right direction, reads right most nibble
  #[inline]
  fn read_nibble(self, direction: Direction, num_bits: u32) -> u4 {
    let bit = match direction {
      Direction::Left => self >> (num_bits - 4),
      Direction::Right => self,
    };
    u4::new(bit as u8 & 0xF)
  }
}


/// Implement the BitOperations trait for u64
impl BitOperations for u64 {
  /// If left direction, writes right most bit
  /// If right direction, writes left most bit
  #[inline]
  fn shift_with_bit(self, direction: Direction, num_bits: u32, bit: bool) -> Self {
    let shifted = match direction {
      Direction::Left => { let num = self << 1; num.mask(num_bits) },
      Direction::Right => self >> 1,
    };
    let num = match direction {
      Direction::Left => 1,
      Direction::Right => 1 << num_bits,
    };
    if bit {
      shifted | num
    } else {
      shifted & !num
    }
  }

  #[inline]
  fn mask(self, bits: u32) -> Self {
    self & (Self::MAX >> (Self::BITS - bits))
  }

  /// If left direction, writes right most nibble
  /// If right direction, writes left most nibble
  #[inline]
  fn shift_with_nibble(self, direction: Direction, num_bits: u32, nibble: u4) -> Self {
    let shifted = match direction {
      Direction::Left => { let num = self << 4; num.mask(num_bits) },
      Direction::Right => self >> 4,
    };
    shifted | match direction {
      Direction::Left => nibble.value() as Self,
      Direction::Right => (nibble.value() as Self) << (num_bits - 4),
    }
  }
  
  /// If left direction, reads left most bit
  /// If right direction, reads right most bit
  #[inline]
  fn read_bit(self, direction: Direction, num_bits: u32) -> bool {
    let num = match direction {
      Direction::Left => self >> (num_bits - 1),
      Direction::Right => self,
    };
    num & 1 == 1
  }
  
  /// If left direction, reads left most nibble
  /// If right direction, reads right most nibble
  #[inline]
  fn read_nibble(self, direction: Direction, num_bits: u32) -> u4 {
    let bit = match direction {
      Direction::Left => self >> (num_bits - 4),
      Direction::Right => self,
    };
    u4::new(bit as u8 & 0xF)
  }
}
*/