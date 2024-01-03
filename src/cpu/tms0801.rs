impl TMS0801 {
  pub fn run_cycle(&mut self, opcode: u11) {
    let class = opcode.value() >> 8;
    match class
    {
      0 => todo!("Jump address if condition reset"),
      1 => todo!("Jump address if condition set"),
      2 => {
      },
      _ => {  //Register Instruction
        let mask = opcode.value() & 0xF;
        let instruction = (opcode.value() >> 4) & 0b11111;
        match instruction {
          0x00 => trace!("A = A + B (dec)"),
          0x01 => trace!("A = A + K (dec)"),
          0x02 => trace!("C = A + K (dec)"),
          0x03 => trace!("A = B"),
          0x04 => trace!("C = B"),
          0x05 => trace!("A = C + K (dec)"),
          0x06 => trace!("B = C + K (dec)"),
          0x07 => trace!("A = A - B (dec)"),
          0x08 => trace!("C = A - B (dec)"),
          0x09 => trace!("A = A - K (dec)"),
          0x0A => trace!("C = C - B (dec)"),
          0x0B => trace!("C = C - K (dec)"),
          0x0C => trace!("? = A - B (dec)"),
          0x0D => trace!("? = A - K (dec)"),
          0x0E => trace!("? = C - B (dec)"),
          0x0F => trace!("? = C - K (dec)"),
          0x10 => trace!("A = K"),
          0x11 => trace!("B = K"),
          0x12 => trace!("C = K"),
          0x13 => trace!("XCHG A, B"),
          0x14 => trace!("A = A << 1"),
          0x15 => trace!("B = B << 1"),
          0x16 => trace!("C = C << 1"),
          0x17 => trace!("A = A >> 1"),
          0x18 => trace!("B = B >> 1"),
          0x19 => trace!("C = C >> 1"),
          0x1A => todo!("Weird AKCN"),
          0x1B => trace!("A = A + K (hex)"),
          0x1C => trace!("A = A - K (hex)"),
          0x1D => trace!("C = C + K"),
          _ => unimplemented!("Unknown command: {}", opcode.value()),
        }
      }
    }
  }
}