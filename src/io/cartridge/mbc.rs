use log::warn;

use super::sizes;

pub enum MbcType {
    RomOnly,
    Mbc1,
    Mbc5,
}

pub struct Mbc {
    /// catridge read only memory
    rom: Vec<u8>,
    /// catridge random access memory
    ram: Vec<u8>,
    /// number of banks within the cartridge (see controllers)
    rom_banks: u8,
    /// currently selected rom bank
    rom_bank: u8,
    /// currently selected ram bank
    ram_bank: u8,
    /// type of controller
    mbc_type: MbcType,
}

impl Mbc {
    pub fn new(rom: Vec<u8>, id: u8) -> Self {
        let mbc_type = match id {
            0 => MbcType::RomOnly,
            1..=3 => MbcType::Mbc1,
            _ => MbcType::Mbc5,
        };

        Self {
            rom,
            ram: Vec::from([0; sizes::RAM_BANK * sizes::RAM_COUNT]),
            rom_banks: 2,
            rom_bank: 1,
            ram_bank: 0,
            mbc_type,
        }
    }

    pub fn fetch_rom_byte(&self, address: u16) -> u8 {
        // check if the address is within the first bank
        if address < sizes::ROM_BANK as u16 {
            return self.rom[address as usize];
        }

        // otherwise return the switchable bank
        self.rom[(address as usize) - sizes::ROM_BANK + (self.rom_bank as usize * sizes::ROM_BANK)]
    }

    pub fn store_rom_byte(&mut self, address: u16, value: u8) {
        match self.mbc_type {
            MbcType::RomOnly => warn!("unhandled rom write: {:04x} = {:02x}", address, value),
            MbcType::Mbc1 => match address {
                0x2000..=0x3fff => {
                    let curr_bank = self.rom_bank & 0xe0;
                    let bank = curr_bank | (value & 0x1f);

                    self.rom_bank = bank;
                }
                _ => warn!("unhandled rom write: {:04x} = {:02x}", address, value),
            },
            MbcType::Mbc5 => warn!("unhandled rom write: {:04x} = {:02x}", address, value),
        }
    }

    pub fn fetch_ram_byte(&self, address: u16) -> u8 {
        self.ram[address as usize + (self.ram_bank as usize * sizes::RAM_BANK)]
    }

    pub fn store_ram_byte(&mut self, address: u16, value: u8) {
        self.ram[address as usize + (self.ram_bank as usize * sizes::RAM_BANK)] = value;
    }
}
