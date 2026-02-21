use crate::GB::memory::addresses::OAM_AREA_ADDRESS as OAM_ADDRESS;

pub const LCDC_ADDRESS: usize = 0xFF40;
pub const LCD_STAT_ADDRESS: usize = 0xFF41;
pub const SCY_ADDRESS: usize = 0xFF42;
pub const SCX_ADDRESS: usize = 0xFF43;
pub const LY_ADDRESS: usize = 0xFF44;
pub const LYC_ADDRESS: usize = 0xFF45;
pub const PALETTE_ADDRESS: usize = 0xFF47;
pub const OBP0_ADDRESS: usize = 0xFF48;
pub const OBP1_ADDRESS: usize = 0xFF49;
pub const WY_ADDRESS: usize = 0xFF4A;
pub const WX_ADDRESS: usize = 0xFF4B;
pub const BG_DATA_1_ADDRESS: usize = 0x9800;
pub const BG_DATA_2_ADDRESS: usize = 0x9C00;
pub const OAM_AREA_ADDRESS: usize = OAM_ADDRESS;
