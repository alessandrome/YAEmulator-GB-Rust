use std::iter::ExactSizeIterator;
use super::mmio::{WAVE_RAM_RANGE};

pub const WAVE_RAM_SIZE: u16 = WAVE_RAM_RANGE.len() as u16; // Bytes
