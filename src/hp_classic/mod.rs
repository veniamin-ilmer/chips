//! HP Classic Calculators includes HP-35 and HP-45.
//! Each clock cycle ends up taking 280 microseconds. (3.671 kHz)

/// Control and Timing chip
pub mod cnt;
/// Arithmetic and Registers chip
pub mod anr;
/// Auxilary Data Storage chips
pub mod ram;
mod rom;

use alloc::vec;

const ROM_CHIP_LEN: usize = 320;  /// 256 * 10 bits = 2560 bits of ROM data. 2560 / 8 = 320 bytes
use arbitrary_int::{
  u3,   //ROM #
  u10,  //ROM opcode
};

use crate::shifter;
type WordSelect = shifter::Shifter16<14>;
/// Each of A&R and RAM shift registers consisted of 14 nibbles (56 bits).
pub type Register = shifter::Shifter64<56>;

/// HP Classic board
pub struct Board<const EXTRA_REGS: usize> {
  /// Arithmetic and Registers chip
  pub anr: anr::AnR,
  /// Control and Timing chip
  pub cnt: cnt::CnT,
  /// ROM data chip
  pub roms: vec::Vec<rom::ROM>,
  /// Auxilary Data Storage chips
  pub ram: ram::RAM<EXTRA_REGS>,
}

impl<const EXTRA_REGS: usize> Board<EXTRA_REGS> {
  /// Create a new board
  pub fn new(packed_rom_data: vec::Vec<u8>) -> Self {
    let mut roms = vec![];
    let mut rom_num = u3::new(0);
    for chunk in packed_rom_data.chunks(ROM_CHIP_LEN) {
      let mut padded_chunk = vec::Vec::from(chunk);
      padded_chunk.resize_with(ROM_CHIP_LEN, Default::default); //This is needed for the last chunk if it is less than the total.
      roms.push(rom::ROM::new(padded_chunk.try_into().unwrap(), rom_num));
      rom_num += u3::new(1);
    }
    Self {
      anr: anr::AnR::new(),
      cnt: cnt::CnT::new(),
      roms,
      ram: ram::RAM::new(),
    }
  }

  /// Run instruction cycle for all chips
  pub fn run_cycle(&mut self) {
    let mut opcode = u10::new(0);
    let mut word_select_data = 0;
    for rom in &mut self.roms {
      let (opcode_rom, word_select_data_rom) = rom.read(self.cnt.next_address);
      opcode |= opcode_rom;
      word_select_data |= word_select_data_rom.read_parallel();
    }
    
    //ROM SELECT Decoding done on all ROMS.
    for rom in &mut self.roms {
      rom.decode(opcode);
    }
    
    //Run C&T and A&R
    word_select_data |= self.cnt.run_cycle(opcode, self.anr.next_carry).read_parallel();
    let ram_data = self.ram.run_cycle(opcode, self.anr.c);
    self.anr.run_cycle(opcode, WordSelect::new(word_select_data), ram_data);
    
    self.cnt.print();
    self.anr.print();
    
  }

}
