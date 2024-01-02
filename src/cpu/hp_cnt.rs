//! The HP 1820-0849, known as the Control and Timing (C&T) chip was used in 1972 in the HP-35, the first handheld calculator.
//!
//! Each instruction was one 10 bits long. The task of decoding and executing the instruction was divided between this C&T chip and the A&R chip.
//! Each clock cycle ends up taking 280 microseconds. (3.671 kHz)
//! The C&T is the control unit of the system; it has the following tasks:
//!
//! * Operating instruction counter and saving the return address,
//! * Keeping the status bits
//! * Scanning the keyboard (8 rows x 5 columns)
//! * Handling the pointer P
//!
//! Useful links
//! * <https://archived.hpcalc.org/laporte/HP%2035%20Saga.htm>
//! * <https://patentimages.storage.googleapis.com/44/5c/ab/197897f4ecaacb/US4001569.pdf>
use arbitrary_int::{u4, u6, u10, u14};
use log::{trace, info};

/// HP 1820-0849 Control and Timing (C&T) chip
#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct HP_CnT {
  /// Program Counter. Read by ROMs.
  pub next_address: u8,
  /// Saved Program Counter for call/return
  pub saved_address: u8,
  /// Status flags. Note that The only status flag connected to hardware is 0, which is set with key press.
  pub status: [bool; 12],
  /// Pointer
  pub pointer: u4,
  /// Carry - Can we jump?
  carry: bool,
    
  pub current_keypress: Option<u6>,
  timer: usize,
}

impl HP_CnT {
  /// Initialize with defaults
  pub fn new() -> Self {
    //wasm_log::init(wasm_log::Config::new(log::Level::Debug));
    //wasm_log::init(wasm_log::Config::new(log::Level::Trace));
    Default::default()
  }
  
  /// Print debug data of all registers
  pub fn print(&self) {
    trace!("Next Address: {:04o} Saved Address: {:04o} Status: {:?} Pointer: {:X} Carry: {}", self.next_address, self.saved_address, self.status, self.pointer, self.carry);
  }

  fn increment_pointer(&mut self) {
    if self.pointer == u4::new(0b1111) {
      self.pointer = u4::new(0);
      //self.carry = true;
    } else {
      self.pointer += u4::new(1);
      //self.carry = false;
    }
  }

  fn decrement_pointer(&mut self) {
    if self.pointer == u4::new(0) {
      self.pointer = u4::new(0b1111);
      //self.carry = true;
    } else {
      self.pointer -= u4::new(1);
      //self.carry = false;
    }
  }

  /// Returns word_select_data
  pub fn run_cycle(&mut self, opcode: u10, mut carry: bool) -> u14 {
    trace!("{:010b}", opcode);
    if self.timer > 0 {
      self.timer -= 1;
      if self.timer == 0 {
        panic!("done");
      }
    } else {
      //self.timer = 200;
    }
    self.next_address += 1;
    carry &= self.carry;  //Merge together carry signal from C&T and A&R.
    self.carry = true;  //Future carry
    let byte_opcode = (opcode.value() >> 2) as u8;

    match opcode.value() & 0b11 {
      //Type 1
      0b11 => {
        trace!("JMP {:04o} (Carry = {})", byte_opcode, carry);
        if carry { self.next_address = byte_opcode; } 
        u14::new(0)
      },
      0b01 => {
        trace!("CALL {:04o}", byte_opcode);
        self.saved_address = self.next_address;
        self.next_address = byte_opcode;
        u14::new(0)
      },
      
      //Type 2, handled by A&R.
      0b10 => {
        let word_select_data = if opcode.value() & 0b11100 == 0b00000 {
          1 << self.pointer.value() //3 => 0b1000
        } else if opcode.value() & 0b11100 == 0b10000 {
          (1 << (self.pointer.value() + 1)) - 1 //3 => 0b1111
        } else {
          0
        };
        u14::new(word_select_data)
      },
      _ => {
        let value = byte_opcode >> 4;
        
        /*if let Some(n) = self.current_keypress {
          if self.timer == 0 && n == u6::new(56) {
            wasm_log::init(wasm_log::Config::new(log::Level::Trace));
            self.timer = 200;
          }
        }*/
        
        match byte_opcode & 0b1111 {
          //Type 10 - NOP
          0b0000 => trace!("NOP"),
          
          //Type 3 - Status
          0b0001 => { trace!("S{} = true", value); self.status[value as usize] = true; },
          0b0101 => { trace!("? S{} != true", value); self.carry = !self.status[value as usize]; },
          0b1001 => { trace!("S{} = false", value); self.status[value as usize] = false; },
          0b1101 => {
             //Starting HP-55 or HP-65, this opcode was modified to add in "delayed select rom". It only clears status if the value is 0.
            if value == 0 {
              trace!("CLEAR STATUSES");
              for i in 0..12 { self.status[i] = false; }
            }
          },
          
          //Type 4 - Pointer
          0b0011 => { trace!("P = ${:X}", value); self.pointer = u4::new(value); },
          0b0111 => { trace!("P--"); self.decrement_pointer(); },
          0b1011 => { trace!("? P != ${:X}", value); self.carry = self.pointer != u4::new(value); } //Note carries are always recorded as opposites.
          0b1111 => { trace!("P++"); self.increment_pointer(); },
          
          //Type 5 - Data Entry. Ignore. Handled by A&R
          0b0010 | 0b1010 | 0b1110 => {},
          //Type 5 - Data Entry. Load Constant. P--
          0b0110 => {
            let word_select_data = 1 << self.pointer.value(); //3 => 0b1000
            self.decrement_pointer();
            // Per documentation: When used with the pointer in position 14, the instruction has no effect.
            if word_select_data <= 0b11111111111111 {
              return u14::new(word_select_data);
            }
          },
          
          //Type 6
          0b0100 | 0b1100 => {
            match byte_opcode & 0b11111 {
              0b00100 => { }, //ROM Select. Handled by ROM.
              0b01100 => { trace!("RET"); self.next_address = self.saved_address; } //Subroutine return
              0b10100 => {
                if byte_opcode >> 5 == 1 {
                  //Key -> ROM Address
                  self.next_address = if let Some(key_code) = self.current_keypress { key_code.value() } else { 0 };
                  
                  /*if self.next_address == 24 {
                    wasm_log::init(wasm_log::Config::new(log::Level::Trace));
                    self.timer = 100;
                  }*/
                  
                  //self.current_keypress = None;

                  info!("Key ({:03o}) -> Address", self.next_address);
                } else {
                  todo!("External Entry");
                }
              },
              0b11100 => {  //Auxilary Data Storage (RAM)
              },
              _ => unimplemented!("Unknown opcode: {:#b}00", byte_opcode),
            }
          },
          _ => unimplemented!("Unknown opcode: {:#b}00", byte_opcode),
        }
        u14::new(0)
      },
    }
  }
}