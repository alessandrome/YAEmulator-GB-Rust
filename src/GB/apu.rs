use crate::GB::bus::{Bus, BusDevice, MmioContext, MmioDevice};
use crate::GB::traits::Tick;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

mod channels;
pub mod constants;
pub mod mmio;
pub mod apu_mmio;

type AudioVolume = u8;
type AudioPeriod = u16;

trait ApuBusChannel: BusDevice {
    fn tick(&mut self, cycles: u32);
    fn sample(&self) -> u8;
    fn output(&self) -> AudioVolume; // Signal intensity from 0 to 15 (4-bit)
}

pub struct APU {
    noise: channels::noise_channel::NoiseChannel,
    div: u16,
}

impl APU {
    pub fn new() -> APU {
        Self {
            noise: channels::noise_channel::NoiseChannel::new(),
            div: u16::MAX,
        }
    }
}

impl Tick for APU {
    fn tick(&mut self, bus: &mut Bus, ctx: &mut MmioContext) {
        todo!()
    }
}

pub struct ApuCtx {
    pub apu: APU,
    pub mmio: ApuCtx
}
