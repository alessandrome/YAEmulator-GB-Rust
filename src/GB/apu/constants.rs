use crate::GB::apu::mmio::{WAVE_RAM_END, WAVE_RAM_START};

pub const WAVE_RAM_SIZE: u16 = WAVE_RAM_END.as_u16() - WAVE_RAM_START.as_u16() + 1; // Bytes
pub const FRAME_SEQUENCER_TICKS: u32  = 8192;  // GB CPU Clock / Frame Sequencer Frequency
pub const FRAME_SEQUENCER_STEP_TICKS: u32  = FRAME_SEQUENCER_TICKS / 8;  // GB CPU Clock / Frame Sequencer Frequency
pub const FRAME_SEQUENCER_FREQUENCY: u32 = 512;  // Hz -> GB CPU Clock/Frame Sequencer Cycle ticks
pub const PERIOD_BITS: u8 = 11;
pub const PERIOD_BITS_MASK: u16 = 0xFF >> (16 - PERIOD_BITS);
