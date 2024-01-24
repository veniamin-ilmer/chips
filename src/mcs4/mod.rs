//! MCS - 4, short for Micro Computer System, 4-bit, was the Intel 4004 family

pub mod cpu4004;
pub mod rom4001;
pub mod ram4002;
pub mod shifter4003;

use bitbybit::bitfield;
use arbitrary_int::{u2, u4};
use alloc::vec;

/// Memory Control, coming from CPU, read by ROM and RAM.
#[bitfield(u8, default: 0)]
struct ControlLines {
  //0..=3
  #[bit(0, rw)]
  ram: [bool; 4],
  #[bit(4, rw)]
  rom: bool,
}

/// 8 bit instruction opcode
#[bitfield(u8, default: 0)]
pub struct Byte {
  /// High Operation Code (OPR)
  #[bits(4..=7, rw)]
  high: u4,

  /// Low Modifer (OPA)
  #[bits(0..=3, rw)]
  low: u4,
}

/// 12 bit instruction address
#[bitfield(u16, default: 0)]
pub struct Address {
  /// Which ROM chip?
  #[bits(8..=11, rw)]
  chip_index: u4,

  /// Which high address inside of the ROM chip?
  #[bits(4..=7, rw)]
  high: u4,

  /// Which low address inside of the ROM chip?
  #[bits(0..=3, rw)]
  low: u4,
}


#[derive(Clone,Copy)]
enum ExecuteOut {
  Nothing,
  /// Chip index=2 bit, Reg Index=2 bit, Char Index=4 bit
  SRC(Byte),
  /// Modifier, Value
  Write(Byte),
}

/// MCS-4 Board
pub struct Board {
  pub roms: vec::Vec<rom4001::ROM>,
  pub rams: vec::Vec<ram4002::RAM>,
  pub cpu: cpu4004::CPU,
}

impl Board {
  /// Create a new board
  pub fn new(data: vec::Vec<u8>, ram_count: u8) -> Self {
    let mut roms = vec![];
    let mut page = 0;
    for chunk in data.chunks(0x100) {
      roms.push(rom4001::ROM::new(chunk.to_vec().try_into().unwrap(), u4::new(page)));
      page += 1;
    }
    
    let mut rams = vec![];
    for page in 0..ram_count {
      rams.push(ram4002::RAM::new(u2::new(page)));
    }
    
    Self {
      roms: roms,
      rams: rams,
      cpu: cpu4004::CPU::new(),
    }
  }
  
  /// Run an instruction cycle with all chips
  pub fn run_cycle(&mut self) {
    //CPU sends address to ROM
    //A1, A2, A3
    //ROM sends data to everyone
    //M1 and M2
    let mut address = self.cpu.get_addr();
    //Evaluation Kit fails without this wrapping..
    address = address.with_chip_index(u4::new(((address.chip_index().value() as usize) % self.roms.len()) as u8));
    
    let opcode = Byte::new_with_raw_value({
      let mut opcode = 0;
      for i in 0..self.roms.len() {
        opcode |= self.roms[i].get_opcode(address);
      }
      opcode
    });
    
    self.cpu.print();
    log::trace!("Opcode: {:02X}", opcode.raw_value());
    
    let control_lines = self.cpu.set_opcode(opcode);

    //The control line will signal which ROM/RAM chip should listen to the command.
    //CPU executes, and ROM or RAM exchange info with CPU
    //X1, X2, X3
    let mut read = Default::default();
    for i in 0..self.roms.len() {
      if control_lines.rom() {
        read |= self.roms[i].io_read(opcode.low());
      }
    }
    for i in 0..self.rams.len() {
      if control_lines.ram(i/4) {
        read |= self.rams[i].io_read(opcode.low());
      }
    }
    let command = self.cpu.run_cycle(read);
    match command {
      ExecuteOut::SRC(data) => {
        for i in 0..self.roms.len() {
          self.roms[i].set_register_control(data);
        }
        for i in 0..self.rams.len() {
          self.rams[i].set_register_control(data);
        }
      }
      ExecuteOut::Write(data) => {
        for i in 0..self.roms.len() {
          if control_lines.rom() {
            self.roms[i].io_write(data);
          }
        }
        for i in 0..self.rams.len() {
          if control_lines.ram(i/4) {
            self.rams[i].io_write(data);
          }
        }
      },
      ExecuteOut::Nothing => {},
    }
  }
}