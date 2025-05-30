pub mod registers;
pub mod addresses;
pub mod BIOS;
pub mod interrupts;

use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use crate::GB::cartridge::{Cartridge, UseCartridge};
use crate::GB::input;
use crate::GB::memory::addresses::{*};
use crate::GB::memory::registers::MemoryRegisters;
use crate::GB::PPU::tile::TILE_SIZE;

use crate::GB::input::{GBInputSelectionBits, GBInputButtonsBits, GBInputDPadBits};

pub const RST_INSTRUCTIONS: usize = 0x0000; // Location in memory for RST instructions (not used on emulation)
pub const CARTRIDGE_HEADER_ADDRESS: usize = 0x0100; // Location for ROM metadata (as name) (not used on emulation)
pub const USER_PROGRAM_ADDRESS: usize = 0x0150; // Location User Program (not used on emulation)
pub const VRAM_ADDRESS: usize = 0x8000; // Video memory
pub const VRAM_BLOCK_SIZE: usize = TILE_SIZE * 128;
pub const VRAM_BLOCK_0_ADDRESS: usize = VRAM_ADDRESS; // Video memory - Block 0
pub const VRAM_BLOCK_1_ADDRESS: usize = VRAM_BLOCK_0_ADDRESS + VRAM_BLOCK_SIZE ; // Video memory - Block 1
pub const VRAM_BLOCK_2_ADDRESS: usize = VRAM_BLOCK_1_ADDRESS + VRAM_BLOCK_SIZE ; // Video memory - Block 2
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

pub struct RAM {
    #[cfg(test)]
    pub memory: Memory<u8>,
    #[cfg(not(test))]
    memory: Memory<u8>,
    inputs: Rc<RefCell<input::GBInput>>,
    cartridge: Rc<RefCell<Option<Cartridge>>>,
}

impl RAM {
    pub fn new(inputs: Rc<RefCell<input::GBInput>>) -> Self {
        RAM {
            memory: Memory::<u8>::new(0, 65536),
            inputs,
            cartridge: Rc::new(RefCell::new(None))
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        let address_usize = address as usize;
        let mut return_val: u8 = 0;
        match address_usize {
            ROM_BANK_0_ADDRESS..=ROM_BANK_1_LAST_ADDRESS | EXTERNAL_RAM_ADDRESS..=EXTERNAL_RAM_LAST_ADDRESS => {
                let c_opt = self.cartridge.borrow();
                match c_opt.as_ref() {
                    None => {
                        return_val = self.memory[address_usize];
                    }
                    Some(cartridge) => {
                        return_val = cartridge.read(address);
                    }
                }
            }
            io::JOYP => {
                //todo!("Test read from input");
                return_val = self.memory[address_usize] & 0xF0;
                if (return_val & GBInputSelectionBits::Buttons == 0 && return_val & GBInputSelectionBits::DPad == 0) {
                    return_val |= 0x0F;
                } else if return_val & GBInputSelectionBits::Buttons == 0 {
                    return_val |= self.inputs.borrow().get_buttons_byte();
                } else if return_val & GBInputSelectionBits::DPad == 0 {
                    return_val |= self.inputs.borrow().get_dpad_byte();
                } else {
                    return_val |= 0x0F;
                }
                // println!("{}", return_val);
            }
            _ => {
                return_val = self.memory[address_usize]
            }
        };
        return_val
    }

    pub fn write(&mut self, address: u16, byte: u8) {
        let address_usize = address as usize;match address_usize {
            ROM_BANK_0_ADDRESS..=ROM_BANK_1_LAST_ADDRESS | EXTERNAL_RAM_ADDRESS..=EXTERNAL_RAM_LAST_ADDRESS => {
                let mut c_opt = self.cartridge.borrow_mut();
                match c_opt.deref_mut() {
                    None => {
                        self.memory[address_usize] = byte;
                    }
                    Some(cartridge) => {
                        cartridge.write(address, byte);
                    }
                }
            }
            _ => {
                self.memory[address_usize] = byte;
            }
        };
    }

    pub fn read_vec(&self, start_address: u16, length: u16) -> &[u8] {
        &self.memory[start_address as usize..(start_address + length) as usize]
    }

    pub fn boot_load(&mut self, bios: &BIOS::BIOS) {
        for i in 0..bios.len() {
            self.memory[i] = self.read(i as u16);
        }
    }

    pub fn get_memory_registers(&self) -> MemoryRegisters {
        MemoryRegisters::new(&self)
    }

    pub fn get_enabled_interrupts(&self) {
        let ie = self.read(registers::IE);
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

impl UseCartridge for RAM {
    fn set_cartridge(&mut self, rom: Rc<RefCell<Option<Cartridge>>>) {
        self.cartridge = rom;
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::GB::input;
    use crate::GB::memory::RAM;

    #[test]
    fn test_memory_read() {
        let inputs_ref = Rc::new(RefCell::new(input::GBInput::default()));
        let mut ram = RAM::new(inputs_ref);
        let address: usize = 0xC0D0;
        let data: u8 = 0x44;
        ram.memory[address] = data;
        assert_eq!(ram.read(address as u16), data);
    }

    #[test]
    fn test_memory_write() {
        let inputs_ref = Rc::new(RefCell::new(input::GBInput::default()));
        let mut ram = RAM::new(inputs_ref);
        let address: usize = 0xC0D0;
        let data: u8 = 0x45;
        ram.memory[address] = 0xFF;
        ram.write(address as u16, data);
        assert_eq!(ram.memory[address], data);
    }


    #[test]
    fn test_memory_read_vec() {
        let inputs_ref = Rc::new(RefCell::new(input::GBInput::default()));
        let mut ram = RAM::new(inputs_ref);
        let start_address: usize = 0xC000;
        let data: Vec<u8> = vec![0x44, 0x55, 0xF0, 0x0F, 0x75, 0x1A, 0xA1, 0x92];
        for i in 0..data.len() {
            ram.memory[start_address + i] = data[i];
        }
        assert_eq!(ram.read_vec(start_address as u16, data.len() as u16), data);
    }
}

pub trait UseMemory {
    fn read_memory(&self, address: u16) -> u8;
    fn write_memory(&self, address: u16, data: u8);
}
