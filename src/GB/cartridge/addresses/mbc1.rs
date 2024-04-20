pub const MBC1_RAM_ENABLE_ADDRESS: (usize, usize) = (0x0, 0x2000); // Range [start, end) -> end not included
pub const MBC1_RAM_ENABLE_ADDRESS_START: usize =MBC1_RAM_ENABLE_ADDRESS.0;
pub const MBC1_RAM_ENABLE_ADDRESS_END: usize = MBC1_RAM_ENABLE_ADDRESS.1 - 1;

pub const MBC1_ROM_BANK_SELECTION_ADDRESS: (usize, usize) = (0x2000, 0x4000); // Range [start, end) -> end not included
pub const MBC1_ROM_BANK_SELECTION_ADDRESS_START: usize = MBC1_ROM_BANK_SELECTION_ADDRESS.0;
pub const MBC1_ROM_BANK_SELECTION_ADDRESS_END: usize = MBC1_ROM_BANK_SELECTION_ADDRESS.1 - 1;
