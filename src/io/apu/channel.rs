use super::{
    duty::Duty, envelope::Envelope, length::Length, noise::Noise, sweep::Sweep, wave::Wave,
};

#[derive(Default)]
pub struct Channel {
    pub dac: bool,
    pub left: bool,
    pub right: bool,
    pub length: Length,
    pub duty: Duty,
    pub envelope: Envelope,
    pub sweep: Sweep,
    pub wave: Wave,
    pub noise: Noise,
    pub enable: bool,
}

impl Channel {
    pub fn length_cycle(&mut self) {
        if self.length.enable && self.length.timer.tick() {
            self.enable = true;
        }
    }
}
