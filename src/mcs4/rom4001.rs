//! The 4001 ROM was limited to only 256 bytes of data. It was also unusual for having 4 io ports for the CPU to read/write with peripheral devices. Up to 16 ROM could be connected together, allowing for a maximum of 4 KB of binary code to be stored.

use arbitrary_int::u4;
use log::trace;
use bitbybit::bitfield;

/// 8 bit ROM address
#[bitfield(u8, default: 0)]
struct Address {
  /// Which high address inside of the ROM chip?
  #[bits(4..=7, rw)]
  high: u4,

  /// Which low address inside of the ROM chip?
  #[bits(0..=3, rw)]
  low: u4,
}

/// Intel 4001 chip
pub struct ROM {
  /// This page mask is added to all memory pointers to know which memory chip is being referenced
  page_mask: u4,
  active: bool,
  /// 0x100 or 256 bytes of ROM data
  data: [u8; 0x100],
  /// 4 bits of io ports
  pub ports: u4,
}

impl ROM {
  /// Create a new 4001 chip. Pass in the rom data.
  #[inline]
  pub fn new(data: [u8; 0x100], page_mask: u4) -> Self {
    Self {
      page_mask,
      active: page_mask == u4::new(0),
      data,
      ports: Default::default(),
    }
  }

  /// Clock A1, A2, A3 - Set address
  /// Clock M1, M2 - Send opcode
  pub fn get_opcode(&self, addr: super::Address) -> u8 {
    if self.page_mask == addr.chip_index() {
      let address = Address::DEFAULT
                  .with_low(addr.low())
                  .with_high(addr.high());
      self.data[address.raw_value() as usize]
    } else {
      0
    }
  }

  /// SRC. We only care about the high bit, which gets compared against the page mask.
  pub fn set_register_control(&mut self, data_in: super::Byte) {
    self.active = data_in.high() == self.page_mask;
  }

  /// Clock X2 - I/O instruction. It can send data back to the CPU
  pub fn io_read(&self, modifier: u4) -> u4 {
    if self.active && matches!(modifier.value(), 0xA) {
      trace!("RDR {}", self.ports); //Read ROM Port to CPU
      self.ports
    } else {
      u4::new(0)
    }
  }

  /// Clock X2 - SRC or I/O instruction. Input: High: modifier. Low: Value
  pub fn io_write(&mut self, data_in: super::Byte) {
    if self.active && matches!(data_in.high().value(), 0x2) {  //modifier
      trace!("WRR {}", data_in.low()); //Write ROM Port from CPU
      self.ports = data_in.low();
    } else {
      u4::new(0);
    }
  }

}
