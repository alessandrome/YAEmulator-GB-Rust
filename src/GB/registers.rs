use std::fmt;
use crate::{GB, mask_flag_enum_default_impl, default_enum_u8_bit_ops};
use crate::GB::debug_print;
use crate::GB::memory::{WRAM_ADDRESS, WRAM_SIZE};

macro_rules! get_set {
    ($reg:ident, $get_name:ident, $set_name:ident, $size:ty) => {
        pub fn $get_name(&self) -> $size {
            self.$reg
        }

        pub fn $set_name(&mut self, val: $size) {
            self.$reg = val;
        }
    };
}

macro_rules! get_set_dual {
    ($reg1:ident, $reg2:ident, $get_name:ident, $set_name:ident) => {
        pub fn $get_name(&self) -> u16 {
            (self.$reg1 as u16) << 8 | self.$reg2 as u16
        }

        pub fn $set_name(&mut self, val: u16) {
            self.$reg1 = (val >> 8) as u8;
            self.$reg2 = val as u8;
        }
    };
}

macro_rules! get_set_flag {
    ($get_name:ident, $set_name:ident, $flag:ident) => {
        pub fn $get_name(&self) -> bool {
            (FlagBits::$flag & self.f) != 0
        }

        pub fn $set_name(&mut self, on: bool) {
            if on {
                self.f |= FlagBits::$flag;
            } else {
                self.f &= FlagBits::$flag ^ 0xFF;
            }
        }
    };
}

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
    sp: u16,
    pc: u16,
}

// trait GetSetRegisters {
//     get_set!(a);
// }

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            sp: (WRAM_ADDRESS + WRAM_SIZE - 1) as u16,
            pc: 0,
        }
    }

    get_set!(a, get_a, set_a, u8);
    get_set!(b, get_b, set_b, u8);
    get_set!(c, get_c, set_c, u8);
    get_set!(d, get_d, set_d, u8);
    get_set!(e, get_e, set_e, u8);
    get_set!(h, get_h, set_h, u8);
    get_set!(l, get_l, set_l, u8);
    get_set!(sp, get_sp, set_sp, u16);
    get_set!(pc, get_pc, set_pc, u16);
    pub fn get_and_inc_pc(&mut self) -> u16 {
        let ret_pc = self.pc;
        self.pc += 1;
        ret_pc
    }

    pub fn inc_pc(&mut self) -> u16 {
        self.pc += 1;
        self.pc
    }
    get_set_dual!(b, c, get_bc, set_bc);
    get_set_dual!(d, e, get_de, set_de);
    get_set_dual!(h, l, get_hl, set_hl);

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

    get_set_flag!(get_zero_flag, set_zero_flag, Z);
    get_set_flag!(get_negative_flag, set_negative_flag, N);
    get_set_flag!(get_half_carry_flag, set_half_carry_flag, H);
    get_set_flag!(get_carry_flag, set_carry_flag, C);
    pub fn get_flags(&self) -> Flags {
        Flags::new(
            (self.f & 0b10000000) != 0,
            (self.f & 0b01000000) != 0,
            (self.f & 0b00100000) != 0,
            (self.f & 0b00010000) != 0,
        )
    }

    // pub fn set_zero_flag(&mut self, on: bool) {
    //     if on {
    //         self.f |= FlagBits::Z;
    //     } else {
    //         self.f &= FlagBits::Z ^ 0xFF;
    //     }
    // }
    //
    // pub fn set_negative_flag(&mut self, on: bool) {
    //     if on {
    //         self.f |= FlagBits::N;
    //     } else {
    //         self.f &= FlagBits::N ^ 0xFF;
    //     }
    // }
    //
    // pub fn set_half_carry_flag(&mut self, on: bool) {
    //     if on {
    //         self.f |= FlagBits::H;
    //     } else {
    //         self.f &= FlagBits::H ^ 0xFF;
    //     }
    // }
    //
    // pub fn set_carry_flag(&mut self, on: bool) {
    //     if on {
    //         self.f |= FlagBits::C;
    //     } else {
    //         self.f &= FlagBits::C ^ 0xFF;
    //     }
    // }
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
