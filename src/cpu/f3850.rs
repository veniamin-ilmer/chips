//! The 3850 CPU, also known as the SL31291, was shipped in 1975.
//!
//! Each clock pulse is 500 ns. (2 MHz)
//! Each instruction is at least 4 clock pulses.
//! That means at least 2 microseconds per instruction.
//!
//! Useful links
//! * <https://wiki.console5.com/tw/images/e/e8/Fairchild_F3850.pdf>
//! * <https://www.seanriddle.com/F8GuideToProgramming.pdf>
//! * <http://www.nyx.net/~lturner/public_html/F8_ins.html>
//! * <https://channelf.se/veswiki/index.php?title=Opcode>
//! * <http://www.bitsavers.org/components/mostek/f8/1981_3870_F8_Microcomputer_Data_Book.pdf>
//! * <https://channelf.se/veswiki/images/1/1d/F8_User%27s_Guide_%281976%29%28Fairchild%29%28Document_67095665%29.pdf>

use bitbybit::bitfield;
use log::{trace,debug};

/// Used to communicate with board
pub trait IO {
  /// Write to interal IO port
  fn output(&mut self, port: u8, value: u8);
  /// Read from interal IO port
  fn input(&self, port: u8) -> u8;
  /// Read from external IO port
  fn read_external_port(&self, port: u8) -> u8;
  
  /// ROMC00 or ROMC03 - Read next code byte
  fn next_code(&mut self) -> u8;
  /// ROMC01 - Read code byte without updating read pointer
  fn peak_code(&self) -> i8;
  /// ROMC02 - Read next data byte
  fn next_data(&mut self) -> u8;
  /// Write next data byte
  fn write_data(&mut self, data: u8);

  /// ROMC14 and ROMC0C - Jump to direct address. push_pc will back up the current position, so you can return to it later. (Call vs Jump)
  fn jump(&mut self, upper: u8, lower: u8, push_pc: bool);
  /// Jump to relative address.
  fn jump_relative(&mut self, relative_addr: i8);
  /// Return from address.
  fn ret_pc(&mut self);
  
  /// Add to dc0 pointer
  fn add_dc0(&mut self, a: i8);
  /// Get dc0 pointer, returns upper, lower
  fn get_dc0(&self) -> (u8, u8);
  /// Set dc0 pointer
  fn set_dc0(&mut self, upper: u8, lower: u8);
  /// Swap DC pointers
  fn swap_dc(&mut self);
  
  /// ROMC07, ROMC0B - Get pc1 pointer, returns upper, lower
  fn get_pc1(&self) -> (u8, u8);
  /// Set pc1 pointer
  fn set_pc1(&mut self, upper: u8, lower: u8);
  
}

/// Status Register (Flags). Also known as the W Register.
#[bitfield(u8, default: 0)]
struct Flags {
  /// Interrupt (ICB)
  #[bit(4, rw)]
  interrupt: bool,

  /// Signed overflow
  #[bit(3, rw)]
  overflow: bool,

  /// Zero (Z): Set if result is zero
  #[bit(2, rw)]
  zero: bool,

  /// Unsigned overflow
  #[bit(1, rw)]
  carry: bool,

  /// Sign / negative (S): Set if result is negative
  #[bit(0, rw)]
  negative: bool,
}

/// Fairchild F3850 chip
pub struct F3850 {
  /// Status Register (Flags)
  flags: Flags,
  /// Accumulator
  acc: u8,
  /// Indirect Scratchpad Address Register (ISAR) - Used to address the registers.
  isar: u8,
  /// General Registers. 64 bytes
  pub regs: [u8; 64],
  /// Setting this to true will make the F3850 "reset" by jumping back to position 0.
  pub reset: bool,
  /// IO Ports (Internal Register)
  pub ports: [u8; 4],
}

impl Default for F3850 {
  fn default() -> Self {
    F3850 {
      flags: Default::default(),
      acc: 0,
      isar: 0,
      regs: [0; 64],
      reset: false,
      ports: [0;4],
    }
  }
}


impl F3850 {
  /// Create a new chip
  pub fn new() -> Self {
    Default::default()
  }

  /// Print debug data of all registers
  pub fn print(&self) {
    trace!("Acc: 0x{:02X} ISAR: 0x{:02X}", self.acc, self.isar);
    trace!("Interrupt: {} Overflow: {} Zero: {} Carry: {} Negative: {}", self.flags.interrupt(), self.flags.overflow(), self.flags.zero(), self.flags.carry(), self.flags.negative());
    trace!("R00: 0x{:02X} R01: 0x{:02X} R02: 0x{:02X} R03: 0x{:02X} R04: 0x{:02X}, R05: 0x{:02X} R06: 0x{:02X} R07: 0x{:02X} R10: 0x{:02X}   J: 0x{:02X}  HU: 0x{:02X}  HL: 0x{:02X}  KU: 0x{:02X},  KL: 0x{:02X}  QU: 0x{:02X}  QL: 0x{:02X}", self.regs[0x00], self.regs[0x01], self.regs[0x02], self.regs[0x03], self.regs[0x04], self.regs[0x05], self.regs[0x06], self.regs[0x07], self.regs[0x08], self.regs[0x09], self.regs[0x0A], self.regs[0x0B], self.regs[0x0C], self.regs[0x0D], self.regs[0x0E], self.regs[0x0F]);
    //trace!("R20: 0x{:02X} R11: 0x{:02X} R12: 0x{:02X} R13: 0x{:02X} R14: 0x{:02X}, R15: 0x{:02X} R16: 0x{:02X} R17: 0x{:02X}", self.regs[0x10], self.regs[0x11], self.regs[0x12], self.regs[0x13], self.regs[0x14], self.regs[0x15], self.regs[0x16], self.regs[0x17]);
    //trace!("R38: 0x{:02X} R19: 0x{:02X} R1A: 0x{:02X} R1B: 0x{:02X} R1C: 0x{:02X}, R1D: 0x{:02X} R1E: 0x{:02X} R1F: 0x{:02X}", self.regs[0x18], self.regs[0x19], self.regs[0x1A], self.regs[0x1B], self.regs[0x1C], self.regs[0x1D], self.regs[0x1E], self.regs[0x1F]);
    //trace!("R40: 0x{:02X} R21: 0x{:02X} R22: 0x{:02X} R23: 0x{:02X} R24: 0x{:02X}, R25: 0x{:02X} R26: 0x{:02X} R27: 0x{:02X}", self.regs[0x20], self.regs[0x21], self.regs[0x22], self.regs[0x23], self.regs[0x24], self.regs[0x25], self.regs[0x26], self.regs[0x27]);
    //trace!("R58: 0x{:02X} R29: 0x{:02X} R2A: 0x{:02X} R2B: 0x{:02X} R2C: 0x{:02X}, R2D: 0x{:02X} R2E: 0x{:02X} R2F: 0x{:02X}", self.regs[0x28], self.regs[0x29], self.regs[0x2A], self.regs[0x2B], self.regs[0x2C], self.regs[0x2D], self.regs[0x2E], self.regs[0x2F]);
    //trace!("R60: 0x{:02X} R31: 0x{:02X} R32: 0x{:02X} R33: 0x{:02X} R34: 0x{:02X}, R35: 0x{:02X} R36: 0x{:02X} R37: 0x{:02X}", self.regs[0x30], self.regs[0x31], self.regs[0x32], self.regs[0x33], self.regs[0x34], self.regs[0x35], self.regs[0x36], self.regs[0x37]);
    //trace!("R78: 0x{:02X} R39: 0x{:02X} R3A: 0x{:02X} R3B: 0x{:02X} R3C: 0x{:02X}, R3D: 0x{:02X} R3E: 0x{:02X} R3F: 0x{:02X}", self.regs[0x38], self.regs[0x39], self.regs[0x3A], self.regs[0x3B], self.regs[0x3C], self.regs[0x3D], self.regs[0x3E], self.regs[0x3F]);
  }

  /// Executes single instruction:
  ///
  /// 1. Read the next byte from ROM
  /// 2. Execute instruction.
  /// 3. Return number of clock pulses this instruction consumed.
  ///
  pub fn run_cycle(&mut self, io: &mut impl IO) -> u8 {
    if self.reset {
      io.jump(0, 0, true);
      self.reset = false;
      return 14;
    }
    
    let opcode = io.next_code();
    match opcode {
      0x00..=0x03 => { let r = opcode + 12;
        debug!("LR Acc, R{:X}", r);  //Load Register
        self.acc = self.regs[r as usize];
        4
      },
      0x04..=0x07 => { let r = opcode - 4 + 12;
        debug!("LR R{:X}, Acc", r);  //Load Register
        self.regs[r as usize] = self.acc;
        4
      },
      0x08 => { debug!("LR K, PC1"); (self.regs[12], self.regs[13]) = io.get_pc1(); 16 },
      0x09 => { debug!("LR PC1, K"); io.set_pc1(self.regs[12], self.regs[13]); 16 },
      0x0A => { debug!("LR Acc, IS"); self.acc = self.isar; 4 },
      0x0B => { debug!("LR IS, Acc"); self.isar = self.acc & 0b11_1111; 4 }, //ISAR is supposed to be a 6 bit register. It loses the top 8 bits when copying from the accumulator.
      
      0x0C => { debug!("PK"); io.jump(self.regs[12], self.regs[13], true); 10 },
      0x0D => { debug!("LR PO, Q"); io.jump(self.regs[14], self.regs[15], false); 16 },
      0x0E => { debug!("LR Q, DC"); (self.regs[14], self.regs[15]) = io.get_dc0(); 16 },
      0x0F => { debug!("LR DC, Q"); io.set_dc0(self.regs[14], self.regs[15]); 16 },
      0x10 => { debug!("LR DC, H"); io.set_dc0(self.regs[10], self.regs[11]); 16 },
      0x11 => { debug!("LR H, DC"); (self.regs[10], self.regs[11]) = io.get_dc0(); 16 },
      
      //Shift
      0x12 => { debug!("SR 1"); self.acc >>= 1; self.set_flags(); 4 }, //TODO - Test negative flag here
      0x13 => { debug!("SL 1"); self.acc <<= 1; self.set_flags(); 4 },
      0x14 => { debug!("SR 4"); self.acc >>= 4; self.set_flags(); 4 }, //TODO - Test negative flag here
      0x15 => { debug!("SL 4"); self.acc <<= 4; self.set_flags(); 4 },
      
      0x16 => { debug!("LM"); self.acc = io.next_data(); 10 }, //Load Memory
      0x17 => { debug!("SM"); io.write_data(self.acc); 10 },   //Store Memory
      
      0x18 => { debug!("COM"); self.acc = !self.acc; self.set_flags(); 4 },  //Complement / invert / not

      0x19 => { debug!("LNK"); self.acc = self.add_and_set_flags(self.acc, self.flags.carry() as u8); 4},
      
      0x1A => { debug!("DI"); self.flags = self.flags.with_interrupt(false); 4 },  //Disable Interrupts
      0x1B => { debug!("EI"); self.flags = self.flags.with_interrupt(true); 4 },  //Enable Interrupts

      0x1C => { debug!("POP"); io.ret_pc(); 8 } //Return from PK / PI / IRQ.
      
      0x1D => { debug!("LR W, J"); self.flags = Flags::new_with_raw_value(self.regs[9]); 4 }
      0x1E => { debug!("LR J, W"); self.regs[9] = self.flags.raw_value(); 8 }

      0x1F => { debug!("INC"); self.acc = self.add_and_set_flags(self.acc, 1); 4 },  //INCrement

      0x20..=0x27 => { let imm = io.next_code();
        match opcode {
          0x20 => { debug!("LI 0x{:02X}", imm); self.acc = imm; 10 }, //Load Immediate
          0x21 => { debug!("NI 0x{:02X}", imm); self.acc &= imm; self.set_flags(); 10 },   //aNd Immediate
          0x22 => { debug!("OI 0x{:02X}", imm); self.acc |= imm; self.set_flags(); 10 },   //Or Immediate
          0x23 => { debug!("XI 0x{:02X}", imm); self.acc ^= imm; self.set_flags(); 10 },  //Xor Immediate
          0x24 => { debug!("AI 0x{:02X}", imm); self.acc = self.add_and_set_flags(imm, self.acc); 10 },   //Add Immediate
          0x25 => { debug!("CI 0x{:02X}", imm); self.subtract_and_set_flags(imm); 10 }, //subtract Immediate but only set status.
          0x26 => { debug!("IN 0x{:02X}", imm); self.input(io, imm); 16 },  //INput
          _ => { debug!("OUT 0x{:02X}", imm); self.output(io, imm); 16 }, //0x27. OUTput
        }
      },
      0x28..=0x2A => {
        let upper = io.next_code();
        let lower = io.next_code();
        match opcode {
          0x28 => { debug!("PI 0x{:02X}{:02X}", upper, lower); self.acc = upper; io.jump(upper, lower, true); 26 },  //call
          0x29 => { debug!("JMP 0x{:02X}{:02X}", upper, lower); self.acc = upper; io.jump(upper, lower, false); 22 }, //JuMP
          _ => { debug!("DCI 0x{:02X}{:02X}", upper, lower); io.set_dc0(upper, lower); 24 },  //0x2A. load DC0 Immediate
        }
      }
      0x2B => { debug!("NOP"); 4 }, //No Operation
      0x2C => { debug!("XDC"); io.swap_dc(); 8 },  //eXchange DC
      
      0x2F => { console_log::init_with_level(log::Level::Trace); 0 }
      0x30..=0x5F => {
        let reg = self.reg_or_isar(opcode & 0xF);
        match opcode {
          0x30..=0x3F => { debug!("DS R{:X}", reg); self.regs[reg] = self.add_and_set_flags(self.regs[reg], (-1_i16) as u8); 6 }, //Decrement Scratchpad
          0x40..=0x4F => { debug!("LR Acc, R{:X}", reg); self.acc = self.regs[reg]; 4 }, //Load Register
          _ => { debug!("LR R{:X}, Acc", reg); self.regs[reg] = self.acc; 4 }, //Load Register (0x50..=0x5F)
        }
      },
      0x60..=0x67 => { let imm = opcode & 0b111; debug!("LISU 0x{:X}", imm); self.isar = (self.isar & 0b111) | (imm << 3); 4 },  //Load Indirect Scratchpad Upper
      0x68..=0x6F => { let imm = opcode & 0b111; debug!("LISL 0x{:X}", imm); self.isar = (self.isar & 0b111000) | imm; 4 },  //Load Indirect Scratchpad Lower
      0x70..=0x7F => { let imm = opcode & 0xF; debug!("LIS 0x{:X}", imm); self.acc = imm; 4 }, //Load Immediate
      0x80..=0x87 => {  //conditional jump
        let relative_addr = io.peak_code();
        let abs_addr = relative_addr.abs();
        let sign = if relative_addr > 0 { '+' } else { '-' };
        let condition = match opcode & 0b111 {
          1 => { debug!("jmp if Positive to {}{}", sign, abs_addr); !self.flags.negative() },
          2 => { debug!("jmp if Carry to {}{}", sign, abs_addr); self.flags.carry() },
          3 => { debug!("jmp if Carry or Positive to {}{}", sign, abs_addr); self.flags.carry() | !self.flags.negative() },
          4 => { debug!("jmp if Zero to {}{}", sign, abs_addr); self.flags.zero() },
          5 => { debug!("jmp if Zero or Positive to {}{}", sign, abs_addr); self.flags.zero() | !self.flags.negative() },
          6 => { debug!("jmp if Zero or Carry to {}{}", sign, abs_addr); self.flags.zero() | self.flags.carry() },
          7 => { debug!("jmp if Zero or Carry or Positive to {}{}", sign, abs_addr); self.flags.zero() | self.flags.carry() | !self.flags.negative() },
          _ => { debug!("Don't jump"); false },
        };
        if condition {
          io.jump_relative(relative_addr);
          14
        } else {
          io.next_code(); //jump over the peaked address.
          12
        }
      },
      0x88..=0x8D => {
        let data = io.next_data();
        match opcode {
          0x88 => { debug!("AM {}", data); self.acc = self.add_and_set_flags(self.acc, data); }, //Add Memory
          0x89 => { debug!("AMD {}", data); self.acc = self.add_bcd_and_set_flags(data); }, //Add Memory Decimal adjusted
          0x8A => { debug!("NM {}", data); self.acc &= data; self.set_flags(); }, //aNd Memory
          0x8B => { debug!("OM {}", data); self.acc |= data; self.set_flags(); }, //Or Memory
          0x8C => { debug!("XM {}", data); self.acc ^= data; self.set_flags(); }, //Xor Memory
          _ => { debug!("CM {}", data); self.subtract_and_set_flags(data); }, //subtract Memory, but only update status
        };
        10
      }
      0x8E => { debug!("ADC"); io.add_dc0(self.acc as i8); 10 },  //Add DC

      0x8F..=0x9F => {  //BRanch (conditional jump)
        let relative_addr = io.peak_code();
        let abs_addr = relative_addr.abs();
        let sign = if relative_addr > 0 { '+' } else { '-' };
        let condition = match opcode {
          0x8F => { debug!("if isar low != 7 to {}{}", sign, abs_addr); self.isar & 0b111 != 0b111 },
          0x91|0x95 => { debug!("jump if Negative to {}{}", sign, abs_addr); self.flags.negative() },
          0x92 => { debug!("jump if No Carry to {}{}", sign, abs_addr); !self.flags.carry() },
          0x93|0x97 => { debug!("jump if No Carry and Negative to {}{}", sign, abs_addr); !self.flags.carry() & self.flags.negative() },
          0x94 => { debug!("jump if Not Zero to {}{}", sign, abs_addr); !self.flags.zero() },
          0x96 => { debug!("jump if Not Carry and Not Zero to {}{}", sign, abs_addr); !self.flags.carry() & !self.flags.zero() },
          0x98 => { debug!("jump if No Overflow to {}{}", sign, abs_addr); !self.flags.overflow() },
          0x99|0x9D => { debug!("jump if negative and No Overflow to {}{}", sign, abs_addr); self.flags.negative() & !self.flags.overflow() },
          0x9A => { debug!("jump if No Overflow and No Carry to {}{}", sign, abs_addr); !self.flags.overflow() & !self.flags.carry() },
          0x9B|0x9F => { debug!("jump if No Overflow and No Carry and negative to {}{}", sign, abs_addr); !self.flags.overflow() & !self.flags.carry() & self.flags.negative() },
          0x9C => { debug!("jump if No Overflow and Not Zero to {}{}", sign, abs_addr); !self.flags.overflow() & !self.flags.zero() },
          0x9E => { debug!("jump if No Overflow and No Carry and Not Zero to {}{}", sign, abs_addr); !self.flags.overflow() & !self.flags.carry() & !self.flags.zero() },
          _ => { debug!("jump to {}{}", sign, abs_addr); true },  //0x90
        };
        if condition {
          io.jump_relative(relative_addr);
          match opcode { 0x8F => 10, _ => 14 }
        } else {
          io.next_code(); //jump over the peaked address.
          match opcode { 0x8F => 8, _ => 12 }
        }
      },
      0xA0..=0xAF => {  //INput
        let port = opcode & 0xF;
        debug!("INS 0x{:02X}", port);
        self.input(io, port);
        match port { 0..=3 => 4, _ => 8 }
      },
      0xB0..=0xBF => {  //OUTput will only set the internal port values. Not external.
        let port = opcode & 0xF;
        debug!("OUTS 0x{:02X}", port);
        self.output(io, port);
        match port { 0..=3 => 4, _ => 8 }
      },
      0xC0..=0xFF => {
        let reg = self.reg_or_isar(opcode & 0xF);
        let value = self.regs[reg];
        match opcode {
          0xC0..=0xCF => { debug!("AS R{:X}", reg); self.acc = self.add_and_set_flags(self.acc, value); 4 }, //Add register
          0xD0..=0xDF => { debug!("ASD R{:X}", reg); self.acc = self.add_bcd_and_set_flags(value); 8 }, //Add register decimal
          0xE0..=0xEF => { debug!("XS R{:X}", reg); self.acc ^= value; self.set_flags(); 4 }, //XOR register
          _ => { debug!("NS R{:X}", reg); self.acc &= value; self.set_flags(); 4 }, //aNd register (0xF0..=0xFF)
        }
      },
      _ => unreachable!("Unknown opcode: {:X}", opcode),
    }
  }
  
  fn reg_or_isar(&mut self, reg: u8) -> usize {
    (match reg {
      0..=11 => reg,
      12 => self.isar,
      _ => {
        let isar = self.isar;
        let isar_low = self.isar & 0b111;
        let new_isar_low = match (reg, isar_low) {
          (13, 0b111) => 0, //Wrap it around
          (13, _) => isar_low + 1,
          
          (14, 0) => 0b111, //Wrap it around
          (14, _) => isar_low - 1,
          
          _ => unreachable!(),
        };
        self.isar = (self.isar & 0b111000) | new_isar_low;
        isar
      },
      _ => unreachable!(),
    }) as usize
  }
  
  /// Adds number to accumulator with BCD logic, all flags.
  /// See F8 Guide to programming page 06-03.
  fn add_bcd_and_set_flags(&mut self, num: u8) -> u8 {
    //Step 1 is done before his instruction of adding 0x66.
    //Step 2 - Binary add the the sum from step 1 to the second number. Record the C and IC as flags.
    let normal_add = self.add_and_set_flags(self.acc, num);
    let (_, intermediate_carry) = (self.acc << 4).overflowing_add(num << 4);
    
    //Step 3 - Add 0xA to the high and/or low of step 2, based on the status of C and IC.
    //In Step 3, any carry from the low order digit to the high order digit is suppressed.
    let high = if self.flags.carry() {
      normal_add >> 4
    } else {
      ((normal_add >> 4) + 0xA) & 0xF
    };
    let low = if intermediate_carry {
      normal_add & 0xF
    } else {
      ((normal_add & 0xF) + 0xA) & 0xF
    };
    
    (high << 4) | low
  }
  
  /// Adds number to accumulator, all flags
  fn add_and_set_flags(&mut self, num1: u8, num2: u8) -> u8 {
    let (result, carry8) = num1.overflowing_add(num2);
    let (_, carry7) = (num1 << 1).overflowing_add(num2 << 1);

    self.flags = self.flags
                  .with_negative(result & 0b1000_0000 == 0b1000_0000)
                  .with_carry(carry8)
                  .with_zero(result == 0)
                  .with_overflow(carry8 ^ carry7);
    result
  }
  
  /// Subtracts number from accumulator, all flags
  fn subtract_and_set_flags(&mut self, num1: u8) -> u8 {
    let num2 = !self.acc; //We will be taking the two's complement by first doing NOT on the accumulator, and then adding 1

    let (intermediate, carry8_first) = num1.overflowing_add(num2);
    let (_, carry7_first) = (num1 << 1).overflowing_add(num2 << 1);

    let (result, carry8_extra) = intermediate.overflowing_add(1);
    let (_, carry7_extra) = (num1 << 1).overflowing_add(1 << 1);

    let carry8 = carry8_first | carry8_extra;
    let carry7 = carry7_first | carry7_extra;

    self.flags = self.flags
                  .with_negative(result & 0b1000_0000 == 0b1000_0000)
                  .with_carry(carry8)
                  .with_zero(result == 0)
                  .with_overflow(carry8 ^ carry7);
    result
  }
  
  fn set_flags(&mut self) {
    self.flags = self.flags
                      .with_negative(self.acc & 0b1000_0000 == 0b1000_0000)
                      .with_carry(false)
                      .with_zero(self.acc == 0)
                      .with_overflow(false);
  }
  
  /// INput will combine the internal and external ports.
  fn input(&mut self, io: &impl IO, port: u8) {
    self.acc = (match port {
      0..=3 => self.ports[port as usize],
      _ => io.input(port),
    }) | io.read_external_port(port);
    self.set_flags()
  }

  /// OUTput will only set the internal port values. Not external.
  fn output(&mut self, io: &mut impl IO, port: u8) {
    match port {
      0..=3 => {self.ports[port as usize] = self.acc; io.output(port, self.acc);},
      _ => io.output(port, self.acc),
    };
  }
}
