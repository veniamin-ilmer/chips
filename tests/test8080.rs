use std::io::prelude::*;
use std::fs::File;
//use simplelog::*;
use chips::{ram,cpu};

#[test]
fn test() {
  //TermLogger::init(LevelFilter::Trace, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

  let mut f = File::open("roms/TEST8080.COM").unwrap();
  let mut test_code = Vec::new();
  f.read_to_end(&mut test_code).unwrap();

  //Reading a COM file. COM files start at memory address 0x100.
  let mut memory = vec![0u8; 0x100];
  memory[0] = 0xc3; memory[1] = 0; memory[2] = 0x1; //JMP 0x100
  memory[5] = 0xc9; //RET
  memory.append(&mut test_code);
  memory.resize(0x1_0000, 0xFF);
  let memory: [u8;0x1_0000] = memory.try_into().unwrap();
  let mut ram = ram::RAM::<0x1_0000>::new();
  ram.set_total(memory);
  
  let mut cpu = cpu::I8080::new();
  
  for _cycle in 0..2000 {
    let mut io = IO {
      memory: &mut ram,
    };
    cpu.run_cycle(&mut io);

    if cpu.cpu.pc == 0x06B4 {
      return; //Success
    }
  }
  panic!("Failed to run all parts of test code successfully.");
}

struct IO<'m> {
  memory: &'m mut ram::RAM::<0x1_0000>,
}

impl cpu::i8080::IO for IO<'_> {
  fn output(&mut self, port: u8, value: u8) {
    panic!("OUT {} {}", port, value);
  }
  
  fn input(&mut self, port: u8) -> u8 {
    panic!("IN {}", port);
  }
}

impl cpu::MemoryIO<u16> for IO<'_> {
  fn read_mem<T: chips::ReadArr>(&self, address: u16) -> T {
    self.memory.read(address as usize)
  }
  fn write_mem<T: chips::WriteArr>(&mut self, address: u16, value: T) {
    self.memory.write(address as usize, value);
  }
}