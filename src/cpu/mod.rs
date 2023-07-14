use crate::io::{cartridge::Cartridge, Bus};

use interrupt::Interrupt;
use log::{error, info};

mod disasm;
pub mod interrupt;
mod opcodes;

pub struct Cpu {
    /// all cpu registers excluding flags
    pub registers: Registers,
    /// cpu flags, aka. 'f' register
    pub flags: Flags,
    /// interrupt enable
    it_master_enable: bool,
    /// set to true if interrupt should be delayed one instruction
    it_master_enable_next: bool,
    /// cpu is stopped
    halted: bool,
    /// access other parts of the machine
    pub bus: Bus,
    /// elapsed cycles after instruction
    cycles: u8,
}

#[derive(Default)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
}

#[derive(Default, Clone, Copy)]
pub struct Flags {
    z: bool,
    n: bool,
    h: bool,
    c: bool,
}

impl Cpu {
    pub fn new(cart: Cartridge) -> Self {
        Self {
            registers: Registers::default(),
            flags: Flags::default(),
            it_master_enable: false,
            it_master_enable_next: false,
            halted: false,
            bus: Bus::new(cart),
            cycles: 0,
        }
    }

    pub fn machine_cycle(&mut self) -> u8 {
        self.cycles = 0;

        if self.it_master_enable {
            if self.handle_interrupt() {
                // interrupt was handled, return cycles
                return self.cycles;
            }
        } else {
            self.it_master_enable = self.it_master_enable_next;
        }

        if self.halted {
            self.bus.tick();
            self.cycles += 1;

            // even if the interrupt flag is disabled
            // the halt instruction should still end
            if !self.it_master_enable && self.check_interrupt() {
                self.halted = false;
            } else {
                return self.cycles;
            }
        }

        // fetch next instruction
        self.execute_next();

        self.cycles
    }

    pub fn get_af(&self) -> u16 {
        let a: u16 = self.registers.a as u16;
        let f: u16 = u8::from(self.flags) as u16;
        f | (a << 8)
    }

    pub fn get_bc(&self) -> u16 {
        let b: u16 = self.registers.b as u16;
        let c: u16 = self.registers.c as u16;
        c | (b << 8)
    }

    pub fn get_de(&self) -> u16 {
        let d: u16 = self.registers.d as u16;
        let e: u16 = self.registers.e as u16;
        e | (d << 8)
    }

    pub fn get_hl(&self) -> u16 {
        let h: u16 = self.registers.h as u16;
        let l: u16 = self.registers.l as u16;
        l | (h << 8)
    }

    pub fn get_r8(&mut self, reg: u8) -> u8 {
        match reg {
            0 => self.registers.b,
            1 => self.registers.c,
            2 => self.registers.d,
            3 => self.registers.e,
            4 => self.registers.h,
            5 => self.registers.l,
            6 => self.fetch_byte(self.get_hl()),
            7 => self.registers.a,
            _ => panic!("unknown register index `{}`", reg),
        }
    }

    pub fn get_r16_sp(&self, reg: u8) -> u16 {
        match reg {
            0 => self.get_bc(),
            1 => self.get_de(),
            2 => self.get_hl(),
            3 => self.registers.sp,
            _ => panic!("unknown register index `{}`", reg),
        }
    }

    pub fn get_r16_af(&self, reg: u8) -> u16 {
        match reg {
            0 => self.get_bc(),
            1 => self.get_de(),
            2 => self.get_hl(),
            3 => self.get_af(),
            _ => panic!("unknown register index `{}`", reg),
        }
    }

    pub fn get_flag(&self, flag: u8) -> bool {
        match flag {
            0 => !self.flags.z, // nz
            1 => self.flags.z,  // z
            2 => !self.flags.c, // nc
            3 => self.flags.c,  // c
            _ => panic!("unknown flag index `{}`", flag),
        }
    }

    pub fn set_r8(&mut self, reg: u8, value: u8) {
        match reg {
            0 => self.registers.b = value,
            1 => self.registers.c = value,
            2 => self.registers.d = value,
            3 => self.registers.e = value,
            4 => self.registers.h = value,
            5 => self.registers.l = value,
            6 => self.store_byte(self.get_hl(), value),
            7 => self.registers.a = value,
            _ => panic!("unknown register index `{}`", reg),
        }
    }

    pub fn set_r16_sp(&mut self, reg: u8, value: u16) {
        match reg {
            0 => self.set_bc(value),
            1 => self.set_de(value),
            2 => self.set_hl(value),
            3 => self.registers.sp = value,
            _ => panic!("unknown register index `{}`", reg),
        }
    }

    pub fn set_r16_af(&mut self, reg: u8, value: u16) {
        match reg {
            0 => self.set_bc(value),
            1 => self.set_de(value),
            2 => self.set_hl(value),
            3 => self.set_af(value),
            _ => panic!("unknown register index `{}`", reg),
        }
    }

    pub fn set_af(&mut self, value: u16) {
        self.registers.a = (value >> 8) as u8;
        self.flags = (value as u8).into();
    }

    pub fn set_bc(&mut self, value: u16) {
        self.registers.b = (value >> 8) as u8;
        self.registers.c = value as u8;
    }

    pub fn set_de(&mut self, value: u16) {
        self.registers.d = (value >> 8) as u8;
        self.registers.e = value as u8;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.registers.h = (value >> 8) as u8;
        self.registers.l = value as u8;
    }

    pub fn fetch_byte(&mut self, address: u16) -> u8 {
        self.delay(1);
        self.bus.fetch_byte(address)
    }

    pub fn store_byte(&mut self, address: u16, value: u8) {
        self.delay(1);
        self.bus.store_byte(address, value);
    }

    pub fn store_word(&mut self, address: u16, value: u16) {
        self.store_byte(address, value as u8);
        self.store_byte(address.wrapping_add(1), (value >> 8) as u8);
    }

    pub fn push_byte(&mut self, value: u8) {
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.store_byte(self.registers.sp, value);
    }

    pub fn push_word(&mut self, value: u16) {
        self.push_byte((value >> 8) as u8);
        self.push_byte(value as u8);
    }

    pub fn pop_byte(&mut self) -> u8 {
        let value = self.fetch_byte(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        value
    }

    pub fn pop_word(&mut self) -> u16 {
        let low = self.pop_byte() as u16;
        let high = self.pop_byte() as u16;

        (high << 8) | low
    }

    pub fn execute_next(&mut self) {
        // fetch opcode at program counter
        let opcode = self.fetch_byte(self.registers.pc);

        // self.trace();

        // increment program counter
        self.registers.pc = self.registers.pc.wrapping_add(1);

        // select opcode depending on whether it has 0xcb prefix
        if opcode != 0xcb {
            // hand off to the opcode executor
            opcodes::execute(self, opcode);
        } else {
            // fetch opcode at program counter
            let bitwise_opcode = self.fetch_byte(self.registers.pc);

            // increment program counter again
            self.registers.pc = self.registers.pc.wrapping_add(1);

            // hand off to the prefix cb opcode executor
            opcodes::execute_cb(self, bitwise_opcode);
        }
    }

    fn advance(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.bus.tick();
        }

        self.cycles += cycles;
    }

    fn delay(&mut self, machine_cycles: u8) {
        self.advance(machine_cycles * 4);
    }

    /// handles interrupt, returns whether it was actually handled
    fn handle_interrupt(&mut self) -> bool {
        if self.bus.it_enable.vblank && self.bus.ppu.vblank_int {
            self.bus.ppu.vblank_int = false;
            self.interrupt(interrupt::irq_vector::VBLANK);
            true
        } else if self.bus.it_enable.lcdc && self.bus.ppu.lcd_stat_int {
            self.bus.ppu.lcd_stat_int = false;
            self.interrupt(interrupt::irq_vector::LCDC);
            true
        } else if self.bus.it_enable.timer && self.bus.timer.interrupt {
            self.bus.timer.interrupt = false;
            self.interrupt(interrupt::irq_vector::TIMER);
            true
        } else {
            // interrupt was not handled
            false
        }
    }

    /// returns whether the interrupt can be handled
    fn check_interrupt(&mut self) -> bool {
        (self.bus.it_enable.vblank && self.bus.ppu.vblank_int)
            || (self.bus.it_enable.lcdc && self.bus.ppu.lcd_stat_int)
            || (self.bus.it_enable.timer && self.bus.timer.interrupt)
    }

    fn interrupt(&mut self, address: u16) {
        self.halted = false;
        self.it_master_enable = false;
        self.it_master_enable_next = false;

        self.push_word(self.registers.pc);
        self.delay(6);
        self.registers.pc = address;
    }

    pub fn fault(&mut self, message: &str) {
        self.registers.pc -= 1; // rewind program counter
        error!("cpu fault encountered");
        self.trace();
        self.dump();
        panic!("critical fault occurred, core dumped: `{}`", message);
    }

    fn trace(&self) {
        info!(
            "0x{:04x} > {} 0x{:02x}",
            self.registers.pc,
            disasm::disassmble(
                self,
                self.registers.pc,
                self.bus.fetch_byte(self.registers.pc)
            ),
            self.bus.fetch_byte(self.registers.pc)
        );
    }

    fn dump(&self) {
        info!(
            "cpu::reg::af = {:04x} | cpu::int::ime    = {}",
            self.get_af(),
            self.it_master_enable
        );
        info!(
            "cpu::reg::bc = {:04x} | cpu::int::vblank = {}",
            self.get_bc(),
            self.bus.it_enable.vblank
        );
        info!(
            "cpu::reg::de = {:04x} | cpu::int::lcdc   = {}",
            self.get_de(),
            self.bus.it_enable.lcdc
        );
        info!(
            "cpu::reg::hl = {:04x} | cpu::int::timer  = {}",
            self.get_hl(),
            self.bus.it_enable.timer
        );
        info!(
            "cpu::reg::pc = {:04x} | cpu::int::serial = {}",
            self.registers.pc, self.bus.it_enable.serial
        );
        info!(
            "cpu::reg::sp = {:04x} | cpu::int::joypad = {}",
            self.registers.sp, self.bus.it_enable.joypad
        );
    }
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        Self {
            z: (value & (1 << 7)) != 0,
            n: (value & (1 << 6)) != 0,
            h: (value & (1 << 5)) != 0,
            c: (value & (1 << 4)) != 0,
        }
    }
}

impl From<Flags> for u8 {
    fn from(flags: Flags) -> Self {
        ((flags.z as u8) << 7)
            | ((flags.n as u8) << 6)
            | ((flags.h as u8) << 5)
            | ((flags.c as u8) << 4)
    }
}
