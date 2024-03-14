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

#[derive(Debug, Clone, Copy)]
pub enum FlagBits {
    Z,
    N,
    H,
    C,
}

pub struct Flags{
    z: bool,
    n: bool,
    h: bool,
    c: bool,
}

impl Flags {
    pub fn new(z:bool, n:bool, h:bool, c:bool) -> Self {
        Self { z, n, h, c }
    }
}

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
    get_set!(a, get_a, set_a, u8);
    get_set!(b, get_b, set_b, u8);
    get_set!(c, get_c, set_c, u8);
    get_set!(d, get_d, set_d, u8);
    get_set!(e, get_e, set_e, u8);
    get_set!(h, get_h, set_h, u8);
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
        self.f = (val & 0xF0) as u8
    }

    pub fn get_flags(&self) -> Flags {
        Flags::new(
            (self.f & 0b10000000) != 0,
            (self.f & 0b01000000) != 0,
            (self.f & 0b00100000) != 0,
            (self.f & 0b00010000) != 0,
        )
    }
}
