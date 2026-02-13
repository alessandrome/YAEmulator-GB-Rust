use crate::GB::APU::constants::{FRAME_SEQUENCER_FREQUENCY, FRAME_SEQUENCER_TICKS};
use std::cmp::{max, min};
use super::super::AudioVolume;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum EnvelopeDirection {
    Down = 0,
    Up = 1,
}

pub struct Envelope {
    volume: u8,
    direction: EnvelopeDirection,
}

impl Envelope {
    const ENVELOPE_TICKS: u16 = (FRAME_SEQUENCER_TICKS / 4) as u16;
    const ENVELOPE_FREQUENCY: u16 = (FRAME_SEQUENCER_FREQUENCY / 4) as u16; // Hz
    pub fn new() -> Envelope {
        Envelope {
            volume: 0,
            direction: EnvelopeDirection::Down,
        }
    }

    pub fn trigger(&mut self, volume: u8, direction: EnvelopeDirection) {
        self.volume = volume;
        self.direction = direction;
    }

    pub fn tick(&mut self) {
        match self.direction {
            EnvelopeDirection::Down => self.volume = self.volume.saturating_sub(1),
            EnvelopeDirection::Up => self.volume = min(self.volume.saturating_add(1), 15),
        }
    }

    #[inline]
    pub fn volume(&self) -> AudioVolume {
        self.volume
    }
}
