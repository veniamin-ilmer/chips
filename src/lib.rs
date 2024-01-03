//! Chips Emulation Crate
//!
//! All chips are written to run independent of each other.
//!
//! Chips which require communicating with other chips will have an IO trait which needs to be implemented to use the chip.
//!
//! Chips will only use the IO trait when you call the `run_cycle` method.
//!
//! By breaking the chip into `run_cycle` vs non cycle methods, it allows us to avoid cyclical reference problems.
//!
//! The CPU is never blocked. No mutex locks, no channel message passing, no atomics, no multithreading.
//!
//! The user handles all communication between chips, essentially building the "Board" to which all chips communicate.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod and; pub use and::AND;
pub mod or; pub use or::OR;
pub mod nand; pub use nand::NAND;
pub mod decoder; pub use decoder::SegmentDecoder;
pub mod counter; pub use counter::Counter;
pub mod shifter; pub use shifter::Shifter;
pub mod rom; pub use rom::ROM;
pub mod ram; pub use ram::RAM;
pub mod cpu;
//pub mod memory_pointer; pub use memory_pointer::MemoryPointer;
pub mod indexer; pub use indexer::{Indexer16, Indexer64};

/// Generic ROM / RAM read operations
pub trait ReadArr {
  /// Generic Read for ROM / RAM. Allows you to read u8, u16, etc
  fn read(data: &[u8]) -> Self;
}
impl ReadArr for u8 {
  fn read(data: &[u8]) -> Self {
    data[0]
  }
}
impl ReadArr for u16 {
  fn read(data: &[u8]) -> Self {
    u16::from_le_bytes([data[0], data[1]])
  }
}

/// Generic ROM / RAM write operations
pub trait WriteArr {
  /// Generic write for ROM / RAM. Allows you to write u8, u16, etc
  fn write(data: &mut [u8], value: Self);
}
impl WriteArr for u8 {
  fn write(data: &mut [u8], value: Self) {
    data[0] = value
  }
}
impl WriteArr for u16 {
  fn write(data: &mut [u8], value: Self) {
    let bytes = value.to_le_bytes();
    data[0] = bytes[0];
    data[1] = bytes[1];
  }
}