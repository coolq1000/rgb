#[derive(Clone, Copy)]
pub struct Lcdc {
    pub lcd_enable: bool,    // 7
    pub win_map: bool,       // 6
    pub win_enable: bool,    // 5
    pub bg_map: bool,        // 3
    pub obj_size: bool,      // 2
    pub obj_enable: bool,    // 1
    pub bg_win_map: bool,    // 4
    pub bg_win_enable: bool, // 0
}

impl Default for Lcdc {
    fn default() -> Self {
        Self {
            lcd_enable: true,
            win_map: false,
            win_enable: false,
            bg_map: false,
            obj_size: false,
            obj_enable: false,
            bg_win_map: false,
            bg_win_enable: false,
        }
    }
}

impl From<u8> for Lcdc {
    fn from(value: u8) -> Self {
        Self {
            lcd_enable: value & 0x80 != 0,
            win_map: value & 0x40 != 0,
            win_enable: value & 0x20 != 0,
            bg_map: value & 0x8 != 0,
            obj_size: value & 0x4 != 0,
            obj_enable: value & 0x2 != 0,
            bg_win_map: value & 0x10 != 0,
            bg_win_enable: value & 0x1 != 0,
        }
    }
}

impl From<Lcdc> for u8 {
    fn from(value: Lcdc) -> Self {
        (value.lcd_enable as u8) << 7
            | (value.win_map as u8) << 6
            | (value.win_enable as u8) << 5
            | (value.bg_map as u8) << 3
            | (value.obj_size as u8) << 2
            | (value.obj_enable as u8) << 1
            | (value.bg_win_map as u8) << 4
            | value.bg_win_enable as u8
    }
}
