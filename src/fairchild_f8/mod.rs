//! Fairchild Channel F, initially named as the Fairchild Video Entertainment System (VES)
//! Released in November 1976

use log::{info, warn};
pub mod cpu3850;
pub mod psu3851;
pub mod dmi3852;
use crate::ram;
use arbitrary_int::u6;
use alloc::vec;

pub struct Board {
  pub cpu: cpu3850::CPU,
  pub roms: vec::Vec<psu3851::F3851>,
  pub rams: vec::Vec<dmi3852::F3852>,
  pub vram: [ram::MK4027; 4],
  pub ports: [u8; 256], //external port values
}

impl Board {
  pub fn new(bios_rom: Option<vec::Vec<u8>>, extra_rom: Option<vec::Vec<u8>>) -> Self {
    let mut roms = vec![];
    
    let mut mask = 0;
    if let Some(data) = bios_rom {
      for chunk in data.chunks(1024) {
        roms.push(psu3851::F3851::new(chunk.to_vec().try_into().unwrap(), u6::new(mask), u6::new(mask + 1)));
        mask += 1;
      }
    }
    
    if let Some(data) = extra_rom {
      for chunk in data.chunks(1024) {
        roms.push(psu3851::F3851::new(chunk.to_vec().try_into().unwrap(), u6::new(mask), u6::new(mask + 1)));
        mask += 1;
      }
    }
    Self {
      cpu: cpu3850::CPU::new(),
      roms,
      rams: vec![
        dmi3852::F3852::new(u6::new(0xA), u6::new(0b1001)), //Port 0x24 and 0x25 set for maze (videocart 10)
        dmi3852::F3852::new(u6::new(0xB), u6::new(0xC)),
      ],
      vram: [
        ram::MK4027::new(),
        ram::MK4027::new(),
        ram::MK4027::new(),
        ram::MK4027::new(),
      ],
      ports: [0; 256],
    }
  }

  /// Runs the CPU and has it interact with the PSU
  pub fn run_cycle(&mut self) -> u8 {
    {
      let mut io = VideoIO {
        board: self,
      };
      io.run_cycle();
    }
    
    {
      let mut io = F3850IO {
        rams: &mut self.rams,
        roms: &mut self.roms,
        ports: &mut self.ports,
      };
      self.cpu.run_cycle(&mut io)
    }

  }
  
  /// Combines internal and external port values together
  pub fn read_port(&self, port: u8) -> u8 {
    let mut ret = 0;
    if port < 4 {
      ret = self.cpu.ports[port as usize];
    } else {
      for rom in self.roms.iter() {
        ret |= rom.read_port(port);
      }
      for ram in self.rams.iter() {
        ret |= ram.read_port(port);
      }
    }
    ret | self.ports[port as usize]
  }
}

struct F3850IO<'a> {
  rams: &'a mut vec::Vec<dmi3852::F3852>,
  roms: &'a mut vec::Vec<psu3851::F3851>,
  ports: &'a mut [u8; 256],
}


/// Outputs (upper, lower)
fn u16_to_u8(source: u16) -> (u8, u8) {
  let bytes = source.to_be_bytes();
  (bytes[0], bytes[1])
}

fn u8_to_u16(upper: u8, lower: u8) -> u16 {
  u16::from_be_bytes([upper, lower])
}


impl cpu3850::IO for F3850IO<'_> {
  fn output(&mut self, port: u8, value: u8) {
    info!("OUT Port: {} Value: {:08b}", port, value);
    for rom in &mut *self.roms {
      rom.write_port(port, value);
    }
    for ram in &mut *self.rams {
      ram.write_port(port, value);
    }
    
    //Hardwired for videocart 10 (maze)
    //Source - https://www.reddit.com/r/ChannelF/comments/91cpj8/reading_and_writing_from_ports_36_37/
    if port == 0x24 {
      let port24 = value as usize;
      let addr1 = (port24 & 0b00000010) << 2  //1 maps to 3
                | (port24 & 0b00000100);      //2 maps to 2

      let port25 = self.input(0x25) as usize;
      let addr2 = (port25 & 0b00000001)       //0 maps to 0
                | (port25 & 0b00000010) << 3  //1 maps to 4
                | (port25 & 0b00000100) << 3  //2 maps to 5
                | (port25 & 0b00001000) << 3  //3 maps to 6
                | (port25 & 0b00010000) >> 3  //4 maps to 1
                | (port25 & 0b00100000) << 2  //5 maps to 7
                | (port25 & 0b01000000) << 2  //6 maps to 8
                | (port25 & 0b10000000) << 2; //7 maps to 9

      let hardwired_address = addr1 | addr2;
      
      let is_write = (port24 & 0b1) != 0;
      if is_write {
        //Write port bit to ram.
        self.rams[0].ram.write_bit(hardwired_address, (port24 & 0b1000) != 0);
      } else {
        //Read. Update the port to contain the ram bit, so it can be read next time.
        let data_bit = (self.rams[0].ram.read_bit(hardwired_address) as u8) << 7;
        self.rams[0].write_port(0x24, (value & 0b01111111) | data_bit);
      }
    }
  }
  /// Read from IO port. Does NOT include external ports, because it doesn't include CPU ports.
  fn input(&self, port: u8) -> u8 {
    info!("IN Port: {}", port);
    let mut ret = 0;
    for rom in self.roms.iter() {
      ret |= rom.read_port(port);
    }
    for ram in self.rams.iter() {
      ret |= ram.read_port(port);
    }
    ret
  }
  
  fn read_external_port(&self, port: u8) -> u8 {
    self.ports[port as usize]
  }
  
  /// Read next code byte
  fn next_code(&mut self) -> u8 {
    let mut ret = 0;
    for rom in &mut *self.roms {
      ret |= rom.next_code();  //We must run it for all ROMS so they update their pc0.
    }
    for ram in &mut *self.rams {
      ret |= ram.next_code();  //We must run it for all RAMS so they update their pc0.
    }
    ret
  }
  /// Read code byte without updating read pointer
  fn peak_code(&self) -> i8 {
    let mut ret = 0;
    for rom in self.roms.iter() {
      ret |= rom.peak_code();
    }
    for ram in self.rams.iter() {
      ret |= ram.peak_code();
    }
    ret
  }
  
  /// Read next data byte
  fn next_data(&mut self) -> u8 {
    let mut ret = 0;
    for rom in &mut *self.roms {
      ret |= rom.next_data();  //We must run it for all ROMS so they update their dc0.
    }
    for ram in &mut *self.rams {
      ret |= ram.next_data();  //We must run it for all RAMS so they update their dc0.
    }
    ret
  }
  /// Write next data byte
  fn write_data(&mut self, data: u8) {
    warn!("Attempted to write {:02X} to {:04X}", data, self.roms[0].dc0);
    for rom in &mut *self.roms {
      rom.next_data();  //We must run it for all ROMS so they update their dc0.
    }
    for ram in &mut *self.rams {
      ram.write_data(data);
    }
  }

  /// Jump to direct address. push_pc will back up the current position, so you can return to it later. (Call vs Jump)
  fn jump(&mut self, upper: u8, lower: u8, push_pc: bool) {
    let addr = u8_to_u16(upper, lower);
    for rom in &mut *self.roms {
      rom.jump(addr, push_pc);
    }
    for ram in &mut *self.rams {
      ram.jump(addr, push_pc);
    }
  }
  /// Jump to relative address.
  fn jump_relative(&mut self, relative_addr: i8) {
    for rom in &mut *self.roms {
      rom.jump_relative(relative_addr);
    }
    for ram in &mut *self.rams {
      ram.jump_relative(relative_addr);
    }
  }
  /// Return from address.
  fn ret_pc(&mut self) {
    for rom in &mut *self.roms {
      rom.ret_pc();
    }
    for ram in &mut *self.rams {
      ram.ret_pc();
    }
  }
  
  /// Used by ADC instruction
  fn add_dc0(&mut self, a: i8) {
    for rom in &mut *self.roms {
      rom.add_dc0(a);
    }
    for ram in &mut *self.rams {
      ram.add_dc0(a);
    }
  }
  /// Get dc0 pointer, returns upper, lower
  fn get_dc0(&self) -> (u8, u8) {
    let mut ret = 0;
    for rom in self.roms.iter() {
      ret |= rom.dc0;
    }
    for ram in self.rams.iter() {
      ret |= ram.dc0;
    }
    u16_to_u8(ret)
  }
  /// Set dc0 pointer
  fn set_dc0(&mut self, upper: u8, lower: u8) {
    let dc0 = u8_to_u16(upper, lower);
    for rom in &mut *self.roms {
      rom.dc0 = dc0;
    }
    for ram in &mut *self.rams {
      ram.dc0 = dc0;
    }
  }
  /// Swap DC pointers
  fn swap_dc(&mut self) {
    for rom in &mut *self.roms {
      rom.swap_dc();
    }
    for ram in &mut *self.rams {
      ram.swap_dc();
    }
  }
  
  /// Get pc1 pointer, returns upper, lower
  fn get_pc1(&self) -> (u8, u8) {
    let mut ret = 0;
    for rom in self.roms.iter() {
      ret |= rom.pc1;
    }
    for ram in self.rams.iter() {
      ret |= ram.pc1;
    }
    u16_to_u8(ret)
  }
  /// Set pc1 pointer
  fn set_pc1(&mut self, upper: u8, lower: u8) {
    let pc1 = u8_to_u16(upper, lower);
    for rom in &mut *self.roms {
      rom.pc1 = pc1;
    }
    for ram in &mut *self.rams {
      ram.pc1 = pc1;
    }
  }
  
}


struct VideoIO<'a> {
  board: &'a mut Board,
}

impl VideoIO<'_> {
  /// This fills in my lacking knowledge of the communication that goes on between the CPU/PSU and the VRAM.
  /// The reason this cannot be done on the CPU out, is that multiple ports needs to be read at the same time, which causes a self reference error.
  fn run_cycle(&mut self) {
    if self.read_cpu_port(0) & 0b100000 == 0b100000 {
      //The pixels 128x64 = 8192 which is twice of our vram 4096. So one chip has the 0 to 4095 and the other chip has 4096 to 8191
      let color = !self.read_cpu_port(1) >> 6; //We only care about the inverted bits 6 and 7.
      let video_x = self.read_rom_port(4) & 0b01111111;  //Don't include the last bit. It keeps getting set for some reason, but is beyond the 128 limit.
      let video_y = self.read_rom_port(5) & 0b00111111;  //Don't include the last 2 bits. It is beyond the 64 limit.
      let address = (video_x as usize) + (video_y as usize) * 128;
      let value = if color & 0b1 == 0b1 { true } else { false };
      if address < 4096 {
        self.board.vram[0].write_bit(address, value);
      } else {
        self.board.vram[1].write_bit(address - 4096, value);
      }
      
      let value = if color & 0b10 == 0b10 { true } else { false };
      if address < 4096 {
        self.board.vram[2].write_bit(address, value);
      } else {
        self.board.vram[3].write_bit(address - 4096, value);
      }
    }
  }


  /// Read from IO port (internal + external)
  fn read_rom_port(&self, port: u8) -> u8 {
    let mut ret = 0;
    for rom in self.board.roms.iter() {
      ret |= rom.read_port(port);
    }
    ret | self.board.ports[port as usize]
  }
  
  /// Read from CPU IO port (Internal + external)
  fn read_cpu_port(&self, port: u8) -> u8 {
    self.board.cpu.ports[port as usize] | self.board.ports[port as usize]
  }

}