# TMS0800

The TMS0800 was introduced in 1973 as a child of the founding TMS0100.

It was a "calculator on a single chip", including the RAM, ROM, Control Unit, ALU, and several Programmable Logic Arrays (PLA).

The PLA is like a configuration used to tell how certain instructions should work.

This emulation reads both the ROM and PLA.

PLAs include ALU instructions, word select, and constants.

## Bits and Words

The TMS0800 was build with a 8 μm process node, with roughly 5000 transistors.

Each instruction takes up taking 264 microseconds.

Each ROM word is 11 bits. Each memory address is 9 bits. All instructions take up only one word each.

Each Register is 44 bits. (11 nibbles).

There is a word select / mask, taking up 11 bits. Notice, they do everything in 11s here..

The original chips ran at 1 bit per clock cycle, though it can update up to 44 bits of a register, within one instruction cycle.

The run_cycle command runs per instruction, not per clock, however inside it emulates much of the shift registers that exist in the chip.