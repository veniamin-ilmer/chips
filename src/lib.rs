//! Chips Emulation Crate
//!
//! All chips are written to run independent of each other.
//! Chips which require communicating with other chips will have an IO trait which needs to be implemented to use the chip.
//! Chips will only use the IO trait when you call the `run_cycle` method.
//! By breaking the chip into `run_cycle` vs non cycle methods, it allows us to avoid cyclical reference problems.
//! The CPU is never blocked. No mutex locks, no channel message passing, no atomics, no multithreading.
//! The user handles all communication between chips, essentially building the "Board" to which all chips communicate.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod shifter;
pub use shifter::Shifter;

mod rom;
pub use rom::ROM;

mod ram;
pub use ram::RAM;

pub mod i4001;
pub mod i4002;
pub mod i4003;
pub mod i4004;