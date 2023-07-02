use crate::{boot, cpu::interrupt::Interrupt};

use cartridge::Cartridge;

use log::error;
pub use ppu::Ppu;

pub use timer::Timer;

pub mod cartridge;
pub mod ppu;
pub mod timer;

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

    /// interrupt flag (IF)
    pub const INTERRUPT_FLAG: u16 = 0xff0f;
    /// switch to unmap the bootrom
    pub const UNMAP_BOOTROM: u16 = 0xff50;
    /// interrupt enable (IE)
    pub const INTERRUPT_ENABLE: u16 = 0xffff;
}

pub struct Bus {
    pub cart: Cartridge,
    pub vram: Box<[u8]>,
    pub wram: Box<[u8]>,
    pub oam: Box<[u8]>,
    pub hram: Box<[u8]>,
    pub it_enable: Interrupt,
    pub it_flag: Interrupt,
    pub ppu: Ppu,
    pub timer: Timer,
    pub boot: bool,
}

impl Bus {
    pub fn new(cart: Cartridge) -> Self {
        Self {
            cart,
            vram: Box::new([0; map::VRAM_SIZE]),
            wram: Box::new([0; map::WRAM_SIZE]),
            oam: Box::new([0; map::OAM_SIZE]),
            hram: Box::new([0; map::HRAM_SIZE]),
            it_enable: Interrupt::from(0),
            it_flag: Interrupt::from(0),
            ppu: Ppu::new(),
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
            map::VRAM_LOW..=map::VRAM_HIGH => self.vram[(address - map::VRAM_LOW) as usize],
            map::XRAM_LOW..=map::XRAM_HIGH => self.cart.fetch_ram_byte(address - map::XRAM_LOW),
            map::WRAM_LOW..=map::WRAM_HIGH => self.wram[(address - map::WRAM_LOW) as usize],
            map::ECHO_LOW..=map::ECHO_HIGH => self.wram[(address - map::ECHO_LOW) as usize],
            map::OAM_LOW..=map::OAM_HIGH => self.oam[(address - map::OAM_LOW) as usize],
            map::HRAM_LOW..=map::HRAM_HIGH => self.hram[(address - map::HRAM_LOW) as usize],
            map::INTERRUPT_ENABLE => self.it_enable.into(),
            _ => {
                error!("attempt to read from unmapped memory `0x{:04x}`", address);
                0xff
            }
        }
    }

    pub fn step(&mut self) {}

    pub fn store_byte(&mut self, address: u16, value: u8) {
        match address {
            map::ROM_LOW..=map::ROM_HIGH => {
                self.cart.store_rom_byte(address, value);
            }
            map::VRAM_LOW..=map::VRAM_HIGH => {
                self.vram[(address - map::VRAM_LOW) as usize] = value;
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
                self.oam[(address - map::OAM_LOW) as usize] = value;
            }
            map::INTERRUPT_FLAG => {
                self.it_flag = Interrupt::from(value);
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
                error!("attempt to write to unmapped memory `0x{:04x}`", address);
            }
        }
    }
}
