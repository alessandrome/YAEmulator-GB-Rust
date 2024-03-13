macro_rules! get_set {
    ($reg:ident, $size:ty) => {
        pub fn get_$reg(&self) -> $size {
            self.$reg
        }

        pub fn set_$reg(&mut self, val: $size) {
            self.$reg = val;
        }
    };
}

macro_rules! get_set_dual {
    ($reg1:ident, $reg2:ident) => {
        pub fn get_$reg1$reg2(&self) -> u16 {
            (self.$reg1 as u16) << 8 | self.$reg2 as u16
        }

        pub fn set_$reg1$reg2(&mut self, val: u16) {
            self.$reg1 = (val >> 8) as u8;
            self.$reg2 = val as u8;
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub enum Flags {
    Z,
    N,
    H,
    C,
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
    get_set!(a, u8);
    get_set!(b, u8);
    get_set!(c, u8);
    get_set!(d, u8);
    get_set!(e, u8);
    get_set!(h, u8);
    get_set!(sp, u16);
    get_set!(pc, u16);
    get_set_dual!(b, c);
    get_set_dual!(d, e);
    get_set_dual!(h, l);

    fn get_f(&self) -> u8 {
        self.f
    }
    fn set_f(&mut self, val: u8) {
        self.f = val & 0xF0
    }

    fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }
    fn set_af(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = (val & 0xF0) as u8
    }

    fn get_flags(&self) -> Flags {
        Flags {
            z: (self.f & 0b10000000) != 0,
            n: (self.f & 0b01000000) != 0,
            h: (self.f & 0b00100000) != 0,
            c: (self.f & 0b00010000) != 0,
        }
    }
}
