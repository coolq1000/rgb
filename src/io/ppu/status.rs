#[derive(Clone, Copy, Default)]
pub struct PpuStatus {
    pub lyc_check: bool,           // 6
    pub m2_oam_interrupt: bool,    // 5
    pub m1_vblank_interrupt: bool, // 4
    pub m0_hblank_interrupt: bool, // 3
    pub coincidence_flag: bool,    // 2
    pub mode_flag: u8,             // 1-0
}

impl From<u8> for PpuStatus {
    fn from(value: u8) -> Self {
        Self {
            lyc_check: value & 0x40 != 0,
            m2_oam_interrupt: value & 0x20 != 0,
            m1_vblank_interrupt: value & 0x8 != 0,
            m0_hblank_interrupt: value & 0x4 != 0,
            coincidence_flag: value & 0x2 != 0,
            mode_flag: value & 0x3,
        }
    }
}

impl From<PpuStatus> for u8 {
    fn from(value: PpuStatus) -> Self {
        (value.lyc_check as u8) << 6
            | (value.m2_oam_interrupt as u8) << 5
            | (value.m1_vblank_interrupt as u8) << 3
            | (value.m0_hblank_interrupt as u8) << 2
            | (value.coincidence_flag as u8) << 1
            | value.mode_flag
    }
}
