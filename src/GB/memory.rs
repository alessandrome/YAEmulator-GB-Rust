pub mod wram;
pub mod hram;
pub mod vram;
pub mod oam_memory;

pub const RST_INSTRUCTIONS: usize = 0x0000; // Location in memory for RST instructions (not used on emulation)
pub const CARTRIDGE_HEADER_ADDRESS: usize = 0x0100; // Location for ROM metadata (as name) (not used on emulation)
pub const USER_PROGRAM_ADDRESS: usize = 0x0150; // Location User Program (not used on emulation)
pub const EXTERNAL_RAM_ADDRESS: usize = 0xA000; // External Extension memory
pub const RST_MEM_SIZE: usize = CARTRIDGE_HEADER_ADDRESS - RST_INSTRUCTIONS;
pub const CARTRIDGE_HEADER_SIZE: usize = USER_PROGRAM_ADDRESS - CARTRIDGE_HEADER_ADDRESS;
pub const USER_PROGRAM_MEM_SIZE: usize = vram::VRAM::VRAM_START_ADDRESS.as_usize() - USER_PROGRAM_ADDRESS;

pub struct Memory<T> where T: Clone {
    #[cfg(test)]
    pub memory: Vec<T>,
    #[cfg(not(test))]
    memory: Vec<T>,
}

impl<T: Clone> std::ops::Index<usize> for Memory<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.memory[index]
    }
}

impl<T: Clone> std::ops::IndexMut<usize> for Memory<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.memory[index]
    }
}

impl<T: Clone> std::ops::Index<std::ops::Range<usize>> for Memory<T> {
    type Output = [T];

    fn index(&self, index: std::ops::Range<usize>) -> &[T] {
        &self.memory[index]
    }
}

impl<T: Clone> std::ops::IndexMut<std::ops::Range<usize>> for Memory<T> {
    fn index_mut(&mut self, index: std::ops::Range<usize>) -> &mut [T] {
        &mut self.memory[index]
    }
}

impl<T: Clone + std::marker::Copy>  Memory<T> {
    pub fn len(&self) -> usize { self.memory.len() }
    pub fn new(default: T, size: usize) -> Self where T: Clone {
        Self {
            memory: vec![default; size]
        }
    }
    pub fn new_from_vec(mem: Vec<T>) -> Self where T: Clone {
        Self {
            memory: mem
        }
    }
}


pub trait Length {
    fn len(&self) -> usize;
}

#[cfg(test)]
mod test {
    use crate::GB::bus::BusDevice;
    use super::wram::WRAM;
    use crate::GB::types::{Byte, address::Address};

    #[test]
    fn test_memory_read() {
        let mut ram = WRAM::new();
        let address = Address(0xC0D0);
        let data: Byte = 0x44;
        ram.memory[address.as_index()] = data;
        assert_eq!(ram.read(address), data);
    }

    #[test]
    fn test_memory_write() {
        let mut ram = WRAM::new();
        let address = Address(0xC0D0);
        let data: Byte = 0x45;
        ram.memory[address.as_index()] = 0xFF;
        ram.write(address, data);
        assert_eq!(ram.memory[address.as_index()], data);
    }


    #[test]
    fn test_memory_read_vec() {
        let mut ram = WRAM::new();
        let start_address = Address(0xC0D0);
        let data: Vec<Byte> = vec![0x44, 0x55, 0xF0, 0x0F, 0x75, 0x1A, 0xA1, 0x92];
        for i in 0..data.len() {
            ram.memory[start_address.as_index() + i] = data[i];
        }
        assert_eq!(ram.read_vec(start_address.as_u16(), data.len() as u16), data);
    }
}
