use crate::GB::bus::BusDevice;
use super::{Length, Memory};
use crate::GB::types::address::{Address, AddressRangeInclusive};
use crate::GB::types::Byte;

pub struct HRAM {
    #[cfg(test)]
    pub memory: Memory<u8>,
    #[cfg(not(test))]
    memory: Memory<u8>,
}

impl HRAM {
    pub const HRAM_START_ADDRESS: Address = Address(0xFF00); // Working memory
    pub const HRAM_END_ADDRESS: Address = Address(0xFFFE); // Working memory
    pub const HRAM_ADDRESS_RANGE: AddressRangeInclusive = Self::HRAM_START_ADDRESS..=Self::HRAM_END_ADDRESS; // Working memory

    pub fn new() -> Self {
        Self {
            memory: Memory::<u8>::new(0, 127),
        }
    }

    pub fn read_vec(&self, start_address: u16, length: u16) -> &[u8] {
        &self.memory[start_address as usize..(start_address + length) as usize]
    }
}

impl Length for HRAM {
    fn len(&self) -> usize {
        self.memory.len()
    }
}

impl BusDevice for HRAM {
    fn read(&self, address: Address) -> Byte {
        match address {
            address if Self::HRAM_ADDRESS_RANGE.contains(&address) => {
                self.memory[address.as_index() - Self::HRAM_START_ADDRESS.as_index()]
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn write(&mut self, address: Address, byte: Byte) {
        match address {
            address if Self::HRAM_ADDRESS_RANGE.contains(&address) => {
                self.memory[address.as_index() - Self::HRAM_START_ADDRESS.as_index()] = byte;
            }
            _ => {
                unreachable!();
            }
        }
    }
}
