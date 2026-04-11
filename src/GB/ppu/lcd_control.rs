use crate::GB::ppu::tile::{TileDataArea, TileMapArea};
use crate::{default_enum_u8_bit_ops, mask_flag_enum_default_impl};
use crate::GB::types::Byte;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum LCDCMasks {
    BgWinEnabled = 0b0000_0001,
    ObjEnabled = 0b0000_0010,
    ObjSize = 0b0000_0100,
    BgTileMapArea = 0b0000_1000,
    BgWinTilesArea = 0b0001_0000,
    WinEnabled = 0b0010_0000,
    WinTileMapArea = 0b0100_0000,
    LcdEnabled = 0b1000_0000,
}

mask_flag_enum_default_impl!(LCDCMasks);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ObjSize {
    Single = 8,
    Double = 16,
}

/// High-Level view of a LCDC register
pub struct LCDC {
    pub lcd_enabled: bool,
    pub window_tile_map: TileMapArea,
    pub window_enabled: bool,
    pub bg_window_tile_area: TileDataArea,
    pub bg_tile_map: TileMapArea,
    pub obj_size: ObjSize,
    pub obj_enabled: bool,
    pub bg_win_enabled: bool,
}

impl LCDC {
    pub fn from_byte(lcdc: Byte) -> Self {
        let obj_size;
        if (lcdc & LCDCMasks::ObjSize) != 0 {
            obj_size = ObjSize::Double
        } else {
            obj_size = ObjSize::Single
        }

        let window_tile_map;
        if (lcdc & LCDCMasks::WinTileMapArea) != 0 {
            window_tile_map = TileMapArea::MapBlock1;
        } else {
            window_tile_map = TileMapArea::MapBlock0;
        }

        let bg_window_tile_area;
        if (lcdc & LCDCMasks::BgWinTilesArea) != 0 {
            bg_window_tile_area = TileDataArea::DataBlock01;
        } else {
            bg_window_tile_area = TileDataArea::DataBlock12;
        }

        let bg_tile_map;
        if (lcdc & LCDCMasks::BgTileMapArea) != 0 {
            bg_tile_map = TileMapArea::MapBlock1;
        } else {
            bg_tile_map = TileMapArea::MapBlock0;
        }

        Self {
            lcd_enabled: (lcdc & LCDCMasks::LcdEnabled) != 0,
            window_tile_map,
            window_enabled: (lcdc & LCDCMasks::WinEnabled) != 0,
            bg_window_tile_area,
            bg_tile_map,
            obj_size,
            obj_enabled: (lcdc & LCDCMasks::ObjEnabled) != 0,
            bg_win_enabled: (lcdc & LCDCMasks::BgWinEnabled) != 0,
        }
    }
}

impl From<Byte> for LCDC {
    fn from(value: Byte) -> Self {
        Self::from_byte(value)
    }
}
