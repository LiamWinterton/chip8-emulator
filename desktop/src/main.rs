use chip8_core::*;

use std::env;
use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

const TICKS_PER_FRAME: usize = 8;
const FRAME_DURATION: Duration = Duration::from_millis(16); // ~60Hz

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");

        return;
    }

    // Setup chip8
    let mut chip8 = Emu::new();

    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();

    rom.read_to_end(&mut buffer).unwrap();

    chip8.load(&buffer);

    // Setup SDL
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Chip 8 Emulator - LW", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .unwrap();

    // Set up double buffering
    canvas
        .set_logical_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    // Setup game
    let mut event_pump = sdl_context.event_pump().unwrap();

    'gameloop: loop {
        let frame_start = Instant::now();

        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'gameloop;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, false);
                    }
                }
                _ => (),
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            chip8.tick();
        }

        chip8.tick_timers();
        draw_screen(&chip8, &mut canvas);

        // Frame rate limiting
        let frame_time = frame_start.elapsed();
        if frame_time < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - frame_time);
        }
    }
}

fn draw_screen(emu: &Emu, canvas: &mut Canvas<Window>) {
    // Clear canvas with black
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    // Set color to white for drawing
    canvas.set_draw_color(Color::RGB(255, 255, 255));

    let screen_buf = emu.get_display();
    let mut rects = Vec::with_capacity(SCREEN_WIDTH * SCREEN_HEIGHT);

    // Collect all rectangles to draw
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;
            rects.push(Rect::new(
                (x * SCALE) as i32,
                (y * SCALE) as i32,
                SCALE,
                SCALE,
            ));
        }
    }

    // Draw all rectangles at once
    if !rects.is_empty() {
        canvas.fill_rects(&rects).unwrap();
    }

    canvas.present();
}

fn key2btn(key: Keycode) -> Option<usize> {
    match key {
        // Original CHIP-8 keypad layout:
        // 1 2 3 C
        // 4 5 6 D
        // 7 8 9 E
        // A 0 B F
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
