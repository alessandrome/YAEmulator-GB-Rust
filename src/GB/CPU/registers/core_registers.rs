use crate::GB::types::address::Address;
use std::fmt;
use crate::{mask_flag_enum_default_impl, default_enum_u8_bit_ops};
use super::macro_registers;
use crate::GB::debug_print;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum FlagBits {
    Z = 0b1000_0000,
    N = 0b0100_0000,
    H = 0b0010_0000,
    C = 0b0001_0000,
}

mask_flag_enum_default_impl!(FlagBits);

#[derive(Debug, Clone, Copy)]
pub struct Flags {
    z: bool,
    n: bool,
    h: bool,
    c: bool,
}

impl Flags {
    pub fn new(z: bool, n: bool, h: bool, c: bool) -> Self {
        Self { z, n, h, c }
    }

    #[inline]
    pub fn z(&self) -> bool {
        self.z
    }

    #[inline]
    pub fn n(&self) -> bool {
        self.n
    }

    #[inline]
    pub fn h(&self) -> bool {
        self.h
    }

    #[inline]
    pub fn c(&self) -> bool {
        self.c
    }

    #[inline]
    pub fn add_carry(lhs: u8, rhs: u8, carry: bool) -> bool {
        (lhs as u16 + rhs as u16 + carry as u16) > 0xff
    }

    #[inline]
    pub fn sub_carry(lhs: u8, rhs: u8, carry: bool) -> bool {
        (lhs as u16) < (rhs as u16 + carry as u16)
    }

    #[inline]
    pub fn add_half_carry(lhs: u8, rhs: u8, carry: bool) -> bool {
        ((lhs & 0x0F) + (rhs & 0x0F) + carry as u8) > 0x0F
    }

    #[inline]
    pub fn sub_half_carry(lhs: u8, rhs: u8, carry: bool) -> bool {
        (lhs & 0x0F) < ((rhs & 0x0F) + carry as u8)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Registers8Bit {
    A, B, C, D, E, F, H, L, W, Z
}

#[derive(Debug, Clone, Copy)]
pub enum Registers16Bit {
    AF, BC, DE, HL, WZ, SP, PC
}

#[derive(Debug, Clone, Copy)]
pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    z: u8,  // Internal use
    w: u8,  // Internal use
    sp: u16,
    pc: u16,
}

// trait GetSetRegisters {
//     get_set!(a);
// }

impl Registers {
    pub fn new() -> Registers {
        // Based on DMG version
        Registers {
            a: 0x01,
            b: 0,
            c: 0x13,
            d: 0,
            e: 0xD8,
            f: 0x0B,
            h: 0x01,
            l: 0x4D,
            z: 0,
            w: 0,
            sp: 0xFFFE,
            pc: 0x0100,
        }
    }

    macro_registers::get_set!(a, get_a, set_a, u8);
    macro_registers::get_set!(b, get_b, set_b, u8);
    macro_registers::get_set!(c, get_c, set_c, u8);
    macro_registers::get_set!(d, get_d, set_d, u8);
    macro_registers::get_set!(e, get_e, set_e, u8);
    macro_registers::get_set!(h, get_h, set_h, u8);
    macro_registers::get_set!(l, get_l, set_l, u8);
    macro_registers::get_set!(z, get_z, set_z, u8);
    macro_registers::get_set!(w, get_w, set_w, u8);
    macro_registers::get_set!(sp, get_sp, set_sp, u16);
    macro_registers::get_set!(pc, get_pc, set_pc, u16);

    #[inline]
    pub fn get_sp_lsb(&self) -> u8 {
        (self.sp & 0xF) as u8
    }

    #[inline]
    pub fn set_sp_lsb(&mut self, value: u8) {
        self.sp = (self.sp & 0xF0) | (value as u16);
    }

    #[inline]
    pub fn get_sp_msb(&self) -> u8 {
        (self.sp >> 8) as u8
    }

    #[inline]
    pub fn set_sp_msb(&mut self, value: u8) {
        self.sp = (self.sp & 0x0F) | ((value as u16) << 8);
    }

    #[inline]
    pub fn get_pc_lsb(&self) -> u8 {
        (self.pc & 0xF) as u8
    }

    #[inline]
    pub fn set_pc_lsb(&mut self, value: u8) {
        self.pc = (self.pc & 0xF0) | (value as u16);
    }

    #[inline]
    pub fn get_pc_msb(&self) -> u8 {
        (self.pc >> 0xF) as u8
    }

    #[inline]
    pub fn set_pc_msb(&mut self, value: u8) {
        self.pc = (self.pc & 0x0F) | ((value as u16) << 8);
    }

    #[inline]
    pub fn get_sp_as_address(&self) -> Address {
        Address(self.sp)
    }

    #[inline]
    pub fn get_pc_as_address(&self) -> Address {
        Address(self.pc)
    }

    pub fn get_and_inc_pc(&mut self) -> u16 {
        let ret_pc = self.pc;
        self.pc += 1;
        ret_pc
    }

    pub fn inc_pc(&mut self) -> u16 {
        self.pc += 1;
        self.pc
    }
    macro_registers::get_set_dual!(b, c, get_bc, get_bc_as_address, set_bc);
    macro_registers::get_set_dual!(d, e, get_de, get_de_as_address, set_de);
    macro_registers::get_set_dual!(h, l, get_hl, get_hl_as_address, set_hl);
    macro_registers::get_set_dual!(w, z, get_wz, get_wz_as_address, set_wz);  // Internal use - Immediate 16-bit Address

    pub fn get_f(&self) -> u8 {
        self.f
    }
    pub fn set_f(&mut self, val: u8) {
        self.f = val & 0xF0
    }

    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }
    pub fn get_af_as_address(&self) -> Address {
        Address(self.get_af())
    }

    pub fn set_af(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = (val & 0x00F0) as u8;
    }

    macro_registers::get_set_flag!(get_zero_flag, set_zero_flag, Z);
    macro_registers::get_set_flag!(get_negative_flag, set_negative_flag, N);
    macro_registers::get_set_flag!(get_half_carry_flag, set_half_carry_flag, H);
    macro_registers::get_set_flag!(get_carry_flag, set_carry_flag, C);
    pub fn get_flags(&self) -> Flags {
        Flags::new(
            (self.f & 0b10000000) != 0,
            (self.f & 0b01000000) != 0,
            (self.f & 0b00100000) != 0,
            (self.f & 0b00010000) != 0,
        )
    }

    pub fn set_flags(&mut self, flags: Flags) {
        self.set_zero_flag(flags.z);
        self.set_negative_flag(flags.n);
        self.set_half_carry_flag(flags.h);
        self.set_carry_flag(flags.c);
    }

    pub fn get_byte(&self, register: Registers8Bit) -> u8 {
        match register {
            Registers8Bit::A => self.get_a(),
            Registers8Bit::B => self.get_b(),
            Registers8Bit::C => self.get_c(),
            Registers8Bit::D => self.get_d(),
            Registers8Bit::E => self.get_e(),
            Registers8Bit::F => self.get_f(),
            Registers8Bit::H => self.get_h(),
            Registers8Bit::L => self.get_l(),
            Registers8Bit::W => self.get_w(),
            Registers8Bit::Z => self.get_z(),
        }
    }

    pub fn get_word(&self, register: Registers16Bit) -> u16 {
        match register {
            Registers16Bit::AF => self.get_af(),
            Registers16Bit::BC => self.get_bc(),
            Registers16Bit::DE => self.get_de(),
            Registers16Bit::HL => self.get_hl(),
            Registers16Bit::WZ => self.get_wz(),
            Registers16Bit::SP => self.get_sp(),
            Registers16Bit::PC => self.get_pc(),
        }
    }

    pub fn get_word_lsb(&self, register: Registers16Bit) -> u8 {
        match register {
            Registers16Bit::AF => self.get_f(),
            Registers16Bit::BC => self.get_c(),
            Registers16Bit::DE => self.get_e(),
            Registers16Bit::HL => self.get_l(),
            Registers16Bit::WZ => self.get_z(),
            Registers16Bit::SP => self.get_sp_lsb(),
            Registers16Bit::PC => self.get_pc_lsb(),
        }
    }

    pub fn get_word_msb(&self, register: Registers16Bit) -> u8 {
        match register {
            Registers16Bit::AF => self.get_a(),
            Registers16Bit::BC => self.get_b(),
            Registers16Bit::DE => self.get_d(),
            Registers16Bit::HL => self.get_h(),
            Registers16Bit::WZ => self.get_w(),
            Registers16Bit::SP => self.get_sp_msb(),
            Registers16Bit::PC => self.get_pc_msb(),
        }
    }

    pub fn set_word_lsb(&mut self, register: Registers16Bit, value: u8) {
        match register {
            Registers16Bit::AF => self.set_f(value),
            Registers16Bit::BC => self.set_c(value),
            Registers16Bit::DE => self.set_e(value),
            Registers16Bit::HL => self.set_l(value),
            Registers16Bit::WZ => self.set_z(value),
            Registers16Bit::SP => self.set_sp_lsb(value),
            Registers16Bit::PC => self.set_pc_lsb(value),
        }
    }

    pub fn set_word_msb(&mut self, register: Registers16Bit, value: u8) {
        match register {
            Registers16Bit::AF => self.set_a(value),
            Registers16Bit::BC => self.set_b(value),
            Registers16Bit::DE => self.set_d(value),
            Registers16Bit::HL => self.set_h(value),
            Registers16Bit::WZ => self.set_w(value),
            Registers16Bit::SP => self.set_sp_msb(value),
            Registers16Bit::PC => self.set_pc_msb(value),
        }
    }

    pub fn set_byte(&mut self, register: Registers8Bit, byte: u8) {
        match register {
            Registers8Bit::A => self.set_a(byte),
            Registers8Bit::B => self.set_b(byte),
            Registers8Bit::C => self.set_c(byte),
            Registers8Bit::D => self.set_d(byte),
            Registers8Bit::E => self.set_e(byte),
            Registers8Bit::F => self.set_f(byte),
            Registers8Bit::H => self.set_h(byte),
            Registers8Bit::L => self.set_l(byte),
            Registers8Bit::W => self.set_w(byte),
            Registers8Bit::Z => self.set_z(byte),
        }
    }

    pub fn set_word(&mut self, register: Registers16Bit, data: u16) {
        match register {
            Registers16Bit::AF => self.set_af(data),
            Registers16Bit::BC => self.set_bc(data),
            Registers16Bit::DE => self.set_de(data),
            Registers16Bit::HL => self.set_hl(data),
            Registers16Bit::WZ => self.set_wz(data),
            Registers16Bit::SP => self.set_sp(data),
            Registers16Bit::PC => self.set_pc(data),
        }
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Registers {{ A: {:#04x}, B: {:#04x}, C: {:#04x}, D: {:#04x}, E: {:#04x}, F: {:#04x}, H: {:#04x}, L: {:#04x}, PC: {:#06x}, SP: {:#06x} }}",
            self.a, self.b, self.c, self.d, self.e, self.f, self.h, self.l, self.pc, self.sp
        )
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}
