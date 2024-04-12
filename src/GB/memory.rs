use std::fs::File;
use std::io::Read;

pub const RST_INSTRUCTIONS: usize = 0x0000; // Location in memory for RST instructions (not used on emulation)
pub const CARTRIDGE_HEADER_ADDRESS: usize = 0x0100; // Location for ROM metadata (as name) (not used on emulation)
pub const USER_PROGRAM_ADDRESS: usize = 0x0150; // Location User Program (not used on emulation)
pub const VRAM_ADDRESS: usize = 0x8000; // Video memory
pub const EXTERNAL_RAM_ADDRESS: usize = 0xA000; // External Extension memory
pub const WRAM_ADDRESS: usize = 0xC000; // Working memory
pub const OAM_RAM_ADDRESS: usize = 0xFE00; // Up to 40 Display Object Data (512B)
pub const INTERNAL_RAM_ADDRESS: usize = 0xFF00; // Instruction Registers & Flags
pub const HRAM_ADDRESS: usize = 0xFF80; // High memory 127B (Memory w/ direct access from CPU)

pub const RST_MEM_SIZE: usize = CARTRIDGE_HEADER_ADDRESS - RST_INSTRUCTIONS;
pub const CARTRIDGE_HEADER_SIZE: usize = USER_PROGRAM_ADDRESS - CARTRIDGE_HEADER_ADDRESS;
pub const USER_PROGRAM_MEM_SIZE: usize = VRAM_ADDRESS - USER_PROGRAM_ADDRESS;
pub const WRAM_SIZE: usize = 0x2000;
pub const HRAM_SIZE: usize = 127;

macro_rules! read_ram_space {
    ($function:ident, $space_address:ident) => {
        pub fn $function(&self, address: u16) -> u8 {
            self.memory[address as usize + $space_address]
        }
    };
}
macro_rules! write_ram_space {
    ($function:ident, $space_address:ident) => {
        pub fn $function(&mut self, address: u16, byte: u8) {
            self.memory[address as usize + $space_address] = byte;
        }
    };
}

pub struct Memory<T, const N: usize> where T: Clone {
    #[cfg(test)]
    pub memory: Box<[T; N]>,
    #[cfg(not(test))]
    memory: Box<[T; N]>,
}

impl<T: Clone, const N: usize> std::ops::Index<usize> for Memory<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.memory[index]
    }
}

impl<T: Clone, const N: usize> std::ops::IndexMut<usize> for Memory<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.memory[index]
    }
}

impl<T: Clone, const N: usize> std::ops::Index<std::ops::Range<usize>> for Memory<T, N> {
    type Output = [T];

    fn index(&self, index: std::ops::Range<usize>) -> &[T] {
        &self.memory[index]
    }
}

impl<T: Clone, const N: usize> std::ops::IndexMut<std::ops::Range<usize>> for Memory<T, N> {
    fn index_mut(&mut self, index: std::ops::Range<usize>) -> &mut [T] {
        &mut self.memory[index]
    }
}

impl<T: Clone + std::marker::Copy, const N: usize>  Memory<T, N> {
    pub fn len(&self) -> usize { self.memory.len() }
    pub fn new(default: T) -> Self where T: Clone {
        Self {
            memory: Box::new([default; N])
        }
    }
}


trait Length {
    fn len(&self) -> usize;
}

pub struct RAM {
    #[cfg(test)]
    pub memory: Memory<u8, 65536>,
    #[cfg(not(test))]
    memory: Memory<u8, 65536>,
}

pub struct ROM {
    #[cfg(test)]
    pub memory: Memory<u8, 256>,
    #[cfg(not(test))]
    memory: Memory<u8, 256>,
    bios: String,
}

impl RAM {
    pub fn new() -> Self {
        RAM { memory: Memory::<u8, 65536>::new(0) }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte;
    }

    pub fn read_vec(&self, start_address: u16, length: u16) -> &[u8] {
        &self.memory[start_address as usize..(start_address + length) as usize]
    }

    pub fn boot_load(&mut self, rom: &ROM) {
        for i in 0..rom.len() {
            self.memory[i] = self.read(i as u16);
        }
    }

    read_ram_space!(read_wram, WRAM_ADDRESS);
    read_ram_space!(read_vram, VRAM_ADDRESS);
    read_ram_space!(read_hram, HRAM_ADDRESS);
    read_ram_space!(read_user_program, USER_PROGRAM_ADDRESS);

    write_ram_space!(write_wram, WRAM_ADDRESS);
    write_ram_space!(write_vram, VRAM_ADDRESS);
    write_ram_space!(write_hram, HRAM_ADDRESS);
    write_ram_space!(write_user_program, USER_PROGRAM_ADDRESS);
}

impl Length for RAM {
    fn len(&self) -> usize {
        self.memory.len()
    }
}

impl ROM {
    pub  fn new() -> Self {
        ROM { memory: Memory::<u8, 256>::new(0), bios: String::from("") }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn load_bios(&mut self, path: &String) -> Result<(), std::io::Error> {
        let mut file = File::open(path)?;
        let mut buffer = [0u8; 256];
        file.read_exact(&mut buffer)?;
        self.memory = Memory { memory: Box::new(buffer) };
        self.bios = path.clone();
        Ok(())
    }
}

impl Length for ROM {
    fn len(&self) -> usize {
        self.memory.len()
    }
}

#[cfg(test)]
mod test {
    use crate::GB::memory::RAM;

    #[test]
    fn test_memory_read() {
        let mut ram = RAM::new();
        let address: usize = 0xC0D0;
        let data: u8 = 0x44;
        ram.memory[address] = data;
        assert_eq!(ram.read(address as u16), data);
    }

    #[test]
    fn test_memory_write() {
        let mut ram = RAM::new();
        let address: usize = 0xC0D0;
        let data: u8 = 0x45;
        ram.memory[address] = 0xFF;
        ram.write(address as u16, data);
        assert_eq!(ram.memory[address], data);
    }


    #[test]
    fn test_memory_read_vec() {
        let mut ram = RAM::new();
        let start_address: usize = 0xC000;
        let data: Vec<u8> = vec![0x44, 0x55, 0xF0, 0x0F, 0x75, 0x1A, 0xA1, 0x92];
        for i in 0..data.len() {
            ram.memory[start_address + i] = data[i];
        }
        assert_eq!(ram.read_vec(start_address as u16, data.len() as u16), data);
    }
}

trait UseMemory {
    fn read_memory(&self, address: u16) -> u8;
    fn write_memory(&self, address: u16, data: u8);
}
