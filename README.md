# Chips Emulator Crate

This is an integrated circuit emulator for various chips.

This crate has went through multiple revisions to include several seemingly conflicting features:

1. All chips can work independent of each other. You can potentially mix and match chips from different origins.
2. All chips can call functions outside of their own declarations. For example, a CPU can request data from the ROM or RAM chips.
3. The user handles all communication between chips, essentially building the "Board" to which all chips communicate.
4. The CPU is never blocked. No mutex locks, no channel message passing, no atomics, no multithreading.
5. This crate is framework independent. That includes no std.
6. As much as possible is done at compile time. There is no `Rc` nor `RefCell`.
7. Zero unsafe code.

In Rust, this was quite hard to implement because of the huge potential for cyclical references between the chips. Along the way I started to think it would be impossible to do in safe Rust.
But now I am proud to note, by carefully managing chip mutability, this crate manages to hit all of these points. As a consequence, Rust ends up compiling the code down to a tiny efficient binary.

Rather than handing individual chip pins, it focuses on the chip functionality, handling register data.

Extra methods are provided to read hidden registers which might not be available on a purely pin level.

It does not emulate any clock, syncing, or bus messaging. Chips which needs to run on an interval, will have a method `run_cycle` which you would need to call on each clock cycle.

This way, you have the flexibility to handle all synchronization, clock, and bus messaging, independently from the chip functionality.

All chips were built to communicate either purely through function calls or via IO trait functions.

## Supported Chips

* Intel MCS-4 (4001 ROM, 4002 RAM, 4003 Shifter, 4004 CPU)
* Fairchild F8 {3850 CPU, 3851 PSU, 3852 DMI)
* HP Classic (Control & Timing, Arithmetic & Register)
* TMS-0800
