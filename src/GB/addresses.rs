use crate::GB::types::address::Address;
pub mod rom {
    use crate::GB::cartridge::{Cartridge, header::RomHeader};
    use super::Address;
    pub const ROM_LOW_BANK_START_ADDRESS: Address = Cartridge::CART_ROM_START_ADDRESS;
    pub const ROM_LOW_BANK_END_ADDRESS: Address = Address(Cartridge::CART_ROM_START_ADDRESS.as_u16() + 0x3FFF);
    pub const ROM_HIGH_BANK_START_ADDRESS: Address = Address(0x4000);
    pub const ROM_HIGH_BANK_END_ADDRESS: Address = Cartridge::CART_ROM_END_ADDRESS;
    pub const RAM_START_ADDRESS: Address = Cartridge::CART_RAM_START_ADDRESS;
    pub const RAM_END_ADDRESS: Address = Cartridge::CART_RAM_END_ADDRESS;
    pub const HEADER_START_ADDRESS: Address = RomHeader::HEADER_START_ADDRESS;
    pub const HEADER_END_ADDRESS: Address = RomHeader::HEADER_END_ADDRESS;
}

pub mod cpu {
    use super::Address;
    use crate::GB::cpu::registers::interrupt_registers::InterruptRegisters;
    use crate::GB::memory::hram::HRAM;

    pub const INTERRUPT_ENABLED_REGISTER: Address = InterruptRegisters::IE_ADDRESS;
    pub const INTERRUPT_FLAGS_REGISTER: Address = InterruptRegisters::IF_ADDRESS;
    pub const HRAM_START_ADDRESS: Address = HRAM::HRAM_START_ADDRESS;
    pub const HRAM_END_ADDRESS: Address = HRAM::HRAM_END_ADDRESS;
}

pub mod wram {
    use super::Address;
    use super::super::memory::wram::WRAM;

    pub const WRAM_START_ADDRESS: Address = WRAM::WRAM_START_ADDRESS;
    pub const WRAM_END_ADDRESS: Address = WRAM::WRAM_END_ADDRESS;
}

pub mod ppu {
    use crate::GB::ppu::PPU;
    use super::Address;
    use super::super::ppu::oam::OAM;
    use super::super::ppu::ppu_mmio::PpuMmio;

    // pub const OAM_START_ADDRESS: Address = OAM::OAM_START_ADDRESS;
    // pub const OAM_END_ADDRESS: Address = OAM::OAM_END_ADDRESS;
    pub const LCDC_REGISTER: Address = PpuMmio::LCDC_ADDRESS;
    pub const STAT_REGISTER: Address = PpuMmio::STAT_ADDRESS;


    pub mod vram {
        use super::Address;
        use super::super::super::memory::vram::VRAM;

        pub const VRAM_START_ADDRESS: Address = VRAM::VRAM_START_ADDRESS;
        pub const VRAM_END_ADDRESS: Address = VRAM::VRAM_END_ADDRESS;
        pub const VRAM_TILE_BLOCK_0_START: Address = VRAM::VRAM_TILE_BLOCK_0_START;
        pub const VRAM_TILE_BLOCK_0_END: Address = VRAM::VRAM_TILE_BLOCK_0_END;
        pub const VRAM_TILE_BLOCK_1_START: Address = VRAM::VRAM_TILE_BLOCK_1_START;
        pub const VRAM_TILE_BLOCK_1_END: Address = VRAM::VRAM_TILE_BLOCK_1_END;
        pub const VRAM_TILE_BLOCK_2_START: Address = VRAM::VRAM_TILE_BLOCK_2_START;
        pub const VRAM_TILE_BLOCK_2_END: Address = VRAM::VRAM_TILE_BLOCK_2_END;
        pub const VRAM_TILE_MAP_0_START: Address = VRAM::VRAM_TILE_MAP_0_START;
        pub const VRAM_TILE_MAP_0_END: Address = VRAM::VRAM_TILE_MAP_0_END;
        pub const VRAM_TILE_MAP_1_START: Address = VRAM::VRAM_TILE_MAP_1_START;
        pub const VRAM_TILE_MAP_1_END: Address = VRAM::VRAM_TILE_MAP_1_END;
    }
}

// TODO: COMPLETE MEMORY MAP
