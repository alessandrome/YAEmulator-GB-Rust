pub mod wram;
pub mod hram;
pub mod vram;

use std::io::Read;
use std::ops::{Deref, DerefMut};
use crate::GB::ppu::tile::TILE_SIZE;

pub const RST_INSTRUCTIONS: usize = 0x0000; // Location in memory for RST instructions (not used on emulation)
pub const CARTRIDGE_HEADER_ADDRESS: usize = 0x0100; // Location for ROM metadata (as name) (not used on emulation)
pub const USER_PROGRAM_ADDRESS: usize = 0x0150; // Location User Program (not used on emulation)
pub const VRAM_ADDRESS: usize = 0x8000; // Video memory
pub const VRAM_BLOCK_SIZE: usize = TILE_SIZE * 128;
pub const VRAM_BLOCK_0_ADDRESS: usize = VRAM_ADDRESS; // Video memory - Block 0
pub const VRAM_BLOCK_1_ADDRESS: usize = VRAM_BLOCK_0_ADDRESS + VRAM_BLOCK_SIZE ; // Video memory - Block 1
pub const VRAM_BLOCK_2_ADDRESS: usize = VRAM_BLOCK_1_ADDRESS + VRAM_BLOCK_SIZE ; // Video memory - Block 2
pub const EXTERNAL_RAM_ADDRESS: usize = 0xA000; // External Extension memory
pub const OAM_RAM_ADDRESS: usize = 0xFE00; // Up to 40 Display Object Data (512B)
pub const INTERNAL_RAM_ADDRESS: usize = 0xFF00; // Instruction Registers & Flags
pub const HRAM_ADDRESS: usize = 0xFF80; // High memory 127B (Memory w/ direct access from CPU)

pub const RST_MEM_SIZE: usize = CARTRIDGE_HEADER_ADDRESS - RST_INSTRUCTIONS;
pub const CARTRIDGE_HEADER_SIZE: usize = USER_PROGRAM_ADDRESS - CARTRIDGE_HEADER_ADDRESS;
pub const USER_PROGRAM_MEM_SIZE: usize = VRAM_ADDRESS - USER_PROGRAM_ADDRESS;
pub const WRAM_SIZE: usize = 0x2000;
pub const HRAM_SIZE: usize = 127;

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
    use super::RAM;
    use crate::GB::types::{Byte, address::Address};

    #[test]
    fn test_memory_read() {
        let mut ram = RAM::new();
        let address = Address(0xC0D0);
        let data: Byte = 0x44;
        ram.memory[address.as_index()] = data;
        assert_eq!(ram.read(address), data);
    }

    #[test]
    fn test_memory_write() {
        let mut ram = RAM::new();
        let address = Address(0xC0D0);
        let data: Byte = 0x45;
        ram.memory[address.as_index()] = 0xFF;
        ram.write(address, data);
        assert_eq!(ram.memory[address.as_index()], data);
    }


    #[test]
    fn test_memory_read_vec() {
        let mut ram = RAM::new();
        let start_address = Address(0xC0D0);
        let data: Vec<Byte> = vec![0x44, 0x55, 0xF0, 0x0F, 0x75, 0x1A, 0xA1, 0x92];
        for i in 0..data.len() {
            ram.memory[start_address.as_index() + i] = data[i];
        }
        assert_eq!(ram.read_vec(start_address.as_u16(), data.len() as u16), data);
    }
}
