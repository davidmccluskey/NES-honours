use crate::cartridge::Cartridge;
use crate::sdl2::pixels::Color;
use std::cell::RefCell;
use std::rc::Rc;
use rand::Rng;

use crate::textures::Tex;

pub const RENDER_WIDTH: usize = 256;
pub const RENDER_HEIGHT: usize = 240;
pub const RENDER_SIZE: usize = RENDER_WIDTH * RENDER_HEIGHT;
pub const RENDER_FULL: usize = RENDER_SIZE * 3;

pub struct PPU {
    pub cartridge: Option<Rc<RefCell<Cartridge>>>,
    pub name_table: [[u8; 2]; 1024],
    pub palette: [u8; 32],
    pub frame_complete: bool,

    sprite_screen: [u8; 256 * 240],
    sprite_name_table: [[u8; 128]; 128],
    sprite_pattern_table: [[u8; 128]; 128],

    scanline: u16,
    cycle: u16,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            cartridge: None,
            name_table: [[0; 2]; 1024],
            palette: [0; 32],
            frame_complete: false,
            sprite_screen: [62; RENDER_SIZE],
            sprite_name_table: [[0; 128]; 128],
            sprite_pattern_table: [[0; 128]; 128],

            scanline: 0,
            cycle: 0,
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
    pub fn cpuWrite(&mut self, addr: u16, data: u8) {
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
    pub fn ppuRead(&mut self, mut addr: u16, _readOnly: bool) -> u8 {
        let mut data: u8 = 0x00;
        addr &= 0x3FFF;
        if let Some(ref c) = self.cartridge {
            if c.borrow_mut().ppu_read(addr, &mut data) {}
        }

        return data;
    }
    pub fn ppuWrite(&mut self, mut addr: u16, data: &mut u8) {
        addr &= 0x3FFF;
        if let Some(ref c) = self.cartridge {
            if c.borrow_mut().ppu_write(addr, data) {}
        }
    }
    pub fn connectCartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>) {
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

    // pub fn getNametable(&mut self, index: usize) -> &Texture
    // {
    //     return &self.sprite_name_table[index];
    // }

    // pub fn getPatterntable(&mut self, index: usize) -> &Texture
    // {
    //     return &self.sprite_pattern_table[index];
    // }
    fn write_system_pixel(&mut self, x: u16, y: u16, c: SystemColor) {
        if x >= 256 || y >= 240 {
            return;
        }
        let i = (x + 256 * y) as usize;
        self.sprite_screen[i] = c;
    }

    pub fn clock(&mut self) 
    {
        if rand::random() { // generates a boolean
            self.write_system_pixel(self.cycle, self.scanline, 33);
        }else {
            self.write_system_pixel(self.cycle, self.scanline, 63);
        }
        self.cycle = self.cycle + 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline = self.scanline + 1;

            if self.scanline >= 261{
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
