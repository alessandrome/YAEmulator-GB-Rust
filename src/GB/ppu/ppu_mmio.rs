use crate::GB::bus::BusDevice;
use crate::GB::memory::vram::VRAM;
use crate::GB::types::address::{Address, AddressRangeInclusive};
use crate::GB::types::Byte;
use super::ppu_mode::PpuMode;
use super::PPU;

pub struct PpuMmio {
    ppu_mode: PpuMode,
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
            ppu_mode: PpuMode::OAMScan,
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

    #[inline]
    pub fn tick(&mut self) {
        self.ly = (self.ly + 1) % (PPU::SCAN_LINES as u8);
    }

    #[inline]
    pub fn ppu_mode(&self) -> PpuMode {
        self.ppu_mode
    }

    #[inline]
    pub fn next_mode(&mut self) {
        match self.ppu_mode {
            PpuMode::OAMScan => self.ppu_mode = PpuMode::Drawing,
            PpuMode::Drawing => self.ppu_mode = PpuMode::HBlank,
            PpuMode::HBlank => {
                if self.ly >= PPU::SCREEN_LINES as u8 {
                    self.ppu_mode = PpuMode::VBlank
                } else {
                    self.ppu_mode = PpuMode::OAMScan
                }
            }
            PpuMode::VBlank => self.ppu_mode = PpuMode::OAMScan,
        }
    }

    #[inline]
    pub fn lcdc(&self) -> Byte {
        self.lcdc
    }

    #[inline]
    pub fn stat(&self) -> Byte {
        self.stat
    }

    #[inline]
    pub fn scy(&self) -> Byte {
        self.scy
    }

    #[inline]
    pub fn scx(&self) -> Byte {
        self.scx
    }

    #[inline]
    pub fn ly(&self) -> Byte {
        self.ly
    }

    #[inline]
    pub fn lyc(&self) -> Byte {
        self.ly
    }

    #[inline]
    pub fn bgp(&self) -> Byte {
        self.bgp
    }

    #[inline]
    pub fn obp0(&self) -> Byte {
        self.obp0
    }

    #[inline]
    pub fn obp1(&self) -> Byte {
        self.obp1
    }

    #[inline]
    pub fn wy(&self) -> Byte {
        self.wy
    }

    #[inline]
    pub fn wx(&self) -> Byte {
        self.wx
    }

    #[inline]
    pub fn wram(&self) -> &VRAM {
        &self.vram
    }

    #[inline]
    pub fn wram_mut(&mut self) -> &mut VRAM {
        &mut self.vram
    }
}

impl BusDevice for PpuMmio {
    fn read(&self, address: Address) -> Byte {
        match address {
            address if VRAM::VRAM_ADDRESS_RANGE.contains(&address) => self.vram.read(address),
            Self::LCDC_ADDRESS => self.lcdc,
            Self::STAT_ADDRESS => self.stat,
            Self::SCY_ADDRESS => self.scy,
            Self::SCX_ADDRESS => self.scx,
            Self::LY_ADDRESS => self.ly,
            Self::LYC_ADDRESS => self.lyc,
            Self::BGP_ADDRESS => self.bgp,
            Self::OBP0_ADDRESS => self.obp0,
            Self::OBP1_ADDRESS => self.obp1,
            Self::WY_ADDRESS => self.wy,
            Self::WX_ADDRESS => self.wx,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: Address, data: Byte) {
        match address {
            address if VRAM::VRAM_ADDRESS_RANGE.contains(&address) => self.vram.write(address, data),
            Self::LCDC_ADDRESS => self.lcdc = data,
            Self::STAT_ADDRESS => self.stat = data,
            Self::SCY_ADDRESS => self.scy = data,
            Self::SCX_ADDRESS => self.scx = data,
            Self::LY_ADDRESS => (), // Read-Only
            Self::LYC_ADDRESS => self.lyc = data,
            Self::BGP_ADDRESS => self.bgp = data,
            Self::OBP0_ADDRESS => self.obp0 = data,
            Self::OBP1_ADDRESS => self.obp1 = data,
            Self::WY_ADDRESS => self.wy = data,
            Self::WX_ADDRESS => self.wx = data,
            _ => unreachable!(),
        }
    }
}
