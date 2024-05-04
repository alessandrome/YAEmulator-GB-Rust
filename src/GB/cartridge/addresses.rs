pub mod mbc1;

pub const ENTRY_POINT: usize = 0x0100;
pub const LOGO: usize = 0x0104;
pub const TITLE: usize = 0x0134; // Oldest Cartridge 16 Bytes, Newer 12
pub const TITLE_OLD_SIZE: usize = 16; // Oldest Cartridge 16 Bytes, Newer 12
pub const TITLE_NEW_SIZE: usize = 12; // Oldest Cartridge 16 Bytes, Newer 12
pub const MANUFACTURER_CODE: usize = 0x013F; // 4 Bytes
pub const NEW_LICENSE_CODE: usize = 0x0144;
pub const SGB_FLAG: usize = 0x0146;
pub const CARTRIDGE_TYPE: usize = 0x0147;
pub const ROM_SIZE: usize = 0x0148;
pub const RAM_SIZE: usize = 0x0149;
pub const DESTINATION_CODE: usize = 0x014A;
pub const OLD_LICENSE_CODE: usize = 0x014B; // If 0x33 Check on New License Code Address
pub const ROM_VERSION: usize = 0x014C;
pub const HEADER_CHECKSUM: usize = 0x014D;
pub const GLOBAL_CHECKSUM: usize = 0x014E; // 16 Bit (Big Endian)

pub const MBC_RAM_ENABLE_ADDRESS: (usize, usize) = (0x0, 0x2000); // Range [start, end) -> end not included
pub const MBC_RAM_ENABLE_ADDRESS_START: usize = MBC_RAM_ENABLE_ADDRESS.0;
pub const MBC_RAM_ENABLE_ADDRESS_END: usize = MBC_RAM_ENABLE_ADDRESS.1 - 1;

pub const MBC_ROM_BANK_SELECTION_ADDRESS: (usize, usize) = (0x2000, 0x4000); // Range [start, end) -> end not included
pub const MBC_ROM_BANK_SELECTION_ADDRESS_START: usize = MBC_ROM_BANK_SELECTION_ADDRESS.0;
pub const MBC_ROM_BANK_SELECTION_ADDRESS_END: usize = MBC_ROM_BANK_SELECTION_ADDRESS.1 - 1;
