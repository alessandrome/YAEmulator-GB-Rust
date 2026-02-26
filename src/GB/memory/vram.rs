use crate::GB::bus::BusDevice;
use super::{Length, Memory};
use crate::GB::types::address::{Address, AddressRangeInclusive};
use crate::GB::types::Byte;

pub struct VRAM {
    #[cfg(test)]
    pub memory: Memory<u8>,
    #[cfg(not(test))]
    memory: Memory<u8>,
}

impl VRAM {
    pub const VRAM_START_ADDRESS: Address = Address(0x8000); // Working memory
    pub const VRAM_END_ADDRESS: Address = Address(0x9FFF); // Working memory
    pub const VRAM_ADDRESS_RANGE: AddressRangeInclusive = Self::VRAM_START_ADDRESS..=Self::VRAM_END_ADDRESS; // Working memory

    pub fn new() -> Self {
        Self {
            memory: Memory::<u8>::new(0, 0x2000),
        }
    }

    pub fn read_vec(&self, start_address: u16, length: u16) -> &[u8] {
        &self.memory[start_address as usize..(start_address + length) as usize]
    }
}

impl Length for VRAM {
    fn len(&self) -> usize {
        self.memory.len()
    }
}

impl BusDevice for VRAM {
    fn read(&self, address: Address) -> Byte {
        match address {
            address if Self::VRAM_ADDRESS_RANGE.contains(&address) => {
                self.memory[address.as_index() - Self::VRAM_START_ADDRESS.as_index()]
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn write(&mut self, address: Address, byte: Byte) {
        match address {
            address if Self::VRAM_ADDRESS_RANGE.contains(&address) => {
                self.memory[address.as_index() - Self::VRAM_START_ADDRESS.as_index()] = byte;
            }
            _ => {
                unreachable!();
            }
        }
    }
}
