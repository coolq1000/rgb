use super::{channel::Channel, duty::Duty, timer::Timer};

#[derive(Default)]
pub struct Sweep {
    pub timer: Timer,
    pub frequency: u16,
    pub shift: u8,
    pub decreasing: bool,
    pub calculated: bool,
    pub enable: bool,
}

impl Sweep {
    pub fn tick(&mut self, duty: &mut Duty, channel_enable: &mut bool) {
        if self.enable && self.timer.tick() && self.timer.period != 0 {
            let sweep_frequency = self.calc_frequency();
            if self.shift != 0 && sweep_frequency < 2048 {
                self.frequency = sweep_frequency;
                duty.frequency = sweep_frequency;
            }

            let sweep_frequency = self.calc_frequency();
            if sweep_frequency >= 2048 {
                *channel_enable = false;
            }
        }
    }

    pub fn calc_frequency(&mut self) -> u16 {
        if self.decreasing {
            self.decreasing = true;
            self.frequency - (self.frequency >> self.shift)
        } else {
            self.frequency + (self.frequency >> self.shift)
        }
    }
}
