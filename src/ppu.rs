use crate::cartridge::Cartridge;
use std::vec::Vec;
use std::cell::RefCell;
use std::rc::Rc;
pub struct PPU{
    pub cartridge: Option<Rc<RefCell<Cartridge>>>,
    pub nameTable:  [[u8; 2];1024],
    pub palette: [u8; 32],
    //pub patternTable:  [[u8; 2];4096],
}

impl<'a> PPU{

    pub fn new() -> PPU{
        PPU{
            cartridge: None,
            nameTable:[[0; 2]; 1024],
            //patternTable:[[0; 2]; 4096],
            palette:[0; 32],

        }
    }

    pub fn cpuRead(&mut self, addr: u16, _readOnly: bool) -> u8 {
        let data: u8 = 0x00;
        match addr {
            0x0000 => (), //Control
            0x0001 => (), //Mask
            0x0002 => (), //Status
            0x0003 => (), //OAM Address
            0x0004 => (), //OAM Data
            0x0005 => (), //Scroll
            0x0006 => (), //PPU Address
            0x0007 => (), //PPU Data

            _ => (), //required by rust
        }
        return 0;
    }
    pub fn cpuWrite(&mut self, addr: u16, data: u8){
        let data: u8 = 0x00;
        match addr {
            0x0000 => (), //Control
            0x0001 => (), //Mask
            0x0002 => (), //Status
            0x0003 => (), //OAM Address
            0x0004 => (), //OAM Data
            0x0005 => (), //Scroll
            0x0006 => (), //PPU Address
            0x0007 => (), //PPU Data

            _ => (), //required by rust
        }
    }
    pub fn ppuRead(&mut self, mut addr: u16, _readOnly: bool) -> u8{
        let mut data:u8 = 0x00;
        addr &= 0x3FFF;
        if let Some(ref c) = self.cartridge 
        {
            if c.borrow_mut().ppu_read(addr, &mut data){

            }
        }

        return data;
    }
    pub fn ppuWrite(&mut self, mut addr: u16, data: &mut u8){
        addr &= 0x3FFF;
        if let Some(ref c) = self.cartridge 
        {
            if c.borrow_mut().ppu_write(addr, data){

            }
        }
    }
    pub fn connectCartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>){
        self.cartridge = Some(cartridge);
    }
    
    pub fn clock(&mut self){

    }
}