mod boot;
mod cpu;
mod dmg;
mod io;

use std::path::Path;

use cpu::Cpu;
use io::cartridge::Cartridge;

fn main() {
    env_logger::init();

    let mut cpu =
        Cpu::new(Cartridge::from_path(Path::new("./tetris.gb")).expect("unable to load rom"));
    loop {
        cpu.machine_cycle();
    }
}
