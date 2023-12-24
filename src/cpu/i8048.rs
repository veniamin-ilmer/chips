//! The 8048 was released 1976.
//!
//! Each clock cycle was 90.91 nanoseconds (11 MHz). Each instruction took 1 - 2 clock cycles.
//! There is an onboard timer which tics every 80 microseconds. 80 microseconds / 90.91 nanoseconds = 880 cycles per tick.
//!
//! Useful links
//! * <https://devsaurus.github.io/mcs-48/mcs-48.pdf>
//! * <https://vtda.org/docs/computing/Intel/98-270B_MCS-48MicrocomputerUsersManualJul77.pdf>
//! * <https://github.com/Halicery/8042/blob/main/8042_INTERN.TXT>

/*
0000 0000 nop
0001 000r A = A + 1 - wrong
0010 000r xchg(A, [Rr])
0100 000r A = A | [Rr]
0101 000r A = A & [Rr]
0110 000r A = A + [Rr]
0111 000r A = A + [Rr] + C
1000 000r A = [Rr] (2 cycle) - External register
1001 000r [Rr] = A (2 cycle) - External register
1010 000r [Rr] = A
1011 000r iiiiiiii [Rr] = imm (2 cycle)
1111 000r A = [Rr]
0000 0010 BUS = A
0100 0010 A = T
0110 0010 T = A
bbb1 0010 aaaaaaaa jmp if bbb = 1
0000 0011 iiiiiiii A = A + imm (2 cycle)
0001 0011 iiiiiiii A = A + imm + C (2 cycle)
0010 0011 iiiiiiii A = imm
0100 0011 iiiiiiii A = A | imm (2 cycle)
0101 0011 iiiiiiii A = A & imm (2 cycle)
1000 0011 ret (2 cycle)
1001 0011 ret and pop psw (2 cycle)
1010 0011 jmp A? (2 cycle)
1011 0011 jmp A (2 cycle)
1110 0011 jmp A? (2 cycle)
aaa1 0100 aaaaaaaa jmp address (2 cycle)
aaa1 0100 aaaaaaaa call address (2 cycle)
0000 0101 Enable interrupts
0001 0101 Disable interrupts
0010 0101 Enable Timer interrupt
0011 0101 Disable timer interrupt
0100 0101 Start Timer
0110 0101 Stop Timer
0111 0101 Enable clock Output
1000 0101 F0 = 0
1001 0101 F0 = !F0
1010 0101 F1 = 0
1011 0101 F1 = !F1
1100 0101 Register Bank = 0
1101 0101 Register Bank = 1
1110 0101 Memory Bank = 0
1111 0101 Memory Bank = 1
0001 0110 aaaaaaaa jmp if tf (2 cycle)
0010 0110 aaaaaaaa jmp if not t0 (2 cycle)
0011 0110 aaaaaaaa jmp if t0 (2 cycle)
0100 0110 aaaaaaaa jmp if not t1 (2 cycle)
0101 0110 aaaaaaaa jmp if t1 (2 cycle)
0111 0110 aaaaaaaa jmp if f1 (2 cycle)
1000 0110 aaaaaaaa jmp if not interrupt input (2 cycle)
1001 0110 aaaaaaaa jmp if acc not 0 (2 cycle)
1011 0110 aaaaaaaa jmp if F0 (2 cycle)
1100 0110 aaaaaaaa jmp if acc 0 (2 cycle)
1110 0110 aaaaaaaa jmp if not carry (2 cycle)
1111 0110 aaaaaaaa jmp if carry (2 cycle)
0000 0111 A = A - 1
0001 0111 A = A + 1
0010 0111 A = 0
0011 0111 A = !A
0100 0111 xchg(low A, high A)
0101 0111 A = DAA(A)
0110 0111 A = ROR(A) and carry
0111 0111 A = ROR(A)
1001 0111 C = 0
1010 0111 C = !C
1100 0111 A = PSW
1101 0111 PSW = A
1110 0111 A = ROL(A)
1111 0111 A = ROL(A) and carry
0000 1000 A = BUS (2 cycle)
1000 1000 iiiiiiii BUS = BUS | imm (2 cycle)
1001 1000 iiiiiiii BUS = BUS & imm (2 cycle)
0001 1rrr Rr = Rr + 1
0010 1rrr xchg(A, Rr)
0100 1rrr A = A | Rr
0101 1rrr A = A & Rr
0110 1rrr A = A + Rr
0111 1rrr A = A + Rr + C
1010 1rrr Rr = A
1011 1rrr iiiiiiii Rr = imm (2 cycle)
1100 1rrr Rr = Rr - 1
1110 1rrr aaaaaaaa Rr = Rr - 1, jnz addr (2 cycle)
1111 1rrr A = Rr
0000 10pp A = Pp (2 cycle) (ports 1 or 2)
0011 10pp Pp = A (ports 1 or 2)
1000 10pp iiiiiiii Pp = Pp | imm (2 cycle) (ports 1 or 2)
1001 10pp iiiiiiii Pp = Pp & imm (2 cycle) (ports 1 or 2)
0000 11pp A = Pp (2 cycle) (ports 4 to 7)
1000 11pp Pp = Pp | A (2 cycle) (ports 4 to 7)
1001 11pp Pp = Pp & A (2 cycle) (ports 4 to 7)

*/