#[derive(Default, Copy, Clone, PartialEq)]
pub struct Register {
  pub data: u56,
}

impl Register {
  pub fn get_nibble(&self, is_left: bool) -> u4 {
    if is_left {
      u4::new((self.data.value() >> 52) as u8)
    } else {
      u4::new((self.data.value() as u8) & 0xF)
    }
  }
  pub fn rotate_with_nibble(&mut self, nibble: u4, is_left: bool) {
    self.data = if is_left {
      (self.data << 4) | u56::new(nibble.value() as u64)
    } else {
      (self.data >> 4) | u56::new((nibble.value() as u64) << 52)
    };
  }
}