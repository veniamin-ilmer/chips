//! The each HP ROM had 256 10 bits of data. It was also responsible for reading decoding a few opcodes:
//! 1. The ROM Select opcode that decides which ROM should run.
//! 2. The WS signal for all codes without a pointer.
//! The C&R would tell it which position to read.

use arbitrary_int::{u3, u10, u14};

/// HP ROM chip
#[allow(non_camel_case_types)]
pub struct HP_ROM {
  /// 0x100 or 256 bytes of ROM data
  data: [u10; 0x100],
  /// ROM number, used to figure out output_enable.
  who_am_i: u3,
  /// ROM Output Enable (ROE) gets set based on the ROM select opcode.
  output_enable: bool,
}

impl HP_ROM {
  /// Create a new HP ROM. Pass in the rom data.
  #[inline]
  pub fn new(data: [u10; 0x100], who_am_i: u3) -> Self {
    Self {
      data,
      who_am_i,
      output_enable: who_am_i == u3::new(0),  //Initially only ROM 0 is enabled.
    }
  }
  
  /// Read and decode an opcode.
  /// Returns the opcode and the Word Select.
  #[inline]
  pub fn read(&mut self, addr: u8) -> (u10, u14) {
    let opcode = self.data[addr as usize];
    if self.output_enable {
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

  //From the ROM Select instruction
  pub fn select_rom(&mut self, rom_num: u3) {
    self.output_enable = self.who_am_i == rom_num;
  }

}
