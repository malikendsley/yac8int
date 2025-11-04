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
}
