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
    /// memory bank controller (mbc)
    pub controller: Mbc,
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
            controller: Mbc::new(rom, mbc_type),
            save_file: OpenOptions::new()
                .write(true)
                .create(true)
                .open(path.with_extension("sav"))
                .expect("unable to write save file"),
        })
    }
}
