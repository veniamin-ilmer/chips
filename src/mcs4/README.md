# MCS-4 (4 bit Micro Computer System) - Intel 4000 family

Produced in 1971, containing 2300 transistors, with a 10 μm process node.

Each clock cycle was 1.35 microseconds (740 kHz). Each instruction took 8 - 16 clock cycles. So, instructions took 10.8 - 21.6 microseconds (46.3 - 92.6 kHz).

Although this emulation closely emulates the instruction cycle, it does NOT imulate the individual clock cycles.

It does emulate each instruction cycle. Instructions which take two instruction cycles will take really take two cycles to run. To accomplish this, the chip remembers the previous state.