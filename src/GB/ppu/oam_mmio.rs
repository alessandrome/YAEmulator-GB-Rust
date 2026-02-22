use crate::GB::bus::BusDevice;
use crate::GB::memory::oam_memory::OamMemory;
use crate::GB::memory::vram::VRAM;
use crate::GB::types::address::{Address, AddressRangeInclusive};
use crate::GB::types::Byte;
use super::ppu_mode::PpuMode;
use super::PPU;

pub struct OamMmio {
    oam: OamMemory,
}

impl OamMmio {
    pub fn new() -> Self {
        Self {
            oam: OamMemory::new(),
        }
    }
}

impl BusDevice for OamMmio {
    fn read(&self, address: Address) -> Byte {
        self.oam.read(address)
    }

    fn write(&mut self, address: Address, data: Byte) {
        self.oam.write(address, data);
    }
}
