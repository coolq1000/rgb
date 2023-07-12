use self::{lcdc::Lcdc, tile::Pixel};

mod fetcher;
mod fifo;
mod lcdc;
mod tile;

pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;

pub const TILE_MAP_ADDRESS_1: u16 = 0x9800;
pub const TILE_MAP_ADDRESS_2: u16 = 0x9c00;

#[derive(Clone, Copy)]
pub enum PpuState {
    HBlank,        // m0
    VBlank,        // m1
    OamSearch,     // m2
    PixelTransfer, // m3
}

pub struct Ppu {
    pub interrupt_trips: InterruptTrips,
    bg_fifo: [Pixel; 16], // background
    sp_fifo: [Pixel; 16], // sprites
    lx: u8,
    ly: u8,
    lyc: u8,
    wx: u8,
    wy: u8,
    ticks: u32,
    state: PpuState,
    pending_state: Option<PpuState>,
    lcdc: Lcdc,

    x: u8,
    window_line_enable: bool,
    window_x: i32,
    line_tiles: []

    pub vram: Box<[u8]>,
    pub oam: Box<[u8]>,
    pub backbuffer: Box<[u8]>,
}

pub struct InterruptTrips {
    pub vblank: bool,
    pub hblank: bool,
    pub lcd: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            interrupt_trips: InterruptTrips {
                vblank: false,
                hblank: false,
                lcd: false,
            },
            bg_fifo: [Pixel::default(); 16],
            sp_fifo: [Pixel::default(); 16],
            lx: 0,
            ly: 0,
            lyc: 0,
            wx: 0,
            wy: 0,
            ticks: 0,
            state: PpuState::HBlank,
            pending_state: None,
            lcdc: Lcdc::default(),
            x: 0,
            window_line_enable: false,
            vram: Box::new([0; super::map::VRAM_SIZE]),
            oam: Box::new([0; super::map::OAM_SIZE]),
            backbuffer: Box::new([0; LCD_WIDTH * LCD_HEIGHT]),
        }
    }

    pub fn tick(&mut self) {
        if !self.lcdc.lcd_enable {
            self.ticks = 0;
            return;
        }

        self.ticks += 1;

        if let Some(pending_state) = self.pending_state {
            self.state = pending_state;
            // TODO: stat update
            self.pending_state = None;
        }

        if self.ly < 144 {
            if self.ticks == 4 && self.ly != 0 {
                // check self.lyc
            }

            match self.state {
                PpuState::OamSearch => {
                    if self.wy == self.ly {
                        self.window_line_enable = true
                    }

                    if self.ticks == 80 {
                        self.state = PpuState::PixelTransfer;
                        self.pending_state = Some(PpuState::PixelTransfer);
                    }
                }
                PpuState::PixelTransfer => todo!(),
                PpuState::HBlank => todo!(),
                PpuState::VBlank => todo!(),
            }
        }
    }

    fn init_pixel_transfer(&mut self) {
        self.x = 0;
        self.window_x = (self.wx as i32) - 7;

    }
}
