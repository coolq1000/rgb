use super::timer::Timer;

#[derive(Default)]
pub struct Envelope {
    pub timer: Timer,
    pub start_volume: u8,
    pub volume: u8,
    pub direction: i8,
    pub enable: bool,
}

impl Envelope {
    pub fn tick(&mut self) {
        if self.enable {
            if self.timer.period == 0 {
                self.timer.period = 8;
            }

            if self.timer.tick() {
                self.volume = (self.volume as i16 + self.direction as i16) as u8;

                if self.volume == 0x10 || self.volume == 0xff {
                    self.volume = (self.volume as i16 - self.direction as i16) as u8;
                    self.enable = false;
                }
            }
        }
    }
}
