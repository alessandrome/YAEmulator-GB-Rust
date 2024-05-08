use crate::{mask_flag_enum_default_impl, default_enum_u8_bit_ops};


#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum InterruptEnableMask {
    JoyPad = 0b0001_0000,
    Serial = 0b0000_1000,
    Timer = 0b0000_0100,
    LCD = 0b0000_0010,
    VBlank = 0b0000_0001,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum InterruptFlagsMask {
    JoyPad = 0b0001_0000,
    Serial = 0b0000_1000,
    Timer = 0b0000_0100,
    LCD = 0b0000_0010,
    VBlank = 0b0000_0001,
}

mask_flag_enum_default_impl!(InterruptEnableMask);
mask_flag_enum_default_impl!(InterruptFlagsMask);

pub struct Interrupts {
    pub joy_pad: bool,
    pub serial: bool,
    pub timer: bool,
    pub lcd: bool,
    pub v_blank: bool,
}

/// Structure that represent status of flags in IF register.
impl Interrupts {
    pub fn new(byte: u8) -> Self {
        Self {
            joy_pad: (byte & InterruptFlagsMask::JoyPad) != 0,
            serial: (byte & InterruptFlagsMask::Serial) != 0,
            timer: (byte & InterruptFlagsMask::Timer) != 0,
            lcd: (byte & InterruptFlagsMask::LCD) != 0,
            v_blank: (byte & InterruptFlagsMask::VBlank) != 0,
        }
    }
}
