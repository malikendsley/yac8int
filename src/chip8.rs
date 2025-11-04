use crate::chip8_stack;

pub struct Chip8 {
    ram: [u8; 4096],
    v_reg: [u8; 16],
    i_reg: u16,
    delay_timer: u16,
    sound_timer: u16,
    pc: u16,
    stack: chip8_stack::Chip8Stack,
    keypad: [bool; 16],
    buffer: [u8; 64 * 32],
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
            buffer: [0; 64 * 32],
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
        println!("Nibble 1: {:01X}", nibble_1);
        match nibble_1 {
            0 => {
                println!("Clearing screen");
            }
            1 => {
                println!("Jump");
            }
            6 => {
                println!("Set register VX");
            }
            7 => {
                println!("Add to VX");
            }
            0xA => {
                println!("Set index register i")
            }
            0xD => {
                println!("Draw");
            }
            _ => {
                println!("Unimplemented");
            }
        }
    }

    pub fn step(&mut self) {
        // Read 2 bytes at the PC
        let op = self.fetch();
        self.pc += 2;
        println!("Fetched {:04X}", op);
        self.decode_execute(op);
    }
}
