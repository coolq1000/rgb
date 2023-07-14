pub mod irq_vector {
    pub const VBLANK: u16 = 0x40;
    pub const LCDC: u16 = 0x48;
    pub const TIMER: u16 = 0x50;
    pub const SERIAL: u16 = 0x58;
    pub const JOYPAD: u16 = 0x60;
}

#[derive(Clone, Copy, Default)]
pub struct Interrupt {
    pub vblank: bool,
    pub lcdc: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool,
}

impl From<u8> for Interrupt {
    fn from(value: u8) -> Self {
        Self {
            vblank: (value & 0x01) != 0,
            lcdc: (value & 0x02) != 0,
            timer: (value & 0x04) != 0,
            serial: (value & 0x08) != 0,
            joypad: (value & 0x10) != 0,
        }
    }
}

impl From<Interrupt> for u8 {
    fn from(value: Interrupt) -> Self {
        (value.vblank as u8)
            | (value.lcdc as u8) << 1
            | (value.timer as u8) << 2
            | (value.serial as u8) << 3
            | (value.joypad as u8) << 4
    }
}
