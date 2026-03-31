use crate::GB::bus::BusDevice;
use crate::GB::types::address::{Address, AddressRangeInclusive};
use crate::GB::types::Byte;
use super::channels;

pub struct ApuMmio {
    // Todo: add APU memory-mapped mapped elements
    sqr0: channels::PulseChannel,
    sqr1: channels::PulseChannel,
    wave: channels::WaveChannel,
    noise: channels::NoiseChannel,
}

impl ApuMmio {
    pub const APU_NR50_MASTER_VOLUME_ADDRESS: Address = Address(0xFF24);
    pub const APU_NR51_SOUND_PANNING_ADDRESS: Address = Address(0xFF25);
    pub const APU_NR52_MASTER_CONTROL_ADDRESS: Address = Address(0xFF26);
    pub const APU_REGISTERS_RANGE: AddressRangeInclusive = channels::PulseChannel::APU_NR10_CHANNEL_SWEEP_ADDRESS..=Self::APU_NR52_MASTER_CONTROL_ADDRESS;
    pub const APU_WAVE_RANGE: AddressRangeInclusive = channels::WaveChannel::APU_WAVE_PATTERN_RANGE;
}

impl ApuMmio {
    pub fn new() -> Self {
        Self {
            sqr0: channels::PulseChannel::new(true),
            sqr1: channels::PulseChannel::new(false),
            wave: channels::WaveChannel::new(),
            noise: channels::NoiseChannel::new(),
        }
    }
}

impl BusDevice for ApuMmio {
    fn read(&self, address: Address) -> Byte {
        // TODO: Add all APU addresses
        0xFF
    }

    fn write(&mut self, address: Address, data: Byte) {
        // TODO: Add all APU addresses
    }
}
