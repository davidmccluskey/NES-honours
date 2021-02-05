use crate::ppu::PPU; 
use crate::cartridge::Cartridge;
use std::cell::RefCell;
use std::rc::Rc;
pub struct Bus {
  pub ram: [u8; 2048], //System memory
  pub ppu: PPU,
  pub system_clock: u32,
  pub cartridge: Option<Rc<RefCell<Cartridge>>>,

  pub controller: [u8; 2],
  pub controller_state: [u8; 2],
  pub nmi_required: bool,
  
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
      controller: [0; 2],
      controller_state: [0; 2],
      nmi_required: false,
      //cpu,
    }
  }

  //Function to write to RAM
  #[allow(unused_comparisons)]
  pub fn cpu_write(&mut self, addr: u16, data: &mut u8, )
  {
    if let Some(ref c) = self.cartridge 
    {
        if c.borrow_mut().cpu_write(addr, data){

        }    
        else if addr >= 0x0000 && addr <= 0x1FFF 
        {
          self.ram[(addr & 0x07FF) as usize] = *data;
        }
        else if addr >= 0x2000 && addr <= 0x3FFF
        {
          self.ppu.cpu_write(addr & 0x0007, data);
        }
        else if addr >= 0x4016 && addr <= 0x4017{
          //let addru8 = addr & 0x0001;
          self.controller_state[0] = self.controller[0];
        }
    }

  }

  //Function to read from RAM
  #[allow(unused_comparisons)]
  pub fn cpu_read(&mut self, addr: u16, read_only: bool) -> u8 {
    let mut data: u8 = 0x00;

    if let Some(ref c) = self.cartridge 
    {
      if c.borrow_mut().cpu_read(addr, &mut data){

      } 
      else if addr >= 0x0000 && addr <= 0x1FFF 
      {
        data = self.ram[(addr & 0x07FF) as usize];
      }
      else if addr >= 0x2000 && addr <= 0x3FFF
      {
        data = self.ppu.cpu_read(addr & 0x0007, read_only);
      }
      else if addr >= 0x4016 && addr <= 0x4017 {
        data = ((self.controller_state[0] & 0x08) > 0) as u8;
        self.controller_state[0] <<= 1;
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


    if self.ppu.nmi_enabled{
      self.ppu.nmi_enabled = false;
      self.nmi_required = true;
  }
  }

  pub fn reset(&mut self){
    self.system_clock = 0;
    self.ppu.reset();
  }
}



