#[derive(Default)]
pub struct Timer {
    pub counter: u16,
    pub period: u16,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            counter: 0,
            period: 0,
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.counter.checked_sub(1).is_none() {
            self.reset();
            true
        } else {
            self.counter -= 1;
            false
        }
    }

    pub fn reset(&mut self) {
        self.counter = self.period;
    }
}
