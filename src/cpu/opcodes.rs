use super::Cpu;

use bitmatch::bitmatch;

#[bitmatch]
pub fn execute(cpu: &mut Cpu, opcode: u8) {
    #[bitmatch]
    match opcode {
        "0000_0000" => control_misc::nop(),
        "00xx_0001" => lsm::ld_r16_u16(cpu, x),
        "00xx_?011" => alu::inc_r16(cpu, x),
        "00xx_x100" => alu::inc_r8(cpu, x),
        "00xx_x101" => alu::dec_r8(cpu, x),
        "00xx_x110" => lsm::ld_r8_u8(cpu, x),
        "0000_0111" => rsb::rlca(cpu),
        "0000_1111" => rsb::rrca(cpu),
        "0001_0111" => rsb::rla(cpu),
        "0001_1111" => rsb::rra(cpu),
        "000x_0010" => lsm::ld_mr16_a(cpu, x),
        "000x_1010" => lsm::ld_a_mr16(cpu, x),
        "0001_1000" => control_br::jr_i8(cpu),
        "001x_x000" => control_br::jr_f_i8(cpu, x),
        "0010_0010" => lsm::ldi_mhl_a(cpu),
        "0010_1010" => lsm::ldi_a_mhl(cpu),
        "0011_0010" => lsm::ldd_mhl_a(cpu),
        "0011_1010" => lsm::ldd_a_mhl(cpu),
        "0000_1000" => lsm::ld_mu16_sp(cpu),
        "01xx_xyyy" => lsm::ld_r8_r8(cpu, x, y),
        "1010_1xxx" => {
            let r8 = cpu.get_r8(x);
            alu::xor_a_val(cpu, r8)
        }
        "110x_x000" => control_br::ret_f(cpu, x),
        "11xx_0001" => lsm::pop_r16(cpu, x),
        "1100_1001" => control_br::ret(cpu),
        "11xx_0101" => lsm::push_r16(cpu, x),
        "1100_1101" => control_br::call_u16(cpu),
        "1110_0000" => lsm::ldh_mu8_a(cpu),
        "1110_0010" => lsm::ldh_mc_a(cpu),
        "1111_0010" => lsm::ldh_a_mc(cpu),
        "1110_1010" => lsm::ld_mu16_a(cpu),
        "1111_1010" => lsm::ld_a_mu16(cpu),
        _ => cpu.fault("unimplemented instruction"),
    }
}

#[bitmatch]
pub fn execute_cb(cpu: &mut Cpu, opcode: u8) {
    #[bitmatch]
    match opcode {
        "0001_0xxx" => cb::rsb::rl_r8(cpu, x),
        "01xx_xyyy" => cb::rsb::bit_u8_r8(cpu, x, y),
        _ => {
            cpu.registers.pc = cpu.registers.pc.wrapping_sub(1); // rewind 0xcb byte from program counter
            cpu.fault("unimplemented prefix cb instruction");
        }
    }
}

fn next_byte(cpu: &mut Cpu) -> u8 {
    // fetch byte at program counter
    let fetched = cpu.fetch_byte(cpu.registers.pc);

    // increment program counter
    cpu.registers.pc = cpu.registers.pc.wrapping_add(1);

    fetched
}

fn next_word(cpu: &mut Cpu) -> u16 {
    next_byte(cpu) as u16 | ((next_byte(cpu) as u16) << 8)
}

/// cpu control miscellaneous instructions
mod control_misc {
    pub fn nop() {}
}

/// load, store & move instructions
mod lsm {
    use crate::cpu::Cpu;

    pub fn ld_r16_u16(cpu: &mut Cpu, dest: u8) {
        let imm16 = super::next_word(cpu);
        cpu.set_r16_sp(dest, imm16);
    }

    pub fn ld_r8_u8(cpu: &mut Cpu, reg: u8) {
        let imm8 = super::next_byte(cpu);
        cpu.set_r8(reg, imm8);
    }

    pub fn ldi_mhl_a(cpu: &mut Cpu) {
        cpu.store_byte(cpu.get_hl(), cpu.registers.a);
        cpu.set_hl(cpu.get_hl().wrapping_add(1));
    }

    pub fn ldi_a_mhl(cpu: &mut Cpu) {
        cpu.registers.a = cpu.fetch_byte(cpu.get_hl());
        cpu.set_hl(cpu.get_hl().wrapping_add(1));
    }

    pub fn ldd_mhl_a(cpu: &mut Cpu) {
        cpu.store_byte(cpu.get_hl(), cpu.registers.a);
        cpu.set_hl(cpu.get_hl().wrapping_sub(1));
    }

    pub fn ldd_a_mhl(cpu: &mut Cpu) {
        cpu.registers.a = cpu.fetch_byte(cpu.get_hl());
        cpu.set_hl(cpu.get_hl().wrapping_sub(1));
    }

    pub fn ld_mr16_a(cpu: &mut Cpu, dest: u8) {
        cpu.store_byte(cpu.get_r16_sp(dest), cpu.registers.a);
    }

    pub fn ld_a_mr16(cpu: &mut Cpu, dest: u8) {
        cpu.registers.a = cpu.fetch_byte(cpu.get_r16_sp(dest));
    }

    pub fn ld_mu16_sp(cpu: &mut Cpu) {
        let imm16 = super::next_word(cpu);
        cpu.store_word(imm16, cpu.registers.sp);
    }

    pub fn ld_r8_r8(cpu: &mut Cpu, dst: u8, src: u8) {
        let r8 = cpu.get_r8(src);
        cpu.set_r8(dst, r8);
    }

    pub fn ldh_mu8_a(cpu: &mut Cpu) {
        let imm8 = super::next_byte(cpu);
        cpu.store_byte(0xff00 + imm8 as u16, cpu.registers.a);
    }

    pub fn ldh_mc_a(cpu: &mut Cpu) {
        cpu.store_byte(0xff00 + cpu.registers.c as u16, cpu.registers.a);
    }

    pub fn ld_mu16_a(cpu: &mut Cpu) {
        let imm16 = super::next_word(cpu);
        cpu.store_byte(imm16, cpu.registers.a);
    }

    pub fn ldh_a_mc(cpu: &mut Cpu) {
        cpu.registers.a = cpu.fetch_byte(0xff00 + cpu.registers.c as u16);
    }

    pub fn ld_a_mu16(cpu: &mut Cpu) {
        let imm16 = super::next_word(cpu);
        cpu.registers.a = cpu.fetch_byte(imm16);
    }

    pub fn pop_r16(cpu: &mut Cpu, reg: u8) {
        let popped = cpu.pop_word();
        cpu.set_r16_af(reg, popped);
    }

    pub fn push_r16(cpu: &mut Cpu, reg: u8) {
        cpu.push_word(cpu.get_r16_af(reg));
        cpu.delay(1);
    }
}

/// arithmetic logic instructions
mod alu {
    use crate::cpu::Cpu;

    pub fn inc_r16(cpu: &mut Cpu, reg: u8) {
        let r16 = cpu.get_r16_sp(reg);
        cpu.set_r16_sp(reg, r16.wrapping_add(1));
        cpu.delay(1);
    }

    pub fn inc_r8(cpu: &mut Cpu, reg: u8) {
        let r8 = cpu.get_r8(reg);
        let added = r8.wrapping_add(1);
        cpu.set_r8(reg, added);

        cpu.flags.z = added == 0;
        cpu.flags.n = false;
        cpu.flags.h = r8 & 0xf == 0xf;
    }

    pub fn dec_r8(cpu: &mut Cpu, reg: u8) {
        let r8 = cpu.get_r8(reg);
        let subbed = r8.wrapping_sub(1);
        cpu.set_r8(reg, subbed);

        cpu.flags.z = subbed == 0;
        cpu.flags.n = true;
        cpu.flags.h = r8 & 0xf == 0;
    }

    pub fn add_a_val(cpu: &mut Cpu, val: u8) {
        let result: u32 = (cpu.registers.a as u32).wrapping_add(val as u32);
        cpu.registers.a = result as u8;

        cpu.flags.z = cpu.registers.a == 0;
        cpu.flags.n = false;
        cpu.flags.h = ((cpu.registers.a & 0xf) + (val & 0xf)) > 0xf;
        cpu.flags.c = (result & 0x100) != 0;
    }

    pub fn adc_a_val(cpu: &mut Cpu, val: u8) {
        let result: u32 = (cpu.registers.a as u32)
            .wrapping_add(val as u32)
            .wrapping_add(cpu.flags.c as u32);
        cpu.registers.a = result as u8;

        cpu.flags.z = cpu.registers.a == 0;
        cpu.flags.n = false;
        cpu.flags.h = ((cpu.registers.a & 0xF)
            .wrapping_add(val & 0xF)
            .wrapping_add(cpu.flags.c as u8))
            > 0xF;
        cpu.flags.c = result > 0xff;
    }

    pub fn xor_a_val(cpu: &mut Cpu, val: u8) {
        cpu.registers.a ^= val;
        cpu.flags.z = cpu.registers.a == 0;
        cpu.flags.n = false;
        cpu.flags.h = false;
        cpu.flags.c = false;
    }

    pub fn xor_a_r8(cpu: &mut Cpu, reg: u8) {
        let r8 = cpu.get_r8(reg);
        xor_a_val(cpu, r8)
    }

    pub fn xor_a_u8(cpu: &mut Cpu) {
        let imm8 = super::next_byte(cpu);
        xor_a_val(cpu, imm8)
    }
}

/// rotate, shift & bitwise instructions
mod rsb {
    use crate::cpu::Cpu;

    pub fn rlca(cpu: &mut Cpu) {
        let carry = cpu.registers.a >> 7;
        cpu.registers.a = (cpu.registers.a << 1) | carry;
        cpu.flags.z = false;
        cpu.flags.n = false;
        cpu.flags.h = false;
        cpu.flags.c = carry == 1;
    }

    pub fn rrca(cpu: &mut Cpu) {
        let carry = cpu.registers.a & 1;
        cpu.registers.a = (cpu.registers.a >> 1) | (carry << 7);
        cpu.flags.z = false;
        cpu.flags.n = false;
        cpu.flags.h = false;
        cpu.flags.c = carry == 1;
    }

    pub fn rla(cpu: &mut Cpu) {
        let carry = cpu.registers.a >> 7;
        cpu.registers.a = (cpu.registers.a << 1) | (cpu.flags.c as u8);
        cpu.flags.z = false;
        cpu.flags.n = false;
        cpu.flags.h = false;
        cpu.flags.c = carry == 1;
    }

    pub fn rra(cpu: &mut Cpu) {
        let carry = cpu.registers.a & 1;
        cpu.registers.a = (cpu.registers.a >> 1) | ((cpu.flags.c as u8) << 7);
        cpu.flags.z = false;
        cpu.flags.n = false;
        cpu.flags.h = false;
        cpu.flags.c = carry == 1;
    }
}

/// control branch instructions
mod control_br {
    use crate::cpu::Cpu;

    pub fn jr_i8(cpu: &mut Cpu) {
        let offset = (super::next_byte(cpu) as i8) as i16;

        cpu.registers.pc = (cpu.registers.pc as i16).wrapping_add(offset) as u16;
        cpu.delay(1);
    }

    pub fn jr_f_i8(cpu: &mut Cpu, flag: u8) {
        let offset = (super::next_byte(cpu) as i8) as i16;

        if cpu.get_flag(flag) {
            cpu.registers.pc = (cpu.registers.pc as i16).wrapping_add(offset) as u16;
            cpu.delay(1);
        }
    }

    pub fn ret_f(cpu: &mut Cpu, flag: u8) {
        if cpu.get_flag(flag) {
            let addr = cpu.pop_word();
            cpu.registers.pc = addr;
            cpu.delay(1);
        }

        cpu.delay(1);
    }

    pub fn ret(cpu: &mut Cpu) {
        let address = cpu.pop_word();
        cpu.registers.pc = address;
        cpu.delay(1);
    }

    pub fn call_u16(cpu: &mut Cpu) {
        let imm16 = super::next_word(cpu);
        cpu.push_word(cpu.registers.pc);
        cpu.registers.pc = imm16;
        cpu.delay(1);
    }
}

mod cb {
    pub mod rsb {
        use crate::cpu::Cpu;

        pub fn rl_r8(cpu: &mut Cpu, reg: u8) {
            let r8 = cpu.get_r8(reg);
            let r = (r8 << 1) | cpu.flags.c as u8;
            cpu.set_r8(reg, r);

            cpu.flags.z = r == 0;
            cpu.flags.n = false;
            cpu.flags.h = false;
            cpu.flags.c = (r8 & 0x80) != 0;
        }

        pub fn bit_u8_r8(cpu: &mut Cpu, bit: u8, reg: u8) {
            cpu.flags.z = (cpu.get_r8(reg) & (1 << bit)) == 0;
            cpu.flags.n = false;
            cpu.flags.h = true;
        }
    }
}
