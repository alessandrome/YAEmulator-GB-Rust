use crate::GB::bus::BusDevice;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;
use super::macro_registers;

use crate::{mask_flag_enum_default_impl, default_enum_u8_bit_ops};


pub const INTERRUPT_VBLANK_ADDR: u16 = 0x40;
pub const INTERRUPT_STAT_ADDR: u16 = 0x48;
pub const INTERRUPT_TIMER_ADDR: u16 = 0x50;
pub const INTERRUPT_SERIAL_ADDR: u16 = 0x58;
pub const INTERRUPT_JOYPAD_ADDR: u16 = 0x60;

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

/// Structure for a quick view of Interrupt Flags status in IF and IE registers
pub struct InterruptFlags {
    pub joy_pad: bool,
    pub serial: bool,
    pub timer: bool,
    pub lcd: bool,
    pub v_blank: bool,
}

impl InterruptFlags {
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


pub struct InterruptRegisters {
    ie: u8,
    iflag: u8,  // IF but if is a reserved keyword
}

impl InterruptRegisters {
    pub const IE_ADDRESS: Address = Address(0xFFFF);
    pub const IF_ADDRESS: Address = Address(0xFF0F);

    #[inline]
    pub fn new() -> Self {
        Self {
            ie: 0,
            iflag: 0,
        }
    }

    macro_registers::get_set!(ie, get_ie, set_ie, u8);
    macro_registers::get_set!(iflag, get_if, set_if, u8);

    #[inline]
    pub fn get_vblank_enabled(&self) -> bool {
        (self.ie & 0b0000_0001) != 0
    }

    #[inline]
    pub fn get_lcd_enabled(&self) -> bool {
        (self.ie & 0b0000_0010) != 0
    }

    #[inline]
    pub fn get_timer_enabled(&self) -> bool {
        (self.ie & 0b0000_0100) != 0
    }

    #[inline]
    pub fn get_serial_enabled(&self) -> bool {
        (self.ie & 0b0000_1000) != 0
    }

    #[inline]
    pub fn get_joypad_enabled(&self) -> bool {
        (self.ie & 0b0001_0000) != 0
    }

    #[inline]
    pub fn get_vblank_interrupt(&self) -> bool {
        (self.iflag & 0b0000_0001) != 0
    }

    #[inline]
    pub fn get_lcd_interrupt(&self) -> bool {
        (self.iflag & 0b0000_0010) != 0
    }

    #[inline]
    pub fn get_timer_interrupt(&self) -> bool {
        (self.iflag & 0b0000_0100) != 0
    }

    #[inline]
    pub fn get_serial_interrupt(&self) -> bool {
        (self.iflag & 0b0000_1000) != 0
    }

    #[inline]
    pub fn get_joypad_interrupt(&self) -> bool {
        (self.iflag & 0b0001_0000) != 0
    }
}

impl BusDevice for InterruptRegisters {
    fn read(&self, address: Address) -> Byte {
        match address {
            Self::IE_ADDRESS => self.ie,
            Self::IF_ADDRESS => self.iflag,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: Address, data: Byte) {
        match address {
            Self::IE_ADDRESS => self.ie = data,
            Self::IF_ADDRESS => self.iflag = data,
            _ => unreachable!(),
        }
    }
}

impl Default for InterruptRegisters {
    fn default() -> Self {
        Self::new()
    }
}
