///! The TMS0800 family of chips "calculator on a chip". Introduced in 1973

pub mod alu;
pub mod control;

use arbitrary_int::{u2,u4,u5,u11};
use crate::shifter;

/// WordSelect, Mapped from the "mask".
type WordSelect = shifter::Shifter16<11>;

pub struct TMS0800 {
  rom: [u11; 320],
  pub alu: alu::ALU,
  pub control: control::ControlUnit,
  word_selects: [u11; 16],
  /// The one branching condition
  carry: bool,
}

impl TMS0800 {
  pub fn new(rom: [u11; 320], pla: [u32; 13], word_selects: [u11; 16], constants: [u4; 16]) -> Self {
    Self {
      rom,
      alu: alu::ALU::new(pla, constants),
      control: control::ControlUnit::new(),
      word_selects: word_selects,
      carry: false,
    }
  }
  
  /// Execute 1 instruction
  pub fn run_cycle(&mut self) {
    let opcode = self.rom[self.control.pc.value() as usize];
    let class = (opcode.value() >> 9) as u8;
    let mask = u4::new((opcode.value() as u8) & 0xF);
    let instruction = u5::new(((opcode.value() >> 4) as u8) & 0b11111);
    let word_select = WordSelect::new(self.word_selects[mask.value() as usize].value());
    
    if class == 3 { //Register Instruction
      if self.alu.run_cycle(word_select, instruction, mask) {
        //Not really documented, but based on the code, this only sets carry, doesn't reset it.
        self.carry = true;
      }
    }
    //Run CU in all cases, including for the alu.
    self.carry = self.control.run_cycle(word_select, opcode, u2::new(class), instruction, self.carry);
  }
}