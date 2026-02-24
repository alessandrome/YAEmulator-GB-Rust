use crate::{mask_flag_enum_default_impl, default_enum_u8_bit_ops};

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TileMapArea {
    MapBlock0,
    MapBlock1,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TileDataArea {
    DataBlock01,
    DataBlock12,
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
