use super::timer::Timer;

#[derive(Default)]
pub struct Noise {
    pub timer: Timer,
    shift: u8,
    width_mode: bool,
    pub lfsr: u16,
    pub state: bool,
}

impl Noise {
    pub fn tick(&mut self) {
        if self.timer.tick() {
            let lfsr_low = (self.lfsr & 0xff) as u8;
            let temp = (lfsr_low & 1) ^ ((lfsr_low & 2) >> 1);
            self.lfsr >>= 1;
            self.lfsr &= 0xbfff;
            self.lfsr |= (temp as u16) * 0x4000;
            if self.width_mode {
                self.lfsr &= 0xffbf;
                self.lfsr |= (temp as u16) * 0x40;
            }
            self.state = !(self.lfsr & 1 != 0);
        }
    }
}
