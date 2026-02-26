use crate::GB::bus::BusDevice;
use crate::GB::ppu::oam::OAM;
use super::{Length, Memory};
use crate::GB::types::address::{Address, AddressRangeInclusive};
use crate::GB::types::Byte;

pub struct OamMemory {
    #[cfg(test)]
    pub memory: Memory<u8>,
    #[cfg(not(test))]
    memory: Memory<u8>,
}

impl OamMemory {
    pub const OAM_START_ADDRESS: Address = Address(0xFE00);
    pub const OAM_END_ADDRESS: Address = Address(0xFE9F);
    pub const OAM_ADDRESS_RANGE: AddressRangeInclusive = Self::OAM_START_ADDRESS..=Self::OAM_END_ADDRESS; // Working memory
    pub const OAM_ITEMS: u8 = 40;

    pub fn new() -> Self {
        Self {
            memory: Memory::<u8>::new(0, 0xA0),
        }
    }

    pub fn read_vec(&self, start_address: u16, length: u16) -> &[u8] {
        &self.memory[start_address as usize..(start_address + length) as usize]
    }

    pub fn oam(&self, id: u8) -> OAM {
        if id >= Self::OAM_ITEMS {
            panic!("OAM ID out of range. It should be between 0 and {}", Self::OAM_ITEMS);
        }
        OAM::new(
            self.memory[OAM::OAM_BYTES as usize * id as usize],
            self.memory[OAM::OAM_BYTES as usize * id as usize + 1],
            self.memory[OAM::OAM_BYTES as usize * id as usize + 2],
            self.memory[OAM::OAM_BYTES as usize * id as usize + 3],
            Some(id)
        )
    }
}

impl Length for OamMemory {
    fn len(&self) -> usize {
        self.memory.len()
    }
}

impl BusDevice for OamMemory {
    fn read(&self, address: Address) -> Byte {
        match address {
            address if Self::OAM_ADDRESS_RANGE.contains(&address) => {
                self.memory[address.as_index() - Self::OAM_START_ADDRESS.as_index()]
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn write(&mut self, address: Address, byte: Byte) {
        match address {
            address if Self::OAM_ADDRESS_RANGE.contains(&address) => {
                self.memory[address.as_index() - Self::OAM_START_ADDRESS.as_index()] = byte;
            }
            _ => {
                unreachable!();
            }
        }
    }
}
