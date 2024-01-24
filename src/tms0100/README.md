# TMS0100

The TMS0100 was the first "calculator on a chip" designed by Texas Instruments in 1971.

I am currently reverse engineering the instruction set using this patent - https://patentimages.storage.googleapis.com/4b/13/d2/5c4391af1c98a1/USH1970.pdf - And comparing it to the TMS0800 instruction set.

This is a bit difficult, because it is possible all of these opcodes are actually configurable via PLA.

But I am trying to at least recreate the variant in the patent, and then go from there.

Decoded instructions:

Word        | ASM | Explanation
------------|-----|--------------------------
10000000001 | CLA | Set reg A to 0
10000000011 | CLC | Set reg C to 0
1000001XXXX | EXAB | Exchange reg A and reg B
10000011100 | SPWD | Wait unit the D register is reset
1110000XXXX | SFA | Set A flag bits to True
1110010XXXX | SFB | Set B flag bits to True
1110100XXXX | ZFA | Set A flag bits to False
1110110XXXX | ZFB | Set B flag bits to False
1111000XXXX | FFA | A flag = !A flag
1111010XXXX | FFB | B flag = !B flag
1111100XXXX | TFA | Test if A flag bits are True
1111110XXXX | TFB | Test if B flag bits are True
