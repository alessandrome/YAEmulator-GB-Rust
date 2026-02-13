use crate::GB::traits::BusDevice;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

pub struct Interrupt {
    // todo!()
}

impl Interrupt {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(&mut self) {
        todo!()
    }
}

impl Default for Interrupt {
    fn default() -> Self {
        Self::new()
    }
}

impl BusDevice for Interrupt {
    fn read(&self, address: Address) -> Byte {
        todo!()
    }

    fn write(&mut self, address: Address, data: Byte) {
        todo!()
    }
}
