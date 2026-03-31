pub mod sweep;
pub mod envelope;
pub mod pulse_channel;
mod wave_channel;
pub mod noise_channel;

pub(super) use wave_channel::*;
pub(super) use pulse_channel::*;
pub(super) use noise_channel::*;
