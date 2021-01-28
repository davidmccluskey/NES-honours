use crate::ppu::PPU; 
use crate::cartridge::Cartridge;
use std::cell::RefCell;
use std::rc::Rc;
pub struct Bus {
  pub ram: [u8; 2048], //System memory
  pub ppu: PPU,
  pub system_clock: u32,
  pub cartridge: Option<Rc<RefCell<Cartridge>>>,
  //cpu: CPU6502,         //Reference to CPU
}

impl Bus{
  pub fn new() -> Bus {
    let ppu = PPU::new();
    Bus{
      ram:[0; 2048],
      ppu,
      system_clock: 0,
      cartridge: None,
      //cpu,
    }
  }

  //Function to write to RAM
  pub fn cpuWrite(&mut self, addr: u16, data: &mut u8, )
  {
    if let Some(ref c) = self.cartridge 
    {
        if c.borrow_mut().cpu_write(addr, data){

        }    
        else if addr >= 0x0000 && addr <= 0x1FFF 
        {
          self.ram[addr as usize & 0x07FF] = *data;
        }
        else if addr >= 0x2000 && addr <= 0x3FFF
        {
          self.ppu.cpu_write(addr & 0x0007, data)
        }
    }

  }

  //Function to read from RAM
  pub fn cpuRead(&mut self, addr: u16, _readOnly: bool) -> u8 {
    let mut data: u8 = 0x00;

    if let Some(ref c) = self.cartridge 
    {
      if c.borrow_mut().cpu_read(addr, &mut data){

      } 
      else if addr >= 0x0000 && addr <= 0x1FFF 
      {
        data = self.ram[addr as usize & 0x07FF];
      }
      else if addr >= 0x2000 && addr <= 0x3FFF
      {
        data = self.ppu.cpu_read(addr & 0x0007, _readOnly)
      }
    }

    return data;
  }

  pub fn connect_cartridge(&mut self, cartridge:  Rc<RefCell<Cartridge>>){
    self.cartridge = Some(cartridge.clone());
    self.ppu.connect_cartridge(cartridge.clone());
  } 
  pub fn clock(&mut self){
    self.ppu.clock();
    self.ppu.clock();
    self.ppu.clock();

  }

  pub fn reset(&mut self){
    self.system_clock = 0;
  }
}



