use crate::io::{map, Bus};

use super::timer::Timer;

#[derive(Default)]
pub struct Wave {
    pub timer: Timer,
    frequency: u16,
    pub shift: u8,
    pub position: u16,
    pub output: u8,
}

impl Wave {
    pub fn tick(&mut self, wave_memory: &Box<[u8]>) {
        if self.timer.tick() {
            self.position = self.position.wrapping_add(1);
            self.output = wave_memory[((self.position & 0x1f) / 2) as usize];
            if self.position % 2 == 1 {
                self.output &= 0xf;
            } else {
                self.output >>= 4;
            }
        }
    }
}
