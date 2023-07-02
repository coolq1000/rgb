pub struct Ppu {
    pub trips: Trips,
}

pub struct Trips {
    pub vblank: bool,
    pub hblank: bool,
    pub lcd: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            trips: Trips {
                vblank: false,
                hblank: false,
                lcd: false,
            },
        }
    }
}
