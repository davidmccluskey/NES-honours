use crate::Mappers::mapper::{Mapper, Mirroring};

pub struct Mapper1 {
    n_prg_banks: u8,
    n_chr_banks: u8,
    
    chr_bank_low_4: u8,
    chr_bank_high_4: u8,
    chr_banK_full_8: u8,

    prg_bank_low_16: u8,
    prg_bank_high_16: u8,
    prg_bank_full_32: u8,

    load_register: u8,
    control_register: u8,
    count: u8,

    mirror: Mirroring,
    ram: Vec<u8>,
}

impl Mapper1 {
    pub fn new(prg_banks: u8, chr_banks: u8) -> Self {
        let mut ram: Vec<u8> = Vec::new();
        ram.resize(32768, 0);
        Mapper1 {
            n_prg_banks: prg_banks,
            n_chr_banks: chr_banks,
            chr_bank_low_4: 0,
            chr_bank_high_4: 0,
            chr_banK_full_8: 0,
        
            prg_bank_low_16: 0,
            prg_bank_high_16: 0,
            prg_bank_full_32: 0,
        
            load_register: 0,
            control_register: 0,
            count: 0,
            mirror: Mirroring::Horizontal,
            ram,
        }
    }
}

#[allow(unused_comparisons)]
impl Mapper for Mapper1 {
    fn cpu_mapper_read(&mut self, address: u16, mapped_address: &mut u32, data: &mut u8) -> bool {
        if address >= 0x6000 && address <= 0x7FFF 
        {
            *mapped_address = 0xFFFFFFFF as u32;
            *data = self.ram[(address & 0x1FFF) as usize];
            return true;
        }
    
        if address >= 0x8000
        {
            if (self.control_register & 0b01000) > 0 
            {
                if address >= 0x8000 && address <= 0xBFFF
                {
                    *mapped_address = (self.prg_bank_low_16 as u32 * 0x4000) + (address as u32 & 0x3FFF);
                    return true;
                }
    
                if address >= 0xC000 && address <= 0xFFFF
                {
                    *mapped_address = (self.prg_bank_high_16 as u32 * 0x4000) + (address as u32 & 0x3FFF);
                    return true;
                }
            }
            else
            {
                *mapped_address = self.prg_bank_full_32 as u32 * 0x8000 + (address as u32 & 0x7FFF);
                return true;
            }
        }
        return false;
    }
    fn cpu_mapper_write(&mut self, address: u16, mapped_address: &mut u32, data: &mut u8) -> bool {
        if address >= 0x6000 && address <= 0x7FFF
        {
            *mapped_address = 0xFFFFFFFF;
            self.ram[(address & 0x1FFF) as usize] = *data;
            return true;
        }
        if address >= 0x8000
        {
            if (*data & 0x80) > 0
            {
                self.load_register = 0x00;
                self.count = 0;
                self.control_register = self.control_register | 0x0C;
            }
            else
            {
                self.load_register >>= 1;
                self.load_register |= (*data & 0x01) << 4;
                self.count += 1;
    
                if self.count == 5
                {
                    let target = (address >> 13) & 0x03;
    
                    if target == 0
                    {
                        self.control_register = self.load_register & 0x1F;

                        let pattern = self.control_register & 0x03;
                        match pattern {
                            2 => {self.mirror = Mirroring::Vertical},
                            3 => {self.mirror = Mirroring::Hardware},
                            _ => {panic!("Bad address")},
                        }
                    }
                    else if target == 1
                    {
                        if (self.control_register & 0b10000) > 0 
                        {
                            self.chr_bank_low_4 = self.load_register & 0x1F;
                        }
                        else
                        {
                            self.chr_banK_full_8 = self.load_register & 0x1E;
                        }
                    }
                    else if target == 2
                    {
                        if (self.control_register & 0b10000) > 0
                        {
                            self.chr_bank_high_4 = self.load_register & 0x1F;
                        }
                    }
                    else if target == 3
                    {
                        let mode = (self.control_register >> 2) & 0x03;
    
                        if mode == 0 || mode == 1
                        {
                            self.prg_bank_full_32 = (self.load_register & 0x0E) >> 1;
                        }
                        else if mode == 2
                        {
                            self.prg_bank_low_16 = 0;
                            self.prg_bank_high_16 = self.load_register & 0x0F;
                        }
                        else if mode == 3
                        {
                            self.prg_bank_low_16 = self.load_register & 0x0F;
                            self.prg_bank_high_16 = self.n_prg_banks - 1;
                        }
                    }
                    self.load_register = 0x00;
                    self.count = 0;
                }
    
            }
    
        }
        return false;
    }

    fn ppu_mapper_read(&mut self, address: u16, mapped_address: &mut u32) -> bool {
        if address < 0x2000
        {
            if self.n_chr_banks == 0
            {
                *mapped_address = address as u32;
                return true;
            }
            else
            {
                if (self.control_register & 0b10000) > 0
                {
                    if address >= 0x0000 && address <= 0x0FFF
                    {
                        *mapped_address = self.chr_bank_low_4 as u32 * 0x1000 + (address as u32 & 0x0FFF);
                        return true;
                    }
    
                    if address >= 0x1000 && address <= 0x1FFF
                    {
                        *mapped_address = self.chr_bank_high_4 as u32 * 0x1000 + (address as u32 & 0x0FFF);
                        return true;
                    }
                }
                else
                {
                    *mapped_address = self.chr_banK_full_8 as u32 * 0x2000 + (address as u32 & 0x1FFF);
                    return true;
                }
            }
        }	
    
        return false;
    }
    fn ppu_mapper_write(&mut self, address: u16, mapped_address: &mut u32) -> bool {
        if address < 0x2000
        {
            if self.n_chr_banks == 0
            {
                *mapped_address = address as u32;
                return true;
            }
            return true;
        }
        else
        {
            return false;
        }
    }

    fn reset(&mut self)
    {
        self.chr_bank_low_4 = 0;
        self.chr_bank_high_4 = 0;
        self.chr_banK_full_8 = 0;
    
        self.prg_bank_low_16 = 0;
        self.prg_bank_high_16 = self.n_prg_banks -1;
        self.prg_bank_full_32 = 0;
    
        self.load_register = 0;
        self.control_register = 0x1C;
        self.count = 0;

    }
    fn mirror(&mut self) -> Mirroring{
        return self.mirror;
    }
}

