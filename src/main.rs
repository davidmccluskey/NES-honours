extern crate sdl2;
#[macro_use]
extern crate bitflags;
use std::path::Path;
use std::fmt::{Error, Write};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::video::Window;
use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
use uwl::StringStream;
use std::fmt;

pub mod bus;
pub mod CPU6502;

static SCREEN_WIDTH : u32 = 1280;
static SCREEN_HEIGHT : u32 = 720;

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);


fn update(nes: &mut CPU6502::CPU6502 ){
    while {
        nes.clock(); 
        !nes.complete()}{}
}

fn writer<W: Write>(f: &mut W, s: &str) -> Result<(), Error> {
    f.write_str(s)
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let font_path: &Path = Path::new("PressStart2P-Regular.ttf");

    let window = video_subsys.window(" ", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;


    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    
    // Load a font
    let mut font = ttf_context.load_font(font_path, 128)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    let mut nOffset = 0x8000;

    let mut nes = CPU6502::CPU6502::new();

    let mut v: Vec<&str> = "A2 0A 8E 00 00 A2 03 8E 01 00 AC 00 00 A9 00 18 6D 01 00 88 D0 FA 8D 02 00 EA EA EA".rsplit(' ').collect();
    v.reverse();
    for c in v.iter() {
        if c.to_string() != " " {
            let z = u8::from_str_radix(c, 16).unwrap();
            nes.bus.ram[nOffset] = z;
        }
        nOffset = nOffset + 1;
    }

    nes.bus.ram[0xFFFC] = 0x00;
    nes.bus.ram[0xFFFD] = 0x80;

    let mut disassembly = nes.disassemble(0x0000, 0xFFFF);
    
    nes.reset();

    let mut count = 0;

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {keycode: Some(Keycode::Space), ..} => {update(&mut nes); count = count + 1;},
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} |
                Event::Quit {..} => break 'mainloop,
                _ => {}
            }
        }
        canvas.clear();
        let pc = nes.pc;
        {
            drawLine(rect!(900, 10, 200, 20), "Status Registers: ", &mut canvas, &font, Color::WHITE);
            if nes.GetFlag(CPU6502::Flags::C) == 0 {
                drawLine(rect!(900, 40, 20, 20), "N", &mut canvas, &font, Color::RED);
            }else{
                drawLine(rect!(900, 40, 20, 20), "N", &mut canvas, &font, Color::GREEN);
            }
            if nes.GetFlag(CPU6502::Flags::V) == 0 {
                drawLine(rect!(930, 40, 20, 20), "V", &mut canvas, &font, Color::RED);
            }else{
                drawLine(rect!(930, 40, 20, 20), "V", &mut canvas, &font, Color::GREEN);
            }
            if nes.GetFlag(CPU6502::Flags::U) == 0 {
                drawLine(rect!(960, 40, 20, 20), "U", &mut canvas, &font, Color::RED);
            }else{
                drawLine(rect!(960, 40, 20, 20), "U", &mut canvas, &font, Color::GREEN);
            }
            if nes.GetFlag(CPU6502::Flags::B) == 0 {
                drawLine(rect!(990, 40, 20, 20), "B", &mut canvas, &font, Color::RED);
            }else{
                drawLine(rect!(990, 40, 20, 20), "B", &mut canvas, &font, Color::GREEN);
            }
            if nes.GetFlag(CPU6502::Flags::D) == 0 {
                drawLine(rect!(1020, 40, 20, 20), "D", &mut canvas, &font, Color::RED);
            }else{
                drawLine(rect!(1020, 40, 20, 20), "D", &mut canvas, &font, Color::GREEN);
            }
            if nes.GetFlag(CPU6502::Flags::I) == 0 {
                drawLine(rect!(1050, 40, 20, 20), "I", &mut canvas, &font, Color::RED);
            }else{
                drawLine(rect!(1050, 40, 20, 20), "I", &mut canvas, &font, Color::GREEN);
            }
            if nes.GetFlag(CPU6502::Flags::Z) == 0 {
                drawLine(rect!(1080, 40, 20, 20), "Z", &mut canvas, &font, Color::RED);
            }else{
                drawLine(rect!(1080, 40, 20, 20), "Z", &mut canvas, &font, Color::GREEN);
            }
            if nes.GetFlag(CPU6502::Flags::C) == 0 {
                drawLine(rect!(1110, 40, 20, 20), "C", &mut canvas, &font, Color::RED);
            }else{
                drawLine(rect!(1110, 40, 20, 20), "C", &mut canvas, &font, Color::GREEN);
            }
        }
        {
            let mut aReg = "A: ".to_owned();
            aReg.push_str(&(format!("{:X}", &nes.a)));
            aReg.push_str(" [");
            aReg.push_str(&(nes.a).to_string());
            aReg.push_str("]");
            drawLine(rect!(900, 80, 200, 20), &aReg, &mut canvas, &font, Color::WHITE);

            let mut aReg = "X: ".to_owned();
            aReg.push_str(&(format!("{:X}", &nes.x)));
            aReg.push_str(" [");
            aReg.push_str(&(nes.x).to_string());
            aReg.push_str("]");
            drawLine(rect!(900, 110, 200, 20), &aReg, &mut canvas, &font, Color::WHITE);

            let mut aReg = "Y: ".to_owned();
            aReg.push_str(&(format!("{:X}", &nes.y)));
            aReg.push_str(" [");
            aReg.push_str(&(nes.y).to_string());
            aReg.push_str("]");
            drawLine(rect!(900, 140, 200, 20), &aReg, &mut canvas, &font, Color::WHITE);

        }
        let mut pcText = ("PC: ").to_owned();
        pcText.push_str(&(format!("{:X}", &pc)));
        drawLine(rect!(900, 170, 200, 20), &pcText, &mut canvas, &font, Color::WHITE);
        let mut i = 0;

        for x in 0..50 {
            let val = (nes.pc + x) as u32;
            let end = disassembly.capacity() as u32;
            if val <= end {
                let iteration = disassembly.get(&(val));
                let text = String::from("Error");
                let val = iteration.unwrap_or(&text);
                if val != "Error"{
                    i = i + 1;
                    drawLine(rect!(900, 170 + (i * 50), 300, 20), &val, &mut canvas, &font, Color::WHITE);
                }
            }
        }




        //canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.present();
    }
    Ok(())
}

fn drawLine(rect: sdl2::rect::Rect, text: &str, canvas: &mut WindowCanvas, font: &sdl2::ttf::Font, color: sdl2::pixels::Color){
    let texture_creator = canvas.texture_creator();
    let surface = font.render(&text)
        .blended(color).map_err(|e| e.to_string());
    let texture = texture_creator.create_texture_from_surface(&surface.unwrap())
        .map_err(|e| e.to_string());

    canvas.copy(&texture.unwrap(), None, Some(rect));

}

#[cfg(test)]
mod test;