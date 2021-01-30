use crate::cartridge::Cartridge;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

pub const RENDER_WIDTH: usize = 256;
pub const RENDER_HEIGHT: usize = 240;
pub const RENDER_SIZE: usize = RENDER_WIDTH * RENDER_HEIGHT;
pub const RENDER_FULL: usize = RENDER_SIZE * 3;

//https://wiki.nesdev.com/w/index.php/PPU_registers
bitfield! {
    #[derive(Copy, Clone)]
    pub struct Status(u8);
    pub sprite_overflow, set_sprite_overflow:        5;
    pub sprite_0, set_sprite_0:        6;
    pub vertical_blank,          set_vblank:                 7;
    pub get,             _:                      7,  0; // Full data
}
bitfield! {
    #[derive(Copy, Clone)]
    pub struct Mask(u8);
    pub greyscale,              _: 0;
    pub show_background_left,   _: 1;
    pub show_sprites_left,      _: 2;
    pub show_background,        _: 3;
    pub show_sprites,           _: 4;
    pub emphasize_red,          _: 5;
    pub emphasize_green,        _: 6;
    pub emphasize_blue,         _: 7;
}
bitfield! {
    #[derive(Copy, Clone)]
    pub struct Controller(u8);
    pub nametable_high,          _: 1, 0;
    pub nametable_low,          _:    2;
    pub sprite_table,           _:    3;
    pub background_table,       _:    4;
    pub sprite_size,            _:    5;
    pub master_slave,           _:    6;
    pub generate_nmi,           _:    7;
}

pub struct PPU {
    cartridge: Option<Rc<RefCell<Cartridge>>>,
    name_table: [[u8; 1024]; 2],
    palette_table: [u8; 32],
    pattern_table: [[u8; 4096]; 2],
    pub frame_complete: bool,

    sprite_screen: [u8; 256 * 240],
    sprite_name_table: [[u8; 256 * 240]; 2],
    sprite_pattern_table: [[u8; 128 * 128]; 2],

    scanline: u16,
    cycle: u16,

    controller: Controller,
    mask: Mask,
    status: Status,
    address_latch: u8,
    data_buffer: u8,
    buffer_address: u16,
    pub nmi_enabled: bool,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            cartridge: None,
            name_table: [[0; 1024]; 2],
            palette_table: [0; 32],
            pattern_table: [[0; 4096]; 2],
            frame_complete: false,
            sprite_screen: [62; RENDER_SIZE],
            sprite_name_table: [[0; 256 * 240]; 2],
            sprite_pattern_table: [[0; 128 * 128]; 2],
            scanline: 0,
            cycle: 0,
            controller: Controller(0),
            mask: Mask(0),
            status: Status(0),
            address_latch: 0,
            data_buffer: 0,
            buffer_address: 0,
            nmi_enabled: true,
        }
    }

    pub fn cpu_read(&mut self, addr: u16, _readOnly: bool) -> u8 {
        let mut data: u8 = 0x00;
        match addr {
            0x0000 => (), //Control
            0x0001 => (), //Mask
            0x0002 => {  //Status
                data = (self.status.get() & 0xE0) | (self.data_buffer & 0x1F);
                self.status.set_vblank(false);
                self.address_latch = 0;
            }
            0x0003 => (), //OAM Address
            0x0004 => (), //OAM Data
            0x0005 => (), //Scroll
            0x0006 => (), //PPU Address
            0x0007 => {   //PPU data
                data = self.data_buffer;
                self.data_buffer = self.ppu_read(self.buffer_address, false);

                if self.buffer_address > 0x3F00 {
                    data = self.data_buffer;
                }
                self.buffer_address += 1;
            }
            _ => (), //required by rust
        }
        return data;
    }
    pub fn cpu_write(&mut self, addr: u16, data: &mut u8) {
        match addr {
            0x0000 => self.controller = Controller(*data), //Control
            0x0001 => self.mask = Mask(*data),             //Mask
            0x0002 => (),                                  //Status
            0x0003 => (),                                  //OAM Address
            0x0004 => (),                                  //OAM Data
            0x0005 => (),                                  //Scroll
            0x0006 =>
            //PPU Address
            {
                if self.address_latch == 0 {
                    self.buffer_address = (self.buffer_address & 0x00FF) | (*data as u16) << 8; //KEEP AN EYE ON THIS
                    self.address_latch = 1;
                } else {
                    self.buffer_address = (self.buffer_address & 0xFF00) | *data as u16;
                    self.address_latch = 0;
                }
            }
            0x0007 => {
                //PPU Data
                self.ppu_write(self.buffer_address, data);
                self.buffer_address += 1;
            }

            _ => (), //required by rust
        }
    }
    pub fn ppu_read(&mut self, mut addr: u16, _readOnly: bool) -> u8 {
        let mut data: u8 = 0x00;
        addr &= 0x3FFF;
        if let Some(ref c) = self.cartridge {
            if c.borrow_mut().ppu_read(addr, &mut data) {
                //Should always be false
            } else if addr >= 0x0000 && addr <= 0x1FFF {
                //Pattern memory
                //First index chooses whether it's the left or the right pattern table, second is index within that table
                let first_index = ((addr & 0x1000) >> 12).to_be_bytes()[1] as usize;
                let second_index = (addr & 0x0FFF) as usize;
                data = self.pattern_table[first_index][second_index];
            } else if addr >= 0x2000 && addr <= 0x3EFF { //Nametable memory
            } else if addr >= 0x3F00 && addr <= 0x3FFF {
                //Palette memory
                addr = addr & 0x001F;
                if addr == 0x0010 {
                    addr = 0x0000
                }
                if addr == 0x0014 {
                    addr = 0x0004
                }
                if addr == 0x0018 {
                    addr = 0x0008
                }
                if addr == 0x001C {
                    addr = 0x000C
                }
                let addr_u8 = addr.to_be_bytes()[1];
                data = self.palette_table[addr_u8 as usize];
            }
        }

        return data;
    }
    pub fn ppu_write(&mut self, mut addr: u16, data: &mut u8) {
        addr &= 0x3FFF;
        if let Some(ref c) = self.cartridge {
            if c.borrow_mut().ppu_write(addr, data) {
                //Should always be false
            } else if addr >= 0x0000 && addr <= 0x1FFF {
                //Pattern memory, usually a ROM however some games need to write to it
                let first_index = ((addr & 0x1000) >> 12).to_be_bytes()[1] as usize;
                self.pattern_table[first_index][(addr & 0x0FFF) as usize] = *data;
            } else if addr >= 0x2000 && addr <= 0x3EFF {
                //Nametable memory
            } else if addr >= 0x3F00 && addr <= 0x3FFF {
                //Palette memory
                addr = addr & 0x001F;
                if addr == 0x0010 {
                    addr = 0x0000
                }
                if addr == 0x0014 {
                    addr = 0x0004
                }
                if addr == 0x0018 {
                    addr = 0x0008
                }
                if addr == 0x001C {
                    addr = 0x000C
                }
                let addr_u8 = addr.to_be_bytes()[1];
                self.palette_table[addr_u8 as usize] = *data;
            }
        }
    }
    pub fn connect_cartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>) {
        self.cartridge = Some(cartridge);
    }

    pub fn render(&self) -> [u8; RENDER_FULL] {
        let mut ret = [0; RENDER_FULL];
        for i in 0..RENDER_SIZE {
            let c = self.sprite_screen[i];
            let (r, g, b) = PALETTE[c as usize];
            ret[i * 3 + 0] = r;
            ret[i * 3 + 1] = g;
            ret[i * 3 + 2] = b;
        }
        return ret;
    }

    pub fn get_name_table(&mut self, index: usize) -> [u8; 256 * 240] {
        return self.sprite_name_table[index];
    }

    pub fn get_pattern_table(&mut self, index: u8, palette: u8) -> [u8; (128 * 128) * 3] {
        for tile_y in 0..16 {
            for tile_x in 0..16 {
                let offset: u16 = tile_y * 256 + tile_x * 16;

                for row in 0..8 {
                    let addr: u16 = index as u16 * 0x1000 + offset + row;
                    let mut tile_ls = self.ppu_read(addr, false);
                    let mut tile_ms = self.ppu_read(addr + 8, false);

                    for column in 0..8 {
                        let pixel = (tile_ls & 0x01) + (tile_ms & 0x01);
                        tile_ls = tile_ls >> 1;
                        tile_ms = tile_ms >> 1;

                        let x = tile_x * 8 + (7 - column);
                        let y = tile_y * 8 + row;
                        let colour = self.get_colour(palette, pixel);
                        self.write_pattern_pixel(x, y, colour, index as usize);
                    }
                }
            }
        }
        return self.render_palette(index);
    }

    pub fn render_palette(&self, index: u8) -> [u8; (128 * 128) * 3] {
        let mut ret = [0; (128 * 128) * 3];
        for i in 0..(128 * 128) {
            let c = self.sprite_pattern_table[index as usize][i];
            let (r, g, b) = PALETTE[c as usize];
            ret[i * 3 + 0] = r;
            ret[i * 3 + 1] = g;
            ret[i * 3 + 2] = b;
        }
        return ret;
    }

    pub fn get_colour(&mut self, palette: u8, pixel: u8) -> u8 {
        let addr: u16 = 0x3F00 + (palette as u16 * 4) + pixel as u16;
        let i = self.ppu_read(addr, false);
        return i;
    }

    fn draw_pixel(&mut self, x: u16, y: u16, c: SystemColor) {
        if x >= 256 || y >= 240 {
            return;
        }
        let i = (x + 256 * y) as usize;
        self.sprite_screen[i] = c;
    }
    fn write_nametable_pixel(&mut self, x: u16, y: u16, c: SystemColor, index: usize) {
        let i = (x + 256 * y) as usize;
        self.sprite_pattern_table[index][i] = c;
    }
    fn write_pattern_pixel(&mut self, x: u16, y: u16, c: SystemColor, index: usize) {
        let i = (x + 128 * y) as usize;
        self.sprite_pattern_table[index][i] = c;
    }
    pub fn clock(&mut self) {
        if self.scanline == 261 && self.cycle == 1 {
            self.status.set_vblank(false);
        }
        if self.scanline == 241 && self.cycle == 1 {
            self.status.set_vblank(true);
            if self.controller.generate_nmi() == true {
                self.nmi_enabled = true;
            }
        }

        // if rand::random() {
        //     self.draw_pixel(self.cycle, self.scanline, 61);
        // } else {
        //     self.draw_pixel(self.cycle, self.scanline, 63);
        // }

        self.cycle = self.cycle + 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline = self.scanline + 1;

            if self.scanline >= 261 {
                self.scanline = 0;
                self.frame_complete = true;
            }
        }
    }
}

type Palette = [(u8, u8, u8); 64];
pub type SystemColor = u8;
// The NES can refer to 64 separate colors. This table has RGB values for each.
pub const PALETTE: Palette = [
    (124, 124, 124), // x0
    (0, 0, 252),     // x1
    (0, 0, 188),     // x2
    (68, 40, 188),   // x3
    (148, 0, 132),   // x4
    (168, 0, 32),    // x5
    (168, 16, 0),    // x6
    (136, 20, 0),    // x7
    (80, 48, 0),     // x8
    (0, 120, 0),     // x9
    (0, 104, 0),     // xA
    (0, 88, 0),      // xB
    (0, 64, 88),     // xC
    (0, 0, 0),       // xD
    (0, 0, 0),       // xE
    (0, 0, 0),       // xF
    // 1x
    (188, 188, 188), // x0
    (0, 120, 248),   // x1
    (0, 88, 248),    // x2
    (104, 68, 252),  // x3
    (216, 0, 204),   // x4
    (228, 0, 88),    // x5
    (248, 56, 0),    // x6
    (228, 92, 16),   // x7
    (172, 124, 0),   // x8
    (0, 184, 0),     // x9
    (0, 168, 0),     // xA
    (0, 168, 68),    // xB
    (0, 136, 136),   // xC
    (0, 0, 0),       // xD
    (0, 0, 0),       // xE
    (0, 0, 0),       // xF
    // 2x
    (248, 248, 248), // x0
    (60, 188, 252),  // x1
    (104, 136, 252), // x2
    (152, 120, 248), // x3
    (248, 120, 248), // x4
    (248, 88, 152),  // x5
    (248, 120, 88),  // x6
    (252, 160, 68),  // x7
    (248, 184, 0),   // x8
    (184, 248, 24),  // x9
    (88, 216, 84),   // xA
    (88, 248, 152),  // xB
    (0, 232, 216),   // xC
    (120, 120, 120), // xD
    (0, 0, 0),       // xE
    (0, 0, 0),       // xF
    // 3x
    (252, 252, 252), // x0
    (164, 228, 252), // x1
    (184, 184, 248), // x2
    (216, 184, 248), // x3
    (248, 184, 248), // x4
    (248, 164, 192), // x5
    (240, 208, 176), // x6
    (252, 224, 168), // x7
    (248, 216, 120), // x8
    (216, 248, 120), // x9
    (184, 248, 184), // xA
    (184, 248, 216), // xB
    (0, 252, 252),   // xC
    (216, 216, 216), // xD
    (0, 0, 0),       // xE
    (0, 0, 0),       // xF
];
