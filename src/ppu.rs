use crate::cartridge::Cartridge;
use crate::cartridge;
use std::cell::RefCell;
use std::rc::Rc;
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
    pub nametable_x,                    _:   0;
    pub nametable_y,                    _:   1;
    pub increment,              _:    2;
    pub sprite_table,           _:    3;
    pub background_table,       _:    4;
    pub sprite_size,            _:    5;
    pub master_slave,           _:    6;
    pub generate_nmi,           _:    7;
}

bitfield!{
    #[derive(Copy, Clone, PartialEq)]
    pub struct Address(u16);
    impl Debug;
    pub u8,  coarse_x,   set_coarse_x:    4,  0;
    pub u8,  coarse_y,   set_coarse_y:    9,  5;
    pub u8,  nametable_x,  set_nametable_x:      10;
    pub u8,  nametable_y,  set_nametable_y:      11;
    pub u8,  fine_y,     set_fine_y:     14, 12;
    pub u16, address,    _:              13,  0; // Full 14-bit address from PPUADDR
    pub u8,  high_byte,  set_high_byte:  13,  8; // High 6 bits of PPUADDR address
    pub u8,  low_byte,   set_low_byte:    7,  0; // Low 7 bits of PPUADDR address
    pub u16, get,        _:              14,  0; // Full data
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
    v_address_register: Address,
    t_address_register: Address,
    fine_x: u8,
    address_latch: u8,
    data_buffer: u8,
    pub nmi_enabled: bool,

    bg_tile_id: u8,
    bg_tile_attr: u8,
    bg_tile_lsb: u8,
    bg_tile_msb: u8,

    bg_shifter_lsb: u16,
    bg_shifter_msb: u16,
    bg_shifter_attr_low: u16,
    bg_shifter_attr_high: u16,
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
            v_address_register: Address(0),
            t_address_register: Address(0),
            fine_x: 0,
            address_latch: 0,
            data_buffer: 0,
            nmi_enabled: true,
            bg_tile_id: 0,
            bg_tile_attr: 0,
            bg_tile_lsb: 0,
            bg_tile_msb: 0,
            bg_shifter_lsb: 0,
            bg_shifter_msb: 0,
            bg_shifter_attr_low: 0,
            bg_shifter_attr_high: 0,
        }
    }

    pub fn cpu_read(&mut self, addr: u16, read_only: bool) -> u8 {
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
                self.data_buffer = self.ppu_read(self.v_address_register.get(), read_only);

                if self.v_address_register.get() > 0x3F00 {
                    data = self.data_buffer;
                }
                if self.controller.increment() == true{
                    self.v_address_register = Address(self.v_address_register.get().wrapping_add(32));
                    //self.v_address_register.get().wrapping_add(32);
                }else{
                    self.v_address_register = Address(self.v_address_register.get().wrapping_add(1));
                    //self.v_address_register.get().wrapping_add(1);
                }
            }
            _ => (), //required by rust
        }
        return data;
    }
    pub fn cpu_write(&mut self, addr: u16, data: &mut u8) {
        match addr {
            0x0000 => {//Control
                self.controller = Controller(*data);
                self.t_address_register.set_nametable_x(self.controller.nametable_x());
                self.t_address_register.set_nametable_y(self.controller.nametable_y());

            }, 
            0x0001 => self.mask = Mask(*data),             //Mask
            0x0002 => (),                                  //Status
            0x0003 => (),                                  //OAM Address
            0x0004 => (),                                  //OAM Data
            0x0005 => { //Scroll
                if self.address_latch == 0{
                    self.fine_x = *data & 0x07;
                    self.t_address_register.set_coarse_x(*data >> 3);
                    self.address_latch = 1;
                }else{
                    self.t_address_register.set_fine_y(*data & 0x07);
                    self.t_address_register.set_coarse_y(*data >> 3);
                    self.address_latch = 0;
                }
            },                                 
            0x0006 =>
            //PPU Address
            {
                if self.address_latch == 0 {
                    self.t_address_register = Address((self.t_address_register.get() & 0x00FF) | (*data as u16) << 8); //KEEP AN EYE ON THIS
                    self.address_latch = 1;
                } else {
                    self.t_address_register = Address((self.t_address_register.get() & 0xFF00) | *data as u16);
                    self.v_address_register = self.t_address_register;
                    self.address_latch = 0;
                }
            }
            0x0007 => {
                //PPU Data
                self.ppu_write(self.v_address_register.get(), data);
                if self.controller.increment() == true{
                    self.v_address_register = Address(self.v_address_register.get().wrapping_add(32));
                }else{
                    self.v_address_register = Address(self.v_address_register.get().wrapping_add(1));
                }
            }

            _ => (), //required by rust
        }
    }
    #[allow(unused_comparisons)]
    pub fn ppu_read(&mut self, mut addr: u16, _read_only: bool) -> u8 {
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
            } else if addr >= 0x2000 && addr <= 0x3EFF { 
                //Nametable memory
                if c.borrow_mut().mirror == cartridge::Mirroring::Vertical {
                    if addr >= 0x0000 && addr <= 0x03FF{
                        data = self.name_table[0][(addr & 0x03FF) as usize];
                    }
                    if addr >= 0x0400 && addr <= 0x07FF{
                        data = self.name_table[1][(addr & 0x03FF) as usize];
                    }
                    if addr >= 0x0800 && addr <= 0x0BFF{
                        data = self.name_table[0][(addr & 0x03FF) as usize];
                    }
                    if addr >= 0x0C00 && addr <= 0x0FFF{
                        data = self.name_table[1][(addr & 0x03FF) as usize];
                    }
                }
                else if c.borrow_mut().mirror == cartridge::Mirroring::Horizontal {
                    if addr >= 0x0000 && addr <= 0x03FF{
                        data = self.name_table[0][(addr & 0x03FF) as usize];
                    }
                    if addr >= 0x0400 && addr <= 0x07FF{
                        data = self.name_table[0][(addr & 0x03FF) as usize];
                    }
                    if addr >= 0x0800 && addr <= 0x0BFF{
                        data = self.name_table[1][(addr & 0x03FF) as usize];
                    }
                    if addr >= 0x0C00 && addr <= 0x0FFF{
                        data = self.name_table[1][(addr & 0x03FF) as usize];
                    }
                }

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
    #[allow(unused_comparisons)]
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
                if c.borrow_mut().mirror == cartridge::Mirroring::Vertical {
                    if addr >= 0x0000 && addr <= 0x03FF{
                        self.name_table[0][(addr & 0x03FF) as usize] = *data;
                    }
                    if addr >= 0x0400 && addr <= 0x07FF{
                        self.name_table[1][(addr & 0x03FF) as usize] = *data;
                    }
                    if addr >= 0x0800 && addr <= 0x0BFF{
                        self.name_table[0][(addr & 0x03FF) as usize] = *data;
                    }
                    if addr >= 0x0C00 && addr <= 0x0FFF{
                        self.name_table[1][(addr & 0x03FF) as usize] = *data;
                    }
                }
                else if c.borrow_mut().mirror == cartridge::Mirroring::Horizontal {
                    if addr >= 0x0000 && addr <= 0x03FF{
                        self.name_table[0][(addr & 0x03FF) as usize] = *data;
                    }
                    if addr >= 0x0400 && addr <= 0x07FF{
                        self.name_table[0][(addr & 0x03FF) as usize] = *data;
                    }
                    if addr >= 0x0800 && addr <= 0x0BFF{
                        self.name_table[1][(addr & 0x03FF) as usize] = *data;
                    }
                    if addr >= 0x0C00 && addr <= 0x0FFF{
                        self.name_table[1][(addr & 0x03FF) as usize] = *data;
                    }
                }
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
        let mut frame = [0; RENDER_FULL];
        for i in 0..RENDER_SIZE {
            let c = self.sprite_screen[i];
            let (r, g, b) = PALETTE[c as usize];
            frame[i * 3 + 0] = r;
            frame[i * 3 + 1] = g;
            frame[i * 3 + 2] = b;
        }
        return frame;
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
        let mut frame = [0; (128 * 128) * 3];
        for i in 0..(128 * 128) {
            let c = self.sprite_pattern_table[index as usize][i];
            let (r, g, b) = PALETTE[c as usize];
            frame[i * 3 + 0] = r;
            frame[i * 3 + 1] = g;
            frame[i * 3 + 2] = b;
        }
        return frame;
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
    // fn write_nametable_pixel(&mut self, x: u16, y: u16, c: SystemColor, index: usize) {
    //     let i = (x + 256 * y) as usize;
    //     self.sprite_pattern_table[index][i] = c;
    // }
    fn write_pattern_pixel(&mut self, x: u16, y: u16, c: SystemColor, index: usize) {
        let i = (x + 128 * y) as usize;
        self.sprite_pattern_table[index][i] = c;
    }

    fn scroll_x(&mut self){
        if self.mask.show_background() || self.mask.show_sprites(){
            if self.v_address_register.coarse_x() == 31{
                self.v_address_register.set_coarse_x(0);
                self.v_address_register.set_nametable_x(!self.v_address_register.nametable_x());
            }else{
                self.v_address_register.set_coarse_x(self.v_address_register.coarse_x() + 1);
            }
        }
    }
    fn scroll_y(&mut self){
        if self.mask.show_background() || self.mask.show_sprites(){
            if self.v_address_register.fine_y() < 7{
                self.v_address_register.set_fine_y(self.v_address_register.fine_y() + 1);
            }else{
                self.v_address_register.set_fine_y(0);

                if self.v_address_register.coarse_y() == 29{

                    self.v_address_register.set_coarse_y(0);
                    self.v_address_register.set_nametable_y(!self.v_address_register.nametable_y());

                }else if self.v_address_register.coarse_y() == 31{
                    self.v_address_register.set_coarse_y(0);
                }else{
                    self.v_address_register.set_coarse_y(self.v_address_register.coarse_y() + 1);
                }
            }
        }
    }
    fn reset_x(&mut self){
        if self.mask.show_background() || self.mask.show_sprites(){
            self.v_address_register.set_nametable_x(self.t_address_register.nametable_x());
            self.v_address_register.set_coarse_x(self.t_address_register.coarse_x());
        }
    }
    fn reset_y(&mut self){
        if self.mask.show_background() || self.mask.show_sprites(){
            self.v_address_register.set_nametable_y(self.t_address_register.nametable_y());
            self.v_address_register.set_coarse_y(self.t_address_register.coarse_y());
            self.v_address_register.set_fine_y(self.t_address_register.fine_y());

        }
    }

    fn load_bg_shifters(&mut self){
        self.bg_shifter_lsb = (self.bg_shifter_lsb & 0x00FF) | self.bg_tile_lsb as u16; 
        self.bg_shifter_msb = (self.bg_shifter_msb & 0x00FF) | self.bg_tile_msb as u16; 

        if (self.bg_tile_attr & 0b01) > 0{
            self.bg_shifter_attr_low = (self.bg_shifter_attr_low & 0xFF00) | 0xFF;
        }else{
            self.bg_shifter_attr_low = (self.bg_shifter_attr_low & 0xFF00) | 0x00;
        }

        if (self.bg_tile_attr & 0b10) > 0{
            self.bg_shifter_attr_high = (self.bg_shifter_attr_high & 0xFF00) | 0xFF;
        }else{
            self.bg_shifter_attr_high = (self.bg_shifter_attr_high & 0xFF00) | 0x00;
        }
    }

    fn update_shifters(&mut self){
        if self.mask.show_background(){
            self.bg_shifter_attr_high <<= 1;
            self.bg_shifter_attr_low <<= 1;

            self.bg_shifter_lsb <<= 1;
            self.bg_shifter_msb <<= 1;

        }
    }
    pub fn clock(&mut self) {
        if self.scanline > 240 {
            if self.scanline == 261 && self.cycle == 1 {
                self.status.set_vblank(false);
            }

            if (self.cycle >= 1 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338){
                self.update_shifters();
                match self.cycle % 8{
                    0 => {
                        self.load_bg_shifters();
                        self.bg_tile_id= self.ppu_read(0x2000 | (self.v_address_register.get() & 0x0FFF), false);
                    },
                    2 => {
                        self.bg_tile_attr = self.ppu_read
                        (0x23C0 
                        | ((self.v_address_register.nametable_y() as u16) << 11) 
                        | ((self.v_address_register.nametable_x() as u16) << 10)
                        | (((self.v_address_register.coarse_y() as u16) >> 2) << 3)
                        | ((self.v_address_register.coarse_x() as u16) >> 2), false);

                        if (self.v_address_register.coarse_y() & 0x02) > 0 {
                            self.bg_tile_attr >>= 4;
                        }
                        if (self.v_address_register.coarse_x() & 0x02) > 0 {
                            self.bg_tile_attr >>= 2;
                        }
                        self.bg_tile_attr &= 0x03;
                    },
                    4 => {
                        self.bg_tile_lsb = self.ppu_read(
                            ((self.controller.background_table() as u16) << 12)
                        + (((self.bg_tile_id) as u16) << 4)
                        + ((self.v_address_register.fine_y() as u16) + 0), false);
                    },
                    6 => {
                        self.bg_tile_lsb = self.ppu_read(((self.controller.background_table() as u16) << 12)
                        + (((self.bg_tile_id) as u16) << 4)
                        + ((self.v_address_register.fine_y() as u16) + 8), false);
                    },
                    7 => {
                        self.scroll_x();
                    },
                    _ => (),
                }
            }
            if self.cycle == 256{
               self.scroll_y();
            }
            if self.cycle == 257{
                self.load_bg_shifters();
                self.reset_x();
            }
            if self.cycle == 338 || self.cycle == 340{
                self.bg_tile_id = self.ppu_read(0x2000 | (self.v_address_register.get() & 0x0FFF), false);
            }

            if self.scanline == 261 && self.cycle >= 280 && self.cycle < 305 {
                self.reset_y();
            }
        }

        if self.scanline == 241 && self.cycle == 1 {
            self.status.set_vblank(true);
            if self.controller.generate_nmi() == true {
                self.nmi_enabled = true;
            }
        }

        let mut background_pixel: u8 = 0x00;
        let mut background_palette: u8 = 0x00;

        if self.mask.show_background(){
            let bit_mask = 0x8000 >> self.fine_x;
            let plane_0 = (self.bg_shifter_lsb & bit_mask) > 0;
            let plane_1 = (self.bg_shifter_msb & bit_mask) > 0;
            background_pixel = ((plane_1 as u8) << 1) | plane_0 as u8;

            let palette_0 = (self.bg_shifter_attr_low & bit_mask) > 0;
            let palette_1 = (self.bg_shifter_attr_high & bit_mask) > 0;

            background_palette = ((palette_1 as u8) << 1) | palette_0 as u8;
        }
        
        let colour = self.get_colour(background_palette, background_pixel);
        self.draw_pixel(self.cycle, self.scanline, colour);

        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;

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
