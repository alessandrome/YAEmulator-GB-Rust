use crate::GB::bus::BusDevice;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;
use super::macro_registers;

pub struct InterruptRegisters {
    ie: u8,
    iflag: u8,  // IF but if is a reserved keyword
}

impl InterruptRegisters {
    pub const IE_ADDRESS: Address = Address(0xFFFF);
    pub const IF_ADDRESS: Address = Address(0xFF0F);

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
