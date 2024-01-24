//! The 4004 was produced in 1971, containing 2300 transistors, with a 10 μm process node.
//!
//! Each clock cycle was 1.35 microseconds (740 kHz). Each instruction took 8 - 16 clock cycles. So, instructions took 10.8 - 21.6 microseconds (46.3 - 92.6 kHz).
//!
//! It was the first commercially produced microprocessor.
//!
//! Primarily produced for the Busicom calculator.
//!
//! Useful links:
//! * <https://pyntel4004.readthedocs.io/en/latest/intro/opcodes.html>
//! * <http://e4004.szyc.org/>

use arbitrary_int::u4;
use log::{trace,debug};
use crate::Indexer64;
use super::{ControlLines, Address, Byte};

#[derive(Default)]
enum ContinueFrom {
  #[default]
  StartOver,
  JumpConditional,
  CallFar,
  JumpFar,
  SetReg,
  SetIndirectReg,
}

/// Intel 4004 chip
#[derive(Default)]
pub struct CPU {
  continue_from: ContinueFrom,
  previous_modifier: u4,
  
  control_output: ControlLines,
  
  /// Program Counter
  pc: Address,
  
  stack: [Address; 4],
  effective_address: usize,
  push_count: u8, //Undocumented feature
  
  /// Currently processing opcode
  opcode: Byte,
  
  carry: bool,
  test: bool,
  acc: u4,
  
  regs: Indexer64,
}

impl CPU {
  
  pub(super) fn new() -> Self {
    Self {
      control_output: ControlLines::DEFAULT.with_rom(true).with_ram(0, true),
      ..Default::default()
    }
  }
  
  /// Print debug data of all registers
  pub(super) fn print(&self) {
    if matches!(self.continue_from, ContinueFrom::StartOver) {
      debug!("R0: {:X} R1: {:X} R2: {:X} R3: {:X} R4: {:X} R5: {:X} R6: {:X} R7: {:X} R8: {:X} R9: {:X} RA: {:X} RB: {:X} RC: {:X} RD: {:X} RE: {:X} RF: {:X}", self.regs.read_nibble(0).value(), self.regs.read_nibble(1).value(), self.regs.read_nibble(2).value(), self.regs.read_nibble(3).value(), self.regs.read_nibble(4).value(), self.regs.read_nibble(5).value(), self.regs.read_nibble(6).value(), self.regs.read_nibble(7).value(), self.regs.read_nibble(8).value(), self.regs.read_nibble(9).value(), self.regs.read_nibble(10).value(), self.regs.read_nibble(11).value(), self.regs.read_nibble(12).value(), self.regs.read_nibble(13).value(), self.regs.read_nibble(14).value(), self.regs.read_nibble(15).value());
      debug!("PC: {:02X} Acc: {:X} Carry: {} Test: {}", self.pc.raw_value(), self.acc.value(), self.carry, self.test);
    }
  }

  /// Set test flag. This is the only way chips could signal the 4004 directly.
  pub fn set_test_flag(&mut self, test: bool) {
    self.test = test;
  }

  fn set_acc_carry(&mut self, val: u8) {
    self.carry = val > 0xF;
    self.acc = u4::new(val & 0xF);
  }
  
  /// A1, A2, A3 clock - Send address
  pub(super) fn get_addr(&self) -> Address {
    if let ContinueFrom::SetIndirectReg = self.continue_from {
      //Special case for FIN command
      self.pc.with_high(self.regs.read_nibble(0))
             .with_low(self.regs.read_nibble(1))
    } else {
      self.pc
    }
  }

  /// M1 and M2 clock - Receive opcode. Returns the control lines being on or off, depending if there is an io instruction coming up.
  pub(super) fn set_opcode(&mut self, opcode: Byte) -> ControlLines {
    self.opcode = opcode;
    if matches!(self.continue_from, ContinueFrom::StartOver) && self.opcode.high().value() == 0xE {
      self.control_output
    } else {
      Default::default()
    }
  }

  /// X1 and X2 clock - Execute. It can read from memory, or write to memory
  pub(super) fn run_cycle(&mut self, data_in: u4) -> super::ExecuteOut {
    let mut data_out = super::ExecuteOut::Nothing;
    
    //FIN and JIN require pc to not change. Everyone else should increment. Including BBL!
    if !(self.opcode.high().value() == 3 && matches!(self.continue_from, ContinueFrom::StartOver)) {
      self.pc = Address::new_with_raw_value((self.pc.raw_value() + 1) & 0xFFF);  //If we reach 0xFFF, we wrap back around
    }
    
    match self.continue_from {
      ContinueFrom::StartOver => {
        let modifier = self.opcode.low().value();
        self.previous_modifier = self.opcode.low();
        match self.opcode.high().value() {
          0x1 => { trace!("JCN");  //Jump CoNditional
            self.continue_from = ContinueFrom::JumpConditional;
          },
          0x2 => {
            if matches!(modifier & 1, 0) {
              trace!("FIM");  //Fetch IMmediate
              self.continue_from = ContinueFrom::SetReg;
            } else {
              trace!("SRC R{:X} R{:X}", modifier & 0b1110, modifier);  //Send Register Control. Handled by X2 and X3
              //modifier is guaranteed to be odd, so doing & 0b1111 will always subtract 1. I expect this was a trick done by the original circuit.
              data_out = super::ExecuteOut::SRC(super::Byte::builder()
                                          .with_high(self.regs.read_nibble(modifier & 0b1110))  //Contains the Chip index, and the Register Index
                                          .with_low(self.regs.read_nibble(modifier))  //Contains the character index
                                          .build());
            }
          },
          0x3 => {
            if matches!(modifier & 1, 0) {  trace!("FIN");  //Fetch INdirect
              self.continue_from = ContinueFrom::SetIndirectReg;
            } else {  trace!("JIN to R{:X} R{:X}", modifier & 0b1110, modifier);  //Jump INdirect
              //modifier is guaranteed to be odd, so doing & 0b1111 will always subtract 1. I expect this was a trick done by the original circuit.
              let high = self.regs.read_nibble(modifier & 0b1110);
              let low = self.regs.read_nibble(modifier);
              self.pc = self.pc.with_high(high)
                               .with_low(low);
            }
          },
          0x4 => { trace!("JUN");  //Jump UNconditional
            self.continue_from = ContinueFrom::JumpFar;
          },
          0x5 => { trace!("JMS"); //JuMp to Subroutine  (Call function. Push current position to stack.)
            self.continue_from = ContinueFrom::CallFar;
          },
          0x6 => {
            trace!("INC R{:X}", modifier);  //INCrement
            let val = self.regs.read_nibble(modifier).value();
            self.regs.write_nibble(modifier, u4::new((val + 1) & 0xF));
            //No flags are set.
          },
          0x7 => {
            trace!("ISZ R{:X}", modifier);  //Increment index register, Skip if Zero (Loop until wrapped to 0)
            match self.regs.read_nibble(modifier).value() {
              0xF => {
                self.regs.write_nibble(modifier, u4::new(0));
                self.previous_modifier = u4::new(0b0000); //False condition. Don't Jump
              },
              val => {
                self.regs.write_nibble(modifier, u4::new(val + 1));
                self.previous_modifier = u4::new(0b1000); //True condition. Jump
              }
            }
            self.continue_from = ContinueFrom::JumpConditional;
          },
          0x8 => { trace!("Acc = Acc + R{:X} + Carry", modifier);  //ADD R
            self.set_acc_carry(self.acc.value() + self.regs.read_nibble(modifier).value() + self.carry as u8);
          },
          0x9 => { trace!("Acc = Acc - R{:X} - Carry", modifier);  //SUB R
            self.set_acc_carry(self.acc.value() + (!self.regs.read_nibble(modifier)).value() + (!self.carry) as u8);
          },
          0xA => { trace!("Acc = R{:X}", modifier); //LD R
            self.acc = self.regs.read_nibble(modifier);
          },
          0xB => { trace!("Exchange Acc, R{:X}", modifier); //XCH
            let nibble = self.regs.read_nibble(modifier);
            self.regs.write_nibble(modifier, self.acc);
            self.acc = nibble;
          },
          0xC => { trace!("BBL Acc={:X}", self.opcode.low()); //Branch Back and Load (Return from function)
            self.acc = self.opcode.low();
            if self.push_count > 0 {  //Undocumented feature...
              self.push_count -= 1;
              self.effective_address = ((self.effective_address as i8 - 1) & 0x3) as usize;  //Loops back around
              self.pc = self.stack[self.effective_address]; //Reset to address from stack
              trace!("PC:{:X}", self.pc.raw_value());
            }
          },
          0xD => { trace!("Acc = {:X}", self.opcode.low());  //LDM - LoaD iMmediate to accumulator
            self.acc = self.opcode.low();
          },
          0xE => match modifier {
            0x8 => { trace!("Acc = Acc - RAM Character - Carry");  //SBM - SuBtract data raM
              self.set_acc_carry(self.acc.value() + (!data_in).value() + (!self.carry) as u8);
            },
            0x9 | 0xA | 0xC..=0xF => { trace!("Acc = Memory data or IO");  //RDM - ReaD raM character to acc, or RDR - ReaD Rom port to acc, or RD{} - Read Status to Acc
              self.acc = data_in;
            },
            0xB => { trace!("Acc = Acc + RAM Character + Carry");  //ADM - ADd data raM
              self.set_acc_carry(self.acc.value() + data_in.value() + self.carry as u8);
            }
            _ => {  //Writing to RAM or ROM.
              data_out = super::ExecuteOut::Write(Byte::builder()
                                            .with_high(self.opcode.low()) //modifier = io command
                                            .with_low(self.acc)   //value
                                            .build());
            },
          }
          0xF => match modifier {
            0x0 => { trace!("Acc = 0, Carry = 0"); //CLB - CLear Both
              self.acc = u4::new(0);
              self.carry = false;
            },
            0x1 => { trace!("Carry = 0"); //CLC - CLear Carry
              self.carry = false;
            },
            0x2 => { trace!("Acc += 1"); //IAC - Increment ACcumulator
              self.set_acc_carry(self.acc.value() + 1);
            },
            0x3 => { trace!("Carry = !Carry"); //CMC - CoMplement Carry
              self.carry = !self.carry;
            },
            0x4 => { trace!("Acc = !Acc"); //CMA - CoMplement Accumulator
              self.acc = !self.acc;
            },
            0x5 => { trace!("RAL Acc"); //RotAte Left
              let new_carry = self.acc.value() & 8 == 8;
              self.acc <<= 1;
              if self.carry {
                self.acc |= u4::new(1);
              }
              self.carry = new_carry;
            },
            0x6 => { trace!("RAR Acc"); //RotAte Right
              let new_carry = self.acc.value() & 1 == 1;
              self.acc >>= 1;
              if self.carry {
                self.acc |= u4::new(8);
              }
              self.carry = new_carry;
            },
            0x7 => { trace!("Acc = Carry"); //TCC - Transmit Carry and Clear
              self.acc = u4::new(self.carry as u8);
              self.carry = false;
            },
            0x8 => { trace!("Acc -= 1");  //DAC - Decrement Accumulator
              if self.acc.value() == 0 {
                self.acc = u4::new(0xF);
                self.carry = false; //Carry is reversed
              } else {
                self.acc -= u4::new(1);
                self.carry = true;  //Carry is reversed
              }
            },
            0x9 => {  //TCS - Transfer Carry Subtract. Used for BCD stuff
              self.acc = u4::new(if self.carry { 10 } else { 9 });
              trace!("Acc = {}", self.acc);
              self.carry = false;
            },
            0xA => { trace!("Carry = true"); //STC - Set Carry
              self.carry = true;
            },
            0xB => { trace!("DAA"); //Decimal Adjust Accumulator
              let mut val = self.acc.value();
              if val > 9 || self.carry {
                val += 6;
                self.acc = u4::new(val & 0xF);
                if val > 0xF {
                  self.carry = true;  //Important note here that carry does NOT set to false if val is small.
                }
              }
            },
            0xC => { trace!("KBP"); //Keyboard Process - Convert single accumulator bit location into a number.
              self.acc = u4::new(match self.acc.value() {
                0b0000 => 0,
                0b0001 => 1,
                0b0010 => 2,
                0b0100 => 3,
                0b1000 => 4,
                _ => 0xF,
              });
            },
            0xD => { trace!("DCL"); //DCL - Designate Command Line - Set RAM Bank
              /*if self.acc.value() & 0b1000 == 0b1000 {
                panic!("Invalid command control value for DCL: {:b}", self.acc);
              }*/
              self.control_output = if self.acc.value() == 0 {
                ControlLines::DEFAULT.with_rom(true)
                  .with_ram(0, true)
              } else {
                ControlLines::DEFAULT.with_rom(true)
                  .with_ram(1, self.acc.value() & 0b1 == 0b1)
                  .with_ram(2, self.acc.value() & 0b10 == 0b10)
                  .with_ram(3, self.acc.value() & 0b100 == 0b100)
              };
            },
            0xE => { trace!("Invalid code 0xFE run by exerciser. Does nothing."); },
            _ => unreachable!("Unknown opcode: F{:X}", modifier),
          },
          _ => trace!("NOP"), //0x0
        }

      },
      ContinueFrom::JumpConditional => {
        let modifier = self.previous_modifier.value();
        let c1 = matches!(modifier & 0x8, 0x8); //Inverse
        let c2 = matches!(modifier & 0x4, 0x4); //Acc
        let c3 = matches!(modifier & 0x2, 0x2); //Carry
        let c4 = matches!(modifier & 0x1, 0x1); //Test
        trace!("Jump to {:X}{:X} Inverse:{} Acc:{} Carry:{} Test:{}", self.opcode.high(), self.opcode.low(), c1, c2, c3, c4);
        let partial_cond = c4 & self.test || c3 & self.carry || c2 & (self.acc == u4::new(0));
        if (!c1 & partial_cond) || (c1 & !partial_cond) {
          self.pc = self.pc.with_high(self.opcode.high())
                           .with_low(self.opcode.low());
        }
        self.continue_from = ContinueFrom::StartOver;
      },
      ContinueFrom::CallFar => {
        trace!("Call to {:X}{:X}{:X}", self.previous_modifier, self.opcode.high(), self.opcode.low());
        self.stack[self.effective_address] = self.pc;
        self.effective_address = (self.effective_address + 1) & 0x3;  //Loops around
        self.push_count += 1;
        self.pc = Address::builder()
                        .with_chip_index(self.previous_modifier)
                        .with_high(self.opcode.high())
                        .with_low(self.opcode.low())
                        .build();
        self.continue_from = ContinueFrom::StartOver;
      },
      ContinueFrom::JumpFar => {
        trace!("Jump to {:X}{:X}{:X}", self.previous_modifier, self.opcode.high(), self.opcode.low());
        self.pc = Address::builder()
                        .with_chip_index(self.previous_modifier)
                        .with_high(self.opcode.high())
                        .with_low(self.opcode.low())
                        .build();
        self.continue_from = ContinueFrom::StartOver;
      },
      ContinueFrom::SetReg | ContinueFrom::SetIndirectReg => {  //Fetch Immediate | Fetch Indirect
        let index = self.previous_modifier.value();
        //index is guaranteed to be a multiple of 2, so doing | 1 will always add 1. I expect this was a trick done by the original circuit.
        trace!("R{:X}={:X}, R{:X}={:X}", index, self.opcode.high(), index | 1, self.opcode.low());
        self.regs.write_nibble(index, self.opcode.high());
        self.regs.write_nibble(index | 1, self.opcode.low());
        self.continue_from = ContinueFrom::StartOver;
      },
    }
    data_out
  }

}