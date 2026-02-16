macro_rules! get_set {
    ($reg:ident, $get_name:ident, $set_name:ident, $size:ty) => {
        #[inline]
        pub fn $get_name(&self) -> $size {
            self.$reg
        }

        #[inline]
        pub fn $set_name(&mut self, val: $size) {
            self.$reg = val;
        }
    };
}

#[macro_export]
macro_rules! get_set_dual {
    ($reg1:ident, $reg2:ident, $get_name:ident, $set_name:ident) => {
        #[inline]
        pub fn $get_name(&self) -> u16 {
            (self.$reg1 as u16) << 8 | self.$reg2 as u16
        }

        #[inline]
        pub fn $set_name(&mut self, val: u16) {
            self.$reg1 = (val >> 8) as u8;
            self.$reg2 = val as u8;
        }
    };
}

#[macro_export]
macro_rules! get_set_flag {
    ($get_name:ident, $set_name:ident, $flag:ident) => {
        #[inline]
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

pub(crate) use get_set;
pub(crate) use get_set_dual;
pub(crate) use get_set_flag;
