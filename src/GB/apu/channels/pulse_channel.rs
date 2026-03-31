use crate::GB::apu::channels::sweep::Sweep;
use crate::GB::types::address::{Address, AddressRangeInclusive};

pub struct PulseChannel {

}

impl PulseChannel {
    pub const APU_NR10_CHANNEL_SWEEP_ADDRESS: Address = Address(0xFF10);
    pub const APU_NR11_CHANNEL_TIMER_ADDRESS: Address = Address(0xFF11);
    pub const APU_NR12_CHANNEL_VOLUME_ADDRESS: Address = Address(0xFF12);
    pub const APU_NR13_CHANNEL_FREQUENCY_ADDRESS: Address = Address(0xFF13);
    pub const APU_NR14_CHANNEL_CONTROL_ADDRESS: Address = Address(0xFF14);
    pub const APU_PULSE_CHANNEL_1_RANGE: AddressRangeInclusive = Self::APU_NR10_CHANNEL_SWEEP_ADDRESS..=Self::APU_NR14_CHANNEL_CONTROL_ADDRESS;
    pub const APU_NR21_CHANNEL_TIMER_ADDRESS: Address = Address(0xFF16);
    pub const APU_NR22_CHANNEL_VOLUME_ADDRESS: Address = Address(0xFF17);
    pub const APU_NR23_CHANNEL_FREQUENCY_ADDRESS: Address = Address(0xFF18);
    pub const APU_NR24_CHANNEL_CONTROL_ADDRESS: Address = Address(0xFF19);
    pub const APU_PULSE_CHANNEL_2_RANGE: AddressRangeInclusive = Self::APU_NR21_CHANNEL_TIMER_ADDRESS..=Self::APU_NR24_CHANNEL_CONTROL_ADDRESS;
}

impl PulseChannel {
    pub fn new(sweep: bool) -> Self {
        Self {}
    }
}
