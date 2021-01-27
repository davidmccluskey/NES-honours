extern crate sdl2;
#[macro_use]
extern crate bitflags;
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
use std::fmt::{Error, Write};
use std::path::Path;
use std::rc::Rc;
use std::time::{Instant};

pub mod CPU6502;
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

fn update(nes: &mut CPU6502::CPU6502) {
    while {
        nes.clock();
        !nes.complete()
    } {}
}

fn render_frame(nes: &mut CPU6502::CPU6502) {
    while {
        nes.clock();
        !nes.bus.ppu.frame_complete
    } {}
    while {
        nes.clock();
        !nes.complete()
    } {}
}

fn writer<W: Write>(f: &mut W, s: &str) -> Result<(), Error> {
    f.write_str(s)
}

fn main() -> Result<(), String> {
    let now = Instant::now();
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let font_path: &Path = Path::new("./assets/PressStart2P-Regular.ttf");

    let window = video_subsys
        .window(" ", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut texture = {
        let mut tex = texture_creator
            .create_texture(
                PixelFormatEnum::RGB24,
                TextureAccess::Streaming,
                RENDER_WIDTH as u32,
                RENDER_HEIGHT as u32,
            )
            .unwrap();
        unsafe { Box::new(std::mem::transmute(tex)) }
    };

    // Load a font
    let mut font = ttf_context.load_font(font_path, 128)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    let mut nes = CPU6502::CPU6502::new();
    let mut cartridge =
        cartridge::Cartridge::new("/Users/multivac/NES/source/src/roms/nestest.nes".to_string());
    nes.bus.connect_cartridge(Rc::new(RefCell::new(cartridge)));

    let disassembly = nes.disassemble(0x0000, 0xFFFF);

    nes.reset();

    let mut count = 0;

    let mut emulation_run = false;
    let mut time: f32 = 0.0;

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
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
                    render_frame(&mut nes);
                    nes.bus.ppu.frame_complete = false;
                    count = count + 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
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
        draw_sprite(&mut canvas, &mut nes, rect!(0, 0, RENDER_WIDTH * 3, RENDER_HEIGHT* 3), &mut texture);

        let pc = nes.pc;
        {
            drawLine(
                rect!(900, 10, 200, 20),
                "Status Registers: ",
                &mut canvas,
                &font,
                Color::WHITE,
            );
            if nes.GetFlag(CPU6502::Flags::C) == 0 {
                drawLine(rect!(900, 40, 20, 20), "N", &mut canvas, &font, Color::RED);
            } else {
                drawLine(
                    rect!(900, 40, 20, 20),
                    "N",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
            if nes.GetFlag(CPU6502::Flags::V) == 0 {
                drawLine(rect!(930, 40, 20, 20), "V", &mut canvas, &font, Color::RED);
            } else {
                drawLine(
                    rect!(930, 40, 20, 20),
                    "V",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
            if nes.GetFlag(CPU6502::Flags::U) == 0 {
                drawLine(rect!(960, 40, 20, 20), "U", &mut canvas, &font, Color::RED);
            } else {
                drawLine(
                    rect!(960, 40, 20, 20),
                    "U",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
            if nes.GetFlag(CPU6502::Flags::B) == 0 {
                drawLine(rect!(990, 40, 20, 20), "B", &mut canvas, &font, Color::RED);
            } else {
                drawLine(
                    rect!(990, 40, 20, 20),
                    "B",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
            if nes.GetFlag(CPU6502::Flags::D) == 0 {
                drawLine(rect!(1020, 40, 20, 20), "D", &mut canvas, &font, Color::RED);
            } else {
                drawLine(
                    rect!(1020, 40, 20, 20),
                    "D",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
            if nes.GetFlag(CPU6502::Flags::I) == 0 {
                drawLine(rect!(1050, 40, 20, 20), "I", &mut canvas, &font, Color::RED);
            } else {
                drawLine(
                    rect!(1050, 40, 20, 20),
                    "I",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
            if nes.GetFlag(CPU6502::Flags::Z) == 0 {
                drawLine(rect!(1080, 40, 20, 20), "Z", &mut canvas, &font, Color::RED);
            } else {
                drawLine(
                    rect!(1080, 40, 20, 20),
                    "Z",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
            if nes.GetFlag(CPU6502::Flags::C) == 0 {
                drawLine(rect!(1110, 40, 20, 20), "C", &mut canvas, &font, Color::RED);
            } else {
                drawLine(
                    rect!(1110, 40, 20, 20),
                    "C",
                    &mut canvas,
                    &font,
                    Color::GREEN,
                );
            }
        }
        {
            let mut aReg = "A: ".to_owned();
            aReg.push_str(&(format!("{:X}", &nes.a)));
            aReg.push_str(" [");
            aReg.push_str(&(nes.a).to_string());
            aReg.push_str("]");
            drawLine(
                rect!(900, 80, 200, 20),
                &aReg,
                &mut canvas,
                &font,
                Color::WHITE,
            );

            let mut aReg = "X: ".to_owned();
            aReg.push_str(&(format!("{:X}", &nes.x)));
            aReg.push_str(" [");
            aReg.push_str(&(nes.x).to_string());
            aReg.push_str("]");
            drawLine(
                rect!(900, 110, 200, 20),
                &aReg,
                &mut canvas,
                &font,
                Color::WHITE,
            );

            let mut aReg = "Y: ".to_owned();
            aReg.push_str(&(format!("{:X}", &nes.y)));
            aReg.push_str(" [");
            aReg.push_str(&(nes.y).to_string());
            aReg.push_str("]");
            drawLine(
                rect!(900, 140, 200, 20),
                &aReg,
                &mut canvas,
                &font,
                Color::WHITE,
            );
        }
        let mut pcText = ("PC: ").to_owned();
        pcText.push_str(&(format!("{:X}", &pc)));
        drawLine(
            rect!(900, 170, 200, 20),
            &pcText,
            &mut canvas,
            &font,
            Color::WHITE,
        );
        let mut i = 0;

        for x in 0..50 {
            let val = (nes.pc + x) as u32;
            let end = disassembly.capacity() as u32;
            if val <= end {
                let iteration = disassembly.get(&(val));
                let text = String::from("Error");
                let val = iteration.unwrap_or(&text);
                if val != "Error" {
                    i = i + 1;
                    drawLine(
                        rect!(900, 170 + (i * 50), 300, 20),
                        &val,
                        &mut canvas,
                        &font,
                        Color::WHITE,
                    );
                }
            }
        }
        canvas.present();
    }
    Ok(())
}

fn drawLine(
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

fn draw_sprite(canvas: &mut WindowCanvas, nes: &mut CPU6502::CPU6502, rect: sdl2::rect::Rect, tex: &mut Texture) {
    let frame_data = nes.bus.ppu.render();
    tex.update(None, &frame_data, RENDER_WIDTH * 3).unwrap();
    canvas.clear();
    canvas.copy(&tex, None, Some(rect)).unwrap();
}

//#[cfg(test)]
