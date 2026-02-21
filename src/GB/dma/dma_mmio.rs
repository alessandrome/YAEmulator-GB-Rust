use crate::GB::bus::BusDevice;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;
use super::DMA;

pub struct DmaMmio {
    enabled: bool,
    value: Byte,
}

impl DmaMmio {
    pub fn new() -> Self {
        Self {
            enabled: false,
            value: 0,
        }
    }

    #[inline]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    #[inline]
    pub fn value(&self) -> Byte {
        self.value
    }

    #[inline]
    pub fn reset(&mut self) {
        self.enabled = false;
    }
}

impl BusDevice for DmaMmio {
    fn read(&self, address: Address) -> Byte {
        match address {
            DMA::DMA_SOURCE_ADDRESS => {
                self.value
            }
            _ => unimplemented!()
        }
    }

    fn write(&mut self, address: Address, data: Byte) {
        match address {
            DMA::DMA_SOURCE_ADDRESS => {
                self.value = data;
                self.enabled = true;
            }
            _ => unimplemented!()
        }
    }
}
