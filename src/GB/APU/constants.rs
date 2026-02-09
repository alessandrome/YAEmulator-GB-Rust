use crate::GB::APU::mmio::{WAVE_RAM_END, WAVE_RAM_START};

pub const WAVE_RAM_SIZE: u16 = WAVE_RAM_END.as_u16() - WAVE_RAM_START.as_u16() + 1; // Bytes
