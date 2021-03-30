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
use sdl2::render::Texture;
use sdl2::render::TextureAccess;

use sdl2::render::WindowCanvas;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;

pub mod apu;
pub mod bus;
pub mod cartridge;
pub mod cpu_6502;
pub mod Mappers;
pub mod ppu;
use sdl2::audio::{AudioSpecDesired, AudioQueue};

static mut NES: Option<cpu_6502::CPU6502> = None;

fn queue_audio(audio: &AudioQueue<i16>, nes: &mut cpu_6502::CPU6502) {
    let samples = &nes.bus.apu.samples;
    let sample = samples.as_slice();
    if audio.size() as usize <= 2 * 8 
    {
        audio.queue(&sample);
    }
    nes.bus.apu.samples.clear();
}

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

fn validate_rom() -> cartridge::Cartridge
{
    use std::io::{stdin};
    print!("{}[2J", 27 as char);
    println!("Please enter a ROM path: ");
    loop
    {
        let mut rom = String::new();
        stdin().read_line(&mut rom).unwrap().to_string();
        rom.truncate(rom.len() - 1);

        if rom == "/Users/multivac/NES/source/src/roms/cpu.nes".to_string()
        {
            println!("valid");
        }
        let cartridge = cartridge::Cartridge::new(rom);
        match cartridge {
            Ok(file) => return file,
            Err(error) => println!("\n{}", error),
        };
    }
}

fn main() -> Result<(), String> {
    let mut nes = cpu_6502::CPU6502::new();
    let cartridge = validate_rom();
    nes.bus.connect_cartridge(Rc::new(RefCell::new(cartridge)));

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let font_path: &Path = Path::new("./assets/monogram.ttf");

    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    let device = audio_subsystem.open_queue::<i16, _>(None, &desired_spec)?;
    device.resume();

    let debug_window = video_subsys
        .window("Debug Window", 1024, 960)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let main_window = video_subsys
        .window("NES Emulator", 1024, 960)
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut main_canvas = main_window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;

    let mut debug_canvas = debug_window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;

    let tx1 = main_canvas.texture_creator();
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

    let tx2 = debug_canvas.texture_creator();
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

    let tx3 = debug_canvas.texture_creator();
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

    nes.reset();
    let disassembly = nes.disassemble(0x0000, 0xFFFF);
    let mut emulation_run = true;
    let mut time: f32 = 0.0;

    let mut debug = false;
    debug_canvas.window_mut().hide();

    let mut a_pressed = false;
    let mut b_pressed = false;
    let mut start_pressed = false;
    let mut select_pressed = false;
    let mut up_pressed = false;
    let mut down_pressed = false;
    let mut right_pressed = false;
    let mut left_pressed = false;

    unsafe {
        NES = Some(nes);
    }
    let mut now = Instant::now();
    'mainloop: loop {
        let mut global_nes = unsafe { NES.as_mut().unwrap() };
        global_nes.bus.controller[0] = 0x00;

        if b_pressed {
            global_nes.bus.controller[0] |= 0x80;
        }
        if a_pressed {
            global_nes.bus.controller[0] |= 0x40;
        }
        if start_pressed {
            global_nes.bus.controller[0] |= 0x20;
        }
        if select_pressed {
            global_nes.bus.controller[0] |= 0x10;
        }
        if up_pressed {
            global_nes.bus.controller[0] |= 0x08;
        }
        if down_pressed {
            global_nes.bus.controller[0] |= 0x04;
        }
        if right_pressed {
            global_nes.bus.controller[0] |= 0x01;
        }
        if left_pressed {
            global_nes.bus.controller[0] |= 0x02;
        }
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::X), //B
                    ..
                } => {
                    b_pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::X), //B
                    ..
                } => {
                    b_pressed = false;
                }
                /////////////////////////////////
                Event::KeyDown {
                    keycode: Some(Keycode::Z), //A
                    ..
                } => {
                    a_pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Z), //A
                    ..
                } => {
                    a_pressed = false;
                }

                /////////////////////////////////
                Event::KeyDown {
                    keycode: Some(Keycode::A), //Start
                    ..
                } => {
                    start_pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::A), //Start
                    ..
                } => {
                    start_pressed = false;
                }
                /////////////////////////////////
                Event::KeyDown {
                    keycode: Some(Keycode::S), //Select
                    ..
                } => {
                    select_pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::S), //Select
                    ..
                } => {
                    select_pressed = false;
                }
                /////////////////////////////////
                Event::KeyDown {
                    keycode: Some(Keycode::Up), //D-Pad up
                    ..
                } => {
                    up_pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Up), //D-Pad up
                    ..
                } => {
                    up_pressed = false;
                }
                /////////////////////////////////
                Event::KeyDown {
                    keycode: Some(Keycode::Down), //D-Pad down
                    ..
                } => {
                    down_pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Down), //D-Pad down
                    ..
                } => {
                    down_pressed = false;
                }
                /////////////////////////////////
                Event::KeyDown {
                    keycode: Some(Keycode::Left), //D-pad Left
                    ..
                } => {
                    left_pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Left), //D-pad Left
                    ..
                } => {
                    left_pressed = false;
                }
                /////////////////////////////////
                Event::KeyDown {
                    keycode: Some(Keycode::Right), //D-Pad Right
                    ..
                } => {
                    right_pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Right), //D-pad Right
                    ..
                } => {
                    right_pressed = false;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    global_nes.reset();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    emulation_run = !emulation_run;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::M),
                    ..
                } => {
                    debug = !debug;

                    if debug == true {
                        debug_canvas.window_mut().show();
                    } else {
                        debug_canvas.window_mut().hide();
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                Event::Quit { .. } => break 'mainloop,
                _ => {}
            }
        }
        //clock_nes(global_nes);
        main_canvas.clear();
        debug_canvas.clear();
        if time > 0.0 {
            time = time - (now.elapsed().as_secs_f32());
        } 
        else 
        {
            time = time + (0.16666) - now.elapsed().as_secs_f32();
            clock_nes(global_nes);
            queue_audio(&device, global_nes);
            now = Instant::now();
        }
        if debug == true {
            draw_debug(&mut debug_canvas, &mut global_nes, &font, &disassembly);
            render_pattern_table(
                &mut debug_canvas,
                &mut global_nes,
                rect!(10, 694, 256, 256),
                &mut pattern_one,
                0,
            );
            render_pattern_table(
                &mut debug_canvas,
                &mut global_nes,
                rect!(276, 694, 256, 256),
                &mut pattern_two,
                1,
            );
        }
        render_frame(
            &mut main_canvas,
            &mut global_nes,
            rect!(0, 0, RENDER_WIDTH * 4, RENDER_HEIGHT * 4),
            &mut screen_texture,
        );
        main_canvas.present();
        debug_canvas.present();
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

fn draw_debug(
    debug_canvas: &mut WindowCanvas,
    nes: &mut cpu_6502::CPU6502,
    font: &sdl2::ttf::Font,
    disassembly: &HashMap<u32, String>,
) {
    let pc = nes.pc;
    {
        draw_line(
            rect!(10, 10, 200, 30),
            "Status Registers: ",
            debug_canvas,
            &font,
            Color::WHITE,
        );
        if nes.get_flag(cpu_6502::Flags::N) == 0 {
            draw_line(rect!(10, 40, 30, 30), "N", debug_canvas, &font, Color::RED);
        } else {
            draw_line(
                rect!(10, 40, 30, 30),
                "N",
                debug_canvas,
                &font,
                Color::GREEN,
            );
        }
        if nes.get_flag(cpu_6502::Flags::V) == 0 {
            draw_line(rect!(40, 40, 30, 30), "V", debug_canvas, &font, Color::RED);
        } else {
            draw_line(
                rect!(40, 40, 30, 30),
                "V",
                debug_canvas,
                &font,
                Color::GREEN,
            );
        }
        if nes.get_flag(cpu_6502::Flags::U) == 0 {
            draw_line(rect!(70, 40, 30, 30), "U", debug_canvas, &font, Color::RED);
        } else {
            draw_line(
                rect!(70, 40, 30, 30),
                "U",
                debug_canvas,
                &font,
                Color::GREEN,
            );
        }
        if nes.get_flag(cpu_6502::Flags::B) == 0 {
            draw_line(rect!(100, 40, 30, 30), "B", debug_canvas, &font, Color::RED);
        } else {
            draw_line(
                rect!(100, 40, 30, 30),
                "B",
                debug_canvas,
                &font,
                Color::GREEN,
            );
        }
        if nes.get_flag(cpu_6502::Flags::D) == 0 {
            draw_line(rect!(130, 40, 30, 30), "D", debug_canvas, &font, Color::RED);
        } else {
            draw_line(
                rect!(130, 40, 30, 30),
                "D",
                debug_canvas,
                &font,
                Color::GREEN,
            );
        }
        if nes.get_flag(cpu_6502::Flags::I) == 0 {
            draw_line(rect!(160, 40, 30, 30), "I", debug_canvas, &font, Color::RED);
        } else {
            draw_line(
                rect!(160, 40, 30, 30),
                "I",
                debug_canvas,
                &font,
                Color::GREEN,
            );
        }
        if nes.get_flag(cpu_6502::Flags::Z) == 0 {
            draw_line(rect!(190, 40, 30, 30), "Z", debug_canvas, &font, Color::RED);
        } else {
            draw_line(
                rect!(190, 40, 30, 30),
                "Z",
                debug_canvas,
                &font,
                Color::GREEN,
            );
        }
        if nes.get_flag(cpu_6502::Flags::C) == 0 {
            draw_line(rect!(220, 40, 30, 30), "C", debug_canvas, &font, Color::RED);
        } else {
            draw_line(
                rect!(220, 40, 30, 30),
                "C",
                debug_canvas,
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
            rect!(10, 80, 200, 30),
            &a_reg,
            debug_canvas,
            &font,
            Color::WHITE,
        );

        let mut x_reg = "X: ".to_owned();
        x_reg.push_str(&(format!("{:X}", &nes.x)));
        x_reg.push_str(" [");
        x_reg.push_str(&(nes.x).to_string());
        x_reg.push_str("]");
        draw_line(
            rect!(10, 110, 200, 30),
            &x_reg,
            debug_canvas,
            &font,
            Color::WHITE,
        );

        let mut y_reg = "Y: ".to_owned();
        y_reg.push_str(&(format!("{:X}", &nes.y)));
        y_reg.push_str(" [");
        y_reg.push_str(&(nes.y).to_string());
        y_reg.push_str("]");
        draw_line(
            rect!(10, 140, 200, 30),
            &y_reg,
            debug_canvas,
            &font,
            Color::WHITE,
        );
    }
    let mut pc_txt = ("PC: ").to_owned();
    pc_txt.push_str(&(format!("{:X}", &pc)));
    draw_line(
        rect!(10, 170, 200, 30),
        &pc_txt,
        debug_canvas,
        &font,
        Color::WHITE,
    );
    let mut i = 0;

    for x in 0..20 {
        let val = nes.pc as u32 + x as u32;
        let end = disassembly.capacity() as u32;
        if val <= end {
            let iteration = disassembly.get(&(val));
            let text = String::from("Error");
            let val = iteration.unwrap_or(&text);
            if val != "Error" {
                i = i + 1;
                draw_line(
                    rect!(10, 170 + (i * 50), 300, 40),
                    &val,
                    debug_canvas,
                    &font,
                    Color::WHITE,
                );
            }
        }
    }

    for x in 0..10 {
        let mut sprite_debug = "".to_owned();
        sprite_debug.push_str(&(format!("{}: ", x)));
        sprite_debug.push_str(&(format!("({}", nes.bus.ppu.oam_ram[x * 4 + 3])));
        sprite_debug.push_str(&(format!(", {})", nes.bus.ppu.oam_ram[x * 4 + 0])));
        sprite_debug.push_str(&(format!(" ID:{}", nes.bus.ppu.oam_ram[x * 4 + 1])));
        sprite_debug.push_str(&(format!(" AT:{}", nes.bus.ppu.oam_ram[x * 4 + 2])));

        draw_line(
            rect!(400, 170 + (x * 50), 300, 40),
            &sprite_debug,
            debug_canvas,
            &font,
            Color::WHITE,
        )
    }
}

fn clock_nes(global_nes: &mut cpu_6502::CPU6502){
    let mut clock_count = 0;
    while global_nes.bus.ppu.frame_complete == false 
    {
        clock_count += 1;
        global_nes.bus.clock();
        if global_nes.bus.dma_transfer == true {
            if global_nes.bus.dma_buffer == true {
                if clock_count % 2 == 1 {
                    global_nes.bus.dma_buffer = false;
                }
            } else {
                if clock_count % 2 == 0 {
                    let page = (global_nes.bus.dma_page as u16) << 8;
                    let addr = global_nes.bus.dma_addr as u16;
                    global_nes.bus.dma_data = global_nes.bus.cpu_read(page | addr, false);
                } else {
                    global_nes.bus.ppu.oam_ram[global_nes.bus.dma_addr as usize] = global_nes.bus.dma_data;
                    if global_nes.bus.dma_addr != 255 {
                        global_nes.bus.dma_addr += 1;
                    } else {
                        global_nes.bus.dma_addr = 0x00;
                        global_nes.bus.dma_transfer = false;
                        global_nes.bus.dma_buffer = true;
                    }
                }
            }
        } else {
            global_nes.clock();
        }
    }
    global_nes.bus.ppu.frame_complete = false;
}
fn render_frame(
    canvas: &mut WindowCanvas,
    nes: &mut cpu_6502::CPU6502,
    rect: sdl2::rect::Rect,
    tex: &mut Texture,
) {
    let frame_data = nes.bus.ppu.render();
    tex.update(None, &frame_data, 256 * 3).unwrap();
    canvas.copy(&tex, None, Some(rect)).unwrap();
}

fn render_pattern_table(
    canvas: &mut WindowCanvas,
    nes: &mut cpu_6502::CPU6502,
    rect: sdl2::rect::Rect,
    tex: &mut Texture,
    index: u8,
) {
    let frame_data = nes.bus.ppu.get_pattern_table(index, 0);
    tex.update(None, &frame_data, 128 * 3).unwrap();
    canvas.copy(&tex, None, Some(rect)).unwrap();
}

//#[cfg(test)]
