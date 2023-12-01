use super::{timer::Timer, DUTY_TABLE};

#[derive(Default)]
pub struct Duty {
    pub timer: Timer,
    pub pattern: u8,
    position: u8,
    pub frequency: u16,
    pub state: bool,
    pub enable: bool,
}

impl Duty {
    pub fn tick(&mut self) {
        self.timer.period = (2048 - self.frequency) * 4;

        if self.timer.tick() {
            self.position = self.position.wrapping_add(1);
            self.state = DUTY_TABLE[self.pattern as usize][self.position as usize % 8];
        }
    }
}
