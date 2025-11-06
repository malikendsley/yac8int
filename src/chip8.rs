use crate::chip8_stack;
use rand::prelude::*;

pub static SCREEN_WIDTH: usize = 64;
pub static SCREEN_HEIGHT: usize = 32;
pub static SCREEN_AREA: usize = SCREEN_WIDTH * SCREEN_HEIGHT;
pub static FONT_SIZE: usize = 5;
pub static FONT_COUNT: usize = 16;

// TODO: Simple tests
// TODO: Depending on usage, this (these) may become *size
const fn x(op: u16) -> usize {
    ((op & 0x0F00) >> 8) as usize
}
const fn y(op: u16) -> usize {
    ((op & 0x00F0) >> 4) as usize
}
const fn n(op: u16) -> usize {
    (op & 0x000F) as usize
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
    v: [u8; 16],
    idx: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    stack: chip8_stack::Chip8Stack,
    pub keypad: [bool; 16],
    display_buffer: [u8; SCREEN_AREA],
    pub dirty: bool,
    pub load_store_quirk: bool, // When true, increments idx on save and load
    pub jp_v0_quirk: bool,      // When true, interprets BNNN as BXNN
    pub shift_quirk: bool,      // When true, left and right shift use Y as source operand
}

impl Default for Chip8 {
    fn default() -> Self {
        let mut chip_8 = Self {
            ram: [0; 4096],
            v: [0; 16],
            idx: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            stack: chip8_stack::Chip8Stack::default(),
            keypad: [false; 16],
            display_buffer: [0; 64 * 32],
            dirty: false,
            load_store_quirk: false,
            jp_v0_quirk: true,
            shift_quirk: true,
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
        match nibble_1 {
            0 => match op {
                0x00E0 => {
                    self.display_buffer = [0; SCREEN_AREA];
                    self.dirty = true;
                }
                0x00EE => self.pc = self.stack.pop(),

                _ => {
                    println!("SYS addr instruction, addr: {}", nnn(op));
                }
            },
            1 => self.pc = nnn(op),

            2 => {
                self.stack.push(self.pc);
                self.pc = nnn(op);
            }
            3 => {
                if self.v[x(op)] == nn(op) {
                    self.pc += 2;
                }
            }
            4 => {
                if self.v[x(op)] != nn(op) {
                    self.pc += 2;
                }
            }
            5 => {
                if n(op) == 0x0000 {
                    if self.v[x(op)] == self.v[y(op)] {
                        self.pc += 2;
                    }
                } else {
                    panic!("Unrecognized 5 instruction");
                }
            }
            6 => self.v[x(op)] = nn(op),

            7 => self.v[x(op)] = self.v[x(op)].wrapping_add(nn(op)), // Cover overflow

            8 => match n(op) {
                0x0000 => self.v[x(op)] = self.v[y(op)],
                0x0001 => self.v[x(op)] = self.v[x(op)] | self.v[y(op)],
                0x0002 => self.v[x(op)] = self.v[x(op)] & self.v[y(op)],
                0x0003 => self.v[x(op)] = self.v[x(op)] ^ self.v[y(op)],
                0x0004 => {
                    let (sum, carry) = self.v[x(op)].overflowing_add(self.v[y(op)]);
                    self.v[x(op)] = sum;
                    self.v[0xF] = carry as u8;
                }
                0x0005 => {
                    let (sum, borrow) = self.v[x(op)].overflowing_sub(self.v[y(op)]);
                    self.v[x(op)] = sum;
                    self.v[0xF] = !borrow as u8;
                }
                0x0006 => {
                    let xi = x(op) as usize;
                    let yi = y(op) as usize;

                    let src = if self.shift_quirk {
                        self.v[yi]
                    } else {
                        self.v[xi]
                    };
                    let carry = src & 1;
                    self.v[xi] = src >> 1;
                    self.v[0xF] = carry;
                }
                0x0007 => {
                    let (sum, borrow) = self.v[y(op)].overflowing_sub(self.v[x(op)]);
                    self.v[x(op)] = sum;
                    self.v[0xF] = !borrow as u8;
                }
                0x000E => {
                    let xi = x(op) as usize;
                    let yi = y(op) as usize;

                    let src = if self.shift_quirk {
                        self.v[yi]
                    } else {
                        self.v[xi]
                    };
                    let carry = (src >> 7) & 1;
                    self.v[xi] = src << 1;
                    self.v[0xF] = carry;
                }
                _ => {
                    panic!("Unrecognized 8 instruction");
                }
            },

            9 => {
                if self.v[x(op)] != self.v[y(op)] {
                    self.pc += 2;
                }
            }

            0xA => self.idx = nnn(op),

            0xB => {
                self.pc = nnn(op) + self.v[0] as u16;
                if self.jp_v0_quirk {
                    self.pc += self.v[x(op)] as u16;
                }
            }

            0xC => self.v[x(op)] = ((rand::rng().next_u32() & 0xFFFF) as u8) & nn(op),

            0xD => {
                // Coordinates
                let x0 = self.v[x(op)] as usize;
                let y0 = self.v[y(op)] as usize;
                // Clip the sprite
                let h = n(op).min(SCREEN_HEIGHT - y0);
                let w = 8usize.min(SCREEN_WIDTH - x0);
                self.v[0xF] = 0;

                for row in 0..h {
                    let b = self.ram[self.idx as usize + row];
                    for col in 0..w {
                        let bit = (b >> (7 - col)) & 1;
                        if bit == 0 {
                            continue;
                        }
                        let idx = idx_from_coord(x0 + col, y0 + row);
                        let prev = self.display_buffer[idx];
                        self.display_buffer[idx] ^= 1;
                        if prev == 1 {
                            self.v[0xF] = 1;
                        }
                    }
                }
                self.dirty = true;
            }

            0xE => match nn(op) {
                0x009E => {
                    if self.keypad[self.v[x(op)] as usize] == true {
                        self.pc += 2;
                    }
                }
                0x00A1 => {
                    if self.keypad[self.v[x(op)] as usize] == false {
                        self.pc += 2;
                    }
                }

                _ => {
                    panic!("Unsupported E-type instruction");
                }
            },

            0xF => match nn(op) {
                0x0007 => self.v[x(op)] = self.delay_timer,
                0x000A => {
                    if let Some(k) = self.keypad.iter().position(|&p| p) {
                        self.v[x(op)] = k as u8;
                    } else {
                        self.pc -= 2; // stall until a key is down
                    }
                }
                0x0015 => self.delay_timer = self.v[x(op)],

                0x0018 => self.sound_timer = self.v[x(op)],
                0x001E => {
                    let vx = self.v[x(op)] as u16;
                    self.v[0xF] = (self.idx > 0x0FFF - vx) as u8;
                    self.idx = self.idx.saturating_add(vx) & 0x0FFF;
                }
                0x0029 => self.idx = (FONT_SIZE as u16) * (self.v[x(op)] as u16),
                0x0033 => {
                    let vx = self.v[x(op)];
                    self.ram[self.idx as usize] = vx / 100;
                    self.ram[(self.idx + 1) as usize] = (vx / 10) % 10;
                    self.ram[(self.idx + 2) as usize] = vx % 10;
                }
                0x0055 => {
                    let num_v = x(op);
                    for i in 0..=num_v {
                        self.ram[self.idx as usize + i] = self.v[i];
                    }
                    if self.load_store_quirk {
                        self.idx += (num_v as u16) + 1;
                    }
                }
                0x0065 => {
                    let num_v = x(op);
                    for i in 0..=num_v {
                        self.v[i] = self.ram[self.idx as usize + i];
                    }
                    if self.load_store_quirk {
                        self.idx += (num_v as u16) + 1;
                    }
                }
                _ => {
                    panic!("Unsupported F-type instruction");
                }
            },

            _ => {
                panic!("Unrecognized instruction");
            }
        }
    }

    pub fn step(&mut self) {
        let op = self.fetch();
        self.pc += 2;
        self.decode_execute(op);
    }

    pub fn display_buffer(&self) -> &[u8] {
        &self.display_buffer
    }
}
