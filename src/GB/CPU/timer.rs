use crate::{define_enum_u8, default_enum_u8_bit_ops, mask_flag_enum_default_impl};


define_enum_u8! {
    pub TACMask {
        Enabled = 0b0000_0100,
        TimerClock = 0b0000_0011
    }
}
mask_flag_enum_default_impl!(TACMask);

define_enum_u8! {
    pub TACClock {
        M256 =  0b0000_0000,
        M4 =    0b0000_0001,
        M16 =   0b0000_0010,
        M64 =   0b0000_0011
    }
}
mask_flag_enum_default_impl!(TACClock);

pub const M256_CLOCK_CYCLES: u64 = 256;
pub const M4_CLOCK_CYCLES: u64 = 4;
pub const M16_CLOCK_CYCLES: u64 = 16;
pub const M64_CLOCK_CYCLES: u64 = 64;

pub const M256_CLOCK_MODE: u8 = 0b00;
pub const M4_CLOCK_MODE: u8 = 0b01;
pub const M16_CLOCK_MODE: u8 = 0b10;
pub const M64_CLOCK_MODE: u8 = 0b11;
