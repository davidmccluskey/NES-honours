extern crate sdl2;
#[macro_use]
extern crate bitflags;
use std::path::Path;
use std::fmt::{Error, Write};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
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

// Scale fonts to a reasonable size when they're too big (though they might look less smooth)
fn get_centered_rect(rect_width: u32, rect_height: u32, cons_width: u32, cons_height: u32) -> Rect {
    let wr = rect_width as f32 / cons_width as f32;
    let hr = rect_height as f32 / cons_height as f32;

    let (w, h) = if wr > 1f32 || hr > 1f32 {
        if wr > hr {
            println!("Scaling down! The text will look worse!");
            let h = (rect_height as f32 / wr) as i32;
            (cons_width as i32, h)
        } else {
            println!("Scaling down! The text will look worse!");
            let w = (rect_width as f32 / hr) as i32;
            (w, cons_height as i32)
        }
    } else {
        (rect_width as i32, rect_height as i32)
    };
    let cx = (SCREEN_WIDTH as i32 - w) / 2;
    let cy = (SCREEN_HEIGHT as i32 - h) / 2;
    rect!(cx, cy, w, h)
}

fn draw() {
    
}


fn run(font_path: &Path, text: String) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let window = video_subsys.window(" ", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;


    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    
    // Load a font
    let mut font = ttf_context.load_font(font_path, 128)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    // render a surface, and convert it to a texture bound to the canvas
    let surface = font.render(&text)
        .blended(Color::RGBA(255, 255, 255, 255)).map_err(|e| e.to_string())?;
    let texture = texture_creator.create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGBA(0, 50, 200, 255));
    canvas.clear();

    let TextureQuery { width, height, .. } = texture.query();

    // If the example text is too big for the screen, downscale it (and center irregardless)
    let padding = 64;
    let target = get_centered_rect(width, height, SCREEN_WIDTH - padding, SCREEN_HEIGHT - padding);

    canvas.copy(&texture, None, Some(target))?;
    canvas.present();

    //let mut stream = StringStream::new("A2 0A 8E 00 00 A2 03 8E 01 00 AC 00 00 A9 00 18 6D 01 00 88 D0 FA 8D 02 00 EA EA EA");

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

    // while !stream.at_end(){
    //     let b = stream.current().unwrap();
    //     nOffset = (nOffset + 1) as usize;
    //     if b != " " {
    //         let z = u8::from_str_radix(b, 16).unwrap();
    //         nes.bus.ram[nOffset] = z;
    //     }
    //     stream.next();
    // }

    nes.bus.ram[0xFFFC] = 0x00;
    nes.bus.ram[0xFFFD] = 0x80;
    
    nes.reset();

    'mainloop: loop {

        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {keycode: Some(Keycode::Space), ..} => run_clock(&mut nes),
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} |
                Event::Quit {..} => break 'mainloop,
                _ => {}
            }
        }
    }

    Ok(())
}

fn run_clock(nes: &mut CPU6502::CPU6502){    
    while {
        nes.clock(); 
        nes.disassemble(32768, 32796);
        !nes.complete()}{}
}

fn writer<W: Write>(f: &mut W, s: &str) -> Result<(), Error> {
    f.write_str(s)
}

fn main() -> Result<(), String> {
    let path: &Path = Path::new("PressStart2P-Regular.ttf");
    let val: String = "10 x 3".to_string();
    run(path, val)?;

    Ok(())
}