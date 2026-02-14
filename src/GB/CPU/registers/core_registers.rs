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
    pub fn get_and_inc_pc(&mut self) -> u16 {
        let ret_pc = self.pc;
        self.pc += 1;
        ret_pc
    }

    pub fn inc_pc(&mut self) -> u16 {
        self.pc += 1;
        self.pc
    }
    macro_registers::get_set_dual!(b, c, get_bc, set_bc);
    macro_registers::get_set_dual!(d, e, get_de, set_de);
    macro_registers::get_set_dual!(h, l, get_hl, set_hl);
    macro_registers::get_set_dual!(z, w, get_zw, set_zw);  // Internal use - Immediate 16-bit Address

    pub fn get_f(&self) -> u8 {
        self.f
    }
    pub fn set_f(&mut self, val: u8) {
        self.f = val & 0xF0
    }

    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
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
