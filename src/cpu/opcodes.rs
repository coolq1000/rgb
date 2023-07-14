use super::Cpu;

use bitmatch::bitmatch;

#[bitmatch]
pub fn execute(cpu: &mut Cpu, opcode: u8) {
    #[bitmatch]
    match opcode {
        "0000_0000" => misc::nop(),
        "00xx_0001" => lsm::ld_r16_u16(cpu, x),
        "00xx_1001" => alu::add_hl_r16(cpu, x),
        "00xx_0011" => alu::inc_r16(cpu, x),
        "00xx_1011" => alu::dec_r16(cpu, x),
        "00xx_x100" => alu::inc_r8(cpu, x),
        "00xx_x101" => alu::dec_r8(cpu, x),
        "00xx_x110" => lsm::ld_r8_u8(cpu, x),
        "0000_0111" => rsb::rlca(cpu),
        "0000_1111" => rsb::rrca(cpu),
        "0001_0111" => rsb::rla(cpu),
        "0001_1111" => rsb::rra(cpu),
        "000x_0010" => lsm::ld_mr16_a(cpu, x),
        "000x_1010" => lsm::ld_a_mr16(cpu, x),
        "0001_1000" => ctrl::jr_i8(cpu),
        "001x_x000" => ctrl::jr_f_i8(cpu, x),
        "0010_0010" => lsm::ldi_mhl_a(cpu),
        "0010_1010" => lsm::ldi_a_mhl(cpu),
        "0010_0111" => alu::daa(cpu),
        "0010_1111" => alu::cpl(cpu),
        "0011_0010" => lsm::ldd_mhl_a(cpu),
        "0011_1010" => lsm::ldd_a_mhl(cpu),
        "0000_1000" => lsm::ld_mu16_sp(cpu),
        "01xx_xyyy" => lsm::ld_r8_r8(cpu, x, y),
        "1000_0xxx" => alu::add_a_r8(cpu, x),
        "1000_1xxx" => alu::adc_a_r8(cpu, x),
        "1001_0xxx" => alu::sub_a_r8(cpu, x),
        "1001_1xxx" => alu::sbc_a_r8(cpu, x),
        "1010_0xxx" => alu::and_a_r8(cpu, x),
        "1010_1xxx" => alu::xor_a_r8(cpu, x),
        "1011_0xxx" => alu::or_a_r8(cpu, x),
        "1011_1xxx" => alu::cp_a_r8(cpu, x),
        "110x_x000" => ctrl::ret_f(cpu, x),
        "110x_x010" => ctrl::jp_a16_f(cpu, x),
        "1100_0011" => ctrl::jp_a16(cpu),
        "11xx_0001" => lsm::pop_r16(cpu, x),
        "1100_1001" => ctrl::ret(cpu),
        "11xx_0101" => lsm::push_r16(cpu, x),
        "1100_0110" => alu::add_a_u8(cpu),
        "1100_1110" => alu::adc_a_u8(cpu),
        "1101_0110" => alu::sub_a_u8(cpu),
        "1101_1110" => alu::sbc_a_u8(cpu),
        "1110_0110" => alu::and_a_u8(cpu),
        "1110_1110" => alu::xor_a_u8(cpu),
        "1111_0110" => alu::or_a_u8(cpu),
        "1111_1110" => alu::cp_a_u8(cpu),
        "1101_1001" => ctrl::reti(cpu),
        "1100_1101" => ctrl::call_u16(cpu),
        "11xx_x111" => ctrl::rst_u8(cpu, x),
        "1110_1001" => ctrl::jp_hl(cpu),
        "1110_0000" => lsm::ldh_mu8_a(cpu),
        "1110_0010" => lsm::ldh_mc_a(cpu),
        "1111_0010" => lsm::ldh_a_mc(cpu),
        "1110_1010" => lsm::ld_mu16_a(cpu),
        "1111_0000" => lsm::ldh_a_mu8(cpu),
        "1111_0011" => misc::di(cpu),
        "1111_1011" => misc::ei(cpu),
        "1111_1010" => lsm::ld_a_mu16(cpu),
        _ => cpu.fault("unimplemented instruction"),
    }
}

#[bitmatch]
pub fn execute_cb(cpu: &mut Cpu, opcode: u8) {
    #[bitmatch]
    match opcode {
        "0001_0xxx" => cb::rsb::rl_r8(cpu, x),
        "0010_0xxx" => cb::rsb::sla_r8(cpu, x),
        "0011_1xxx" => cb::rsb::srl_r8(cpu, x),
        "0011_0xxx" => cb::rsb::swap_r8(cpu, x),
        "01xx_xyyy" => cb::rsb::bit_u8_r8(cpu, x, y),
        "10xx_xyyy" => cb::rsb::res_u8_r8(cpu, x, y),
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
mod misc {
    use crate::cpu::Cpu;

    pub fn nop() {}

    pub fn di(cpu: &mut Cpu) {
        cpu.it_master_enable = false;
    }

    pub fn ei(cpu: &mut Cpu) {
        cpu.it_master_enable_next = true;
    }
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
        cpu.store_byte(0xff00 | imm8 as u16, cpu.registers.a);
    }

    pub fn ldh_mc_a(cpu: &mut Cpu) {
        cpu.store_byte(0xff00 | cpu.registers.c as u16, cpu.registers.a);
    }

    pub fn ld_mu16_a(cpu: &mut Cpu) {
        let imm16 = super::next_word(cpu);
        cpu.store_byte(imm16, cpu.registers.a);
    }

    pub fn ldh_a_mc(cpu: &mut Cpu) {
        cpu.registers.a = cpu.fetch_byte(0xff00 | cpu.registers.c as u16);
    }

    pub fn ldh_a_mu8(cpu: &mut Cpu) {
        let imm8 = super::next_byte(cpu);
        cpu.registers.a = cpu.fetch_byte(0xff00 | imm8 as u16);
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

    pub fn add_a_r8(cpu: &mut Cpu, reg: u8) {
        let r8 = cpu.get_r8(reg);
        add_a_v8(cpu, r8)
    }

    pub fn add_a_u8(cpu: &mut Cpu) {
        let imm8 = super::next_byte(cpu);
        add_a_v8(cpu, imm8)
    }

    pub fn add_a_v8(cpu: &mut Cpu, val: u8) {
        let x = cpu.registers.a as u32;
        let y = val as u32;

        let result = x.wrapping_add(y);
        let result_byte = result as u8;

        cpu.flags.z = result_byte == 0;
        cpu.flags.n = false;
        cpu.flags.h = (x ^ y ^ result) & 0x10 != 0;
        cpu.flags.c = result & 0x100 != 0;

        cpu.registers.a = result_byte;
    }

    pub fn adc_a_r8(cpu: &mut Cpu, reg: u8) {
        let r8 = cpu.get_r8(reg);
        adc_a_v8(cpu, r8)
    }

    pub fn adc_a_u8(cpu: &mut Cpu) {
        let imm8 = super::next_byte(cpu);
        adc_a_v8(cpu, imm8)
    }

    pub fn adc_a_v8(cpu: &mut Cpu, val: u8) {
        let x = cpu.registers.a as u32;
        let y = val as u32;
        let c = cpu.flags.c as u32;

        let result = x.wrapping_add(y).wrapping_add(c);
        let result_byte = result as u8;

        cpu.flags.z = result_byte == 0;
        cpu.flags.n = false;
        cpu.flags.h = (x ^ y ^ result) & 0x10 != 0;
        cpu.flags.c = result & 0x100 != 0;

        cpu.registers.a = result_byte;
    }

    pub fn sub_a_r8(cpu: &mut Cpu, reg: u8) {
        let r8 = cpu.get_r8(reg);
        sub_a_v8(cpu, r8)
    }

    pub fn sub_a_u8(cpu: &mut Cpu) {
        let imm8 = super::next_byte(cpu);
        sub_a_v8(cpu, imm8)
    }

    pub fn sub_a_v8(cpu: &mut Cpu, val: u8) {
        let x = cpu.registers.a as u32;
        let y = val as u32;

        let result = x.wrapping_sub(y);
        let result_byte = result as u8;

        cpu.flags.z = result_byte == 0;
        cpu.flags.n = true;
        cpu.flags.h = (x ^ y ^ result) & 0x10 != 0;
        cpu.flags.c = result & 0x100 != 0;

        cpu.registers.a = result_byte;
    }

    pub fn sbc_a_r8(cpu: &mut Cpu, reg: u8) {
        let r8 = cpu.get_r8(reg);
        sbc_a_v8(cpu, r8)
    }

    pub fn sbc_a_u8(cpu: &mut Cpu) {
        let imm8 = super::next_byte(cpu);
        sbc_a_v8(cpu, imm8)
    }

    pub fn sbc_a_v8(cpu: &mut Cpu, val: u8) {
        let x = cpu.registers.a as u32;
        let y = val as u32;
        let c = cpu.flags.c as u32;

        let result = x.wrapping_sub(y).wrapping_sub(c);
        let result_byte = result as u8;

        cpu.flags.z = result_byte == 0;
        cpu.flags.n = false;
        cpu.flags.h = (x ^ y ^ result) & 0x10 != 0;
        cpu.flags.c = result & 0x100 != 0;

        cpu.registers.a = result_byte;
    }

    pub fn and_a_r8(cpu: &mut Cpu, reg: u8) {
        let r8 = cpu.get_r8(reg);
        and_a_v8(cpu, r8)
    }

    pub fn and_a_u8(cpu: &mut Cpu) {
        let imm8 = super::next_byte(cpu);
        and_a_v8(cpu, imm8)
    }

    pub fn and_a_v8(cpu: &mut Cpu, val: u8) {
        cpu.registers.a &= val;

        cpu.flags.z = cpu.registers.a == 0;
        cpu.flags.n = false;
        cpu.flags.h = true;
        cpu.flags.c = false;
    }

    pub fn xor_a_r8(cpu: &mut Cpu, reg: u8) {
        let r8 = cpu.get_r8(reg);
        xor_a_v8(cpu, r8)
    }

    pub fn xor_a_u8(cpu: &mut Cpu) {
        let imm8 = super::next_byte(cpu);
        xor_a_v8(cpu, imm8)
    }

    pub fn xor_a_v8(cpu: &mut Cpu, val: u8) {
        cpu.registers.a ^= val;

        cpu.flags.z = cpu.registers.a == 0;
        cpu.flags.n = false;
        cpu.flags.h = false;
        cpu.flags.c = false;
    }

    pub fn or_a_r8(cpu: &mut Cpu, reg: u8) {
        let r8 = cpu.get_r8(reg);
        or_a_v8(cpu, r8)
    }

    pub fn or_a_u8(cpu: &mut Cpu) {
        let imm8 = super::next_byte(cpu);
        or_a_v8(cpu, imm8)
    }

    pub fn or_a_v8(cpu: &mut Cpu, val: u8) {
        cpu.registers.a |= val;

        cpu.flags.z = cpu.registers.a == 0;
        cpu.flags.n = false;
        cpu.flags.h = false;
        cpu.flags.c = false;
    }

    pub fn cp_a_r8(cpu: &mut Cpu, reg: u8) {
        let r8 = cpu.get_r8(reg);
        cp_a_v8(cpu, r8)
    }

    pub fn cp_a_u8(cpu: &mut Cpu) {
        let imm8 = super::next_byte(cpu);
        cp_a_v8(cpu, imm8)
    }

    pub fn cp_a_v8(cpu: &mut Cpu, val: u8) {
        let x = cpu.registers.a as u32;
        let y = val as u32;

        let result = x.wrapping_sub(y);
        let result_byte = result as u8;

        cpu.flags.z = result_byte == 0;
        cpu.flags.n = true;
        cpu.flags.h = (x ^ y ^ result) & 0x10 != 0;
        cpu.flags.c = result & 0x100 != 0;
    }

    pub fn add_hl_r16(cpu: &mut Cpu, reg: u8) {
        let x = cpu.get_hl() as u32;
        let y = cpu.get_r16_sp(reg) as u32;

        let result = x.wrapping_add(y);
        cpu.set_hl(result as u16);

        cpu.flags.n = false;
        cpu.flags.h = (x ^ y ^ result) & 0x1000 != 0;
        cpu.flags.c = result & 0x10000 != 0;

        cpu.delay(1);
    }

    pub fn inc_r16(cpu: &mut Cpu, reg: u8) {
        let r16 = cpu.get_r16_sp(reg);
        cpu.set_r16_sp(reg, r16.wrapping_add(1));
        cpu.delay(1);
    }

    pub fn dec_r16(cpu: &mut Cpu, reg: u8) {
        let r16 = cpu.get_r16_sp(reg);
        cpu.set_r16_sp(reg, r16.wrapping_sub(1));
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

    pub fn daa(cpu: &mut Cpu) {
        let mut adjust = 0;

        if cpu.flags.h {
            adjust |= 0x6;
        }

        if cpu.flags.c {
            adjust |= 0x60;
        }

        let result = if cpu.flags.n {
            cpu.registers.a.wrapping_sub(adjust)
        } else {
            if cpu.registers.a & 0xf > 0x9 {
                adjust |= 0x6;
            }

            if cpu.registers.a > 0x99 {
                adjust |= 0x60;
            }

            cpu.registers.a.wrapping_add(adjust)
        };

        cpu.registers.a = result;

        cpu.flags.z = result == 0;
        cpu.flags.h = false;
        cpu.flags.c = adjust & 0x60 != 0;
    }

    pub fn cpl(cpu: &mut Cpu) {
        cpu.registers.a = !cpu.registers.a;

        cpu.flags.n = true;
        cpu.flags.h = true;
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
mod ctrl {
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

    pub fn jp_a16_f(cpu: &mut Cpu, flag: u8) {
        let addr = super::next_word(cpu);

        if cpu.get_flag(flag) {
            cpu.registers.pc = addr;
            cpu.delay(1);
        }
    }

    pub fn jp_a16(cpu: &mut Cpu) {
        let addr = super::next_word(cpu);
        cpu.registers.pc = addr;
        cpu.delay(1);
    }

    pub fn ret(cpu: &mut Cpu) {
        let address = cpu.pop_word();
        cpu.registers.pc = address;
        cpu.delay(1);
    }

    pub fn reti(cpu: &mut Cpu) {
        let address = cpu.pop_word();
        cpu.registers.pc = address;
        cpu.it_master_enable = true;
        cpu.it_master_enable_next = true;
        cpu.delay(1);
    }

    pub fn call_u16(cpu: &mut Cpu) {
        let imm16 = super::next_word(cpu);
        cpu.push_word(cpu.registers.pc);
        cpu.registers.pc = imm16;
        cpu.delay(1);
    }

    pub fn rst_u8(cpu: &mut Cpu, n: u8) {
        cpu.push_word(cpu.registers.pc);
        cpu.registers.pc = n as u16 * 8;
        cpu.delay(1);
    }

    pub fn jp_hl(cpu: &mut Cpu) {
        cpu.registers.pc = cpu.get_hl();
    }
}

/// prefix cb
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

        pub fn sla_r8(cpu: &mut Cpu, reg: u8) {
            let r8 = cpu.get_r8(reg);

            let result = r8 << 1;
            cpu.set_r8(reg, result);

            cpu.flags.z = result == 0;
            cpu.flags.n = false;
            cpu.flags.h = false;
            cpu.flags.c = r8 & 0x80 != 0;
        }

        pub fn srl_r8(cpu: &mut Cpu, reg: u8) {
            let r8 = cpu.get_r8(reg);

            let result = r8 >> 1;
            cpu.set_r8(reg, result);

            cpu.flags.z = result == 0;
            cpu.flags.n = false;
            cpu.flags.h = false;
            cpu.flags.c = r8 & 0x1 != 0;
        }

        pub fn swap_r8(cpu: &mut Cpu, reg: u8) {
            let r8 = cpu.get_r8(reg);
            cpu.set_r8(reg, (r8 << 4) | (r8 >> 4));

            cpu.flags.z = r8 == 0;
            cpu.flags.n = false;
            cpu.flags.h = false;
            cpu.flags.c = false;
        }

        pub fn bit_u8_r8(cpu: &mut Cpu, bit: u8, reg: u8) {
            cpu.flags.z = (cpu.get_r8(reg) & (1 << bit)) == 0;
            cpu.flags.n = false;
            cpu.flags.h = true;
        }

        pub fn res_u8_r8(cpu: &mut Cpu, bit: u8, reg: u8) {
            let reset = cpu.get_r8(reg) & !(1 << bit);
            cpu.set_r8(reg, reset);
        }
    }
}
