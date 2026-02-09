use crate::GB::traits::BusDevice;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;
use crate::GB::APU::{mmio, ApuBusDevice};
use crate::{default_enum_u8_bit_ops, mask_flag_enum_default_impl};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum NoiseNR43Masks {
    Z = 0b1000_0000,
    LfsrWidth = 0b0001_0000,
}

mask_flag_enum_default_impl!(NoiseNR43Masks);

pub struct NoiseChannel {
    nr41: u8, // Initial Length Timer
    nr42: u8, // Volume & Envelope
    nr43: u8, // Frequency & Randomness
    nr44: u8, // Control
    lfsr: u16,
}

impl NoiseChannel {
    const LFSR_BIT: u8 = 14; // After the right shift
    const LFSR_SHORT_BIT: u8 = 6; // After the right shift
    const LFSR_BIT_SET_MASK: u16 = 1 << 14; // After the right shift
    const LFSR_SHORT_BIT_SET_MASK: u16 = 1 << 6; // After the right shift

    pub fn new() -> NoiseChannel {
        NoiseChannel {
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            lfsr: 0,
        }
    }

    #[inline]
    pub fn short_mode(&self) -> bool {
        (self.nr43 & NoiseNR43Masks::LfsrWidth) != 0
    }

    #[inline]
    pub fn short_mode_as_u8(&self) -> u8 {
        self.nr43 & NoiseNR43Masks::LfsrWidth
    }

    fn shift(&mut self) {
        let bit14: u16 = (self.lfsr & 0x01) ^ ((self.lfsr & 0x02) >> 1);
        self.lfsr = ((self.lfsr >> 1) & !Self::LFSR_BIT_SET_MASK) | (bit14 << Self::LFSR_BIT);
        if self.short_mode() {
            self.lfsr =
                (self.lfsr & !Self::LFSR_SHORT_BIT_SET_MASK) | (bit14 << Self::LFSR_SHORT_BIT);
        }
    }
}

impl BusDevice for NoiseChannel {
    fn read(&self, address: Address) -> Byte {
        match address {
            mmio::NR41 => self.nr41,
            mmio::NR42 => self.nr42,
            mmio::NR43 => self.nr43,
            mmio::NR44 => self.nr44,
            _ => unreachable!(),
        }
    }

    fn write(&self, address: Address, data: Byte) {
        match address {
            mmio::NR41 => todo!(),
            mmio::NR42 => todo!(),
            mmio::NR43 => todo!(),
            mmio::NR44 => todo!(),
            _ => unreachable!(),
        }
    }
}

impl ApuBusDevice for NoiseChannel {
    fn tick(cycles: u32) {
        todo!()
    }

    fn sample() -> u8 {
        todo!()
    }
}
