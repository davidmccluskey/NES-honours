use crate::mapper::Mapper;

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

impl Mapper for Mapper0 {
    fn cpu_mapper_read(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr >= 0x8000 && addr <= 0xFFFF {
            if self.n_prg_banks > 1 
            {
                let mut m_addr = (addr & 0x7FFF) as u32;
                *mapped_addr = m_addr;
            }else
            {
                let mut m_addr = (addr & 0x3FFF) as u32;
                *mapped_addr = m_addr;
            }
            return true;
        }
        return false;
    }
    fn cpu_mapper_write(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr >= 0x0000 && addr <= 0xFFFF {
            return true;
        }
        return false;
    }

    fn ppu_mapper_read(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr >= 0x8000 && addr <= 0x1FFF {
            *mapped_addr = addr as u32;
            return true;
        }
        return false;
    }
    fn ppu_mapper_write(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        // if addr >= 0x0000 && addr <= 0x1FFF {
        //     return true;
        // }
        return false;
    }
}

