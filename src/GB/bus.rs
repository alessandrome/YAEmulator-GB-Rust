use std::ops::RangeInclusive;
use crate::GB::cartridge::ROM;
use crate::GB::memory::RAM;
use crate::GB::CPU::CPU;
use crate::GB::PPU::PPU;
use crate::GB::traits::BusDevice;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

pub struct Bus {
    // todo!
}

impl Bus {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl BusDevice for Bus {
    fn read(&self, address: Address) -> Byte {
        todo!()
    }

    fn write(&self, address: Address, data: Byte) {
        todo!()
    }
}
