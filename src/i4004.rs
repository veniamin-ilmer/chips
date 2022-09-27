//!The 4004 was produced in 1971, containing 2300 transistors.
//!It ran at 740 kHz.
//!It was the first commercially produce microprocessor.
//!Primarily produced for the Busicom calculator.

use bitbybit::bitfield;
use arbitrary_int::{u2,u4};

use log::{trace,debug};

pub trait IO {
  fn read_rom_byte(&self, address: ROMAddress) -> u8;
  fn read_rom_ports(&self, designated_index: DesignatedIndex) -> u4;
  fn read_ram_character(&self, command_control: u4, designated_index: DesignatedIndex) -> u4;
  fn write_ram_character(&mut self, command_control: u4, designated_index: DesignatedIndex, value: u4);
  fn read_ram_status(&self, command_control: u4, designated_index: DesignatedIndex, status_index: u2) -> u4;
  fn write_ram_status(&mut self, command_control: u4, designated_index: DesignatedIndex, status_index: u2, value: u4);
  fn write_ram_ports(&mut self, command_control: u4, designated_index: DesignatedIndex, value: u4);
  fn write_rom_ports(&mut self, designated_index: DesignatedIndex, value: u4);
}

#[bitfield(u8, default: 0)]
#[derive(Default)]
pub struct DesignatedIndex {
  #[bits(6..=7, rw)]
  chip_index: u2,

  #[bits(4..=5, rw)]
  reg_index: u2,

  #[bits(0..=3, rw)]
  char_index: u4,
}

#[bitfield(u16, default: 0)]
#[derive(Default)]
pub struct ROMAddress {
  #[bits(12..=15, rw)]
  _reserved: u4,

  #[bits(8..=11, rw)]
  chip_index: u4,

  #[bits(0..=7, rw)]
  offset: u8,
}

struct PairOp {
  high: usize,
  low: usize,
}

#[derive(Default)]
pub struct I4004 {
  regs: [u4; 16],
  carry: bool,
  test: bool,
  acc: u4,

  designated_index: DesignatedIndex,
  command_control: u4,
  
  stack: [ROMAddress; 4],
  effective_address: usize,
  push_count: usize,
  pc: ROMAddress, //Program Counter
}
impl I4004 {
  pub fn new() -> Self {
    Default::default()
  }

  fn get_reg(&mut self, opcode: u8) -> usize {
    let index = opcode & 0xF;
    index as usize
  }

  fn get_reg_pair(&mut self, opcode: u8) -> PairOp {
    let index = ((opcode >> 1) & 0b111) as usize;
    PairOp {
      high: index * 2,
      low: index * 2 + 1,
    }
  }
  
  fn jump(&mut self, low_byte: u8, high_nibble: u4) {
    self.pc = ROMAddress::new()
              .with_offset(low_byte)
              .with_chip_index(high_nibble);
  }
  
  fn jump_near(&mut self, low_byte: u8) {
    self.pc = self.pc.with_offset(low_byte);
  }
  
  pub fn print(&self) {
    debug!("R0: {:X} R1: {:X} R2: {:X} R3: {:X} R4: {:X} R5: {:X} R6: {:X} R7: {:X}", self.regs[0].value(), self.regs[1].value(), self.regs[2].value(), self.regs[3].value(), self.regs[4].value(), self.regs[5].value(), self.regs[6].value(), self.regs[7].value());
    debug!("R8: {:X} R9: {:X} RA: {:X} RB: {:X} RC: {:X} RD: {:X} RE: {:X} RF: {:X}", self.regs[8].value(), self.regs[9].value(), self.regs[10].value(), self.regs[11].value(), self.regs[12].value(), self.regs[13].value(), self.regs[14].value(), self.regs[15].value());
    debug!("PC: {:X} Acc: {:X} Carry: {} Test: {} Designated Index: {} {} {}", self.pc.raw_value(), self.acc.value(), self.carry, self.test, self.designated_index.chip_index(), self.designated_index.reg_index(), self.designated_index.char_index());
  }
  
  pub fn get_pc(&self) -> u16 {
    self.pc.raw_value()
  }
  
  fn set_acc_carry(&mut self, val: u8) {
    self.carry = val > 0xF;
    self.acc = u4::new(val & 0xF);
  }
  
  pub fn set_test_flag(&mut self, test: bool) {
    self.test = test;
  }

  fn next_rom_byte(&mut self, io: &mut impl IO) -> u8 {
    let ret = io.read_rom_byte(self.pc);
    self.pc = ROMAddress::new_with_raw_value((self.pc.raw_value() + 1) & 0xFFF);  //If we reach 0xFFF, we wrap back around
    ret
  }

  pub fn run_cycle(&mut self, io: &mut impl IO) {
    let opcode = self.next_rom_byte(io);
    match opcode {
      0 => trace!("NOP"),
      0x10..=0x1f => {
        let opcode_low = self.next_rom_byte(io);
        let c1 = matches!(opcode & 0x8, 0x8); //Inverse
        let c2 = matches!(opcode & 0x4, 0x4); //Acc
        let c3 = matches!(opcode & 0x2, 0x2); //Carry
        let c4 = matches!(opcode & 0x1, 0x1); //Test
        trace!("JCN - to {:X} - Conditional Jump Inverse:{} Acc:{} Carry:{} Test:{}", opcode_low, c1, c2, c3, c4);
        let condition = c4 & self.test || c3 & self.carry || c2 & (self.acc == u4::new(0));
        if (!c1 & condition) || (c1 & !condition) {
          self.jump_near(opcode_low);
        }
      },
      0x20..=0x2f => {
        let pair = self.get_reg_pair(opcode);
        if opcode & 1 == 0 {  //FIM
          let opcode_low = self.next_rom_byte(io);
          self.regs[pair.low] = u4::new(opcode_low & 0xf);
          self.regs[pair.high] = u4::new(opcode_low >> 4);
          trace!("FIM R{:X} = {:X}, R{:X} = {:X} - Fetch Immediate", pair.high, self.regs[pair.high].value(), pair.low, self.regs[pair.low].value());
        } else { //SRC
          trace!("SRC R{:X} R{:X} - Send Register Control - set designated index", pair.low, pair.high);
          self.designated_index = DesignatedIndex::new()
                                   .with_char_index(self.regs[pair.low])
                                   .with_reg_index(u2::new(self.regs[pair.high].value() & 0b11))
                                   .with_chip_index(u2::new(self.regs[pair.high].value() >> 2));
        }
      },
      0x30..=0x3f => {
        if opcode & 1 == 0 { //FIN
          let addr = self.regs[1].value() | (self.regs[0].value() << 4);
          let full_addr = ROMAddress::new().with_chip_index(self.pc.chip_index())
                                           .with_offset(addr);
          let opcode_low = io.read_rom_byte(full_addr);
          let pair = self.get_reg_pair(opcode);
          self.regs[pair.low] = u4::new(opcode_low & 0xf);
          self.regs[pair.high] = u4::new(opcode_low >> 4);
          trace!("FIN R{:X} R{:X} - Fetch Indirect", pair.low, pair.high);
        } else {  //JIN
          let pair = self.get_reg_pair(opcode);
          trace!("JIN R{:X} R{:X} - Jump Indirect", pair.low, pair.high);
          self.jump_near(self.regs[pair.low].value() | self.regs[pair.high].value() << 4);
        }
      },
      0x40..=0x4f => { trace!("JUN - Unconditional Jump");
        let high_nibble = u4::new(opcode & 0xF);
        let opcode_low = self.next_rom_byte(io);
        self.jump(opcode_low, high_nibble);
      },
      0x50..=0x5f => { trace!("JMS - Jump to Subroutine (Call function. Push current position to stack.)");
        let high_nibble = u4::new(opcode & 0xF);
        let opcode_low = self.next_rom_byte(io);
        self.stack[self.effective_address] = self.pc;
        self.effective_address = (self.effective_address + 1) & 0x3;  //Loops around
        self.push_count += 1;
        self.jump(opcode_low, high_nibble);
      },
      0x60..=0x6f => {
        let reg_op = self.get_reg(opcode);
        trace!("INC R{:X} - Increment", reg_op);
        let val = self.regs[reg_op].value();
        self.regs[reg_op] = u4::new((val + 1) & 0xF);
        //No flags are set.
      },
      0x70..=0x7f => {
        let reg_op = self.get_reg(opcode);
        let opcode_low = self.next_rom_byte(io);
        trace!("ISZ R{:X} - Jump to {:X} - Increment and Skip (Loop until wrapped to 0)", reg_op, opcode_low);
        let val = self.regs[reg_op].value();
        match val {
          0xF => self.regs[reg_op] = u4::new(0),
          _ => {
            self.regs[reg_op] = u4::new(val + 1);
            self.jump_near(opcode_low);
          },
        };
      },
      0x80..=0x8f => {
        let reg_op = self.get_reg(opcode);
        trace!("Acc + ADD R{:X} + Carry - Set Accumulator to value of register", reg_op);
        //First add the carry..
        self.set_acc_carry(self.acc.value() + self.regs[reg_op].value() + self.carry as u8);
      },
      0x90..=0x9f => {
        let reg_op = self.get_reg(opcode);
        trace!("SUB Acc - R{:X} - Carry - Subtract index register from accumulator with borrow", reg_op);
        self.set_acc_carry(self.acc.value() + (!self.regs[reg_op]).value() + (!self.carry) as u8);
      },
      0xa0..=0xaf => {
        let reg_op = self.get_reg(opcode);
        trace!("LD Acc, R{:X} - Set Accumulator to value of register", reg_op);
        self.acc = self.regs[reg_op];
      },
      0xb0..=0xbf => {
        let reg_op = self.get_reg(opcode);
        trace!("XCH R{:X} - Exchange register with accumulator", reg_op);
        let temp = self.acc;
        self.acc = self.regs[reg_op];
        self.regs[reg_op] = temp;
      },
      0xc0..=0xcf => {
        self.acc = u4::new(opcode & 0xF);
        trace!("BBL Acc={:X} - Branch Back and Load (Return from function)", self.acc.value());
        if self.push_count > 0 {  //Undocumented feature...
          self.push_count -= 1;
          self.effective_address = ((self.effective_address as i8 - 1) & 0x3) as usize;  //Loops back around
          self.pc = self.stack[self.effective_address]; //Reset to address from stack
        }
      },
      0xd0..=0xdf => {
        self.acc = u4::new(opcode & 0xF);
        trace!("LDM {:X} - Load immediate to accumulator", self.acc.value());
      },
      0xe0 => { trace!("WRM - Write Acc to RAM character");
        io.write_ram_character(self.command_control, self.designated_index, self.acc);
      },
      0xe1 => { trace!("WMP- Write RAM Port");
        io.write_ram_ports(self.command_control, self.designated_index, self.acc);
      },
      0xe2 => { trace!("WRR- Write ROM Port");
        io.write_rom_ports(self.designated_index, self.acc);
      },
      //0xe3 => Write Program RAM
      0xe4..=0xe7 => {
        let status_index = u2::new(opcode & 0b11);
        trace!("WR{} - Write Acc to RAM status", status_index);
        io.write_ram_status(self.command_control, self.designated_index, status_index, self.acc);
      },
      0xe8 => { trace!("SBM - Subtract DATA RAM - ACC + !RAM Character + !Carry");
        let val = io.read_ram_character(self.command_control, self.designated_index);
        self.set_acc_carry(self.acc.value() + (!val).value() + (!self.carry) as u8);
      },
      0xe9 => { trace!("RDM - Read RAM character to Acc");
        self.acc = io.read_ram_character(self.command_control, self.designated_index);
      },
      0xea => { trace!("RDR - Read ROM Port to Acc");
        self.acc = io.read_rom_ports(self.designated_index);
      },
      0xeb => { trace!("ADM - Add DATA RAM - ACC + !RAM Character + !Carry");
        let val = io.read_ram_character(self.command_control, self.designated_index);
        self.set_acc_carry(self.acc.value() + val.value() + (self.carry) as u8);
      },
      0xec..=0xef => {
        let status_index = u2::new(opcode & 0b11);
        trace!("RD{} - Read Status to Acc", status_index.value());
        self.acc = io.read_ram_status(self.command_control, self.designated_index, status_index);
      },
      0xf0 => {
        trace!("CLB - Clear Accumulator and Carry");
        self.acc = u4::new(0);
        self.carry = false;
      },
      0xf1 => { trace!("CLC - Clear Carry");
        self.carry = false;
      }
      0xf2 => { trace!("IAC - Increment Accumulator");
        self.set_acc_carry(self.acc.value() + 1);
      },
      0xf3 => { trace!("CMC - Complement Carry");
        self.carry = !self.carry;
      }
      0xf4 => { trace!("CMA - Complement Accumulator");
        self.acc = !self.acc;
      }
      0xf5 => { trace!("RAL (Acc) - Rotate Left");
        let new_carry = matches!(self.acc.value() & 8, 8);
        self.acc <<= 1;
        if self.carry {
          self.acc |= u4::new(1);
        }
        self.carry = new_carry;
      },
      0xf6 => { trace!("RAR (Acc) - Rotate Right");
        let new_carry = matches!(self.acc.value() & 1, 1);
        self.acc >>= 1;
        if self.carry {
          self.acc |= u4::new(8);
        }
        self.carry = new_carry;
      },
      0xf7 => { trace!("TCC - Transmit Carry and Clear");
        self.acc = u4::new(self.carry as u8);
        self.carry = false;
      },
      0xf8 => { trace!("DAC - Decrement Accumulator");
        if self.acc.value() == 0 {
          self.acc = u4::new(0xF);
          self.carry = false; //Carry is reversed
        } else {
          self.acc -= u4::new(1);
          self.carry = true;  //Carry is reversed
        }
      },
      0xf9 => { trace!("TCS - Transfer Carry Subtract");
        if self.carry {
          self.acc = u4::new(10);
        } else {
          self.acc = u4::new(9);
        }
        self.carry = false;
      },
      0xfa => { trace!("STC - Set Carry");
        self.carry = true;
      }
      0xfb => { trace!("DAA - Decimal Adjust Accumulator");
        let mut val = self.acc.value();
        if val > 9 || self.carry {
          val += 6;
          self.acc = u4::new(val & 0xF);
          if val > 0xF {
            self.carry = true;  //Important note here that carry does NOT set to false if val is small.
          }
        }
      },
      0xfc => { trace!("KBP - Keyboard Process - Convert single accumulator bit location into a number.");
        self.acc = u4::new(match self.acc.value() {
          0b0000 => 0,
          0b0001 => 1,
          0b0010 => 2,
          0b0100 => 3,
          0b1000 => 4,
          _ => 0xF,
        });
      },
      0xfd => { trace!("DCL - Designate Command Line - Set RAM Bank");
        self.command_control = self.acc;
      },
      0xfe => { trace!("Invalid code 0xFE run by exerciser. Does nothing."); }
      _ => unreachable!("Unknown opcode: {:X}", opcode),
    }
  }

}