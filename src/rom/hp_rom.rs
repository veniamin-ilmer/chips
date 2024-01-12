//! The each HP ROM had 256 10 bits of data. It was also responsible for reading decoding a few opcodes:
//! 1. The ROM Select opcode that decides which ROM should run.
//! 2. The WS signal for all codes without a pointer.
//! The C&R would tell it which position to read.

use arbitrary_int::{u3, u10};
use log::trace;
use crate::shifter;
type WordSelect = shifter::Shifter16<14>;

/// HP ROM chip
#[allow(non_camel_case_types)]
pub struct HP_ROM {
  /// 256 * 10 bits = 2560 bits of ROM data. 2560 / 8 = 320 bytes
  packed_data: [u8; 320],
  /// ROM number, used to figure out output_enable.
  who_am_i: u3,
  /// ROM Output Enable (ROE) gets set based on the ROM select opcode.
  output_enable: bool,
  /// DELAYED ROM Output Enable, sets the output_enable on the next cycle.
  delayed_output_enable: Option<bool>
}

impl HP_ROM {
  /// Create a new HP ROM. Pass in the rom data.
  #[inline]
  pub fn new(packed_data: [u8; 320], who_am_i: u3) -> Self {
    Self {
      packed_data,
      who_am_i,
      output_enable: who_am_i == u3::new(0),  //Initially only ROM 0 is enabled.
      delayed_output_enable: None,
    }
  }
  
  /// Read and decode an opcode.
  /// Returns the opcode and the Word Select.
  #[inline]
  pub fn read(&mut self, addr: u8) -> (u10, WordSelect) {
    let result = if self.output_enable {
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
      (opcode, WordSelect::new(word_select))
    } else {  //output disabled.
      (u10::new(0), WordSelect::new(0))
    };
    
    //Read the delayed output enable, only after reading above.
    if let Some(output_enable) = self.delayed_output_enable {
      self.output_enable = output_enable;
      self.delayed_output_enable = None;
    }
    
    result
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

  /// ROM Select decoder
  pub fn decode(&mut self, opcode: u10) {
    let rom_num = (opcode.value() >> 7) as u8;
    match opcode.value() & 0b1111111 {
      0b0010000 => { //ROM SELECT
        if self.who_am_i.value() == rom_num {
          trace!("SELECT ROM {}", rom_num);
          self.output_enable = true;
        } else {
          self.output_enable = false;
        }
      },
      0b0110100 => {
        if opcode.value() >> 9 == 1 {
          trace!("DELAYED ROM GROUP {}", rom_num & 0b11);
          todo!();
        }
      },
      0b1110100 => {
        if self.who_am_i.value() == rom_num {
          trace!("DELAYED SELECT ROM {}", rom_num);
          self.delayed_output_enable = Some(true);
        } else {
          self.delayed_output_enable = Some(false);
        }
      }
      _ => (),
    }
  }
}
