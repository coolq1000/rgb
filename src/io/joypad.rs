#[derive(Default)]
pub struct Joypad {
    directions: bool,
    actions: bool,
    right: bool,
    left: bool,
    up: bool,
    down: bool,
    a: bool,
    b: bool,
    select: bool,
    start: bool,
}

impl Joypad {
    pub fn set_matrix(&mut self, directions: bool, actions: bool) {
        self.directions = directions;
        self.actions = actions;
    }

    pub fn set_directions(&mut self, right: bool, left: bool, up: bool, down: bool) {
        self.right = right;
        self.left = left;
        self.up = up;
        self.down = down;
    }

    pub fn set_actions(&mut self, a: bool, b: bool, select: bool, start: bool) {
        self.a = a;
        self.b = b;
        self.select = select;
        self.start = start;
    }

    pub fn select_matrix(&self) -> u8 {
        let mut output = 0xc0;
        output |= (self.actions as u8) << 5;
        output |= (self.directions as u8) << 4;

        if self.directions {
            output |= self.select_directions();
        } else if self.actions {
            output |= self.select_actions();
        } else {
            output |= 0xf;
        }

        output
    }

    pub fn select_directions(&self) -> u8 {
        let mut r = !self.right as u8;
        let mut l = (!self.left as u8) << 1;
        let mut u = (!self.up as u8) << 2;
        let mut d = (!self.down as u8) << 3;

        // disallow pressing opposite directions at once
        if self.left && self.right {
            r = 1;
            l = 1;
        }

        if self.up && self.down {
            u = 1;
            d = 1;
        }

        r | l | u | d
    }

    pub fn select_actions(&self) -> u8 {
        let a = !self.a as u8;
        let b = (!self.b as u8) << 1;
        let se = (!self.select as u8) << 2;
        let st = (!self.start as u8) << 3;

        a | b | se | st
    }
}
