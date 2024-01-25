# TMS0100

The TMS0100 was the first "calculator on a chip" designed by Texas Instruments in 1971.

I am currently reverse engineering the instruction set using this patent - https://patentimages.storage.googleapis.com/4b/13/d2/5c4391af1c98a1/USH1970.pdf - And comparing it to the TMS0800 instruction set.

This is a bit difficult, because it is possible all of these opcodes are actually configurable via PLA.

But I am trying to at least recreate the variant in the patent, and then go from there.

## Keyboard

Type | D11 | D10 | D9 | D8 | D7 | D6 | D5 | D4 | D3 | D2 | D1
-----|-----|-----|----|----|----|----|----|----|----|----|----
 KN  |     |  0  |  9 |  8 |  7 |  6 |  5 |  4 |  3 |  2 | 1
 KO  |  C  |  Cl |  . | =  | +/-| -= | += | -  |  ÷ |  x | +
 KP  |     |  0  |    |    |  7 |  6 |  5 |  4 |  3 |  2 | 1
 KQ  |     |  K  |    |    |    |    |    |    |    |    |

## Mask

MMMM | ASM
-----|----
0000 | ALL
0001 | EXP
0010 | MONT / M19
0011 | LSD1
0100 | M11
0101 | MSD1 / M81
0110 | EXP1
0111 | DPT1
1000 | DPT7
1001 | EXP7
1010 | Compare flags
1011 | Exchange flags
1100 | Set flags
1101 | Reset flags
1110 | Toggle flags
1111 | Test flags

Code   | Shift
-------|------
000101 | SLLA
001010 | SLLB
001111 | SLLC
100101 | SRLA
101010 | SRLB
101111 | SRLC

100X0110010

It seems shifts only happen when M = MONT

Code        | Meaning
------------|--------
XXXXXXXXX00 | Compare. Not equal to anything
XXXXXXXXX01 | A =
XXXXXXXXX10 | B =
XXXXXXXXX11 | C =
XXXXX0000XX | K
XXXXX0001XX | A << 4 or A + K
XXXXX0010XX | B << 4 or B + K
XXXXX0011XX | C << 4 or C + K
XXXXX0100XX | A + B
XXXXX0101XX | C + B
XXXXX0110XX | Exchange A, B
XXXXX0111XX | SPWD
XXXXX1000XX | SCAN / SOCN / WD11 / KQCD / DPTA - Some kind of "wait" is here
XXXXX1001XX | A >> 4 or A - K
XXXXX1010XX | B >> 4 or B - K
XXXXX1011XX | C >> 4 or C - K
XXXXX1100XX | A - B
XXXXX1101XX | C - B


Instructions are encoded as:

J/I | Md | Mc | Mb | Ma | Sub | Rc | Rb | Ra | Σb | Σa
----|----|----|----|----|-----|----|----|----|----|----
Jump=0<br/>Instruction = 1 | Mask | Mask | Mask | Mask | Add = 0<br />Subtract = 1 | Arg | Arg | Arg | Where<br/>return | Where<br/>return

### Operation decoder (From Fig 17W)

J/I | Ma | Mb | Mc | Md | Ra | Rb | Rc | Σa | Σb | Sub | Operation   | Result
----|----|----|----|----|----|----|----|----|----|-----|-------------|-----------
 1  | 0  | X  | 0  | 0  | 1  | 0  | 0  | 1  |  0 | 1   | Shift Right | A = A >> 4
 1  | 0  | X  | 0  | 0  | 0  | 1  | 0  | 0  |  1 | 1   | Shift Right | B = B >> 4
 1  | 0  | X  | 0  | 0  | 1  | 1  | 0  | 1  |  1 | 1   | Shift Right | C = C >> 4
 1  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  |  0 | 0   | ININ        | WD11
 1  | 1  | 1  | 0  | 0  | 0  | 0  | 0  | 1  |  0 | 0   | ININ & I    | ???
 1  | 1  | 1  | 0  | 0  | 0  | 0  | 0  | 1  |  0 | 1   | ININ & II   | SOCN
 1  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  |  0 | 1   | ININ & KPCD | ???
 1  | 1  | 0  | 0  | 0  | 0  | 0  | 0  | 0  |  0 | 1   | ININ & KNCD & KOCD | SCAN
 
### Σ Decoder (from Fig 17O, reading from left to right)

Where to Return | Σb | Σa 
-------|----|----
 X     | 0  | 0  
 C     | 1  | 1  
 A     | 0  | 1  
 B     | 1  | 0  

### R Decoder (from Fig 17U, reading from left to right):

 Rc | Rb | Ra | EX | BV | AU | CU | WAIT | Discovered Function
----|----|----|----|----|----|----|------|--------------------
 0  | 0  | 1  |    |    | ✓  |    |      | A is first argument
 0  | 1  | 0  |    | ✓  |    |    |      | B is second argument
 0  | 1  | 1  |    |    |    | ✓  |      | C is first arument
 1  | 0  | 0  |    | ✓  | ✓  |    |      | A is first arg, B is second arg
 1  | 0  | 1  |    | ✓  |    | ✓  |      | C is first arg, B is second arg
 1  | 1  | 1  |    |    |    |    |  ✓   | SPWD
 1  | 0  | 1  | ✓  |    |    |    |      | Exchange A and B

## Instruction Set

* A = Register A
* B = Register B
* C = Register C
* D = Internal shifting bit register
* K = Constant
* FA = Flag A
* FB = Flag B


Word         | ASM  | Explanation
-------------|------|--------------------------
00AAA AAAAAA | BO A | Jump to Addr
01AAA AAAAAA | BZ A | Jump to Addr
10000 000000 | WD11 | ???
10010 000000 | KQCD | Something about the keyboard being KQ?
10111 000001 | DPTA | ???
1MMMM 000001 | CLA M | A = K
1MMMM 000010 | MSDB | B = K
1MMMM 000011 | CLC M | C = K
MONT 000101 | SLLA MONT | A = A << 4 
1MMMM 000101 | AAKA M | A = A + K
1MMMM 000110 | AAKB M | B = A + K
1MMMM 000111 | AAKC M | C = A + K
MONT 001010 | SLLB MONT | B = B << 4
1MMMM 001011 | ABKC M | C = B + K
1MMMM 001011 | ABKA M | A = B + K
1MMMM 001101 | ACKA M | A = C + K
1MMMM 001110 | ACKB M | B = C + K
1MMMM 001111 | ACKC M | C = C + K
MONT 001111 | SLLC MONT | C = C << 4
1MMMM 010001 | AABA M | A = A + B
1MMMM 010010 | AABB M | B = A + B
1MMMM 010011 | AABC M | C = A + B
1MMMM 011000 | EXAB M | Exchange A and B
10000 011100 | SPWD | Wait until the D is reset
10001 100000 | SCAN | Wait until the D is reset
10011 100001 | SOCN | ???
1MMMM 100100 | CAK M | A - K? Set cond if borrow
10010 100101 | SRLA MONT | A = A >> 4
1MMMM 100101 | SAKA M | A = A - K
1MMMM 100110 | SAKB M | B = A - K
1MMMM 100111 | SAKC M | C = A - K
10010 101010 | SRLB MONT | B = B >> 4
1MMMM 101100 | CCK M | C - K? Set cond if borrow
1MMMM 101101 | SCKA M | A = C - K
1MMMM 101110 | SCKB M | B = C - K
1MMMM 101111 | SCKC M | C = C - K
10010 101111 | SRLC MONT | C = C >> 4
1MMMM 110000 | CAB M | A - B? Set cond if borrow
1MMMM 110001 | SABA M | A = A - B
1MMMM 110010 | SABB M | B = A - B
1MMMM 110011 | SABC M | C = A - B
1MMMM 110100 | CCB M | C - B? Set cond if borrow
1MMMM 110101 | SCBA M | A = C - B
1MMMM 110110 | SCBB M | B = C - B
1MMMM 110111 | SCBC M | C = C - B
11010 00XXXX | CFA X | FA{i} != FB{i}? - set borrow if true
11011 00XXXX | XFA X | Exchange FA and FB
11100 00XXXX | SFA X | FA bits = True
11100 10XXXX | SFB X | FB bits = True
11101 00XXXX | ZFA X | FA bits = False
11101 10XXXX | ZFB X | FB bits = False
11110 00XXXX | FFA X | FA = !FA
11110 10XXXX | FFB X | FB = !FB
11111 00XXXX | TFA X | Test if FA bits are True
11111 10XXXX | TFB X | Test if FB bits are True