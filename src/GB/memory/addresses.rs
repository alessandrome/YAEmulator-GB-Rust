pub const INTERRUPT_RST_TABLE_ADDRESS: usize = 0x00;
pub const INTERRUPT_RST_TABLE_SIZE: usize = 0x100;

pub const ROM_ADDRESS: usize = 0x00;
pub const ROM_SIZE: usize = 0x8000;

pub const ROM_DATA_ADDRESS: usize = 0x00;
pub const ROM_DATA_SIZE: usize = 0x50;

pub const ROM_PROGRAM_ADDRESS: usize = 0x150;
pub const ROM_PROGRAM_SIZE: usize = ROM_SIZE - ROM_PROGRAM_ADDRESS;

pub const ROM_BANK_0_ADDRESS: usize = 0x00;
pub const ROM_BANK_1_ADDRESS: usize = 0x4000; // Switchable ROM Bank 01-NN
pub const ROM_BANK_SIZE: usize = 0x4000;
pub const ROM_BANK_0_LAST_ADDRESS: usize = ROM_BANK_0_ADDRESS + ROM_BANK_SIZE - 1;
pub const ROM_BANK_1_LAST_ADDRESS: usize = ROM_BANK_1_ADDRESS + ROM_BANK_SIZE - 1; // Switchable ROM Bank 01-NN

pub const VRAM_ADDRESS: usize = 0x8000;
pub const VRAM_SIZE: usize = 0x2000; // 8KiB

pub const EXTERNAL_RAM_ADDRESS: usize = 0xA000; // 8KB external (in cartridge) RAM - Switchable if possible
pub const EXTERNAL_RAM_SIZE: usize = 0x2000; // 8KB external (in cartridge) RAM - Switchable if possible
pub const EXTERNAL_RAM_LAST_ADDRESS: usize = EXTERNAL_RAM_ADDRESS + EXTERNAL_RAM_SIZE - 1; // Last usable address of external RAM

pub const WRAM_ADDRESS: usize = 0xC000; // 8KB Working RAM (normal used RAM mounted on GB)
pub const WRAM_SIZE: usize = 0x2000; // 8KB Working RAM (normal used RAM mounted on GB)

pub const ECHO_WRAM_ADDRESS: usize = 0xE000; // Use Prohibited
pub const ECHO_WRAM_SIZE: usize = OAM_ADDRESS - ECHO_WRAM_ADDRESS; // Use Prohibited

pub const OAM_ADDRESS: usize = 0xFE00; // OAM Items zone

pub const PROHIBITED_AREA_ADDRESS: usize = 0xFEA0; // Prohibited by GB documents
pub const PROHIBITED_AREA_SIZE: usize = IO_REGISTERS_ADDRESS - PROHIBITED_AREA_ADDRESS; // Prohibited by GB documents

pub const IO_REGISTERS_ADDRESS: usize = 0xFF00; // I/O Mapped
pub const IO_REGISTERS_SIZE: usize = 0x80;

pub const HRAM_ADDRESS: usize = 0xFF80; // High-RAM - High Speed memory zone, not locked during oam transfer ops
pub const HRAM_SIZE: usize = 127; // 127 Bytes of HRAM

pub const INTERRUPT_ENABLED_ADDRESS: usize = 0xFFFF; // High-RAM - High Speed memory zone, not locked during oam transfer ops
