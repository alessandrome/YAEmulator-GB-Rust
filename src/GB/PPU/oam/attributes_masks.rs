use crate::{default_enum_u8, default_enum_u8_bit_ops};

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum AttributesMasks {
    Priority = 0b1000_0000,
    YFlip = 0b0100_0000,
    XFlip = 0b0010_0000,
    Palette = 0b0001_0000,
}

default_enum_u8!(AttributesMasks {Priority = 128, YFlip = 64, XFlip = 32, Palette = 16});
