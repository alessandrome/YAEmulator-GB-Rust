use std::ops::RangeInclusive;
use crate::GB::cartridge::ROM;
use crate::GB::memory::RAM;
use crate::GB::CPU::CPU;
use crate::GB::PPU::PPU;

pub type MemoryAddressRange = RangeInclusive<u16>;

pub trait AddressableDevice {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, value: u8) -> bool;
    fn address_range(&self) -> &MemoryAddressRange;
}

pub struct Bus {
    devices: Vec<Box<dyn AddressableDevice>>,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    pub fn add_device(&mut self, device: Box<dyn AddressableDevice>) {
        self.devices.push(device);
    }

    pub fn read(&self, addr: u16) -> u8 {
        for device in &self.devices {
            if let Some(val) = device.read(addr) {
                return val;
            }
        }
        0xFF // valore di default se nulla risponde
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        for device in &mut self.devices {
            if device.write(addr, val) {
                break; // primo device che risponde gestisce la scrittura
            }
        }
    }
}
