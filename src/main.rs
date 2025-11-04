use std::{
    env,
    time::{Duration, Instant},
};

use sdl2::{self, event::Event, keyboard::Scancode};

mod chip8;
mod chip8_stack;

static ZOOM: u32 = 10;
static CHIP8_IPS: f64 = 700.;
static TIMER_HZ: f64 = 60.;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    if args.len() != 2 {
        println!("usage: yac8int path/to/chip8/program");
        std::process::exit(1);
    }

    let mut chip8 = chip8::Chip8::default();

    if let Err(e) = chip8.load_program(&args[1]) {
        eprintln!("Program load error: {e}");
        std::process::exit(1);
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("yac8int", 64 * ZOOM, 32 * ZOOM)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut last_time = Instant::now();

    let mut chip8_acc: f64 = 0.;
    let mut timer_acc: f64 = 0.;
    'game: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    scancode: Some(Scancode::Escape),
                    ..
                } => break 'game,
                _ => {}
            }
        }

        let time_now = Instant::now();
        let dt = (time_now - last_time).as_secs_f64();
        last_time = time_now;

        chip8_acc += dt * CHIP8_IPS;
        timer_acc += dt * TIMER_HZ;
        while chip8_acc > 0. {
            todo!(); // Step the chip8
            chip8_acc -= 1.;
        }
        while timer_acc > 0. {
            chip8.step_timers();
            timer_acc -= 1.;
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
