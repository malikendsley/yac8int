use sdl2::{event::Event, keyboard::Scancode, pixels::PixelFormatEnum, render::TextureAccess};
use std::{env, time::Instant};

mod chip8;
mod chip8_stack;

const W: u32 = 64;
const H: u32 = 32;
const SCALE: u32 = 10;
static CHIP8_IPS: f64 = 700.;
static TIMER_HZ: f64 = 60.0;

// TODO: once_cell hashmap?
const KEYMAP: &[(Scancode, usize)] = &[
    (Scancode::Num1, 1),
    (Scancode::Num2, 2),
    (Scancode::Num3, 3),
    (Scancode::Num4, 0xC),
    (Scancode::Q, 4),
    (Scancode::W, 5),
    (Scancode::E, 6),
    (Scancode::R, 0xD),
    (Scancode::A, 7),
    (Scancode::S, 8),
    (Scancode::D, 9),
    (Scancode::F, 0xE),
    (Scancode::Z, 0xA),
    (Scancode::X, 0),
    (Scancode::C, 0xB),
    (Scancode::V, 0xF),
];

fn map_key(scancode: Scancode) -> Option<usize> {
    KEYMAP.iter().find(|(s, _)| *s == scancode).map(|(_, i)| *i)
}

fn draw_to_texture(chip8: &chip8::Chip8, tex: &mut sdl2::render::Texture, rgb_buf: &mut [u8]) {
    let fb = chip8.display_buffer();
    for y in 0..H as usize {
        for x in 0..W as usize {
            let i = (y * W as usize + x) * 3;
            let on = fb[y * W as usize + x] != 0;
            let v = if on { 0xFF } else { 0x00 };
            rgb_buf[i..i + 3].copy_from_slice(&[v, v, v]);
        }
    }
    tex.update(None, rgb_buf, (W * 3) as usize).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: yac8int path/to/chip8/program");
        std::process::exit(1);
    }

    let mut chip8 = chip8::Chip8::default();
    chip8.load_program(&args[1]).unwrap();

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video
        .window("yac8int", W * SCALE, H * SCALE)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().accelerated().build().unwrap();

    let creator = canvas.texture_creator();
    let mut tex = creator
        .create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, W, H)
        .unwrap();
    let mut rgb_buf = vec![0u8; (W * H * 3) as usize];

    let mut event_pump = sdl.event_pump().unwrap();
    let mut last_time = Instant::now();
    let mut chip8_acc = 0.0;
    let mut timer_acc = 0.0;
    let mut cycles_since_last_second = 0;
    let mut wall_acc = 0.0;

    'game: loop {
        for e in event_pump.poll_iter() {
            match e {
                Event::Quit { .. }
                | Event::KeyDown {
                    scancode: Some(Scancode::Escape),
                    ..
                } => break 'game,

                Event::KeyDown {
                    scancode: Some(code),
                    ..
                } => {
                    if let Some(idx) = map_key(code) {
                        // println!("Pressed {}", code.name());
                        chip8.keypad[idx] = true;
                    }
                }

                Event::KeyUp {
                    scancode: Some(code),
                    ..
                } => {
                    if let Some(idx) = map_key(code) {
                        // println!("Released {}", code.name());
                        chip8.keypad[idx] = false;
                    }
                }

                _ => {}
            }
        }

        let now = Instant::now();
        let dt = (now - last_time).as_secs_f64();
        last_time = now;

        chip8_acc += dt * CHIP8_IPS;
        timer_acc += dt * TIMER_HZ;
        wall_acc += dt;

        while chip8_acc >= 1.0 {
            cycles_since_last_second += 1;
            chip8.step();
            chip8_acc -= 1.0;
        }
        while timer_acc >= 1.0 {
            chip8.step_timers();
            timer_acc -= 1.0;
        }
        while wall_acc >= 1.0 {
            // Technically, if this runs too slow (somehow) this will be wrong
            wall_acc -= 1.;
            println!("Cycles per second: {}", cycles_since_last_second);
            cycles_since_last_second = 0;
        }

        if chip8.dirty {
            draw_to_texture(&chip8, &mut tex, &mut rgb_buf);
            chip8.dirty = false;
        }

        canvas.clear();
        canvas.copy(&tex, None, None).unwrap();
        canvas.present();
    }
}
