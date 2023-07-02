use std::{
    fs::{File, OpenOptions},
    io::{Read, Result as IoResult},
    path::Path,
};

use self::mbc::Mbc;

mod mbc;

mod sizes {
    /// each rom bank is 16KiB
    pub const ROM_BANK: usize = 1024 * 16;
    /// each ram bank is 8KiB
    pub const RAM_BANK: usize = 1024 * 8;
    /// number of ram banks in mb5
    pub const RAM_COUNT: usize = 16;
}

mod offsets {
    pub const TITLE: usize = 0x134;
    pub const TYPE: usize = 0x147;
    pub const ROM_SIZE: usize = 0x148;
    pub const RAM_SIZE: usize = 0x149;
}

pub struct Cartridge {
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
    /// memory bank controller (mbc)
    controller: Mbc,
    /// file to save external ram to
    save_file: File,
}

impl Cartridge {
    pub fn from_path(path: &Path) -> IoResult<Cartridge> {
        // open rom file
        let mut cart_file = File::open(path)?;

        // read contents of rom into vector
        let mut rom = Vec::new();
        cart_file.read_to_end(&mut rom)?;

        // ensure there are at least two rom banks
        assert!(rom.len() >= sizes::ROM_BANK * 2);

        let mbc_type = rom[offsets::TYPE];

        Ok(Self {
            rom,
            ram: Vec::from([0; sizes::RAM_BANK * sizes::RAM_COUNT]),
            rom_banks: 2,
            rom_bank: 1,
            ram_bank: 0,
            controller: Mbc::from_id(mbc_type).expect("unknown mbc type"),
            save_file: OpenOptions::new()
                .write(true)
                .create(true)
                .open(path.with_extension("sav"))
                .expect("unable to write save file"),
        })
    }

    pub fn fetch_rom_byte(&self, address: u16) -> u8 {
        // check if the address is within the first bank
        if address < sizes::ROM_BANK as u16 {
            return self.rom[address as usize];
        }

        // otherwise return the switchable bank
        self.rom[address as usize + (self.rom_bank as usize * sizes::ROM_BANK)]
    }

    pub fn fetch_ram_byte(&self, address: u16) -> u8 {
        self.ram[address as usize + (self.ram_bank as usize * sizes::RAM_BANK)]
    }

    pub fn store_rom_byte(&mut self, address: u16, value: u8) {
        self.controller.write(address, value);
    }

    pub fn store_ram_byte(&mut self, address: u16, value: u8) {
        self.ram[address as usize + (self.ram_bank as usize * sizes::RAM_BANK)] = value;
    }
}
