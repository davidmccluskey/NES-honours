use crate::Mappers::mapper::{Mapper, Mirroring};

pub struct Mapper2 {
    n_prg_banks: u8,
    n_chr_banks: u8,
}

impl Mapper2 {
    pub fn new(prg_banks: u8, chr_banks: u8) -> Self {
        Mapper2 {
            n_prg_banks: prg_banks,
            n_chr_banks: chr_banks,
        }
    }
}

#[allow(unused_comparisons)]
impl Mapper for Mapper2 {
    fn cpu_mapper_read(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {

        return false;
    }
    fn cpu_mapper_write(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
 
        return false;
    }

    fn ppu_mapper_read(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {

        return false;
    }
    fn ppu_mapper_write(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {

        return false;
    }

    fn mirror(&mut self) -> Mirroring{
        return Mirroring::Hardware;
    }

    fn reset(&mut self){

    }
}

