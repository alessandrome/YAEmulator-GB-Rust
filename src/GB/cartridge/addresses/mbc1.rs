pub const MBC1_RAM_ENABLE_ADDRESS: (usize, usize) = (0x0, 0x2000); // Range [start, end) -> end not included
pub const MBC1_RAM_ENABLE_ADDRESS_START: usize =MBC1_RAM_ENABLE_ADDRESS.0;
pub const MBC1_RAM_ENABLE_ADDRESS_END: usize = MBC1_RAM_ENABLE_ADDRESS.1 - 1;

pub const MBC1_ROM_BANK_SELECTION_ADDRESS: (usize, usize) = (0x2000, 0x4000); // Range [start, end) -> end not included
pub const MBC1_ROM_BANK_SELECTION_ADDRESS_START: usize = MBC1_ROM_BANK_SELECTION_ADDRESS.0;
pub const MBC1_ROM_BANK_SELECTION_ADDRESS_END: usize = MBC1_ROM_BANK_SELECTION_ADDRESS.1 - 1;

pub const MBC1_RAM_BANK_SELECTION_ADDRESS: (usize, usize) = (0x4000, 0x6000); // RAM bank selection OR High bits ROM bank selection
pub const MBC1_RAM_BANK_SELECTION_ADDRESS_START: usize = MBC1_RAM_BANK_SELECTION_ADDRESS.0;
pub const MBC1_RAM_BANK_SELECTION_ADDRESS_END: usize = MBC1_RAM_BANK_SELECTION_ADDRESS.1 - 1;

pub const MBC1_BANKING_MODE_ADDRESS: (usize, usize) = (0x6000, 0x8000); // Banking mode selection
pub const MBC1_BANKING_MODE_ADDRESS_START: usize = MBC1_BANKING_MODE_ADDRESS.0;
pub const MBC1_BANKING_MODE_ADDRESS_END: usize = MBC1_BANKING_MODE_ADDRESS.1 - 1;
