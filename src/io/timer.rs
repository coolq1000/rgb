pub struct Timer {
    counter: u8,
    modulo: u8,
    enabled: bool,
    divider: Divider,
    counter_16k: u32,
    pub interrupt: bool,
}

#[derive(Clone, Copy)]
enum Divider {
    Div16 = 4,
    Div64 = 6,
    Div256 = 8,
    Div1024 = 10,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            counter: 0,
            modulo: 0,
            enabled: false,
            divider: Divider::Div1024,
            counter_16k: 0,
            interrupt: false,
        }
    }

    pub fn tick(&mut self) {
        self.counter_16k += 1;

        if !self.enabled {
            return;
        }

        let mask = (1 << (self.divider as usize)) - 1;

        if self.counter_16k & mask == 0 {
            self.counter = self.counter.wrapping_add(1);

            if self.counter == 0 {
                // timer has overflowed, trip interrupt
                self.interrupt = true;
                self.counter = self.modulo;
            }
        }
    }

    pub fn div(&self) -> u8 {
        (self.counter_16k >> 8) as u8
    }
}
