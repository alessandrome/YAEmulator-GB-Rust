use crate::GB::traits::BusDevice;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;
use crate::GB::APU::channels::envelope::{Envelope, EnvelopeDirection};
use crate::GB::APU::{mmio, ApuBusChannel, AudioPeriod, AudioVolume};
use crate::{default_enum_u8_bit_ops, mask_flag_enum_default_impl};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum NoiseNR42Masks {
    Volume = 0b1111_0000,
    EnvDir = 0b0000_1000,
    SweepPace = 0b0000_0111,
}

mask_flag_enum_default_impl!(NoiseNR42Masks);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum NoiseNR43Masks {
    Z = 0b1000_0000,
    LfsrWidth = 0b0001_0000,
}

mask_flag_enum_default_impl!(NoiseNR43Masks);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum NoiseNR44Masks {
    Trigger = 0b1000_0000,
    LengthEnable = 0b0100_0000,
    UpperPeriod = 0b0000_0111,
    UnusedBits = 0b0011_1000,
}

mask_flag_enum_default_impl!(NoiseNR44Masks);

pub struct NoiseChannel {
    nr41: u8, // Initial Length Timer
    nr42: u8, // Volume & Envelope
    nr43: u8, // Frequency & Randomness
    nr44: u8, // Control
    lfsr: u16,
    envelope: Envelope,
}

/**
gate(t) ∈ {0, 1}
volume(t) ∈ {0..=15}
*/
impl NoiseChannel {
    const LFSR_BIT: u8 = 14; // After the right shift
    const LFSR_SHORT_BIT: u8 = 6; // After the right shift
    const LFSR_BIT_SET_MASK: u16 = 1 << 14; // After the right shift
    const LFSR_SHORT_BIT_SET_MASK: u16 = 1 << 6; // After the right shift
    const LFSR_OUTPUT_BIT: u16 = 0;
    const LFSR_OUTPUT_MASK: u16 = 1;

    pub fn new() -> NoiseChannel {
        NoiseChannel {
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            lfsr: 0,
            envelope: Envelope::new(),
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

    #[inline]
    fn lfsr_output_bit(&self) -> u8 {
        (self.lfsr & Self::LFSR_OUTPUT_MASK) as u8 ^ 1
    }

    #[inline]
    pub fn volume(&self) -> u8 {
        (self.nr42 & NoiseNR42Masks::Volume) >> (NoiseNR42Masks::Volume as u8).trailing_zeros()
    }

    pub fn envelope_direction(&self) -> EnvelopeDirection {
        if (self.nr42 & NoiseNR42Masks::EnvDir) != 0 {
            return EnvelopeDirection::Down;
        }
        EnvelopeDirection::Up
    }

    fn trigger(&mut self) {
        self.lfsr = 0xFFFF;
        self.envelope
            .trigger(self.volume(), self.envelope_direction());
    }

    #[inline]
    fn set_nr41(&mut self, val: Byte) {
        self.nr41 = val;
    }

    #[inline]
    fn set_nr42(&mut self, val: Byte) {
        self.nr42 = val;
    }

    #[inline]
    fn set_nr43(&mut self, val: Byte) {
        self.nr43 = val;
    }

    #[inline]
    fn set_nr44(&mut self, val: Byte) {
        self.nr44 = val;
        if (self.nr44 & NoiseNR44Masks::Trigger) != 0 {
            self.trigger();
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

    fn write(&mut self, address: Address, data: Byte) {
        match address {
            mmio::NR41 => self.set_nr41(data),
            mmio::NR42 => self.set_nr42(data),
            mmio::NR43 => self.set_nr43(data),
            mmio::NR44 => self.set_nr44(data),
            _ => unreachable!(),
        }
    }
}

impl ApuBusChannel for NoiseChannel {
    fn tick(&mut self, cycles: u32) {
        todo!()
    }

    fn sample(&self) -> u8 {
        todo!()
    }

    fn output_volume(&self) -> AudioVolume {
        self.volume() * self.lfsr_output_bit()
    }

    fn output_period(&self) -> AudioPeriod {
        todo!()
    }
}
