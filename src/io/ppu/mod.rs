pub use self::{lcdc::Lcdc, status::PpuStatus};

use super::map::{OAM_SIZE, VRAM_SIZE};

pub mod lcdc;
pub mod status;

mod timing {
    pub const OAM_ACCESS: usize = 83;
    pub const HBLANK: usize = 207;
    pub const PIXEL_TRANSFER: usize = 175;
    pub const SCANLINE: usize = 456;
}

pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;
pub const SCANLINE_MAX: usize = 153;

const DMG_PALETTE: [Colour; 4] = [
    Colour {
        r: 0xe0,
        g: 0xf8,
        b: 0xd0,
    }, // 100%
    Colour {
        r: 0x88,
        g: 0xc0,
        b: 0x70,
    }, // 66%
    Colour {
        r: 0x34,
        g: 0x68,
        b: 0x56,
    }, // 33%
    Colour {
        r: 0x08,
        g: 0x18,
        b: 0x20,
    }, // 0%
];

#[derive(Clone, Copy)]
enum PpuStage {
    HBlank,        // m0
    VBlank,        // m1
    OamSearch,     // m2
    PixelTransfer, // m3
}

#[derive(Clone, Copy, Default)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Ppu {
    stage: PpuStage,
    pub ly: u8,
    pub lyc: u8,
    pub scx: u8,
    pub scy: u8,
    pub wx: u8,
    pub wy: u8,
    pub bgp: u8,
    pub obp0: u8,
    pub obp1: u8,
    pub lcdc: Lcdc,
    pub stat: PpuStatus,
    pub vblank_int: bool,
    pub lcd_stat_int: bool,
    pub vram: Box<[u8]>,
    pub oam: Box<[u8]>,
    pub framebuffer: Box<[Colour]>,
    pub frame: u32,
    ticks: u32,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            stage: PpuStage::OamSearch,
            ly: 0,
            lyc: 0,
            scx: 0,
            scy: 0,
            wx: 0,
            wy: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            lcdc: Lcdc::default(),
            stat: PpuStatus::default(),
            vblank_int: false,
            lcd_stat_int: false,
            vram: vec![0; VRAM_SIZE].into_boxed_slice(),
            oam: vec![0; OAM_SIZE].into_boxed_slice(),
            framebuffer: vec![Colour::default(); LCD_WIDTH * LCD_HEIGHT].into_boxed_slice(),
            frame: 0,
            ticks: 0,
        }
    }

    fn update_ly(&mut self) {
        self.ly = (self.ly + 1) % SCANLINE_MAX as u8;
    }

    fn compare_ly_lyc(&mut self) {
        if self.ly == self.lyc {
            self.stat.coincidence_flag = true;
            self.lcd_stat_int = true;
        }
    }

    fn update_stat_interrupt(&mut self) {
        self.lcd_stat_int = match self.stage {
            PpuStage::HBlank => self.stat.m0_hblank_interrupt,
            PpuStage::VBlank => self.stat.m1_vblank_interrupt,
            PpuStage::OamSearch => self.stat.m2_oam_interrupt,
            PpuStage::PixelTransfer => panic!("invalid lcd interrupt status"),
        };
    }

    pub fn tick(&mut self) {
        if !self.lcdc.lcd_enable {
            self.stage = PpuStage::HBlank;
            self.lyc = 0;
            self.ticks = 0;
            return;
        }

        match self.stage {
            PpuStage::OamSearch => {
                if self.ticks >= timing::OAM_ACCESS as u32 {
                    self.stage = PpuStage::PixelTransfer;
                    self.ticks -= timing::OAM_ACCESS as u32;
                }
            }
            PpuStage::PixelTransfer => {
                if self.ticks >= timing::PIXEL_TRANSFER as u32 {
                    // TODO: investigate why ly can be higher than 144
                    if self.ly < LCD_HEIGHT as u8 {
                        self.render_line();
                    }
                    // TODO: hdma transfer
                    self.stage = PpuStage::HBlank;
                    self.update_stat_interrupt();
                    self.ticks -= timing::PIXEL_TRANSFER as u32;
                }
            }
            PpuStage::HBlank => {
                if self.ticks >= timing::HBLANK as u32 {
                    self.update_ly();
                    self.compare_ly_lyc();

                    self.stage = if self.ly == LCD_HEIGHT as u8 {
                        self.vblank_int = true;
                        PpuStage::VBlank
                    } else {
                        PpuStage::OamSearch
                    };

                    self.update_stat_interrupt();
                    self.ticks -= timing::HBLANK as u32;
                }
            }
            PpuStage::VBlank => {
                if self.ticks >= timing::SCANLINE as u32 {
                    self.update_ly();
                    self.compare_ly_lyc();

                    if self.ly == 0 {
                        self.frame = self.frame.wrapping_add(1);
                        self.stage = PpuStage::OamSearch;
                        self.update_stat_interrupt();
                    }

                    self.ticks -= timing::SCANLINE as u32;
                }
            }
        }

        self.stat.mode_flag = self.stage as u8;
        self.ticks += 1;
    }

    fn get_pixel(&self, x: u32, y: u32) -> Colour {
        self.framebuffer[(x + y * LCD_WIDTH as u32) as usize]
    }

    fn set_pixel(&mut self, x: u32, y: u32, value: Colour) {
        self.framebuffer[(x + y * LCD_WIDTH as u32) as usize] = value;
    }

    fn get_tile(&self, tile_id: u8, x: u8, y: u8, sprite: bool) -> u8 {
        let tile_addr: u16 = if self.lcdc.bg_win_map || sprite {
            tile_id as u16 * 16
        } else {
            0x1000 + tile_id as u16 * 16
        };

        let line_1 = self.vram[(tile_addr + y as u16 * 2) as usize + 1];
        let line_2 = self.vram[(tile_addr + y as u16 * 2) as usize];

        let mask: u8 = 0x80 >> x;

        ((((line_1 & mask) > 0) as u8) << 1) | ((line_2 & mask) > 0) as u8
    }

    fn convert_dmg_palette(&self, palette: u8, id: u8) -> u8 {
        (palette >> (id * 2)) & 3
    }

    fn render_background_pixel(&self, x: u8, y: u8, window: bool) -> Colour {
        let map_area: u16 = if (self.lcdc.bg_map && !window) || (self.lcdc.win_map && window) {
            0x1c00
        } else {
            0x1800
        };

        let tile_x = x / 8;
        let tile_y = y / 8;

        let map_address = map_area + (tile_x as u16) + (tile_y as u16 * 32);
        let tile_id = self.vram[map_address as usize];
        let tile = self.get_tile(tile_id, x % 8, y % 8, false);

        DMG_PALETTE[self.convert_dmg_palette(self.bgp, tile) as usize]
    }

    fn render_sprite_pixel(&mut self, x: u8, y: u8) {
        for i in 0..40 {
            let sprite_address = i * 4;

            // check y-coordinate isn't 0
            if self.oam[sprite_address] != 0 {
                let sprite_x = self.oam[sprite_address + 1] as i16 - 8;
                let sprite_y = self.oam[sprite_address] as i16 - 16;
                let sprite_tile_id =
                    self.oam[sprite_address + 2] & if self.lcdc.obj_size { 0xfe } else { 0xff };
                let sprite_tile_attributes = self.oam[sprite_address + 3];

                let palette = if sprite_tile_attributes & 0x10 > 0 {
                    self.obp0
                } else {
                    self.obp1
                };

                let sprite_height = if self.lcdc.obj_size { 16 } else { 8 };

                if (y as i16) >= sprite_y
                    && (y as i16) < sprite_y + sprite_height
                    && (x as i16) >= sprite_x
                    && (x as i16) < sprite_x + 8
                {
                    self.set_pixel(
                        x as u32,
                        y as u32,
                        Colour {
                            r: 255,
                            g: 255,
                            b: 255,
                        },
                    );
                    let flip_x = (sprite_tile_attributes & 0x20) > 0;
                    let flip_y = (sprite_tile_attributes & 0x40) > 0;
                    let priority = (sprite_tile_attributes & 0x80) > 0;

                    let mut pixel_x = (x as i16 - sprite_x) % 8;
                    let mut pixel_y = (y as i16 - sprite_y) % sprite_height;

                    if flip_x {
                        pixel_x = 7 - pixel_x;
                    }

                    if flip_y {
                        pixel_y = sprite_height - 1 - pixel_y;
                    }

                    let sprite = self.get_tile(sprite_tile_id, pixel_x as u8, pixel_y as u8, true);
                    let pixel = self.convert_dmg_palette(palette, sprite);

                    if sprite != 0 {
                        self.set_pixel(x as u32, y as u32, DMG_PALETTE[pixel as usize]);
                    }
                }
            }
        }
    }

    fn render_line(&mut self) {
        let window_x = self.wx.wrapping_sub(7);
        let window_y = self.wy;

        for x in 0..LCD_WIDTH {
            if self.lcdc.bg_win_enable
            /* TODO: change in cgb */
            {
                let pixel = self.render_background_pixel(
                    (x as u8).wrapping_add(self.scx),
                    (self.ly).wrapping_add(self.scy),
                    false,
                );

                self.set_pixel(x as u32, self.ly as u32, pixel);
            }
            // if self.lcdc.obj_enable {
            self.render_sprite_pixel(x as u8, self.ly);
            // }
        }
    }
}
