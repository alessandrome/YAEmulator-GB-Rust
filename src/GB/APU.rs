use crate::GB::traits::{BusDevice, MmioDevice};
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

mod channels;
pub mod constants;
pub mod mmio;

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
            div: u16::MAX_VALUE,
        }
    }
    pub fn tick(&mut self) {
        self.div = self.div.wrapping_add(1);
    }
}

impl BusDevice for APU {
    fn read(&self, address: Address) -> Byte {
        todo!()
    }

    fn write(&mut self, address: Address, data: Byte) {
        match address {
            mmio::NR10 => {}
            a if mmio::NOISE_CHANNEL_RANGE.contains(&a) => {
                self.noise.write(address, data);
            }
            a if mmio::MATER_VOLUME_RANGE.contains(&a) => {}
            a if mmio::WAVE_RAM_RANGE.contains(&a) => {}
            _ => unimplemented!(),
        }
    }
}

impl MmioDevice for APU {}
