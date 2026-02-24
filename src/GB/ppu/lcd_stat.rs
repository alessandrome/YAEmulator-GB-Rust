use crate::{mask_flag_enum_default_impl, default_enum_u8_bit_ops};
use super::ppu_mode::PpuMode;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum LCDStatMasks {
    PPUMode = 0b0000_0011,
    LYCeLY = 0b0000_0100,
    Mode0Interrupt = 0b0000_1000,   // HBlank Mode
    Mode1Interrupt = 0b0001_0000,   // VBlank Mode
    Mode2Interrupt = 0b0010_0000,   // OAM Scan Mode
    LYCInterrupt = 0b0100_0000,
}

mask_flag_enum_default_impl!(LCDStatMasks);

/// High-Level view of a STAT register
pub struct LcdStat {
    pub lyc_interrupt_enabled: bool,
    pub hblank_interrupt_enabled: bool,
    pub vblank_interrupt_enabled: bool,
    pub oam_scan_interrupt_enabled: bool,
    pub lcy_eq_ly: bool,
    pub ppu_mode: PpuMode,
}
