pub mod cartridge;
pub mod joypad;
pub mod ppu;
pub mod timer;

use crate::{boot, cpu::interrupt::Interrupt};

use cartridge::Cartridge;

use log::{error, warn};
pub use ppu::Ppu;

pub use timer::Timer;

use self::{
    joypad::Joypad,
    ppu::{Lcdc, PpuStatus},
};

mod map {
    /// cartridge rom
    pub const ROM_LOW: u16 = 0x0000;
    pub const ROM_HIGH: u16 = 0x7fff;
    pub const ROM_SIZE: usize = 0x8000;
    /// video ram
    pub const VRAM_LOW: u16 = 0x8000;
    pub const VRAM_HIGH: u16 = 0x9fff;
    pub const VRAM_SIZE: usize = 0x2000;
    /// external cartridge ram
    pub const XRAM_LOW: u16 = 0xa000;
    pub const XRAM_HIGH: u16 = 0xbfff;
    pub const XRAM_SIZE: usize = 0x2000;
    /// internal work ram
    pub const WRAM_LOW: u16 = 0xc000;
    pub const WRAM_HIGH: u16 = 0xdfff;
    pub const WRAM_SIZE: usize = 0x2000;
    /// echo ram, mirror of 0xc000-0xddff
    pub const ECHO_LOW: u16 = 0xe000;
    pub const ECHO_HIGH: u16 = 0xfdff;
    pub const ECHO_SIZE: usize = 0x1e00;
    /// sprite attribute memory
    pub const OAM_LOW: u16 = 0xfe00;
    pub const OAM_HIGH: u16 = 0xfe9f;
    pub const OAM_SIZE: usize = 0xa0;
    /// high ram
    pub const HRAM_LOW: u16 = 0xff80;
    pub const HRAM_HIGH: u16 = 0xfffe;
    pub const HRAM_SIZE: usize = 0x7f;

    pub mod joyp_io {
        pub const JOYP_ADDR: u16 = 0xff00;
    }

    pub mod lcd_io {
        pub const LCDC_ADDR: u16 = 0xff40;
        pub const STAT_ADDR: u16 = 0xff41;
        pub const SCY_ADDR: u16 = 0xff42;
        pub const SCX_ADDR: u16 = 0xff43;
        pub const LY_ADDR: u16 = 0xff44;
        pub const LYC_ADDR: u16 = 0xff45;
        pub const DMA_ADDR: u16 = 0xff46;
        pub const BGP_ADDR: u16 = 0xff47;
        pub const OBP0_ADDR: u16 = 0xff48;
        pub const OBP1_ADDR: u16 = 0xff49;
        pub const WY_ADDR: u16 = 0xff4a;
        pub const WX_ADDR: u16 = 0xff4b;
    }

    /// interrupt flag (IF)
    pub const INTERRUPT_FLAG: u16 = 0xff0f;
    /// switch to unmap the bootrom
    pub const UNMAP_BOOTROM: u16 = 0xff50;
    /// interrupt enable (IE)
    pub const INTERRUPT_ENABLE: u16 = 0xffff;
}

pub struct Bus {
    pub cart: Cartridge,
    pub wram: Box<[u8]>,
    pub hram: Box<[u8]>,
    dma_src: u16,
    dma_idx: u16,
    /// set of cpu interrupts, disrupt control flow
    pub it_enable: Interrupt,
    pub it_flag: Interrupt,
    pub ppu: Ppu,
    pub joypad: Joypad,
    pub timer: Timer,
    pub boot: bool,
}

impl Bus {
    pub fn new(cart: Cartridge) -> Self {
        Self {
            cart,
            wram: Box::new([0; map::WRAM_SIZE]),
            hram: Box::new([0; map::HRAM_SIZE]),
            dma_src: 0,
            dma_idx: map::OAM_SIZE as u16,
            it_enable: Interrupt::from(0),
            it_flag: Interrupt::from(0),
            ppu: Ppu::new(),
            joypad: Joypad::default(),
            timer: Timer::new(),
            boot: true,
        }
    }

    pub fn fetch_byte(&self, address: u16) -> u8 {
        match address {
            map::ROM_LOW..=map::ROM_HIGH => {
                if self.boot && (address as usize) < boot::BOOTROM.len() {
                    boot::BOOTROM[address as usize]
                } else {
                    self.cart.fetch_rom_byte(address)
                }
            }
            map::VRAM_LOW..=map::VRAM_HIGH => self.ppu.vram[(address - map::VRAM_LOW) as usize],
            map::XRAM_LOW..=map::XRAM_HIGH => self.cart.fetch_ram_byte(address - map::XRAM_LOW),
            map::WRAM_LOW..=map::WRAM_HIGH => self.wram[(address - map::WRAM_LOW) as usize],
            map::ECHO_LOW..=map::ECHO_HIGH => self.wram[(address - map::ECHO_LOW) as usize],
            map::OAM_LOW..=map::OAM_HIGH => self.ppu.oam[(address - map::OAM_LOW) as usize],
            map::joyp_io::JOYP_ADDR => self.joypad.select_matrix(),
            map::lcd_io::LCDC_ADDR => self.ppu.lcdc.into(),
            map::lcd_io::STAT_ADDR => self.ppu.stat.into(),
            map::lcd_io::SCY_ADDR => self.ppu.scy,
            map::lcd_io::SCX_ADDR => self.ppu.scx,
            map::lcd_io::LY_ADDR => self.ppu.ly,
            map::lcd_io::LYC_ADDR => self.ppu.lyc,
            map::lcd_io::DMA_ADDR => (self.dma_src >> 8) as u8,
            map::lcd_io::BGP_ADDR => self.ppu.bgp,
            map::lcd_io::OBP0_ADDR => self.ppu.obp0,
            map::lcd_io::OBP1_ADDR => self.ppu.obp1,
            map::lcd_io::WY_ADDR => self.ppu.wy,
            map::lcd_io::WX_ADDR => self.ppu.wx,
            map::HRAM_LOW..=map::HRAM_HIGH => self.hram[(address - map::HRAM_LOW) as usize],
            map::INTERRUPT_ENABLE => self.it_enable.into(),
            _ => {
                warn!("attempt to read from unmapped memory `0x{:04x}`", address);
                0xff
            }
        }
    }

    pub fn tick(&mut self) {
        self.ppu.tick();

        if self.dma_idx < map::OAM_SIZE as u16 {
            let direct_byte = self.fetch_byte(self.dma_src);
            self.ppu.oam[self.dma_idx as usize] = direct_byte;

            self.dma_idx += 1;
            self.dma_src += 1;
        }
    }

    pub fn store_byte(&mut self, address: u16, value: u8) {
        match address {
            map::ROM_LOW..=map::ROM_HIGH => {
                self.cart.store_rom_byte(address, value);
            }
            map::VRAM_LOW..=map::VRAM_HIGH => {
                self.ppu.vram[(address - map::VRAM_LOW) as usize] = value;
            }
            map::XRAM_LOW..=map::XRAM_HIGH => {
                self.cart.store_ram_byte(address - map::XRAM_LOW, value);
            }
            map::WRAM_LOW..=map::WRAM_HIGH => {
                self.wram[(address - map::WRAM_LOW) as usize] = value;
            }
            map::ECHO_LOW..=map::ECHO_HIGH => {
                self.wram[(address - map::ECHO_LOW) as usize] = value;
            }
            map::OAM_LOW..=map::OAM_HIGH => {
                self.ppu.oam[(address - map::OAM_LOW) as usize] = value;
            }
            map::INTERRUPT_FLAG => {
                self.it_flag = Interrupt::from(value);
            }
            map::joyp_io::JOYP_ADDR => {
                let matrix_col_2x2 = (value & 0x30) >> 4;
                let least = (matrix_col_2x2 & 1) != 0;
                let most = (matrix_col_2x2 & 2) != 0;
                self.joypad.set_matrix(most, least);
            }
            map::lcd_io::LCDC_ADDR => {
                self.ppu.lcdc = Lcdc::from(value);
            }
            map::lcd_io::STAT_ADDR => {
                self.ppu.stat = PpuStatus::from(value & 0xfc);
            }
            map::lcd_io::SCY_ADDR => {
                self.ppu.scy = value;
            }
            map::lcd_io::SCX_ADDR => {
                self.ppu.scx = value;
            }
            map::lcd_io::LYC_ADDR => {
                self.ppu.lyc = value;
            }
            map::lcd_io::DMA_ADDR => {
                self.dma_idx = 0;
                self.dma_src = (value as u16) << 8;
            }
            map::lcd_io::BGP_ADDR => {
                self.ppu.bgp = value;
            }
            map::lcd_io::OBP0_ADDR => {
                self.ppu.obp0 = value;
            }
            map::lcd_io::OBP1_ADDR => {
                self.ppu.obp1 = value;
            }
            map::lcd_io::WY_ADDR => {
                self.ppu.wy = value;
            }
            map::lcd_io::WX_ADDR => {
                self.ppu.wx = value;
            }
            map::UNMAP_BOOTROM => {
                if self.boot && value == 1 {
                    self.boot = false;
                }
            }
            map::HRAM_LOW..=map::HRAM_HIGH => {
                self.hram[(address - map::HRAM_LOW) as usize] = value;
            }
            map::INTERRUPT_ENABLE => {
                self.it_enable = Interrupt::from(value);
            }
            _ => {
                warn!("attempt to write to unmapped memory `0x{:04x}`", address);
            }
        }
    }
}
