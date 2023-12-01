pub struct Timer {
    pub counter: u8,
    pub modulo: u8,
    enable: bool,
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
            enable: false,
            divider: Divider::Div1024,
            counter_16k: 0,
            interrupt: false,
        }
    }

    pub fn tick(&mut self) {
        self.counter_16k += 1;

        if !self.enable {
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

    pub fn get_div(&self) -> u8 {
        (self.counter_16k >> 8) as u8
    }

    pub fn reset_div(&mut self) {
        self.counter_16k = 0;
    }

    pub fn set_control(&mut self, ctrl: u8) {
        self.enable = ctrl & 4 != 0;

        self.divider = match ctrl & 3 {
            0 => Divider::Div1024,
            1 => Divider::Div16,
            2 => Divider::Div64,
            3 => Divider::Div256,
            _ => unreachable!(),
        };
    }

    pub fn get_control(&self) -> u8 {
        let mut ctrl = 0;

        ctrl |= (self.enable as u8) << 2;
        ctrl |= self.divider as u8;

        ctrl
    }
}
