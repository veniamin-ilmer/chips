# HP Classic

The HP Classic chipset, from 1972, was used by the first HP calculators. These included the HP-35, HP-45, and several others.

Each instruction cycle ends up taking 280 microseconds. (3.671 kHz)

It was composed of several chips:

* Arithmetic and Registers chip - The ALU
* Control and Timing Chip - The Control Unit
* ROMs

The chips ran at 1 bit per clock cycle, though it can update up to 56 bits of a register, within one instruction cycle.

Each ROM word is 10 bits. Each address is 8 bits. Word selects are 14 bits. Each register is 56 bits (14 nibbles).

The ALU was only capable of doing BCD math, with 4 bits per digit. 9 + 1 = 0 with a carry, not 0xA.