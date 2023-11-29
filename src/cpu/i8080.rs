//! The 8080 was produced in April 1974, containing 4500 transistors.
//!
//! Each clock cycle was 0.5 microseconds (2 MHz). Each instruction took 4 - 18 clock cycles. So, instructions took 2 - 9 microseconds (111 - 500 kHz).
//!
//! Useful links
//! * <http://www.piclist.com/techref/intel/8080.htm>
//! * <https://www.pastraiser.com/cpu/i8080/i8080_opcodes.html>

use bitbybit::bitfield;
use log::{trace,debug};
use arbitrary_int::{u2,u3};
use crate::cpu;

/// Used to communicate with board
pub trait IO: crate::cpu::MemoryIO<u16> {
  /// Write to IO port
  fn output(&mut self, port: u8, value: u8);
  /// Read from IO port
  fn input(&mut self, port: u8) -> u8;
}

/// Program Status Word
#[bitfield(u16, default: 0)]
struct PSW {
  /// Accumulator
  #[bits(8..=15, rw)]
  acc: u8,
  
  /// Sign: Set if result is negative
  #[bit(7, rw)]
  sign: bool,

  /// Zero: Set if result is zero
  #[bit(6, rw)]
  zero: bool,

  /// Auxiliary Carry: Used for Binary Coded Decimal (DCD) arithmetic
  #[bit(4, rw)]
  aux: bool,

  /// Parity: Set if the number of 1 bits in the result is even.
  #[bit(2, rw)]
  parity: bool,

  /// Carry: Set if the last addition operation resulted in a carry or if the last subtraction operation required a borrow
  #[bit(0, rw)]
  carry: bool,
}

#[bitfield(u16, default: 0)]
struct Word {
  #[bits(8..=15, rw)]
  high: u8,
  
  #[bits(0..=7, rw)]
  low: u8,
}

/// Main Registers
#[derive(Default)]
struct Registers {
  /// Program Status Word: Accumulator, Flags
  psw: PSW,
  /// B, C bytes
  bc: Word,
  /// D, E bytes
  de: Word,
  /// H, L bytes
  hl: Word,
}

/// Intel 8080 chip
#[derive(Default)]
pub struct I8080 {
  /// stack and counter registers. Public for debugging purposes
  pub cpu: cpu::CPU<u16>,
  /// Main Registers.
  regs: Registers,
  /// Interrupts Enabled
  interrupts_enabled: bool,
}

impl I8080 {
  /// Create a new chip
  pub fn new() -> Self {
    Self {
      cpu: cpu::CPU::<u16> {
        sp: 0xFFFF,
        pc: 0,
      },
      regs: Default::default(),
      interrupts_enabled: false,
    }
  }
  
  fn debug_reg(&self, reg_index: u3) -> &str {
    match reg_index.value() {
      0 => "B", 1 => "C", 2 => "D", 3 => "E",
      4 => "H", 5 => "L", 6 => "[HL]", _ => "A",
    }
  }
  fn read_reg(&self, io: &mut impl IO, reg_index: u3) -> u8 {
    match reg_index.value() {
      0 => self.regs.bc.high(), //B
      1 => self.regs.bc.low(),  //C
      2 => self.regs.de.high(), //D
      3 => self.regs.de.low(),  //E
      4 => self.regs.hl.high(), //H
      5 => self.regs.hl.low(),  //L
      6 => io.read_mem(self.regs.hl.raw_value()), //[HL]
      _ => self.regs.psw.acc(), //A
    }
  }
  fn write_reg(&mut self, io: &mut impl IO, reg_index: u3, value: u8) {
    match reg_index.value() {
      0 => self.regs.bc = self.regs.bc.with_high(value), //B
      1 => self.regs.bc = self.regs.bc.with_low(value),  //C
      2 => self.regs.de = self.regs.de.with_high(value), //D
      3 => self.regs.de = self.regs.de.with_low(value),  //E
      4 => self.regs.hl = self.regs.hl.with_high(value), //H
      5 => self.regs.hl = self.regs.hl.with_low(value),  //L
      6 => io.write_mem(self.regs.hl.raw_value(), value),  //[HL]
      _ => self.regs.psw = self.regs.psw.with_acc(value), //A
    };
  }

  fn debug_pair(&self, pair_index: u2) -> &str {
    match pair_index.value() {
      0 => "BC", 1 => "CD", 2 => "HL", _ => "SP",
    }
  }
  fn read_pair(&mut self, pair_index: u2) -> u16 {
    match pair_index.value() {
      0 => self.regs.bc.raw_value(),
      1 => self.regs.de.raw_value(),
      2 => self.regs.hl.raw_value(),
      _ => self.cpu.sp,
    }
  }
  fn write_pair(&mut self, pair_index: u2, value: u16) {
    match pair_index.value() {
      0 => self.regs.bc = Word::new_with_raw_value(value),
      1 => self.regs.de = Word::new_with_raw_value(value),
      2 => self.regs.hl = Word::new_with_raw_value(value),
      _ => self.cpu.sp = value,
    };
  }


  fn debug_pair_push_pop(&self, pair_index: u2) -> &str {
    match pair_index.value() {
      0 => "BC", 1 => "CD", 2 => "HL", _ => "PSW",
    }
  }
  fn read_pair_push(&mut self, pair_index: u2) -> u16 {
    match pair_index.value() {
      0 => self.regs.bc.raw_value(),
      1 => self.regs.de.raw_value(),
      2 => self.regs.hl.raw_value(),
      _ => self.regs.psw.raw_value(),
    }
  }
  fn write_pair_pop(&mut self, pair_index: u2, value: u16) {
    match pair_index.value() {
      0 => self.regs.bc = Word::new_with_raw_value(value),
      1 => self.regs.de = Word::new_with_raw_value(value),
      2 => self.regs.hl = Word::new_with_raw_value(value),
      _ => self.regs.psw = PSW::new_with_raw_value(value),
    };
  }
  
  fn debug_condition(&self, condition: u3) -> &str {
    match condition.value() {
      0 => "NZ",  //Not Zero
      1 => "Z",   //Zero
      2 => "NC",  //Not Carry
      3 => "C",   //Carry
      4 => "PO",  //Odd
      5 => "PE",  //Even
      6 => "P",   //Plus
      _ => "M",   //Minus
    }
  }
  
  fn test_condition(&self, condition: u3) -> bool {
    match condition.value() {
      0 => !self.regs.psw.zero(),
      1 => self.regs.psw.zero(),
      2 => !self.regs.psw.carry(),
      3 => self.regs.psw.carry(),
      4 => !self.regs.psw.parity(),
      5 => self.regs.psw.parity(),
      6 => !self.regs.psw.sign(),
      _ => self.regs.psw.sign(),
    }
  }

  /// Print debug data of all registers
  pub fn print(&self) {
    debug!("A: 0x{:02X} B: 0x{:02X} C: 0x{:02X} D: 0x{:02X} E: 0x{:02X} H: 0x{:02X} L: 0x{:02X} SP: 0x{:04X}", self.regs.psw.acc(), self.regs.bc.high(), self.regs.bc.low(), self.regs.de.high(), self.regs.de.low(), self.regs.hl.high(), self.regs.hl.low(), self.cpu.sp);
    debug!("PC: 0x{:04X} Zero: {} Carry: {} Aux: {} Parity: {} Sign: {}", self.cpu.pc, self.regs.psw.zero(), self.regs.psw.carry(), self.regs.psw.aux(), self.regs.psw.parity(), self.regs.psw.sign());
  }

  /// Executes single instruction:
  ///
  /// 1. Read the next byte from ROM
  /// 2. Check top 2 bits to divide up decoding.
  /// 3. Some decoders might swap the two nibbles for easier decoding.
  /// 4. If multiple byte instruction found, read more bytes from ROM.
  /// 5. Execute instruction.
  pub fn run_cycle(&mut self, io: &mut impl IO) {
    let opcode = self.cpu.next_code_byte(io);
    //Looking at the opcode map, it makes sense to chop up the instruction set into 4 chunks..
    match opcode >> 6 {
      0 => self.decode0(io, opcode),
      1 => {  //MOV
        let from_index = u3::new(opcode & 0b111);
        let to_index = u3::new((opcode >> 3) & 0b111);
        trace!("Copy {} into {}", self.debug_reg(from_index), self.debug_reg(to_index));
        let value = self.read_reg(io, from_index);
        self.write_reg(io, to_index, value);
      },
      2 => self.decode2(io, opcode),
      _ => self.decode3(io, opcode),
    }
  }
  
  fn decode0(&mut self, io: &mut impl IO, opcode: u8) {
    let opcode = opcode.rotate_left(4); //Rotating makes decoding a lot easier..
    match opcode {
      0x00..=0x03 | 0x80..=0x83 => trace!("NOP"),
      0x10..=0x13 => {  //LXI
        let word = self.cpu.next_code_word(io);
        let to_index = u2::new(opcode & 0b11);
        trace!("Copy 0x{:04X} into {}", word, self.debug_pair(to_index));
        self.write_pair(to_index, word);
      },
      0x20 => { trace!("Copy Acc into [BC]"); //STAX BC
        let to_addr = self.regs.bc.raw_value();
        let byte = self.regs.psw.acc();
        io.write_mem(to_addr, byte);
      },
      0x21 => { trace!("Copy Acc into [DE]"); //STAX DE
        let to_addr = self.regs.de.raw_value();
        let byte = self.regs.psw.acc();
        io.write_mem(to_addr, byte);
      },
      0x22 => { trace!("Copy HL into [word]"); //SHLD
        let to_addr = self.cpu.next_code_word(io);
        let value = self.regs.hl.raw_value();
        io.write_mem(to_addr, value);
      },
      0x23 => { trace!("Copy Acc into [word]"); //STA
        let to_addr = self.cpu.next_code_word(io);
        let value = self.regs.psw.acc();
        io.write_mem(to_addr, value);
      },
      0x30..=0x33 => {  //INX
        let index = u2::new(opcode & 0b11);
        trace!("Increment {}", self.debug_pair(index));
        let value = self.read_pair(index);
        self.write_pair(index, value.wrapping_add(1));
      },
      0x40..=0x43 | 0xC0..=0xC3 => {  //INR
        let index = rotate_index(opcode);
        trace!("Increment {}", self.debug_reg(index));
        let byte = self.read_reg(io, index);
        let (result, _, aux) = cpu::execute_add(byte, 1);
        self.write_reg(io, index, result);
        self.regs.psw = self.regs.psw.with_aux(aux);
        self.set_result_flags(result);
      },
      0x50..=0x53 | 0xD0..=0xD3 => {  //DCR
        let index = rotate_index(opcode);
        trace!("Decrement {}", self.debug_reg(index));
        let byte = self.read_reg(io, index);
        let (result, _, aux) = cpu::execute_sub(byte, 1);
        self.write_reg(io, index, result);
        self.regs.psw = self.regs.psw.with_aux(aux);
        self.set_result_flags(result);
      },
      0x60..=0x63 | 0xE0..=0xE3 => {  //MVI
        let byte = self.cpu.next_code_byte(io);
        let to_index = rotate_index(opcode);
        trace!("Copy 0x{:02X} into {}", byte, self.debug_reg(to_index));
        self.write_reg(io, to_index, byte);
      },
      0x70 => { trace!("RLC");
        let acc = self.regs.psw.acc().rotate_left(1);
        let carry = acc & 0b1 == 0b1;
        self.regs.psw = self.regs.psw.with_acc(acc).with_carry(carry);
      },
      0x71 => { trace!("RAL");
        let new_carry = self.regs.psw.acc() >> 7 == 0b1;
        let acc = self.regs.psw.acc() << 1 | self.regs.psw.carry() as u8;
        self.regs.psw = self.regs.psw.with_acc(acc).with_carry(new_carry);
      },
      0x72 => { trace!("DAA");
        let (acc, carry, aux) = cpu::execute_daa(self.regs.psw.acc(), self.regs.psw.carry(), self.regs.psw.aux());
        self.regs.psw = self.regs.psw.with_acc(acc).with_carry(carry).with_aux(aux);
        self.set_result_flags(acc);
      },
      0x73 => { trace!("STC");
        self.regs.psw = self.regs.psw.with_carry(true);
      },
      0x90..=0x93 => {  //HL += pair
        let index = u2::new(opcode & 0b11);
        trace!("DAD {}", self.debug_pair(index));
        let (hl, carry) = self.regs.hl.raw_value().overflowing_add(self.read_pair(index));
        self.regs.hl = Word::new_with_raw_value(hl);
        self.regs.psw = self.regs.psw.with_carry(carry);
      },
      0xA0 => { trace!("Copy [BC] into Acc"); //LDAX BC
        let value = io.read_mem(self.regs.bc.raw_value());
        self.regs.psw = self.regs.psw.with_acc(value);
      },
      0xA1 => { trace!("Copy [DE] into Acc"); //LDAX DE
        let value = io.read_mem(self.regs.de.raw_value());
        self.regs.psw = self.regs.psw.with_acc(value);
      },
      0xA2 => { trace!("Copy [word] into HL"); //LHLD
        let from_addr = self.cpu.next_code_word(io);
        let value = io.read_mem(from_addr);
        self.regs.hl = Word::new_with_raw_value(value);
      },
      0xA3 => { trace!("Copy [word] into Acc"); //LDA
        let from_addr = self.cpu.next_code_word(io);
        let value = io.read_mem(from_addr);
        self.regs.psw = self.regs.psw.with_acc(value);
      },
      0xB0..=0xB3 => {  //DCX
        let index = u2::new(opcode & 0b11);
        trace!("Decrement {}", self.debug_pair(index));
        let value = self.read_pair(index);
        self.write_pair(index, value.wrapping_sub(1));
      },
      0xF0 => { trace!("RRC");
        let carry = self.regs.psw.acc() & 0b1 == 0b1;
        let acc = self.regs.psw.acc().rotate_right(1);
        self.regs.psw = self.regs.psw.with_acc(acc).with_carry(carry);
      },
      0xF1 => { trace!("RAR");
        let new_carry = self.regs.psw.acc() & 1 == 1;
        let acc = self.regs.psw.acc() >> 1 | ((self.regs.psw.carry() as u8) << 7);
        self.regs.psw = self.regs.psw.with_acc(acc).with_carry(new_carry);
      },
      0xF2 => { trace!("CMA");
        self.regs.psw = self.regs.psw.with_acc(!self.regs.psw.acc());
      },
      0xF3 | _ => { trace!("CMC");
        self.regs.psw = self.regs.psw.with_carry(!self.regs.psw.carry());
      },
    };
  }

  fn decode2(&mut self, io: &mut impl IO, opcode: u8) {
    let index = u3_index(opcode);
    let byte = self.read_reg(io, index);
    let (acc, carry, aux) = match opcode {
      0x80..=0x87 => { trace!("ADD {}", self.debug_reg(index));
        cpu::execute_add(self.regs.psw.acc(), byte)
      },
      0x88..=0x8F => { trace!("ADC {}", self.debug_reg(index));
        cpu::execute_add_carry(self.regs.psw.acc(), byte, self.regs.psw.carry())
      },
      0x90..=0x97 => { trace!("SUB {}", self.debug_reg(index));
        cpu::execute_sub(self.regs.psw.acc(), byte)
      },
      0x98..=0x9F => { trace!("SBB {}", self.debug_reg(index));
        cpu::execute_sub_carry(self.regs.psw.acc(), byte, self.regs.psw.carry())
      },
      0xA0..=0xA7 => { trace!("ANA {}", self.debug_reg(index));
        (self.regs.psw.acc() & byte, false, false)  //Logical expression will never carry..
      },
      0xA8..=0xAF => { trace!("XRA {}", self.debug_reg(index));
        (self.regs.psw.acc() ^ byte, false, false)  //Logical expression will never carry..
      },
      0xB0..=0xB7 => { trace!("ORA {}", self.debug_reg(index));
        (self.regs.psw.acc() | byte, false, false)  //Logical expression will never carry..
      },
      0xB8..=0xBF | _ => { trace!("CMP {}", self.debug_reg(index));
        cpu::execute_sub(self.regs.psw.acc(), byte)
      },
    };
    match opcode {
      0xB8..=0xBF => (),  //Skip CMP. Don't write to accumulator if CMP.
      _ => self.regs.psw = self.regs.psw.with_acc(acc),
    };
    self.regs.psw = self.regs.psw.with_carry(carry).with_aux(aux);
    self.set_result_flags(acc);
  }

  fn decode3(&mut self, io: &mut impl IO, opcode: u8) {
    let opcode = opcode.rotate_left(4); //Rotating makes decoding a lot easier..
    match opcode {
      0x0C..=0x0F | 0x8C..=0x8F => {
        let index = rotate_index(opcode);
        trace!("R{}", self.debug_condition(index));
        if self.test_condition(index) {
          self.cpu.pc = self.cpu.pop(io); //RET
        }
      },
      0x1C..=0x1F => {
        let to_index = u2::new(opcode & 0b11);
        trace!("POP {}", self.debug_pair_push_pop(to_index));
        let value = self.cpu.pop(io);
        self.write_pair_pop(to_index, value);
      },
      0x2C..=0x2F | 0xAC..=0xAF => {
        let index = rotate_index(opcode);
        let word = self.cpu.next_code_word(io);
        trace!("J{} 0x{:04X}", self.debug_condition(index), word);
        if self.test_condition(index) {
          self.cpu.pc = word; //JMP
        }
      },
      0x3C | 0xBC => {
        let word = self.cpu.next_code_word(io);
        trace!("JMP 0x{:04X}", word);
        self.cpu.pc = word;
      },
      0x3D => { let port = self.cpu.next_code_byte(io);
        trace!("Output acc into port 0x{:02X}", port);
        io.output(port, self.regs.psw.acc());
      },
      0x3E => { trace!("Xchange HL and [SP]"); //XTHL
        let value = io.read_mem(self.cpu.sp);
        io.write_mem(self.cpu.sp, self.regs.hl.raw_value());
        self.regs.hl = Word::new_with_raw_value(value);
      },
      0x3F => { trace!("Disable Interrupts"); //DI
        self.interrupts_enabled = false;
      },
      0x4C..=0x4F | 0xCC..=0xCF => {
        let index = rotate_index(opcode);
        let word = self.cpu.next_code_word(io);
        trace!("C{} 0x{:04X}", self.debug_condition(index), word);
        if self.test_condition(index) {
          self.cpu.push(io, self.cpu.pc);
          self.cpu.pc = word; //CALL
        }
      },
      0x5C..=0x5F => {
        let from_index = u2::new(opcode & 0b11);
        trace!("PUSH {}", self.debug_pair_push_pop(from_index));
        let value = self.read_pair_push(from_index);
        self.cpu.push(io, value);
      },
      0x6C => {
        let byte = self.cpu.next_code_byte(io);
        trace!("ADI 0x{:02X}", byte);
        let (result, carry, aux) = cpu::execute_add(self.regs.psw.acc(), byte);
        self.regs.psw = self.regs.psw.with_acc(result).with_carry(carry).with_aux(aux);
        self.set_result_flags(result);
      },
      0x6D => {
        let byte = self.cpu.next_code_byte(io);
        trace!("SUI 0x{:02X}", byte);
        let (result, carry, aux) = cpu::execute_sub(self.regs.psw.acc(), byte);
        self.regs.psw = self.regs.psw.with_acc(result).with_carry(carry).with_aux(aux);
        self.set_result_flags(result);
      },
      0x6E => {
        let byte = self.cpu.next_code_byte(io);
        trace!("ANI 0x{:02X}", byte);
        let result = self.regs.psw.acc() & byte;
        self.regs.psw = self.regs.psw.with_acc(result).with_carry(false).with_aux(false);
        self.set_result_flags(result);
      },
      0x6F => {
        let byte = self.cpu.next_code_byte(io);
        trace!("ORI 0x{:02X}", byte);
        let result = self.regs.psw.acc() | byte;
        self.regs.psw = self.regs.psw.with_acc(result).with_carry(false).with_aux(false);
        self.set_result_flags(result);
      },
      0x7C..=0x7F | 0xFC..=0xFF => {  //RST weird call function. The program counter is set to the value 0000 0000 00nn n000.
        let index = rotate_index(opcode);
        trace!("RST {}", index);
        self.cpu.push(io, self.cpu.pc);
        self.cpu.pc = (index.value() as u16) << 3;
      },
      0x9C..=0x9D => { trace!("RET");
        self.cpu.pc = self.cpu.pop(io);
      },
      0x9E => { trace!("Copy HL into PC"); //PCHL
        self.cpu.pc = self.regs.hl.raw_value();
      },
      0x9F => { trace!("Copy HL into SP"); //SPHL
        self.cpu.sp = self.regs.hl.raw_value();
      },
      0xBD => {
        let port = self.cpu.next_code_byte(io);
        trace!("Input acc from port 0x{:02X}", port);
        self.regs.psw = self.regs.psw.with_acc(io.input(port));
      },
      0xBE => { trace!("Xchange HL and DE");
        core::mem::swap(&mut self.regs.hl, &mut self.regs.de);
      },
      0xBF => { trace!("Enable Interrupts");  //EI
        self.interrupts_enabled = true;
      },
      0xDC..=0xDF => {
        let word = self.cpu.next_code_word(io);
        trace!("CALL 0x{:04X}", word);
        self.cpu.push(io, self.cpu.pc);
        self.cpu.pc = word;
      },
      0xEC => {
        let byte = self.cpu.next_code_byte(io);
        trace!("ACI 0x{:02X}", byte);
        let (result, carry, aux) = cpu::execute_add_carry(self.regs.psw.acc(), byte, self.regs.psw.carry());
        self.regs.psw = self.regs.psw.with_acc(result).with_carry(carry).with_aux(aux);
        self.set_result_flags(result);
      },
      0xED => {
        let byte = self.cpu.next_code_byte(io);
        trace!("SBI 0x{:02X}", byte);
        let (result, carry, aux) = cpu::execute_sub_carry(self.regs.psw.acc(), byte, self.regs.psw.carry());
        self.regs.psw = self.regs.psw.with_acc(result).with_carry(carry).with_aux(aux);
        self.set_result_flags(result);
      },
      0xEE => {
        let byte = self.cpu.next_code_byte(io);
        trace!("XRI 0x{:02X}", byte);
        let result = self.regs.psw.acc() ^ byte;
        self.regs.psw = self.regs.psw.with_acc(result).with_carry(false).with_aux(false);
        self.set_result_flags(result);
      },
      0xEF | _ => {
        let byte = self.cpu.next_code_byte(io);
        trace!("CPI 0x{:02X}", byte);
        let (result, carry, aux) = cpu::execute_sub(self.regs.psw.acc(), byte);
        self.regs.psw = self.regs.psw.with_carry(carry).with_aux(aux);
        self.set_result_flags(result);
      },
    };
  }
  
  fn set_result_flags(&mut self, result: u8) {
    self.regs.psw = self.regs.psw
                      .with_sign(result & 0b1000_0000 == 0b1000_0000)
                      .with_zero(result == 0)
                      .with_parity(result.count_ones() % 2 == 0);
  }
}

/// A lot of opcodes store index this way
fn u3_index(opcode: u8) -> u3 {
  u3::new(opcode & 0b111)
}

/// After some examination, I found that a lot of the opcodes are written in this format:
/// 0,2,4,6 or 1,3,5,7
/// Rotating left converts all of the registers into standardized 1,2,3,4,5,6,7
fn rotate_index(opcode: u8) -> u3 {
  u3::new(opcode.rotate_left(1) & 0b111)
}
