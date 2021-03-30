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
    fn cpu_mapper_read(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr >= 0x8000 && addr <= 0xFFFF {
            if self.n_prg_banks > 1 
            {
                let m_addr = (addr & 0x7FFF) as u32;
                *mapped_addr = m_addr;
            }else
            {
                let m_addr = (addr & 0x3FFF) as u32;
                *mapped_addr = m_addr;
            }
            return true;
        }
        return false;
    }
    fn cpu_mapper_write(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr >= 0x8000 && addr <= 0xFFFF {
            if self.n_prg_banks > 1 
            {
                let m_addr = (addr & 0x7FFF) as u32;
                *mapped_addr = m_addr;
            }else
            {
                let m_addr = (addr & 0x3FFF) as u32;
                *mapped_addr = m_addr;
            }
            return true;
        }
        return false;
    }

    fn ppu_mapper_read(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr >= 0x0000 && addr <= 0x1FFF {
            *mapped_addr = addr as u32;
            return true;
        }
        return false;
    }
    fn ppu_mapper_write(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr >= 0x0000 && addr <= 0x1FFF {
            if self.n_chr_banks == 0 {
                *mapped_addr = addr as u32;
                return true;
            }
        }
        return false;
    }

    fn mirror(&mut self) -> Mirroring{
        return Mirroring::Hardware;
    }

    fn reset(&mut self){

    }
}

