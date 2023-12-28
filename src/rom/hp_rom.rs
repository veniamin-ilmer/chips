//! The each HP ROM had 256 10 bits of data. It was also responsible for reading decoding a few opcodes:
//! 1. The ROM Select opcode that decides which ROM should run.
//! 2. The WS signal for all codes without a pointer.
//! The C&R would tell it which position to read.

use arbitrary_int::{u3, u10, u14};

/// HP ROM chip
#[allow(non_camel_case_types)]
pub struct HP_ROM {
  /// 256 * 10 bits = 2560 bits of ROM data. 2560 / 8 = 320 bytes
  packed_data: [u8; 320],
  /// ROM number, used to figure out output_enable.
  who_am_i: u3,
  /// ROM Output Enable (ROE) gets set based on the ROM select opcode.
  output_enable: bool,
}

impl HP_ROM {
  /// Create a new HP ROM. Pass in the rom data.
  #[inline]
  pub fn new(packed_data: [u8; 320], who_am_i: u3) -> Self {
    Self {
      packed_data,
      who_am_i,
      output_enable: who_am_i == u3::new(0),  //Initially only ROM 0 is enabled.
    }
  }
  
  /// Read and decode an opcode.
  /// Returns the opcode and the Word Select.
  #[inline]
  pub fn read(&mut self, addr: u8) -> (u10, u14) {
    if self.output_enable {
      let opcode = self.unpack_data(addr);
      let word_select = match opcode.value() & 0b11111 { //0bxxx10 = Type 2. Generate the word select
        //From FIG 9 of the patent. Note that the codes are reversed.
        0b01010 => 0b00000000000111,  //Exponent
        0b11010 => 0b00000000000100,  //Exponent Sign
        0b00110 => 0b01111111111000,  //Mantissa Only
        0b10110 => 0b11111111111000,  //Mantissa with Sign
        0b01110 => 0b11111111111111,  //Entire Word
        0b11110 => 0b10000000000000,  //Mantissa Sign
        _ => 0, //Pointer or up to pointer, handled by C & T
      };
      (opcode, u14::new(word_select))
    } else {  //output disabled.
      (u10::new(0), u14::new(0))
    }
  }

  fn unpack_data(&self, index: u8) -> u10 {
    // Calculate byte positions
    let bit_start = (index as usize) * 10;
    let first_byte_index = bit_start / 8;
    let second_byte_index = first_byte_index + 1;

    // Ensure the byte indices are within bounds
    if second_byte_index >= self.packed_data.len() {
        panic!("Index out of bounds");
    }

    // Extract bytes
    let first_byte = self.packed_data[first_byte_index] as u16;
    let second_byte = self.packed_data[second_byte_index] as u16;

    // Calculate how many bits of the first byte belong to the 10-bit value
    let bits_from_first_byte = 8 - (bit_start % 8);

    // Combine the bits from the two bytes to form the 10-bit value
    let value = (first_byte >> (8 - bits_from_first_byte)) | (second_byte << bits_from_first_byte) & 0x3FF;
    
    u10::new(value)
  }

  //From the ROM Select instruction
  pub fn select_rom(&mut self, rom_num: u3) {
    self.output_enable = self.who_am_i == rom_num;
  }

}
