pub struct ROM<const LENGTH: usize> {
  data: [u8; LENGTH],
}

impl<const LENGTH: usize> ROM<LENGTH> {
  #[inline]
  pub fn new(data: [u8; LENGTH]) -> Self {
    Self {
      data,
    }
  }
  #[inline]
  pub fn read_byte(&self, addr: usize) -> u8 {
    self.data[addr]
  }
}