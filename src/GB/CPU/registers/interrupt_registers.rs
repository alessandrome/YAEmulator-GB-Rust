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
