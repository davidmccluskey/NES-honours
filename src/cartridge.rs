use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io;
use std::io::Read;

use crate::mapper::Mapper;
use crate::mapper::Mirroring;
use crate::mapper_0::Mapper0;

#[allow(dead_code)]
pub struct Cartridge {
    pub vec_prg_memory: Vec<u8>,
    pub vec_chr_memory: Vec<u8>,
    pub c_mapper_id: u8,
    pub c_prg_banks: u8,
    pub c_chr_banks: u8,
    pub mapper: Box<dyn Mapper>,
    pub header: CartridgeHeader,
    pub hardware_mirror: Mirroring,
}

pub struct CartridgeHeader {
    pub name: [char; 4],
    pub prg_rom_pages: u8,
    pub chr_rom_pages: u8,
    pub mapper_1: u8,
    pub mapper_2: u8,
    pub prg_ram_size: u8,
}

impl CartridgeHeader{
    pub fn new(data: &[u8]) -> Self{
        let mut name: [char;4] = ['A','B','C','D'];
        if data[0] == 0x4e && data[1] == 0x45 && data[2] == 0x53 && data[3] == 0x1a {
            name[0] = 'N';
            name[1] = 'E';
            name[2] = 'S';
            name[3] = '\x1a';

        }else{
            panic!("rom file not valid!")
        }
        CartridgeHeader {
            name,
            prg_rom_pages: data[4],
            chr_rom_pages: data[5],
            mapper_1: data[6],
            mapper_2: data[7],
            prg_ram_size: data[8],
        }
    }
}

impl Cartridge {
    pub fn new(filename: String) -> Result<Cartridge, io::Error> {
        let f = Cartridge::read_rom(filename);

        let _f = match f {
            Ok(file) => return Ok(file),
            Err(error) => return Err(error),
        };
    }

    pub fn read_rom(filename: String) -> Result<Cartridge, io::Error> {
        let mut file = File::open(filename)?;
        // Header
        let mut header: [u8; 16] = [0; 16];
        file.read_exact(&mut header)?;
        let cartridge_header = CartridgeHeader::new(&header);
        if (cartridge_header.mapper_1 & 0x04) > 0
        {
            file.seek(SeekFrom::Current(512)).unwrap();
        }

        let mapper_id = ((cartridge_header.mapper_2 >> 4) << 4) | (cartridge_header.mapper_1 >> 4);

        let file_type = 1;

        let mut vec_prg_memory: Vec<u8> = Vec::new();
        let mut vec_chr_memory: Vec<u8> = Vec::new();


        // if file_type == 0{
        //     //TODO
        // }else if file_type == 1
        // {
        //     let prg: usize = cartridge_header.prg_rom_pages as usize;
        //     let prg_size = prg * 16384;
        //     vec_prg_memory.resize(prg_size, 0);
        //     file.read_exact(&mut vec_prg_memory)?;

        //     let chr: usize = cartridge_header.chr_rom_pages as usize;
        //     let chr_size = chr * 8192;
        //     vec_chr_memory.resize(chr_size, 0);
        //     file.read_exact(&mut vec_chr_memory)?;
        // }else if file_type == 2{
        //     //TODO
        // }
        let prg: usize = cartridge_header.prg_rom_pages as usize;
        let prg_size = prg * 16384;
        vec_prg_memory.resize(prg_size, 0);
        file.read_exact(&mut vec_prg_memory)?;

        let chr: usize = cartridge_header.chr_rom_pages as usize;
        let mut chr_size = 0;
        if chr == 0
        {
            chr_size = 8192;
        }else
        {
            chr_size = chr * 8192;
        }
        vec_chr_memory.resize(chr_size, 0);
        let mut handle = file.take(chr_size as u64);
        handle.read(&mut vec_chr_memory);
        //file.take(chr_size as u64).read_to_end(&mut vec_chr_memory)?;
        //file.read_exact(&mut vec_chr_memory)?;

        println!("mapper: {}", mapper_id);
        let mapper: Box<dyn Mapper> = match mapper_id {
            0 => Box::new(Mapper0::new(
                cartridge_header.prg_rom_pages,
                cartridge_header.chr_rom_pages,
            )),
            n => panic!("Mapper {} not implemented", n),
        };

        let cartridge = Cartridge {
            vec_prg_memory,
            vec_chr_memory,
            c_mapper_id: mapper_id,
            c_prg_banks: cartridge_header.prg_rom_pages,
            c_chr_banks: cartridge_header.chr_rom_pages,
            mapper: mapper,
            hardware_mirror: if cartridge_header.mapper_1 & 0x01 == 0
            {
                Mirroring::Horizontal
            }else{
                Mirroring::Vertical
            },
            header: cartridge_header,
        };
        return Ok(cartridge);
    }

    pub fn reset(&mut self){
        self.mapper.reset();
    }
    pub fn cpu_write(&mut self, addr: u16, data: u8) -> bool {
        let mut mapped_addr: u32 = 0;
        if self.mapper.cpu_mapper_write(addr, &mut mapped_addr){
            self.vec_prg_memory[mapped_addr as usize] = data;
            return true;
        }else{
            return false;
        }
    }

    pub fn cpu_read(&mut self, addr: u16, data: &mut u8) -> bool {
        let mut mapped_addr: u32 = 0;
        if self.mapper.cpu_mapper_read(addr, &mut mapped_addr){
            *data = self.vec_prg_memory[mapped_addr as usize];
            return true;
        }
        return false;
    }

    pub fn ppu_read(&mut self, addr: u16, data: &mut u8) -> bool {
        let mut mapped_addr: u32 = 0;
        if self.mapper.ppu_mapper_read(addr, &mut mapped_addr){
            *data = self.vec_chr_memory[mapped_addr as usize];
            return true;
        }else{
            return false;
        }
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) -> bool {
        let mut mapped_addr: u32 = 0;
        if self.mapper.ppu_mapper_write(addr, &mut mapped_addr){
            self.vec_chr_memory[mapped_addr as usize] = data;
            return true;
        }else{
            return false;
        }
    }

    pub fn mirror(&mut self) -> Mirroring{
        let mirror = self.mapper.mirror();
        if mirror == Mirroring::Hardware
        {
            return self.hardware_mirror;
        }else
        {
            return mirror;
        }
    }
}


#[test]
fn test_new()
{
    let car = Cartridge::new("src/test/nestest.nes".to_string()).unwrap();

    assert_eq!(car.header.mapper_1, 0);
    assert_eq!(car.header.name, ['N', 'E', 'S', '\x1a']);
    assert_eq!(car.header.prg_rom_pages, 1);
    assert_eq!(car.header.chr_rom_pages, 1);
    assert_eq!(car.header.prg_ram_size, 0);

    assert_eq!(car.vec_prg_memory.len(), 16384);
    assert_eq!(car.vec_chr_memory.len(), 8192);
    assert_eq!(car.c_mapper_id, 0);
    assert_eq!(car.c_chr_banks, 1);
    assert_eq!(car.c_prg_banks, 1);
}
