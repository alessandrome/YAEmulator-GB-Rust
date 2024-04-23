use crate::{mask_flag_enum_default_impl, default_enum_u8_bit_ops};

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum LCDStatMasks {
    PPUMode = 0b0000_0011,
    LYCeLY = 0b0000_0100,
    Mode0Interrupt = 0b0000_1000,
    Mode1Interrupt = 0b0001_0000,
    Mode2Interrupt = 0b0010_0000,
    LYCInterrupt = 0b0100_0000,
}

mask_flag_enum_default_impl!(LCDStatMasks);
