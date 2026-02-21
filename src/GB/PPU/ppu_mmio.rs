use crate::GB::memory::vram::VRAM;
use crate::GB::types::address::{Address, AddressRangeInclusive};
use crate::GB::types::Byte;
use super::PPU;

pub struct PpuMmio {
    vram: VRAM,
    lcdc: Byte,
    stat: Byte,
    scy: Byte,
    scx: Byte,
    ly: Byte,
    lyc: Byte,
    bgp: Byte,
    obp0: Byte,
    obp1: Byte,
    wy: Byte,
    wx: Byte,
}

impl PpuMmio {
    pub const VRAM_BLOCK_0_START: Address = Address(0x8000);
    pub const VRAM_BLOCK_0_END: Address = Address(0x87FF);
    pub const VRAM_BLOCK_0_RANGE: AddressRangeInclusive = Self::VRAM_BLOCK_0_START..=Self::VRAM_BLOCK_0_END;
    pub const VRAM_BLOCK_1_START: Address = Address(0x8800);
    pub const VRAM_BLOCK_1_END: Address = Address(0x8FFF);
    pub const VRAM_BLOCK_1_RANGE: AddressRangeInclusive = Self::VRAM_BLOCK_1_START..=Self::VRAM_BLOCK_1_END;
    pub const VRAM_BLOCK_2_START: Address = Address(0x9000);
    pub const VRAM_BLOCK_2_END: Address = Address(0x97FF);
    pub const VRAM_BLOCK_2_RANGE: AddressRangeInclusive = Self::VRAM_BLOCK_2_START..=Self::VRAM_BLOCK_2_END;
    pub const VRAM_TILE_MAP_0_START: Address = Address(0x9800);
    pub const VRAM_TILE_MAP_0_END: Address = Address(0x9BFF);
    pub const VRAM_TILE_MAP_0_RANGE: AddressRangeInclusive = Self::VRAM_TILE_MAP_0_START..=Self::VRAM_TILE_MAP_0_END;
    pub const VRAM_TILE_MAP_1_START: Address = Address(0x9C00);
    pub const VRAM_TILE_MAP_1_END: Address = Address(0x9FFF);
    pub const VRAM_TILE_MAP_1_RANGE: AddressRangeInclusive = Self::VRAM_TILE_MAP_1_START..=Self::VRAM_TILE_MAP_1_END;

    pub const LCDC_ADDRESS: Address = Address(0xFF40);
    pub const STAT_ADDRESS: Address = Address(0xFF41);
    pub const SCY_ADDRESS: Address = Address(0xFF42);
    pub const SCX_ADDRESS: Address = Address(0xFF43);
    pub const LY_ADDRESS: Address = Address(0xFF44);
    pub const LYC_ADDRESS: Address = Address(0xFF45);
    pub const BGP_ADDRESS: Address = Address(0xFF47);
    pub const OBP0_ADDRESS: Address = Address(0xFF48);
    pub const OBP1_ADDRESS: Address = Address(0xFF49);
    pub const WY_ADDRESS: Address = Address(0xFF4A);
    pub const WX_ADDRESS: Address = Address(0xFF4B);
}

impl PpuMmio {
    pub fn new() -> Self {
        Self {
            vram: VRAM::new(),
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,
        }
    }

    pub fn tick(&mut self) {
        self.ly = (self.ly + 1) % PPU::SCAN_LINES;
    }
}
