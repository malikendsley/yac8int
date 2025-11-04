use std::{
    env,
    io::{self, Write},
    time::{Duration, Instant},
};

mod chip8;
mod chip8_stack;

static CHIP8_IPS: f64 = 700.;
static TIMER_HZ: f64 = 60.0;

fn clear_screen() {
    // ANSI: clear screen and move cursor to 1;1
    print!("\x1B[2J\x1B[H");
}

fn draw_console(buf: &[u8], width: usize, height: usize) {
    let mut out = String::with_capacity((width + 1) * height);
    for y in 0..height {
        let row = &buf[y * width..(y + 1) * width];
        for &px in row {
            // Full block for "on", space for "off"
            out.push(if px != 0 { '█' } else { ' ' });
        }
        out.push('\n');
    }
    print!("{}", out);
    let _ = io::stdout().flush();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: yac8int path/to/chip8/program");
        std::process::exit(1);
    }

    let mut chip8 = chip8::Chip8::default();
    if let Err(e) = chip8.load_program(&args[1]) {
        eprintln!("Program load error: {e}");
        std::process::exit(1);
    }

    let mut last_time = Instant::now();
    let mut chip8_acc = 0.0;
    let mut timer_acc = 0.0;

    clear_screen();

    // Assumes CHIP-8 logical size 64x32
    let w = 64usize;
    let h = 32usize;

    loop {
        let now = Instant::now();
        let dt = (now - last_time).as_secs_f64();
        last_time = now;

        chip8_acc += dt * CHIP8_IPS;
        timer_acc += dt * TIMER_HZ;

        while chip8_acc >= 1.0 {
            chip8.step();
            chip8_acc -= 1.0;
        }
        while timer_acc >= 1.0 {
            chip8.step_timers();
            timer_acc -= 1.0;
        }

        if chip8.dirty() {
            // Reposition cursor to top-left before redraw to avoid full clear flicker
            print!("\x1B[H");
            draw_console(chip8.display_buffer(), w, h);
            chip8.clear_dirty();
        }

        std::thread::sleep(Duration::from_micros(1_000_000 / 60));
    }
}
