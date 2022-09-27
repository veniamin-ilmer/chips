//! Testing using the official MCS4 Evaluation Kit with 4001-0009

use arbitrary_int::{u2,u4};

use log::warn;
use chips::i4001;
use chips::i4002;
use chips::i4004;

use simplelog::*;

use std::time;

fn convert_ram_index(command_control: u4, designated_index: i4004::DesignatedIndex) -> usize {
  let bank = match command_control.value() {
    0b000 => 0,
    0b001 => 1,
    0b010 => 2,
    0b100 => 3,
    _ => { warn!("Invalid command control register: {}", command_control);
      0
    },
  };
  //bank takes upper 2 bits. designated index takes lower 2 bits.
  (designated_index.chip_index().value() | (bank << 2)) as usize
}

#[test]
fn test() {
//  TermLogger::init(LevelFilter::Trace, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

  let mut binary = vec![0xE2, 0xCF, 0x2A, 0x41, 0x50, 0xDE, 0x50, 0xE5,
                        0x30, 0xFE, 0x50, 0xEE, 0x50, 0xE5, 0x50, 0xEE,
                        0x50, 0xE5, 0x2A, 0x42, 0x5F, 0xFF, 0x57, 0x1A,
                        0x48, 0x24, 0x5F, 0xFF, 0x53, 0x20, 0x4C, 0x18,
                        0x5F, 0xFF, 0x4F, 0xFF, 0x22, 0xCB, 0xF0, 0x2B,
                        0xE1, 0x21, 0xE0, 0xF2, 0x71, 0x29, 0xE4, 0xF2,
                        0xE5, 0xF2, 0xE6, 0xF2, 0xE7, 0x60, 0x72, 0x29,
                        0xFA, 0x50, 0xF7, 0x73, 0x39, 0x25, 0xFA, 0xF5,
                        0xE1, 0x1A, 0x47, 0x1C, 0x4F, 0x19, 0x50, 0x12,
                        0x50, 0x14, 0x52, 0x11, 0x43, 0x40, 0x45, 0xF0,
                        0x40, 0x3F, 0x2C, 0x66, 0x2E, 0x59, 0x20, 0x00,
                        0x3D, 0x21, 0x84, 0x85, 0xE0, 0xF6, 0x74, 0x59,
                        0x75, 0x59, 0x50, 0xDE, 0x40, 0x75, 0x50, 0xDE,
                        0x21, 0x94, 0x95, 0xE0, 0xF0, 0x74, 0x68, 0x75,
                        0x68, 0xF0, 0x2B, 0xE1, 0x3F, 0xFA, 0x68, 0xA8,
                        0xE0, 0xB9, 0xA9, 0xE2, 0xFB, 0xE0, 0x74, 0x75,
                        0xF0, 0xF8, 0xE0, 0xFC, 0xE0, 0x74, 0x81, 0xF0,
                        0xFB, 0xE0, 0xF2, 0x74, 0x88, 0xDF, 0xE0, 0xF7,
                        0xE0, 0x1C, 0x8D, 0xF0, 0x2B, 0xE1, 0xDF, 0xF9,
                        0xE0, 0xFA, 0xF9, 0xE0, 0xF3, 0xF6, 0xE0, 0x74,
                        0x9C, 0x24, 0xC0, 0x21, 0xE9, 0x71, 0xA3, 0xEC,
                        0xED, 0xEE, 0xEF, 0x60, 0x74, 0xA3, 0x20, 0x20,
                        0x22, 0x30, 0x21, 0xE8, 0x61, 0x23, 0xE8, 0xE0,
                        0x73, 0xB2, 0x20, 0x00, 0x20, 0x10, 0xF0, 0x2B,
                        0xE1, 0x21, 0xEB, 0x61, 0x23, 0xEB, 0xE0, 0x73,
                        0xC1, 0x2B, 0xEC, 0x14, 0xD7, 0xD8, 0x21, 0xE1,
                        0xF0, 0x2B, 0xE4, 0x19, 0xD3, 0x40, 0x20, 0xF2,
                        0xE4, 0xD2, 0x21, 0xE1, 0x40, 0x20, 0x2B, 0xAB,
                        0xF1, 0xE1, 0xF5, 0xBB, 0xC0, 0x21, 0x23, 0x25,
                        0x27, 0x29, 0x2B, 0x2D, 0x2F, 0xC0, 0x32, 0x34,
                        0x36, 0x38, 0x3A, 0x3C, 0x3E, 0x30, 0xC0, 0xA4,
                        0xF5, 0xFD, 0xB4, 0xEA, 0xC0, 0x00, 0xFF, 0x00];
    
  binary.resize(0x100, 0xFF);

  let mut i4001s = [i4001::I4001::new(binary.try_into().unwrap())];
  let mut i4002s = [i4002::I4002::new(), i4002::I4002::new()];
  let mut i4004 = i4004::I4004::new();

  let mut next_time = time::Instant::now();
  let mut cycles = 0;
  while cycles < 500 {
    if time::Instant::now() >= next_time {
      cycles += 1;
      //Make Rust happy by borrowing things one at a time, then releasing them when done.
      {
        let mut i4004_io = I4004IO {
          i4001s: &mut i4001s,
          i4002s: &mut i4002s,
        };
        i4004.single_run(&mut i4004_io);
      }
      
      {
        let mut i4002_io = I4002IO {
          i4004: &mut i4004,
        };
        for i4002 in &mut i4002s {
          i4002.single_run(&mut i4002_io);
        }
      }
      {
        let mut i4001_io = I4001IO {
        };
        for i4001 in &mut i4001s {
          i4001.single_run(&mut i4001_io);
        }
      }
      if i4004.get_pc() & 0xFF == 0x50 {
        return; //Success
      }
      next_time = time::Instant::now() + time::Duration::from_nanos(1 * 10_800); //10.8 microsecond delay. (8 clock ticket at 740 khz)
    }
  }
  panic!("Failed to run all parts of test code successfully.");
}

struct I4004IO<'a> {
  i4001s: &'a mut [i4001::I4001; 1],
  i4002s: &'a mut [i4002::I4002; 2],
}
impl i4004::IO for I4004IO<'_> {
  fn read_rom(&self, address: i4004::ROMAddress) -> u8 {
    let high_addr = address.chip_index().value() as usize;
    let low_addr = address.offset();
    let i4001 = &self.i4001s[high_addr % self.i4001s.len()];  //Wrap around
    i4001.read_rom(low_addr)
  }
  
  fn read_rom_port(&self, designated_index: i4004::DesignatedIndex) -> u4 {
    let high_addr = (designated_index.chip_index() << 2 | designated_index.reg_index()).value() as usize;
    let i4001 = &self.i4001s[high_addr % self.i4001s.len()];  //Wrap around
    i4001.read_port()
  }
  fn write_rom_port(&mut self, designated_index: i4004::DesignatedIndex, value: u4) {
    let high_addr = (designated_index.chip_index() << 2 | designated_index.reg_index()).value() as usize;
    let i4001 = &mut self.i4001s[high_addr % self.i4001s.len()];  //Wrap around
    i4001.write_port(value);
  }
  
  fn read_ram(&self, command_control: u4, designated_index: i4004::DesignatedIndex) -> u4 {
    let high_addr = convert_ram_index(command_control, designated_index);
    if let Some(i4002) = self.i4002s.get(high_addr) {
      i4002.read_ram(designated_index.reg_index(), designated_index.char_index())
    } else {
      warn!("Write to nonexisting ram {}.", high_addr);
      u4::new(0)
    }
  }
  fn write_ram(&mut self, command_control: u4, designated_index: i4004::DesignatedIndex, value: u4) {
    let high_addr = convert_ram_index(command_control, designated_index);
    if let Some(i4002) = self.i4002s.get_mut(high_addr) {
      i4002.write_ram(designated_index.reg_index(), designated_index.char_index(), value);
    } else {
      warn!("Write to nonexisting ram {}.", high_addr);
    }
  }
  fn read_ram_status(&self, command_control: u4, designated_index: i4004::DesignatedIndex, status_index: u2) -> u4 {
    let high_addr = convert_ram_index(command_control, designated_index);
    if let Some(i4002) = self.i4002s.get(high_addr) {
      i4002.read_status(designated_index.reg_index(), status_index)
    } else {
      warn!("Write to nonexisting ram {}.", high_addr);
      u4::new(0)
    }
  }
  fn write_ram_status(&mut self, command_control: u4, designated_index: i4004::DesignatedIndex, status_index: u2, value: u4) {
    let high_addr = convert_ram_index(command_control, designated_index);
    if let Some(i4002) = self.i4002s.get_mut(high_addr) {
      i4002.write_status(designated_index.reg_index(), status_index, value);
    } else {
      warn!("Write to nonexisting ram {}.", high_addr);
    }
  }
  fn write_ram_port(&mut self, command_control: u4, designated_index: i4004::DesignatedIndex, value: u4) {
    let high_addr = convert_ram_index(command_control, designated_index);
    if let Some(i4002) = self.i4002s.get_mut(high_addr) {
      i4002.write_port(value);
    } else {
      warn!("Write to nonexisting ram {}.", high_addr);
    }
  }
}


struct I4001IO {
}
impl i4001::IO for I4001IO {
  fn port0(&mut self, _value: bool) {
  }
  fn port1(&mut self, _value: bool) {
  }
  fn port2(&mut self, _value: bool) {
  }
  fn port3(&mut self, _value: bool) {
  }
}


struct I4002IO<'a> {
  i4004: &'a mut i4004::I4004,
}
impl i4002::IO for I4002IO<'_> {
  fn port0(&mut self, value: bool) {
    self.i4004.set_test_flag(value);
  }
  fn port1(&mut self, _value: bool) {
  }
  fn port2(&mut self, _value: bool) {
  }
  fn port3(&mut self, _value: bool) {
  }
}
