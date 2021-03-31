use crate::Mappers::mapper::{Mapper, Mirroring};

pub struct Mapper0 {
    n_prg_banks: u8,
    n_chr_banks: u8,
}

impl Mapper0 {
    pub fn new(prg_banks: u8, chr_banks: u8) -> Self {
        Mapper0 {
            n_prg_banks: prg_banks,
            n_chr_banks: chr_banks,
        }
    }
}

#[allow(unused_comparisons)]
impl Mapper for Mapper0 {
    fn cpu_mapper_read(&mut self, address: u16, mapped_address: &mut u32, data: &mut u8) -> bool {
        if address >= 0x8000 && address <= 0xFFFF {
            if self.n_prg_banks > 1 
            {
                let m_address = (address & 0x7FFF) as u32;
                *mapped_address = m_address;
            }else
            {
                let m_address = (address & 0x3FFF) as u32;
                *mapped_address = m_address;
            }
            return true;
        }
        return false;
    }
    fn cpu_mapper_write(&mut self, address: u16, mapped_address: &mut u32, data: &mut u8) -> bool {
        if address >= 0x8000 && address <= 0xFFFF {
            if self.n_prg_banks > 1 
            {
                let m_address = (address & 0x7FFF) as u32;
                *mapped_address = m_address;
            }else
            {
                let m_address = (address & 0x3FFF) as u32;
                *mapped_address = m_address;
            }
            return true;
        }
        return false;
    }

    fn ppu_mapper_read(&mut self, address: u16, mapped_address: &mut u32) -> bool {
        if address >= 0x0000 && address <= 0x1FFF {
            *mapped_address = address as u32;
            return true;
        }
        return false;
    }
    fn ppu_mapper_write(&mut self, address: u16, mapped_address: &mut u32) -> bool {
        if address >= 0x0000 && address <= 0x1FFF {
            if self.n_chr_banks == 0 {
                *mapped_address = address as u32;
                return true;
            }
        }
        return false;
    }
    fn reset(&mut self){

    }
    fn mirror(&mut self) -> Mirroring{
        return Mirroring::Hardware;
    }
}

