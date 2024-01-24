///! The TMS0800 family of chips had the ROM built into them. Introduced in 1973
use arbitrary_int::{u4,u5,u9,u11};
use log::{trace,debug};
use crate::shifter;
use crate::cpu::tms_alu;

/// 11 bit flag
pub type Flag = shifter::Shifter16<11>;

/// WordSelect, Mapped from the "mask".
type WordSelect = shifter::Shifter16<11>;

pub struct TMS0800 {
  rom: [u11; 320],
  /// Program Counter. 9 bits accesses up to 512 words of data.
  pub pc: u9,
  pub alu: tms_alu::ALU,
  word_selects: [u11; 16],
  pub fa: Flag,
  pub fb: Flag,
  d: shifter::Shifter16<10>,  //In the patent, this is a 11 bit shifter, but it keeps skipping over the 11th bit, so I make it 10 bits.
  /// The one branching condition
  carry: bool,
  /// Set by keyboard
  pub current_keypress: u16,
  count: usize,
}

impl TMS0800 {
  pub fn new(rom: [u11; 320], alu_map: [u32; 13], word_selects: [u11; 16], constants: [u4; 16]) -> Self {
    Self {
      rom,
      pc: u9::new(0),
      alu: tms_alu::ALU::new(alu_map, constants),
      word_selects: word_selects,
      fa: Flag::new(0),
      fb: Flag::new(0),
      d: shifter::Shifter16::<10>::new(0b1000000000),
      carry: false,
      current_keypress: 0,
      count:0,
    }
  }
  
  /// Execute 1 instruction
  pub fn run_cycle(&mut self) {
    let d = self.d.read_bit(shifter::Direction::Right);
    let opcode = self.rom[self.pc.value() as usize];
    self.pc += u9::new(1);
    let mask = u4::new((opcode.value() as u8) & 0xF);
    let instruction = u5::new(((opcode.value() >> 4) as u8) & 0b11111);
    let class = opcode.value() >> 9;
    let addr = opcode.value() & 0b111111111;
    let mut word_select = WordSelect::new(self.word_selects[mask.value() as usize].value());
    if self.count > 0 {
      self.count -= 1;
      if self.count == 0 {
        panic!("done");
      }
    }
    match class {
      0 => {
        trace!("Jump if not carry to {:03X}", addr);
        if !self.carry {
          self.pc = u9::new(addr);
        } else {
          self.carry = false; //Although not well documented, the patent shows comments how jumping seems to reset the carry
        }
      },
      1 => {
        trace!("Jump if carry to {:03X}", addr);
        if self.carry {
          self.pc = u9::new(addr);
          self.carry = false; //Although not well documented, the patent shows comments how jumping seems to reset the carry
        }
      },
      2 => {
        let direction = shifter::Direction::Right;
        for i in 0..11 {
          let mut fa = self.fa.read_bit(direction);
          let mut fb = self.fb.read_bit(direction);
          if word_select.read_and_shift_bit(direction, false) {
            match instruction.value() {
              0..=15 => {
                trace!("Jump on key match to {:03X}", addr);
                //self.count = 100;

                if self.d.read_parallel() == self.current_keypress {
                  self.pc = u9::new(addr);
                }
              },
              17 | 18 => { //This command is amusing. The jump is part of the command...
                if self.current_keypress == 0 {
                  self.pc -= u9::new(1);  //Keep waiting until there is a keypress.
                } else {
                  match instruction.value() {
                    17 => trace!("WAITDK {:03X}", addr),
                    _ => trace!("WAITNO {:03X}", addr),
                  }
                  self.pc = u9::new(addr);
                }
              },
              19 => { trace!("FB{i} = true"); fb = true; },
              20 => { trace!("FA{i} = true"); fa = true; },
              21 => { trace!("SYNC"); if !d && i == 0 { self.pc -= u9::new(1); } },  //Keep going back until d is reset.
              22 => { trace!("SCAN");
                if !d && i == 0 {
                  self.pc -= u9::new(1);  //Keep going back until d is reset.
                }
                self.carry = self.current_keypress != 0;
              },
              23 => { trace!("FB{i} = false"); fb = false; },
              24 => { trace!("FA{i} = false"); fa = false; },
              25 => { trace!("? FB{i}"); if fb { self.carry = true; } },  //Not really documented, but based on the code, this only sets carry, doesn't reset it.
              26 => { trace!("? FA{i}"); if fa { self.carry = true; } },  //Not really documented, but based on the code, this only sets carry, doesn't reset it.
              27 => { trace!("FB{i} = !FB{i}"); fb = !fb; }
              28 => { trace!("FA{i} = !FA{i}"); fa = !fa; }
              29 => { trace!("? FB{i} != FA{i}"); if fb != fa { self.carry = true; } }, //Not really documented, but based on the code, this only sets carry, doesn't reset it.
              31 => { trace!("XCHG FB{i}, FA{i}"); (fb, fa) = (fa, fb); },
              _ => trace!("NOP"), //16 | 30
            }
          }
          self.fa.shift_with_bit(direction, fa);
          self.fb.shift_with_bit(direction, fb);
        }
      },
      _ => {  //3 - Register Instruction
        if let Some(carry) = self.alu.run_cycle(word_select, instruction, mask) {
          if carry {  //Not really documented, but based on the code, this only sets carry, doesn't reset it.
            self.carry = true;
          }
        }
        if instruction.value() == 0x1A {  //AKCN instruction needs to keep repeating as it tallies up the key code.
          if self.d.read_parallel() | 0b1 != self.current_keypress {  //Keep trying until we are done reading all of d.
            self.pc -= u9::new(1);
          }
        }
      }
    }
    self.d.shift_with_bit(shifter::Direction::Right, d);
    
    if class==2 && matches!(instruction.value(), 17 | 18) { //Don't debug while waiting.
    } else {
      debug!("PC: {:03X} FA: {:011b} FB: {:011b} D:{:010b} key:{:011b} Carry: {}", self.pc, self.fa.read_parallel(), self.fb.read_parallel(), self.d.read_parallel(), self.current_keypress, self.carry);
    }
  }
}