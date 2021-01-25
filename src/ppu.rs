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
    palette_colours: [Color; 64],

    sprite_screen: [u8; RENDER_SIZE],
    sprite_patterns: [u16; 8],
    sprite_palettes: [u8; 8],

    scanline: u16,
    cycle: u16,
}

impl PPU {
    pub fn new() -> PPU {
        let palette_colours = PPU::setPal();
        PPU {
            cartridge: None,
            name_table: [[0; 2]; 1024],
            palette: [0; 32],
            frame_complete: false,
            palette_colours,
            sprite_screen: [62; RENDER_SIZE],
            sprite_name_table: None,
            sprite_pattern_table: None,

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

    pub fn getFrame(&mut self) -> [u8; RENDER_FULL]
    {
        let mut ret = [0; RENDER_FULL];
        for i in 0..RENDER_SIZE {
            let c = self.sprite_screen[i];
            let color = self.palette_colours[c as usize];
            ret[i * 3 + 0] = color.r;
            ret[i * 3 + 1] = color.g;
            ret[i * 3 + 2] = color.b;
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

    pub fn setPal() -> [Color; 64]{
        let mut array: [Color; 64] = [Color::BLACK; 64];

        array[0x00] = Color::RGB(84,84,84);
        array[0x01] = Color::RGB(0, 30, 116);
        array[0x02] = Color::RGB(8, 16, 144);
        array[0x03] = Color::RGB(48, 0, 136);
        array[0x04] = Color::RGB(68, 0, 100);
        array[0x05] = Color::RGB(92, 0, 48);
        array[0x06] = Color::RGB(84, 4, 0);
        array[0x07] = Color::RGB(60, 24, 0);
        array[0x08] = Color::RGB(32, 42, 0);
        array[0x09] = Color::RGB(8, 58, 0);
        array[0x0A] = Color::RGB(0, 64, 0);
        array[0x0B] = Color::RGB(0, 60, 0);
        array[0x0C] = Color::RGB(0, 50, 60);
        array[0x0D] = Color::RGB(0, 0, 0);
        array[0x0E] = Color::RGB(0, 0, 0);
        array[0x0F] = Color::RGB(0, 0, 0);

        array[0x10] = Color::RGB(152, 150, 152);
        array[0x11] = Color::RGB(8, 76, 196);
        array[0x12] = Color::RGB(48, 50, 236);
        array[0x13] = Color::RGB(92, 30, 228);
        array[0x14] = Color::RGB(136, 20, 176);
        array[0x15] = Color::RGB(160, 20, 100);
        array[0x16] = Color::RGB(152, 34, 32);
        array[0x17] = Color::RGB(120, 60, 0);
        array[0x18] = Color::RGB(84, 90, 0);
        array[0x19] = Color::RGB(40, 114, 0);
        array[0x1A] = Color::RGB(8, 124, 0);
        array[0x1B] = Color::RGB(0, 118, 40);
        array[0x1C] = Color::RGB(0, 102, 120);
        array[0x1D] = Color::RGB(0, 0, 0);
        array[0x1E] = Color::RGB(0, 0, 0);
        array[0x1F] = Color::RGB(0, 0, 0);

        array[0x20] = Color::RGB(236, 238, 236);
        array[0x21] = Color::RGB(76, 154, 236);
        array[0x22] = Color::RGB(120, 124, 236);
        array[0x23] = Color::RGB(176, 98, 236);
        array[0x24] = Color::RGB(228, 84, 236);
        array[0x25] = Color::RGB(236, 88, 180);
        array[0x26] = Color::RGB(236, 106, 100);
        array[0x27] = Color::RGB(212, 136, 32);
        array[0x28] = Color::RGB(160, 170, 0);
        array[0x29] = Color::RGB(116, 196, 0);
        array[0x2A] = Color::RGB(76, 208, 32);
        array[0x2B] = Color::RGB(56, 204, 108);
        array[0x2C] = Color::RGB(56, 180, 204);
        array[0x2D] = Color::RGB(60, 60, 60);
        array[0x2E] = Color::RGB(0, 0, 0);
        array[0x2F] = Color::RGB(0, 0, 0);

        array[0x30] = Color::RGB(236, 238, 236);
        array[0x31] = Color::RGB(168, 204, 236);
        array[0x32] = Color::RGB(188, 188, 236);
        array[0x33] = Color::RGB(212, 178, 236);
        array[0x34] = Color::RGB(236, 174, 236);
        array[0x35] = Color::RGB(236, 174, 212);
        array[0x36] = Color::RGB(236, 180, 176);
        array[0x37] = Color::RGB(228, 196, 144);
        array[0x38] = Color::RGB(204, 210, 120);
        array[0x39] = Color::RGB(180, 222, 120);
        array[0x3A] = Color::RGB(168, 226, 144);
        array[0x3B] = Color::RGB(152, 226, 180);
        array[0x3C] = Color::RGB(160, 214, 228);
        array[0x3D] = Color::RGB(160, 162, 16);
        array[0x3E] = Color::RGB(0, 0, 0);
        array[0x3F] = Color::RGB(0, 0, 0);

        return array;
    }

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

type RgbColor = (u8, u8, u8);
type SystemPalette = [RgbColor; 64];
pub type SystemColor = u8;
// The NES can refer to 64 separate colors. This table has RGB values for each.
pub const SYSTEM_PALETTE: SystemPalette = [
    // 0x
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
