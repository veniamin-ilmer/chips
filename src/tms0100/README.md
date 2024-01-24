# TMS0100

The TMS0100 was the first "calculator on a chip" designed by Texas Instruments in 1971.

I am currently reverse engineering the instruction set using this patent - https://patentimages.storage.googleapis.com/4b/13/d2/5c4391af1c98a1/USH1970.pdf - And comparing it to the TMS0800 instruction set.

This is a bit difficult, because it is possible all of these opcodes are actually configurable via PLA.

But I am trying to at least recreate the variant in the patent, and then go from there.

* A = Register A
* B = Register B
* C = Register C
* D = Internal shifting bit register
* K = Constant
* FA = Flag A
* FB = Flag B


Word        | ASM | Explanation
------------|-----|--------------------------
10000000001 | CLA ALL | A = 0
10000000011 | CLC ALL | C = 0
10000000111 | AAKC ALL | C = A + K
10000011000 | EXAB ALL | Exchange A and B
10000011100 | SPWD | Wait unit the D is reset
10001010011 | AABC EXP | C = A + B
10010010001 | AABA MONT | A = A + B
10110000101 | AAKA EXP1 | A = A + K
10111000101 | AAKA DPT1 | A = A + K
10000001101 | ACKA ALL | A = C + K
10000001011 | ABKA ALL | A = B + K
10001110000 | CAB EXP | A - B? Set cond if borrow
10010110000 | CAB MONT | A - B? Set cond if borrow
10010110000 | CAB M19 | A - B? Set cond if borrow
10100100100 | CAK M11 | A - K? Set cond if borrow
11000100100 | CAK DPT7 | A - K? Set cond if borrow
10101100100 | CAK M81 | A - K? Set cond if borrow
10110100100 | CAK EXP1 | A - K? Set cond if borrow
10110101111 | SCKC EXP1 | C = C - K
10001110011 | SABC EXP | C = A - B
10010110001 | SABA MONT | A = A - B
10110100101 | SAKA EXP1 | A = A - K
10010100101 | SRLA MONT | A = A >> 4
10010101010 | SRLB MONT | B = B >> 4
10010000101 | SLLA MONT | A = A << 4
10010001010 | SLLB MONT | B = B << 4
1101000XXXX | CFA X | FA{i} != FB{i}? - set borrow if true
1101100XXXX | XFA X | Exchange FA and FB
1110000XXXX | SFA X | FA bits = True
1110010XXXX | SFB X | FB bits = True
1110100XXXX | ZFA X | FA bits = False
1110110XXXX | ZFB X | FB bits = False
1111000XXXX | FFA X | FA = !FA
1111010XXXX | FFB X | FB = !FB
1111100XXXX | TFA X | Test if FA bits are True
1111110XXXX | TFB X | Test if FB bits are True
