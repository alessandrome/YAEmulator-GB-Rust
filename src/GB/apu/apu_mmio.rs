use crate::GB::bus::BusDevice;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

pub struct ApuMmio {
    // Todo: add APU memory-mapped mapped elements
    // sqr0: SquareChannel
    // sqr1: SquareChannel
    // wave: WaveChannel
    // noise: NoiseChannel
}

impl ApuMmio {
    pub fn new() -> Self {
        Self {}
    }
}

impl BusDevice for ApuMmio {
    fn read(&self, address: Address) -> Byte {
        todo!()
    }

    fn write(&mut self, address: Address, data: Byte) {
        todo!()
    }
}
