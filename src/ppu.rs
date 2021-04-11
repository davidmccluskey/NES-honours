use crate::cartridge;
use crate::cartridge::Cartridge;
use crate::Mappers::mapper::Mirroring;
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
    pub sprite_overflow, set_sprite_overflow: 5;
    pub sprite_0, set_sprite_0:     6;
    pub vertical_blank, set_vblank: 7;
    pub get,_:                      7,  0;
}
bitfield! {
    #[derive(Copy, Clone, PartialEq)]
    pub struct SpriteStatus(u8);
    impl Debug;
    pub palette,            _: 1, 0;
    pub behind_background,  _:    5;
    pub flip_x,             _:    6;
    pub flip_y,             _:    7;
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
    pub get,                    _: 7,  0;
}
bitfield! {
    #[derive(Copy, Clone)]
    pub struct Controller(u8);
    pub nametable_x,            _:    0;
    pub nametable_y,            _:    1;
    pub increment,              _:    2;
    pub sprite_table,           _:    3;
    pub background_table,       _:    4;
    pub sprite_size,            _:    5;
    pub master_slave,           _:    6;
    pub generate_nmi,           _:    7;
    pub get,                    _:    7,  0;
}

bitfield! {
    #[derive(Copy, Clone, PartialEq)]
    pub struct Address(u16);
    impl Debug;
    pub u8,  coarse_x,   set_coarse_x:          4,  0;
    pub u8,  coarse_y,   set_coarse_y:          9,  5;
    pub u8,  nametable_x,  set_nametable_x:     10;
    pub u8,  nametable_y,  set_nametable_y:     11;
    pub u8,  fine_y,     set_fine_y:            14, 12;
    pub u8,  unused,     _:                     15;
    pub u16, get,        _:                     15,  0; // Full data
}

pub struct PPU {
    cartridge: Option<Rc<RefCell<Cartridge>>>,
    name_table: [[u8; 1024]; 2],
    palette_table: [u8; 32],
    pattern_table: [[u8; 4096]; 2],
    pub frame_complete: bool,

    sprite_screen: [u8; 256 * 240],
    sprite_pattern_table: [[u8; 128 * 128]; 2],

    scanline: i32,
    cycle: i32,

    //Background rendering
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

    //Foreground Rendering
    pub oam_ram: [u8; 256],
    oam_sprites: Vec<Sprite>,
    pub oam_address_port: u8,
    sprite_shifter_low: [u8; 8],
    sprite_shifter_high: [u8; 8],

    sprite_zero_hit: bool,
    sprite_zero_rendered: bool,

}

struct Sprite {
    x: u8,
    y: u8,
    id: u8,
    attribute: u8, 
}

impl Sprite {
    pub fn new(bytes: &[u8]) -> Self {
        Sprite {
            y: bytes[0],
            id: bytes[1],
            attribute: bytes[2],
            x: bytes[3],
        }
    }
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            cartridge: None,
            name_table: [[0; 1024]; 2],
            palette_table: [0; 32],
            pattern_table: [[0; 4096]; 2],
            frame_complete: false,
            sprite_screen: [2; RENDER_SIZE],
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
            nmi_enabled: false,
            bg_tile_id: 0,
            bg_tile_attr: 0,
            bg_tile_lsb: 0,
            bg_tile_msb: 0,
            bg_shifter_lsb: 0,
            bg_shifter_msb: 0,
            bg_shifter_attr_low: 0,
            bg_shifter_attr_high: 0,

            oam_ram: [0; 256],
            oam_sprites: Vec::with_capacity(8),
            oam_address_port: 0,
            sprite_shifter_low: [0; 8],
            sprite_shifter_high: [0; 8],

            sprite_zero_hit: false,
            sprite_zero_rendered: false,
        }
    }
    pub fn cpu_read(&mut self, address: u16, read_only: bool) -> u8 {
        let mut data: u8 = 0x00;
        match address {
            0x0000 => { //Control
            }
            0x0001 => (), //Mask
            0x0002 => {
                //Status
                data = (self.status.get() & 0xE0) | (self.data_buffer & 0x1F);
                self.status.set_vblank(false);
                self.address_latch = 0;
            }
            0x0003 => (), //OAM Address
            0x0004 => {
                //OAM Data
                data = self.oam_ram[self.oam_address_port as usize];
            }
            0x0005 => (), //Scroll
            0x0006 => (), //PPU Address
            0x0007 => {
                //PPU data
                data = self.data_buffer;
                self.data_buffer = self.ppu_read(self.v_address_register.get(), read_only);

                if self.v_address_register.get() >= 0x3F00 {
                    data = self.data_buffer;
                }
                if self.controller.increment() == true {
                    self.v_address_register =
                        Address(self.v_address_register.get().wrapping_add(32));
                } else {
                    self.v_address_register =
                        Address(self.v_address_register.get().wrapping_add(1));
                }
            }
            _ => (), //required by rust
        }
        return data;
    }
    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            0x0000 => {
                //Control
                self.controller = Controller(data);
                self.t_address_register
                    .set_nametable_x(self.controller.nametable_x());
                self.t_address_register
                    .set_nametable_y(self.controller.nametable_y());
            }
            0x0001 => {
                //Mask
                self.mask = Mask(data)
            }
            0x0002 => {
                //Status
            }
            0x0003 => {
                //OAM Address
                self.oam_address_port = data;
            }
            0x0004 => {
                //OAM Data
                self.oam_ram[self.oam_address_port as usize] = data;
            }
            0x0005 => {
                //Scroll
                if self.address_latch == 0 {
                    self.fine_x = data & 0x07;
                    self.t_address_register.set_coarse_x(data >> 3);
                    self.address_latch = 1;
                } else {
                    self.t_address_register.set_fine_y(data & 0x07);
                    self.t_address_register.set_coarse_y(data >> 3);
                    self.address_latch = 0;
                }
            }
            0x0006 => {
                //PPU Address
                if self.address_latch == 0 {
                    let t_address =
                        (((data & 0x3F) as u16) << 8) | (self.t_address_register.get() & 0x00FF);
                    self.t_address_register = Address(t_address);
                    self.address_latch = 1;
                } else {
                    let t_address = (self.t_address_register.get() & 0xFF00) | data as u16;
                    self.t_address_register = Address(t_address);
                    self.v_address_register = self.t_address_register;
                    self.address_latch = 0;
                }
            }
            0x0007 => {
                //PPU Data
                self.ppu_write(self.v_address_register.get(), data);

                if self.controller.increment() == true {
                    self.v_address_register =
                        Address(self.v_address_register.get().wrapping_add(32));
                } else {
                    self.v_address_register =
                        Address(self.v_address_register.get().wrapping_add(1));
                }
            }
            _ => (), //required by rust
        }
    }

    pub fn render(&mut self) -> [u8; RENDER_FULL] {
        let mut frame = [0; RENDER_FULL];
        for i in 0..RENDER_SIZE {
            let c = self.sprite_screen[i];
            

            let (mut r, mut g, mut b) = SYSTEM_PALETTE[c as usize];
            if self.mask.emphasize_red(){r = r * 2;}
            if self.mask.emphasize_blue(){b = b * 2;}
            if self.mask.emphasize_green(){g = g * 2;}

            frame[i * 3 + 0] = r;
            frame[i * 3 + 1] = g;
            frame[i * 3 + 2] = b;
        }
        return frame;
    }

    #[allow(unused_comparisons)]
    pub fn ppu_read(&mut self, mut address: u16, _read_only: bool) -> u8 {
        let mut data: u8 = 0x00;
        address &= 0x3FFF;

        if let Some(ref c) = self.cartridge {
            if c.borrow_mut().ppu_read(address, &mut data) {
                //Should always be false
            } else if address >= 0x0000 && address <= 0x1FFF {
                //Pattern memory
                //First index chooses whether it's the left or the right pattern table, second is index within that table
                let first_index = ((address & 0x1000) >> 12) as usize;
                let second_index = (address & 0x0FFF) as usize;
                data = self.pattern_table[first_index][second_index];
            } else if address >= 0x2000 && address <= 0x3EFF {
                //Nametable memory
                address &= 0x0FFF;
                if c.borrow_mut().mirror() == Mirroring::Vertical {
                    if address >= 0x0000 && address <= 0x03FF {
                        data = self.name_table[0][(address & 0x03FF) as usize];
                    }
                    if address >= 0x0400 && address <= 0x07FF {
                        data = self.name_table[1][(address & 0x03FF) as usize];
                    }
                    if address >= 0x0800 && address <= 0x0BFF {
                        data = self.name_table[0][(address & 0x03FF) as usize];
                    }
                    if address >= 0x0C00 && address <= 0x0FFF {
                        data = self.name_table[1][(address & 0x03FF) as usize];
                    }
                } else if c.borrow_mut().mirror() == Mirroring::Horizontal {
                    if address >= 0x0000 && address <= 0x03FF {
                        data = self.name_table[0][(address & 0x03FF) as usize];
                    }
                    if address >= 0x0400 && address <= 0x07FF {
                        data = self.name_table[0][(address & 0x03FF) as usize];
                    }
                    if address >= 0x0800 && address <= 0x0BFF {
                        data = self.name_table[1][(address & 0x03FF) as usize];
                    }
                    if address >= 0x0C00 && address <= 0x0FFF {
                        data = self.name_table[1][(address & 0x03FF) as usize];
                    }
                }
            } else if address >= 0x3F00 && address <= 0x3FFF {
                //Palette memory
                address &= 0x001F;
                if address == 0x0010 {
                    address = 0x0000
                }
                if address == 0x0014 {
                    address = 0x0004
                }
                if address == 0x0018 {
                    address = 0x0008
                }
                if address == 0x001C {
                    address = 0x000C
                }
                if self.mask.greyscale() == true {
                    data = self.palette_table[address as usize] & 0x30;
                } else {
                    data = self.palette_table[address as usize] & 0x3F;
                }
            }
        }

        return data;
    }
    #[allow(unused_comparisons)]
    pub fn ppu_write(&mut self, mut address: u16, data: u8) {
        address &= 0x3FFF;
        if let Some(ref c) = self.cartridge {
            if c.borrow_mut().ppu_write(address, data) {
                //Should always be false
            } else if address >= 0x0000 && address <= 0x1FFF {
                //Pattern memory, usually a ROM however some games need to write to it
                let first_index = ((address & 0x1000) >> 12) as usize;
                self.pattern_table[first_index][(address & 0x0FFF) as usize] = data;
            } else if address >= 0x2000 && address <= 0x3EFF {
                //Nametable memory
                address &= 0x0FFF;
                if c.borrow_mut().mirror() == Mirroring::Vertical {
                    if address >= 0x0000 && address <= 0x03FF {
                        self.name_table[0][(address & 0x03FF) as usize] = data;
                    }
                    if address >= 0x0400 && address <= 0x07FF {
                        self.name_table[1][(address & 0x03FF) as usize] = data;
                    }
                    if address >= 0x0800 && address <= 0x0BFF {
                        self.name_table[0][(address & 0x03FF) as usize] = data;
                    }
                    if address >= 0x0C00 && address <= 0x0FFF {
                        self.name_table[1][(address & 0x03FF) as usize] = data;
                    }
                } else if c.borrow_mut().mirror() == Mirroring::Horizontal {
                    if address >= 0x0000 && address <= 0x03FF {
                        self.name_table[0][(address & 0x03FF) as usize] = data;
                    }
                    if address >= 0x0400 && address <= 0x07FF {
                        self.name_table[0][(address & 0x03FF) as usize] = data;
                    }
                    if address >= 0x0800 && address <= 0x0BFF {
                        self.name_table[1][(address & 0x03FF) as usize] = data;
                    }
                    if address >= 0x0C00 && address <= 0x0FFF {
                        self.name_table[1][(address & 0x03FF) as usize] = data;
                    }
                }
            } else if address >= 0x3F00 && address <= 0x3FFF {
                //Palette memory
                address = address & 0x001F;
                if address == 0x0010 {
                    address = 0x0000
                }
                if address == 0x0014 {
                    address = 0x0004
                }
                if address == 0x0018 {
                    address = 0x0008
                }
                if address == 0x001C {
                    address = 0x000C
                }
                self.palette_table[address as usize] = data;
            }
        }
    }
    pub fn connect_cartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>) {
        self.cartridge = Some(cartridge);
    }

    pub fn get_pattern_table(&mut self, index: u8, palette: u8) -> [u8; (128 * 128) * 3] {
        for tile_y in 0..16 {
            for tile_x in 0..16 {
                let offset: u16 = (tile_y * 256) + (tile_x * 16);

                for row in 0..8 {
                    let address: u16 = index as u16 * 0x1000 + offset + row;
                    let mut tile_ls = self.ppu_read(address, false);
                    let mut tile_ms = self.ppu_read(address + 8, false);

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
            let (r, g, b) = SYSTEM_PALETTE[c as usize];
            frame[i * 3 + 0] = r;
            frame[i * 3 + 1] = g;
            frame[i * 3 + 2] = b;
        }
        return frame;
    }

    pub fn get_colour(&mut self, palette: u8, pixel: u8) -> u8 {
        let address: u16 = 0x3F00 + ((palette << 2) as u16) + pixel as u16;
        let i = self.ppu_read(address, false);
        return i & 0x3F;
    }

    fn draw_pixel(&mut self, x: i32, y: i32, c: SystemColor) {
        if x >= 0 && y >= 0 && x < 256 && y < 240 {
            let i = (x + 256 * y) as usize;
            self.sprite_screen[i] = c;
        }
    }

    fn write_pattern_pixel(&mut self, x: u16, y: u16, c: SystemColor, index: usize) {
        let i = (x + 128 * y) as usize;
        self.sprite_pattern_table[index][i] = c;
    }

    fn scroll_x(&mut self) {
        if self.mask.show_background() || self.mask.show_sprites() {
            if self.v_address_register.coarse_x() == 31 {
                self.v_address_register.set_coarse_x(0);
                self.v_address_register.0 ^= 0x0400;
            } else {
                let nx = self.v_address_register.coarse_x();
                self.v_address_register.set_coarse_x(nx + 1);
            }
        }
    }
    fn scroll_y(&mut self) {
        if self.mask.show_background() || self.mask.show_sprites() {
            if self.v_address_register.fine_y() < 7 {
                self.v_address_register
                    .set_fine_y(self.v_address_register.fine_y() + 1);
            } else {
                self.v_address_register.set_fine_y(0);
                if self.v_address_register.coarse_y() == 29 {
                    self.v_address_register.set_coarse_y(0);
                    self.v_address_register.0 ^= 0x0800; // Switch vertical nametable
                } else if self.v_address_register.coarse_y() == 31 {
                    self.v_address_register.set_coarse_y(0);
                } else {
                    self.v_address_register
                        .set_coarse_y(self.v_address_register.coarse_y() + 1);
                }
            }
        }
    }
    fn reset_x(&mut self) {
        if self.mask.show_background() || self.mask.show_sprites() {
            self.v_address_register
                .set_nametable_x(self.t_address_register.nametable_x());
            self.v_address_register
                .set_coarse_x(self.t_address_register.coarse_x());
        }
    }
    fn reset_y(&mut self) {
        if self.mask.show_background() || self.mask.show_sprites() {
            self.v_address_register
                .set_fine_y(self.t_address_register.fine_y());
            self.v_address_register
                .set_nametable_y(self.t_address_register.nametable_y());
            self.v_address_register
                .set_coarse_y(self.t_address_register.coarse_y());
        }
    }

    pub fn reset(&mut self) {
        self.fine_x = 0x00;
        self.address_latch = 0x00;
        self.data_buffer = 0x00;
        self.scanline = 0;
        self.cycle = 0;
        self.bg_tile_id = 0;
        self.bg_tile_attr = 0;
        self.bg_tile_lsb = 0;
        self.bg_tile_msb = 0;
        self.bg_shifter_attr_high = 0;
        self.bg_shifter_attr_low = 0;
        self.bg_shifter_lsb = 0;
        self.bg_shifter_msb = 0;
        self.status = Status(0);
        self.mask = Mask(0);
        self.controller = Controller(0);
        self.v_address_register = Address(0);
        self.t_address_register = Address(0);
    }
    fn load_bg_shifters(&mut self) {
        self.bg_shifter_lsb = (self.bg_shifter_lsb & 0xFF00) | (self.bg_tile_lsb as u16);
        self.bg_shifter_msb = (self.bg_shifter_msb & 0xFF00) | (self.bg_tile_msb as u16);

        if (self.bg_tile_attr & 0b01) > 0 {
            self.bg_shifter_attr_low = (self.bg_shifter_attr_low & 0xFF00) | 0xFF;
        } else {
            self.bg_shifter_attr_low = (self.bg_shifter_attr_low & 0xFF00) | 0x00;
        }

        if (self.bg_tile_attr & 0b10) > 0 {
            self.bg_shifter_attr_high = (self.bg_shifter_attr_high & 0xFF00) | 0xFF;
        } else {
            self.bg_shifter_attr_high = (self.bg_shifter_attr_high & 0xFF00) | 0x00;
        }
    }

    fn update_shifters(&mut self) {
        if self.mask.show_background() {
            self.bg_shifter_attr_high <<= 1;
            self.bg_shifter_attr_low <<= 1;

            self.bg_shifter_lsb <<= 1;
            self.bg_shifter_msb <<= 1;
        }
        if self.mask.show_sprites() && self.cycle >= 1 && self.cycle < 258{
            for i in 0..self.oam_sprites.len()
            {
                if self.oam_sprites[i].x > 0
                {
                    self.oam_sprites[i].x -= 1;
                }
                else
                {
                    self.sprite_shifter_low[i] <<= 1;
                    self.sprite_shifter_high[i] <<= 1;
                }
            }
        }
    }
    #[allow(unused_assignments)]
    fn evaluate_sprites(&mut self) {
        self.oam_sprites.clear();
        self.sprite_zero_hit = false;
        for i in 0..64 {
            let address = i * 4;
            let sprite = Sprite::new(&self.oam_ram[address..address + 4]);

            let left = self.scanline as usize >= sprite.y as usize;

            let mut right = false;
            if self.controller.sprite_size() 
            {
                right = (self.scanline as usize) < sprite.y as usize + 16 as usize;
            }else
            {
                right = (self.scanline as usize) < sprite.y as usize + 8 as usize;
            }

            if left && right
            {
                if self.oam_sprites.len() == 8 {
                    if self.mask.show_background() || self.mask.show_sprites()
                    {
                        self.status.set_sprite_overflow(true);
                    }
                    break;
                }
                if i == 0
                {
                    self.sprite_zero_hit = true;
                }
                self.oam_sprites.push(sprite);
            }
        }
    }

    fn reverse_bits(mut a: u8) -> u8 {
        let mut out = 0u8;
        for _i in 0..8 {
            out <<= 1;
            out |= a & 1;
            a >>= 1;
        }
        return out;
    }

    #[allow(unused_assignments)]
    pub fn clock(&mut self) {
        if self.scanline >= -1 && self.scanline < 240 {
            if self.scanline == 0 && self.cycle == 0 {
                self.cycle = 1;
            }

            if self.scanline == -1 && self.cycle == 1 {
                self.status.set_vblank(false);
                self.status.set_sprite_0(false);
                self.status.set_sprite_overflow(false);

                for i in 0..8{
                    self.sprite_shifter_low[i] = 0;
                    self.sprite_shifter_high[i] = 0;
                }
            }

            if (self.cycle >= 2 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338) {
                self.update_shifters();

                match (self.cycle - 1) % 8 {
                    0 => {
                        self.load_bg_shifters();
                        let address = 0x2000 | (self.v_address_register.get() & 0x0FFF);
                        self.bg_tile_id = self.ppu_read(address, false); //Correct
                    }
                    2 => {
                        self.bg_tile_attr = self.ppu_read(
                            0x23C0
                                | ((self.v_address_register.nametable_y() as u16) << 11)
                                | ((self.v_address_register.nametable_x() as u16) << 10)
                                | (((self.v_address_register.coarse_y()) >> 2) << 3) as u16
                                | ((self.v_address_register.coarse_x()) >> 2) as u16,
                            false,
                        );

                        if (self.v_address_register.coarse_y() & 0x02) > 0 {
                            self.bg_tile_attr >>= 4;
                        }
                        if (self.v_address_register.coarse_x() & 0x02) > 0 {
                            self.bg_tile_attr >>= 2;
                        }
                        self.bg_tile_attr &= 0x03;
                    }
                    4 => {
                        self.bg_tile_lsb = self.ppu_read(
                            ((self.controller.background_table() as u16) << 12)
                                + (((self.bg_tile_id) as u16) << 4)
                                + ((self.v_address_register.fine_y() as u16) + 0),
                            false,
                        );
                    }
                    6 => {
                        self.bg_tile_msb = self.ppu_read(
                            ((self.controller.background_table() as u16) << 12)
                                + (((self.bg_tile_id) as u16) << 4)
                                + ((self.v_address_register.fine_y() as u16) + 8),
                            false,
                        );
                    }
                    7 => {
                        self.scroll_x();
                    }
                    _ => (),
                }
            }
            if self.cycle == 256 {
                self.scroll_y();
            }
            if self.cycle == 257 {
                self.load_bg_shifters();
                self.reset_x();
            }
            if self.cycle == 338 || self.cycle == 340 {
                self.bg_tile_id =
                    self.ppu_read(0x2000 | (self.v_address_register.get() & 0x0FFF), false);
            }
            if self.scanline == -1 && self.cycle >= 280 && self.cycle < 305 {
                self.reset_y();
            }

            //Sprite Rendering
            if self.cycle == 257 && self.scanline >= 0 
            {
                for i in 0..8{
                    self.sprite_shifter_low[i] = 0;
                    self.sprite_shifter_high[i] = 0;
                }
                self.evaluate_sprites();
            }
            if self.cycle == 340
            {
                for i in 0..self.oam_sprites.len() 
                {
                    let mut sprite_bits_low: u8 = 0;
                    let mut sprite_bits_high: u8 = 0;

                    let mut sprite_address_low: u16 = 0;
                    let mut sprite_address_high: u16 = 0;

                    if !self.controller.sprite_size(){
                        //8x8
                        if !(self.oam_sprites[i].attribute & 0x80 > 0)
                        {
                            let low = (self.controller.sprite_table() as u16) << 12;
                            let high = (self.oam_sprites[i].id as u16) << 4;
                            let sl = (self.scanline as u16) - self.oam_sprites[i].y as u16;

                            sprite_address_low = low | high | sl;
                        }
                        else    //Sprite is flipped
                        {
                            let low = (self.controller.sprite_table() as u16) << 12;
                            let high = (self.oam_sprites[i].id as u16) << 4;
                            let sl = (self.scanline as u16) - self.oam_sprites[i].y as u16;

                            sprite_address_low = low | high | 7 - sl;
                        }
                    }
                    else
                    {
                        //8x16
                        if !(self.oam_sprites[i].attribute & 0x80 > 0)
                        {
                            if (self.scanline as u16 - self.oam_sprites[i].y as u16) < 8{
                                //Top half of sprite
                                let low = ((self.oam_sprites[i].id as u16) & 0x01) << 12;
                                let high = ((self.oam_sprites[i].id as u16) & 0xFE) << 4;
                                let sl = ((self.scanline as u16) - self.oam_sprites[i].y as u16) & 0x07;

                                sprite_address_low = low | high | sl;

                            }else
                            {
                                //Bottom half of sprite
                                let low = ((self.oam_sprites[i].id  as u16) & 0x01) << 12;
                                let high = (((self.oam_sprites[i].id as u16) & 0xFE) + 1) << 4;
                                let sl = ((self.scanline as u16) - self.oam_sprites[i].y as u16) & 0x07;

                                sprite_address_low = low | high | sl;
                            }
                        }
                        else    //Sprite is flipped
                        {
                            if (self.scanline as u16 - self.oam_sprites[i].y as u16) < 8{
                                //Top half of sprite
                                let low = ((self.oam_sprites[i].id as u16) & 0x01) << 12;
                                let high = ((self.oam_sprites[i].id as u16) & 0xFE) << 4;
                                let sl = ((self.scanline as u16) - self.oam_sprites[i].y as u16) & 0x07;
                               
                                sprite_address_low = low | high | 7 - sl;

                            }else
                            {
                                //Bottom half of sprite                               
                                let low = ((self.oam_sprites[i].id as u16) & 0x01) << 12;
                                let high = (((self.oam_sprites[i].id as u16) & 0xFE)+ 1) << 4;
                                let sl = ((self.scanline as u16) - self.oam_sprites[i].y as u16) & 0x07;

                                sprite_address_low = low | high | 7 - sl; //KEEP AN EYE ON THIS WITH ABOVE & OPERATOR
                            }
                        }
                    }

                    sprite_address_high = sprite_address_low + 8;
                    sprite_bits_low = self.ppu_read(sprite_address_low, false);
                    sprite_bits_high = self.ppu_read(sprite_address_high, false);

                    if (self.oam_sprites[i].attribute & 0x40) > 0{
                        sprite_bits_low = PPU::reverse_bits(sprite_bits_low);
                        sprite_bits_high = PPU::reverse_bits(sprite_bits_high);
                    }

                    self.sprite_shifter_low[i] = sprite_bits_low;
                    self.sprite_shifter_high[i] = sprite_bits_high;
                }
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

        if self.mask.show_background() {
            let bit_mask = 0x8000 >> (self.fine_x);
            let plane_0 = ((self.bg_shifter_lsb & bit_mask) > 0) as u8;
            let plane_1 = ((self.bg_shifter_msb & bit_mask) > 0) as u8;
            background_pixel = (plane_1 << 1) | plane_0;

            let palette_0 = ((self.bg_shifter_attr_low & bit_mask) > 0) as u8;
            let palette_1 = ((self.bg_shifter_attr_high & bit_mask) > 0) as u8;

            background_palette = (palette_1 << 1) | palette_0;
        }
        
        let mut sprite_pixel: u8 = 0x00;
        let mut sprite_pal: u8 = 0x00;
        let mut sprite_z_buffer: u8 = 0x00;

        if self.mask.show_sprites(){
            self.sprite_zero_rendered = false;
            for i in 0..self.oam_sprites.len(){
                if self.oam_sprites[i].x == 0 {
                     let sp_pixel_low = ((self.sprite_shifter_low[i] & 0x80) > 0) as u8;
                     let sp_pixel_high = ((self.sprite_shifter_high[i] & 0x80) > 0) as u8;

                     sprite_pixel = (sp_pixel_high << 1) | sp_pixel_low;

                     sprite_pal = (self.oam_sprites[i].attribute & 0x03) + 0x04;
                     sprite_z_buffer = ((self.oam_sprites[i].attribute & 0x20) == 0) as u8;

                     if sprite_pixel != 0
                     {
                        if i == 0
                        {
                            self.sprite_zero_rendered = true;
                        }
                        break;
                     }
                }
            }
        }

        let mut final_pixel: u8 = 0x00;
        let mut final_palette: u8 = 0x00;

        if background_pixel == 0 && sprite_pixel == 0{
            final_pixel = 0x00;
            final_palette = 0x00;
        }
        else if background_pixel == 0 && sprite_pixel > 0
        {
            final_pixel = sprite_pixel;
            final_palette = sprite_pal;
        }        
        else if background_pixel > 0 && sprite_pixel == 0
        {
            final_pixel = background_pixel;
            final_palette = background_palette;
        }
        else if background_pixel > 0 && sprite_pixel > 0
        {
            if sprite_z_buffer > 0{
                final_pixel = sprite_pixel;
                final_palette = sprite_pal;
            }
            else
            {
                final_pixel = background_pixel;
                final_palette = background_palette;
            }

            if self.sprite_zero_hit && self.sprite_zero_rendered 
            {
                if self.mask.show_background() && self.mask.show_sprites()
                {
                    if !(self.mask.show_background_left() | self.mask.show_sprites_left())
                    {
                        if self.cycle >= 9 && self.cycle < 258
                        {
                            self.status.set_sprite_0(true);
                        }
                    }
                    else 
                    {
                        if self.cycle >= 1 && self.cycle < 258
                        {
                            self.status.set_sprite_0(true);
                        }
                    }
                }
            }
        }

        let colour = self.get_colour(final_palette, final_pixel);
        self.draw_pixel(self.cycle - 1, self.scanline, colour);

        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline >= 261 {
                self.scanline = -1;
                self.frame_complete = true;
            }
        }
    }
}
pub type SystemColor = u8;
pub const SYSTEM_PALETTE: [(u8, u8, u8); 64] = [
    //0x00 - 0x0F
    (84, 84, 84),
    (0, 30, 116),
    (8, 16, 144),
    (48, 0, 136),
    (68, 0, 100),
    (92, 0, 48),
    (84, 4, 0),
    (60, 24, 0),
    (32, 42, 0),
    (8, 58, 0),
    (0, 64, 0),
    (0, 60, 0),
    (0, 50, 60),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    // 0x10 - 0x1F
    (152, 150, 152),
    (8, 76, 196),
    (48, 50, 236),
    (92, 30, 228),
    (136, 20, 76),
    (160, 20, 100),
    (152, 34, 32),
    (120, 60, 0),
    (84, 90, 0),
    (40, 114, 0),
    (8, 124, 0),
    (0, 118, 40),
    (0, 102, 120),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    // 0x20 - 0x2F
    (236, 238, 236),
    (76, 154, 236),
    (120, 124, 236),
    (176, 98, 236),
    (228, 84, 236),
    (236, 88, 180),
    (236, 106, 100),
    (212, 136, 32),
    (160, 170, 0),
    (116, 196, 0),
    (76, 208, 32),
    (56, 204, 108),
    (56, 180, 204),
    (60, 60, 60),
    (0, 0, 0),
    (0, 0, 0),
    // 0x30 - 0x3F
    (236, 238, 236),
    (168, 204, 236),
    (188, 188, 236),
    (212, 178, 236),
    (236, 174, 236),
    (236, 174, 212),
    (236, 180, 176),
    (228, 196, 144),
    (204, 210, 120),
    (180, 222, 120),
    (168, 226, 144),
    (152, 226, 180),
    (160, 214, 228),
    (160, 162, 160),
    (0, 0, 0),
    (0, 0, 0),
];
