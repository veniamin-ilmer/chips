//! All CPU chips

pub mod i4004; pub use i4004::I4004;
pub mod i8080; pub use i8080::I8080;
pub mod f3850; pub use f3850::F3850;

/// The data is being requested from RAM and ROM chips with methods in this trait
pub trait MemoryIO<ADDRESS> {
  /// Read from ROM chip
  fn read_mem<T: crate::ReadArr>(&self, address: ADDRESS) -> T;
  /// Write byte to RAM chip
  fn write_mem<T: crate::WriteArr>(&mut self, address: ADDRESS, value: T);
}

/// Generic CPU chip
///
/// Keeps track of current instruction position and stack position
#[derive(Default)]
pub struct CPU<ADDRESS> {
  /// Stack Pointer
  pub sp: ADDRESS,
  /// Program Counter
  pub pc: ADDRESS,
}

impl<ADDRESS: core::default::Default + core::ops::AddAssign<ADDRESS> + core::ops::SubAssign<ADDRESS> + From<u8> + Copy> CPU<ADDRESS> {
  /// Create a new ROM chip
  #[inline]
  pub fn new() -> Self {
    Default::default()
  }
  
  /// Read next ROM byte and move over program counter
  #[inline]
  pub fn next_code_byte(&mut self, io: &impl MemoryIO<ADDRESS>) -> u8 {
    let ret = io.read_mem(self.pc);
    self.pc += ADDRESS::from(1);
    ret
  }
  
  /// Read next ROM word and move over program counter
  #[inline]
  pub fn next_code_word(&mut self, io: &impl MemoryIO<ADDRESS>) -> u16 {
    let ret = io.read_mem(self.pc);
    self.pc += ADDRESS::from(2);
    ret
  }
  
  /// Push data to stack and update stack pointer
  #[inline]
  pub fn push<T: crate::WriteArr>(&mut self, io: &mut impl MemoryIO<ADDRESS>, value: T) {
    self.sp -= ADDRESS::from(2);
    io.write_mem(self.sp, value);
  }
  
  /// Pop data from stack and update stack pointer
  #[inline]
  pub fn pop<T: crate::ReadArr>(&mut self, io: &impl MemoryIO<ADDRESS>) -> T {
    let value = io.read_mem(self.sp);
    self.sp += ADDRESS::from(2);
    value
  }
  
}


/// Add two bytes
///
/// Returns (result, carry, nibble carry)
///
/// ### Example
/// ```
/// use chips::cpu;
/// assert_eq!(cpu::execute_add(0x18, 0x19), (0x31, false, true));
/// assert_eq!(cpu::execute_add(0x13, 0x14), (0x27, false, false));
/// assert_eq!(cpu::execute_add(0x83, 0x94), (0x17, true, false));  //overflowing
/// assert_eq!(cpu::execute_add(0x88, 0x99), (0x21, true, true));   //overflowing
/// ```
pub fn execute_add(byte1: u8, byte2: u8) -> (u8, bool, bool) {
  let (result, carry) = byte1.overflowing_add(byte2);
  let nibble_carry = (byte1 & 0xF) + (byte2 & 0xF) > 9;
  (result, carry, nibble_carry)
}

/// Add two bytes + carry
///
/// Returns (result, carry, nibble carry)
///
/// ### Example
/// ```
/// use chips::cpu;
/// assert_eq!(cpu::execute_add_carry(0x18, 0x19, true), (0x32, false, true));
/// ```
pub fn execute_add_carry(byte1: u8, byte2: u8, carry: bool) -> (u8, bool, bool) {
  let (result, carry1) = byte1.overflowing_add(carry as u8);
  let (result, carry2) = result.overflowing_add(byte2);
  let nibble_carry = (byte1 & 0xF) + (byte2 & 0xF) + ((carry as u8) & 0xF) > 9;
  (result, carry1 | carry2, nibble_carry)
}


/// Subtract two bytes
///
/// ### Example
/// ```
/// use chips::cpu;
/// assert_eq!(cpu::execute_sub(0x5, 0x4), (0x1, false, false));
/// ```
pub fn execute_sub(byte1: u8, byte2: u8) -> (u8, bool, bool) {
  let (result, carry) = byte1.overflowing_sub(byte2);
  let nibble_carry = (result & 0xF) + (byte2 & 0xF) > 9;
  (result, carry, nibble_carry)
}

/// Subtract two bytes - carry
///
/// Returns (result, carry, nibble carry)
///
/// ### Example
/// ```
/// use chips::cpu;
/// assert_eq!(cpu::execute_sub_carry(0x5, 0x4, true), (0x0, false, false));
/// ```
pub fn execute_sub_carry(byte1: u8, byte2: u8, carry: bool) -> (u8, bool, bool) {
  let (result, carry1) = byte1.overflowing_sub(carry as u8);
  let (result, carry2) = result.overflowing_sub(byte2);
  let nibble_carry = (result & 0xF) + (byte2 & 0xF) + ((carry as u8) & 0xF) > 9;
  (result, carry1 | carry2, nibble_carry)
}


/// Decimal Adjust after Addition adjusts numbers to look decimal in hexadecimal.
///
/// ### Example
/// ```
/// use chips::cpu;
/// let (num, carry, nibble_carry) = cpu::execute_add(0x38, 0x45);
/// assert_eq!(num, 0x7D);
/// let daa = cpu::execute_daa(num, carry, nibble_carry);
/// assert_eq!(daa, (0x83, false, true));  //38+45=83 aux is true because 1 was added to 7 to make 8.
///
/// let (num, carry, nibble_carry) = cpu::execute_add(0x38, 0x41);
/// assert_eq!(num, 0x79);
/// let daa = cpu::execute_daa(num, carry, nibble_carry);
/// assert_eq!(daa, (0x79, false, false));  //38+41=79
///
/// let (num, carry, nibble_carry) = cpu::execute_add(0x83, 0x54);
/// assert_eq!(num, 0xD7);
/// let daa = cpu::execute_daa(num, carry, nibble_carry);
/// assert_eq!(daa, (0x37, true, false));  //83+54=137 carry is true because 1 needs to be added to the next byte.
///
/// let (num, carry, nibble_carry) = cpu::execute_add(0x09, 0x08);
/// assert_eq!(num, 0x11);
/// let daa = cpu::execute_daa(num, carry, nibble_carry);
/// assert_eq!(daa, (0x17, false, true));  //9+8=17
/// ```
#[inline]
pub fn execute_daa(mut byte: u8, carry: bool, nibble_carry: bool) -> (u8, bool, bool) {
  let nibble_carry = if nibble_carry || (byte & 0xF) > 9 {
    byte = byte.wrapping_add(6);
    true
  } else {
    false
  };
  let carry = if carry || byte > 0x90 {
    byte = byte.wrapping_add(0x60);
    true
  } else {
    false
  };
  (byte, carry, nibble_carry)
}