pub struct StrBus {
  pub ram: [u8; 64 * 1028], //System memory
  //cpu: CPU6502,         //Reference to CPU
}

#[allow(non_snake_case)]
impl StrBus{
  pub fn new() -> StrBus {
    StrBus{
      ram:[0; 64 * 1028],
      //cpu,
    }
  }

  //Function to write to RAM
  pub fn write(&mut self, addr: u16, data: u8, ){
    if addr >= 0x0000 && addr <= 0xFFFF {
      self.ram[addr as usize] = data;
    }
  }

  //Function to read from RAM
  pub fn read(&mut self, addr: u16, _readOnly: bool) -> u8 {
    if addr >= 0x0000 && addr <= 0xFFFF {
      return self.ram[addr as usize];
    }
    return 0x00;
  }
}



