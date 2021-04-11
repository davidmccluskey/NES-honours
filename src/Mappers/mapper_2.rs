use crate::Mappers::mapper::{Mapper, Mirroring};

pub struct Mapper2 {
    n_prg_banks: u8,
    n_chr_banks: u8,

    prg_bank_low: u8,
    prg_bank_high: u8,
}

impl Mapper2 {
    pub fn new(prg_banks: u8, chr_banks: u8) -> Self {
        Mapper2 {
            n_prg_banks: prg_banks,
            n_chr_banks: chr_banks,

            prg_bank_low: 0x00,
            prg_bank_high: 0x00,
        }
    }
}

#[allow(unused_comparisons)]
impl Mapper for Mapper2 {
    fn cpu_mapper_read(&mut self, address: u16, mapped_address: &mut i32, data: &mut u8) -> bool {

        if address >= 0x8000 && address <= 0xBFFF
        {
            *mapped_address = (self.prg_bank_low as i32 * 0x4000) + (address as i32 & 0x3FFF);
            return true;
        }
    
        if address >= 0xC000 && address <= 0xFFFF
        {
            *mapped_address = (self.prg_bank_high as i32 * 0x4000) + (address as i32 & 0x3FFF);
            return true;
        }
        return false;
    }
    fn cpu_mapper_write(&mut self, address: u16, mapped_address: &mut i32, data: &mut u8) -> bool {
        
        if address >= 0x8000 && address <= 0xFFFF
        {
            self.prg_bank_low = *data & 0x0F;
        }
        return false;
    }

    fn ppu_mapper_read(&mut self, address: u16, mapped_address: &mut u32) -> bool {

        if address < 0x2000
        {
            *mapped_address = address as u32;
            return true;
        }
        else
        {
            return false;
        }
    }
    fn ppu_mapper_write(&mut self, address: u16, mapped_address: &mut u32) -> bool {

        if address < 0x2000
        {
            if self.n_chr_banks == 0 
            {
                *mapped_address = address as u32;
                return true;
            }
        }
        return false;
    }

    fn reset(&mut self)
    {
        self.prg_bank_low = 0;
        self.prg_bank_high = self.n_prg_banks - 1;
    }
    fn mirror(&mut self) -> Mirroring{
        return Mirroring::Hardware;
    }
}

