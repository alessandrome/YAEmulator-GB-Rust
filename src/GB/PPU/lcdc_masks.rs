use crate::mask_flag_enum_default_impl;

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
