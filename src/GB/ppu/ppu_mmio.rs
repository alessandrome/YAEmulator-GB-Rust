use crate::GB::bus::BusDevice;
use crate::GB::memory::vram::VRAM;
use crate::GB::ppu::lcd_control::{LCDCMasks, ObjSize, TileDataArea, TileMapArea, LCDC};
use crate::GB::ppu::lcd_stat::{LCDStatMasks, LcdStat};
use crate::GB::types::address::{Address, AddressRangeInclusive};
use crate::GB::types::Byte;
use super::ppu_mode::PpuMode;
use super::PPU;

pub struct PpuMmio {
    ppu_mode: PpuMode,
    prev_ppu_mode: PpuMode, // PPU Mode in tha last T-Cycle tick
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
            prev_ppu_mode: PpuMode::HBlank,
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
    pub fn ppu_mode(&self) -> PpuMode {
        self.ppu_mode
    }

    #[inline]
    pub fn prev_ppu_mode(&self) -> PpuMode {
        self.prev_ppu_mode
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
    /// Increment LY Register
    pub fn next_ly(&mut self) {
        self.ly = (self.ly + 1) % PPU::SCAN_LINES as u8;
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
    pub fn vram(&self) -> &VRAM {
        &self.vram
    }

    #[inline]
    pub fn vram_mut(&mut self) -> &mut VRAM {
        &mut self.vram
    }

    #[inline]
    pub fn stat_view(&self) -> LcdStat {
        LcdStat {
            lyc_interrupt_enabled: (self.stat & LCDStatMasks::LYCInterrupt) != 0,
            hblank_interrupt_enabled: (self.stat & LCDStatMasks::Mode0Interrupt) != 0,
            vblank_interrupt_enabled: (self.stat & LCDStatMasks::Mode1Interrupt) != 0,
            oam_scan_interrupt_enabled: (self.stat & LCDStatMasks::Mode2Interrupt) != 0,
            lcy_eq_ly: self.ly == self.lyc,
            ppu_mode: self.ppu_mode,
        }
    }

    #[inline]
    pub fn lcdc_view(&self) -> LCDC {
        let obj_size;
        if (self.lcdc & LCDCMasks::ObjSize) != 0 {
            obj_size = ObjSize::Single
        } else {
            obj_size = ObjSize::Double
        }

        let window_tile_map;
        if (self.lcdc & LCDCMasks::WinTileMapArea) != 0 {
            window_tile_map = TileMapArea::MapBlock1;
        } else {
            window_tile_map = TileMapArea::MapBlock0;
        }

        let bg_window_tile_area;
        if (self.lcdc & LCDCMasks::BgWinTilesArea) != 0 {
            bg_window_tile_area = TileDataArea::DataBlock01;
        } else {
            bg_window_tile_area = TileDataArea::DataBlock12;
        }

        let bg_tile_map;
        if (self.lcdc & LCDCMasks::WinTileMapArea) != 0 {
            bg_tile_map = TileMapArea::MapBlock1;
        } else {
            bg_tile_map = TileMapArea::MapBlock0;
        }

        LCDC {
            lcd_enabled: (self.lcdc & LCDCMasks::LcdEnabled) != 0,
            window_tile_map,
            window_enabled: (self.lcdc & LCDCMasks::WinEnabled) != 0,
            bg_window_tile_area,
            bg_tile_map,
            obj_size,
            obj_enabled: (self.lcdc & LCDCMasks::ObjEnabled) != 0,
            bg_win_enabled: (self.lcdc & LCDCMasks::BgWinEnabled) != 0,
        }
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
