mod bus_device;

use crate::GB::APU::APU;
use crate::GB::cartridge::ROM;
use crate::GB::memory::RAM;
use crate::GB::CPU::CPU;
use crate::GB::PPU::PPU;
use super::timer::TimerRegisters;
pub(crate) use bus_device::{BusDevice, MmioDevice, MemoryDevice};
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

pub struct BusContext<'a> {
    // timer: &'a mut TimerRegisters,
    pub apu: &'a mut APU,
}

pub struct Bus {}

impl Bus {
    pub fn new() -> Self {
        Self {}
    }
}

impl BusDevice for Bus {
    fn read(&self, address: Address) -> Byte {
        todo!()
    }

    fn write(&mut self, address: Address, data: Byte) {
        todo!()
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}
