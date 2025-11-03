#[derive(Default)]
pub struct Chip8Stack {
    slots: [u16; 16],
    sp: usize,
}

impl Chip8Stack {
    pub fn push(&mut self, value: u16) {
        if self.sp == 16 {
            panic!("Overfilled Chip-8 stack.");
        }
        self.slots[self.sp] = value;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> u16 {
        if self.sp == 0 {
            panic!("Popped empty Chip-8 stack");
        }

        self.sp -= 1;
        return self.slots[self.sp];
    }
}

#[cfg(test)]
mod tests {
    use crate::chip8_stack::Chip8Stack;

    #[test]
    fn push_pop() {
        let mut s = Chip8Stack::default();
        for i in 0..16 {
            s.push(i);
        }
        for i in 0..15 {
            assert_eq!(s.pop(), 15 - i);
        }
    }

    #[test]
    #[should_panic]
    fn overfill() {
        let mut s = Chip8Stack::default();
        for i in 0..17 {
            s.push(i);
        }
    }

    #[test]
    #[should_panic]
    fn pop_empty() {
        let mut s = Chip8Stack::default();
        s.push(1);
        s.pop();
        s.pop();
    }
}
