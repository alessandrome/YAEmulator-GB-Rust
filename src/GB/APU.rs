use crate::GB::APU::mmio::WAVE_RAM_END;
use crate::GB::traits::{BusDevice, MmioDevice};
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

pub mod constants;
pub mod mmio;
mod channels;

trait ApuBusDevice: BusDevice {
    fn tick(cycles: u32);
    fn sample() -> u8;
}

pub struct APU {
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
    nr20: u8,
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
    nr40: u8,
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,
    nr50: u8,
    nr51: u8,
    nr52: u8,
    nr53: u8,
    nr54: u8,
    wave: [u8; 16],
}

impl APU {
    pub fn new() -> APU {
        Self {
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            nr20: 0,
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,
            nr40: 0,
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            nr50: 0,
            nr51: 0,
            nr52: 0,
            nr53: 0,
            nr54: 0,
            wave: [0; 16],
        }
    }
}

impl BusDevice for APU {
    fn read(&self, address: Address) -> Byte {
        todo!()
    }

    fn write(&self, address: Address, data: Byte) {
        match address {
            mmio::NR10 => {}
            a if mmio::WAVE_RAM_RANGE.contains(&a) => {}
            _ => unimplemented!(),
        }
    }
}

impl MmioDevice for APU {}
