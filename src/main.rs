extern crate sdl2;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate bitfield;
use ppu::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::TextureAccess;
use sdl2::render::Texture;

use sdl2::render::WindowCanvas;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::{Instant};

pub mod cpu_6502;
pub mod bus;
pub mod cartridge;
pub mod mapper;
pub mod mapper_0;
pub mod ppu;

static SCREEN_WIDTH: u32 = 1280;
static SCREEN_HEIGHT: u32 = 720;

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

fn update(nes: &mut cpu_6502::CPU6502) {
    while {
        nes.clock();
        !nes.complete()
    } {}
}

fn update_full_frame(nes: &mut cpu_6502::CPU6502) {
    while {
        nes.clock();
        !nes.bus.ppu.frame_complete
    } {}
    while {
        nes.clock();
        !nes.complete()
    } {}
}

fn main() -> Result<(), String> {
    let now = Instant::now();
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let font_path: &Path = Path::new("./assets/monogram.ttf");

    let window = video_subsys
        .window(" ", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let tx1 = canvas.texture_creator();
    let mut screen_texture: Box<sdl2::render::Texture> = {
        let tex = tx1
            .create_texture(
                PixelFormatEnum::RGB24,
                TextureAccess::Streaming,
                RENDER_WIDTH as u32,
                RENDER_HEIGHT as u32,
            )
            .unwrap();
        unsafe { Box::new(std::mem::transmute(tex)) }
    };

    let tx2 = canvas.texture_creator();
    let mut pattern_one: Box<sdl2::render::Texture> = {
        let tex2 = tx2
            .create_texture(
                PixelFormatEnum::RGB24,
                TextureAccess::Streaming,
                128 as u32,
                128 as u32,
            )
            .unwrap();
        unsafe { Box::new(std::mem::transmute(tex2)) }
    };

    let tx3 = canvas.texture_creator();
    let mut pattern_two: Box<sdl2::render::Texture> = {
        let tex3 = tx3
            .create_texture(
                PixelFormatEnum::RGB24,
                TextureAccess::Streaming,
                128 as u32,
                128 as u32,
            )
            .unwrap();
        unsafe { Box::new(std::mem::transmute(tex3)) }
    };

    // Load a font
    let mut font = ttf_context.load_font(font_path, 128)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    let mut nes = cpu_6502::CPU6502::new();
    let cartridge = cartridge::Cartridge::new("/Users/multivac/NES/source/src/roms/nestest.nes".to_string());
    nes.bus.connect_cartridge(Rc::new(RefCell::new(cartridge)));


    let disassembly = nes.disassemble(0x0000, 0xFFFF);
    nes.reset();

    let mut count = 0;

    let mut emulation_run = false;
    let mut time: f32 = 0.0;

    let mut palette = 0;


    'mainloop: loop {
        nes.bus.controller[0] = 0x00;
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::X),  //B
                    ..
                } => {
                    nes.bus.controller[0] |= 0x80;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Z),  //A
                    ..
                } => {
                    nes.bus.controller[0] |= 0x40;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),  //Start
                    ..
                } => {
                    nes.bus.controller[0] |= 0x20;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),  //Select
                    ..
                } => {
                    nes.bus.controller[0] |= 0x10;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up), //D-Pad up
                    ..
                } => {
                    nes.bus.controller[0] |= 0x08;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down), //D-Pad down
                    ..
                } => {
                    nes.bus.controller[0] |= 0x04;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),   //D-pad Left
                    ..
                } => {
                    nes.bus.controller[0] |= 0x02;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),  //D-Pad Right
                    ..
                } => {
                    nes.bus.controller[0] |= 0x01;

                }





                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => {
                    update(&mut nes);
                    count = count + 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    nes.reset();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    emulation_run = !emulation_run;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => {
                    update_full_frame(&mut nes);
                    nes.bus.ppu.frame_complete = false;
                    count = count + 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                | Event::Quit { .. } => break 'mainloop,
                _ => {}
            }
        }
        if emulation_run == true {
            if time > 0.0 {
                time = time - now.elapsed().as_secs_f32();
            } else {
                time = time + (1.0 / 60.0);
                while nes.bus.ppu.frame_complete == false {
                    nes.clock();
                    
                }
                nes.bus.ppu.frame_complete = false;
            }
        }
        canvas.clear();
        let pc = nes.pc;
        {
            draw_line(
                rect!(900, 10, 200, 20),
                "Status Registers: ",
                &mut canvas,
                &font,
                Color::WHITE,
            );
            if nes.get_flag(cpu_6502::Flags::N) == 0 {
                draw_line(rect!(900, 40, 20, 20), "N", &mut canvas, &font, Color::RED);
            } else {
                draw_line(
                    rect!(900, 40, 20, 20),
                    "N",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
            if nes.get_flag(cpu_6502::Flags::V) == 0 {
                draw_line(rect!(930, 40, 20, 20), "V", &mut canvas, &font, Color::RED);
            } else {
                draw_line(
                    rect!(930, 40, 20, 20),
                    "V",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
            if nes.get_flag(cpu_6502::Flags::U) == 0 {
                draw_line(rect!(960, 40, 20, 20), "U", &mut canvas, &font, Color::RED);
            } else {
                draw_line(
                    rect!(960, 40, 20, 20),
                    "U",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
            if nes.get_flag(cpu_6502::Flags::B) == 0 {
                draw_line(rect!(990, 40, 20, 20), "B", &mut canvas, &font, Color::RED);
            } else {
                draw_line(
                    rect!(990, 40, 20, 20),
                    "B",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
            if nes.get_flag(cpu_6502::Flags::D) == 0 {
                draw_line(rect!(1020, 40, 20, 20), "D", &mut canvas, &font, Color::RED);
            } else {
                draw_line(
                    rect!(1020, 40, 20, 20),
                    "D",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
            if nes.get_flag(cpu_6502::Flags::I) == 0 {
                draw_line(rect!(1050, 40, 20, 20), "I", &mut canvas, &font, Color::RED);
            } else {
                draw_line(
                    rect!(1050, 40, 20, 20),
                    "I",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
            if nes.get_flag(cpu_6502::Flags::Z) == 0 {
                draw_line(rect!(1080, 40, 20, 20), "Z", &mut canvas, &font, Color::RED);
            } else {
                draw_line(
                    rect!(1080, 40, 20, 20),
                    "Z",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
            if nes.get_flag(cpu_6502::Flags::C) == 0 {
                draw_line(rect!(1110, 40, 20, 20), "C", &mut canvas, &font, Color::RED);
            } else {
                draw_line(
                    rect!(1110, 40, 20, 20),
                    "C",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
        }
        {
            let mut a_reg = "A: ".to_owned();
            a_reg.push_str(&(format!("{:X}", &nes.a)));
            a_reg.push_str(" [");
            a_reg.push_str(&(nes.a).to_string());
            a_reg.push_str("]");
            draw_line(
                rect!(900, 80, 200, 30),
                &a_reg,
                &mut canvas,
                &font,
                Color::WHITE,
            );

            let mut x_reg = "X: ".to_owned();
            x_reg.push_str(&(format!("{:X}", &nes.x)));
            x_reg.push_str(" [");
            x_reg.push_str(&(nes.x).to_string());
            x_reg.push_str("]");
            draw_line(
                rect!(900, 110, 200, 30),
                &x_reg,
                &mut canvas,
                &font,
                Color::WHITE,
            );

            let mut y_reg = "Y: ".to_owned();
            y_reg.push_str(&(format!("{:X}", &nes.y)));
            y_reg.push_str(" [");
            y_reg.push_str(&(nes.y).to_string());
            y_reg.push_str("]");
            draw_line(
                rect!(900, 140, 200, 30),
                &y_reg,
                &mut canvas,
                &font,
                Color::WHITE,
            );
        }
        let mut pc_txt = ("PC: ").to_owned();
        pc_txt.push_str(&(format!("{:X}", &pc)));
        draw_line(
            rect!(900, 170, 200, 30),
            &pc_txt,
            &mut canvas,
            &font,
            Color::WHITE,
        );
        let mut i = 0;

        for x in 0..20 {
            let val = (nes.pc as u32 + x as u32);
            let end = disassembly.capacity() as u32;
            if val <= end {
                let iteration = disassembly.get(&(val));
                let text = String::from("Error");
                let val = iteration.unwrap_or(&text);
                if val != "Error" {
                    i = i + 1;
                    draw_line(
                        rect!(900, 170 + (i * 50), 300, 40),
                        &val,
                        &mut canvas,
                        &font,
                        Color::WHITE,
                    );
                }
            }
        }

        render_frame(&mut canvas, &mut nes, rect!(0, 0, RENDER_WIDTH * 3, RENDER_HEIGHT* 3), &mut screen_texture);
        // render_pattern_table(&mut canvas, &mut nes, rect!(900, 500, 256, 256), &mut pattern_one, 0, palette);
        // render_pattern_table(&mut canvas, &mut nes, rect!(1030, 580, 256, 256), &mut pattern_two, 1, palette);
        
        canvas.present();
    }
    Ok(())
}

fn draw_line(
    rect: sdl2::rect::Rect,
    text: &str,
    canvas: &mut WindowCanvas,
    font: &sdl2::ttf::Font,
    color: sdl2::pixels::Color,
) {
    let texture_creator = canvas.texture_creator();
    let surface = font.render(&text).blended(color).map_err(|e| e.to_string());
    let texture = texture_creator
        .create_texture_from_surface(&surface.unwrap())
        .map_err(|e| e.to_string());

    canvas.copy(&texture.unwrap(), None, Some(rect)).unwrap();
}

fn render_frame(canvas: &mut WindowCanvas, nes: &mut cpu_6502::CPU6502, rect: sdl2::rect::Rect, tex: &mut Texture) {
    let frame_data = nes.bus.ppu.render();
    tex.update(None, &frame_data, 256 * 3).unwrap();
    canvas.copy(&tex, None, Some(rect)).unwrap();
}

fn render_pattern_table(canvas: &mut WindowCanvas, nes: &mut cpu_6502::CPU6502, rect: sdl2::rect::Rect, tex: &mut Texture, index: u8, palette: u8){
    let frame_data = nes.bus.ppu.get_pattern_table(index, palette);
    tex.update(None, &frame_data, 128*3).unwrap();
    canvas.copy(&tex, None, Some(rect)).unwrap();
}


//#[cfg(test)]
