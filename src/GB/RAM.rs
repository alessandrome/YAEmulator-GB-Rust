pub const RST_INSTRUCIONS: usize = 0x0000; // Location in RAM for RST instructions (not used on emulation)
pub const ROM_METADATA: usize = 0x0100; // Location for ROM metadata (as name) (not used on emulation)
pub const USER_PROGRAM_ADDRESS: usize = 0x0150; // Location User Program (not used on emulation)
pub const VRAM_ADDRESS: usize = 0x8000; // Video RAM
pub const EXTERNAL_RAM_ADDRESS: usize = 0xA000; // External Extension RAM
pub const WRAM_ADDRESS: usize = 0xC000; // Working RAM
pub const OAM_RAM_ADDRESS: usize = 0xFE00; // Up to 40 Display Object Data (512B)
pub const INTERNAL_RAM_ADDRESS: usize = 0xFF00; // Instruction Registers & Flags
pub const HRAM_ADDRESS: usize = 0xFF80; // High RAM 127B (Memory w/ direct access from CPU)

macro_rules! read_ram_space {
    ($function:ident, $space_address:ident) => {
        pub fn $function(&self, address: u16) -> u8 {
            self.memory[address as usize + $space_address]
        }
    };
}
macro_rules! write_ram_space {
    ($function:ident, $space_address:ident) => {
        pub fn $function(&self, address: u16, byte: u8) {
            self.memory[address as usize + $space_address] = byte;
        }
    };
}

pub struct RAM {
    memory: [u8; 65536],
}

impl RAM {
    pub fn new() -> Self {
        RAM { memory: [0; 65536] }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte;
    }

    pub fn read_vec(&self, start_address: usize, length: usize) -> &[u8] {
        &self.memory[start_address..(start_address + length)]
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
