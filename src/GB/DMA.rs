use crate::GB::bus::{Bus, BusContext, BusDevice};
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

pub struct DMA {
    enabled: bool
}

impl DMA {
    pub fn new() -> Self {
        Self {
            enabled: false,
        }
    }

    pub fn tick(&mut self, bus: &mut Bus, ctx: &mut BusContext) {
        todo!()
    }
    
    #[inline]
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

impl BusDevice for DMA {
    fn read(&self, address: Address) -> Byte {
        todo!()
    }

    fn write(&mut self, address: Address, data: Byte) {
        todo!()
    }
}

impl Default for DMA {
    fn default() -> Self {
        Self::new()
    }
}
