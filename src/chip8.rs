use crate::chip8_stack;

pub static SCREEN_WIDTH: usize = 64;
pub static SCREEN_HEIGHT: usize = 32;
pub static SCREEN_AREA: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

// TODO: Simple tests
// TODO: Depending on usage, this (these) may become *size
const fn x(op: u16) -> u8 {
    ((op & 0x0F00) >> 8) as u8
}
const fn y(op: u16) -> u8 {
    ((op & 0x00F0) >> 4) as u8
}
const fn n(op: u16) -> u8 {
    (op & 0x000F) as u8
}
// kk depending on who you ask
const fn nn(op: u16) -> u8 {
    (op & 0x00FF) as u8
}
const fn nnn(op: u16) -> u16 {
    (op & 0x0FFF) as u16
}
const fn idx_from_coord(x: usize, y: usize) -> usize {
    y * SCREEN_WIDTH + x
}

pub struct Chip8 {
    ram: [u8; 4096],
    v_reg: [u8; 16],
    i_reg: u16,
    delay_timer: u16,
    sound_timer: u16,
    pc: u16,
    stack: chip8_stack::Chip8Stack,
    keypad: [bool; 16],
    display_buffer: [u8; SCREEN_AREA],
    dirty: bool,
}

impl Default for Chip8 {
    fn default() -> Self {
        let mut chip_8 = Self {
            ram: [0; 4096],
            v_reg: [0; 16],
            i_reg: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            stack: chip8_stack::Chip8Stack::default(),
            keypad: [false; 16],
            display_buffer: [0; 64 * 32],
            dirty: false,
        };

        const FONT_SET: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80,
            0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
            0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90,
            0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
            0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];

        chip_8.ram[0..80].copy_from_slice(&FONT_SET);
        chip_8
    }
}

impl Chip8 {
    pub fn load_program(&mut self, path: &str) -> std::io::Result<()> {
        let bytes = std::fs::read(path)?;
        if bytes.len() > self.ram.len() - 0x200 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Program too large",
            ));
        }
        println!("Loading program {} of size {}", path, bytes.len());
        self.ram[0x200..0x200 + bytes.len()].copy_from_slice(&bytes);
        Ok(())
    }

    pub fn step_timers(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
    }

    fn fetch(&self) -> u16 {
        let pc = self.pc as usize;
        ((self.ram[pc] as u16) << 8) | self.ram[pc + 1] as u16
    }

    fn decode_execute(&mut self, op: u16) {
        let nibble_1 = (op & 0xF000) >> 12;
        // println!("Nibble 1: {:01X}", nibble_1);
        match nibble_1 {
            0 => {
                match op {
                    0x00E0 => {
                        self.display_buffer = [0; SCREEN_AREA];
                        // TODO: Later, track lit pixels count
                        self.dirty = true;
                        // println!("Clearing screen");
                    }
                    _ => {
                        println!("Unimplemented");
                        panic!();
                    }
                }
            }
            1 => {
                self.pc = nnn(op);
                // println!("Jump");
            }
            6 => {
                self.v_reg[x(op) as usize] = nn(op);
                // println!("Set register VX");
            }
            7 => {
                // Cover overflow
                self.v_reg[x(op) as usize] = self.v_reg[x(op) as usize].wrapping_add(nn(op));
                // println!("Add to VX");
            }
            0xA => {
                self.i_reg = nnn(op);
                // println!("Set index register i")
            }
            0xD => {
                // Coordinates
                let x0 = self.v_reg[x(op) as usize] as usize;
                let y0 = self.v_reg[y(op) as usize] as usize;
                // Clip the sprite
                let h = (n(op) as usize).min(SCREEN_HEIGHT - y0);
                let w = 8usize.min(SCREEN_WIDTH - x0);
                self.v_reg[0xF] = 0;

                for row in 0..h {
                    let b = self.ram[self.i_reg as usize + row];
                    for col in 0..w {
                        let bit = (b >> (7 - col)) & 1;
                        if bit == 0 {
                            continue;
                        }
                        let idx = idx_from_coord(x0 + col, y0 + row);
                        let prev = self.display_buffer[idx];
                        self.display_buffer[idx] ^= 1;
                        if prev == 1 {
                            self.v_reg[0xF] = 1;
                        }
                    }
                }
                self.dirty = true;
                // println!("Draw");
            }
            _ => {
                println!("Unimplemented");
                panic!();
            }
        }
    }

    pub fn step(&mut self) {
        // Read 2 bytes at the PC
        let op = self.fetch();
        self.pc += 2;
        // println!("Fetched {:04X}", op);
        self.decode_execute(op);
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn display_buffer(&self) -> &[u8] {
        &self.display_buffer
    }
}
