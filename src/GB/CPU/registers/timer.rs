use crate::{define_enum_u8, default_enum_u8_bit_ops, mask_flag_enum_default_impl};
use crate::utils::{falling_edge as utils, falling_edge};

pub const M256_CLOCK_CYCLES: u64 = 256;
pub const M4_CLOCK_CYCLES: u64 = 4;
pub const M16_CLOCK_CYCLES: u64 = 16;
pub const M64_CLOCK_CYCLES: u64 = 64;

pub const M256_CLOCK_MODE: u8 = 0b00;
pub const M4_CLOCK_MODE: u8 = 0b01;
pub const M16_CLOCK_MODE: u8 = 0b10;
pub const M64_CLOCK_MODE: u8 = 0b11;

define_enum_u8! {
    pub TACMask {
        Enabled = 0b0000_0100,
        TimerClock = 0b0000_0011
    }
}
mask_flag_enum_default_impl!(TACMask);

define_enum_u8! {
    pub TACClock {
        M256 =  M256_CLOCK_MODE,
        M4 =    M4_CLOCK_MODE,
        M16 =   M16_CLOCK_MODE,
        M64 =   M64_CLOCK_MODE
    }
}
mask_flag_enum_default_impl!(TACClock);

impl TACClock {
    pub fn get_timer_bit(&self) -> u8 {
        match self {
            Self::M256 => 9,  // 4096 Hz
            Self::M4 => 3,  // 262144 Hz
            Self::M16 => 5,  // 65536 Hz
            Self::M64 => 7,  // 16384 Hz
            _ => unreachable!(),
        }
    }
    
    pub fn get_timer_bit_from_u8(bits: u8) -> u8 {
        match bits {
            M256_CLOCK_MODE => 9,  // 4096 Hz
            M4_CLOCK_MODE => 3,  // 262144 Hz
            M16_CLOCK_MODE => 5,  // 65536 Hz
            M64_CLOCK_MODE => 7,  // 16384 Hz
            _ => unreachable!(),
        }
    }
}

pub struct TimerRegisters {
    cycles: u16,
    div_counter: u16, // DIV is composed of 2 8bit-subregister, the lower one is not visible to dev while the upper one is the one commonly named DIV timer
    tima: u8,
    tma: u8,
    tac: u8,
}

impl TimerRegisters {
    pub fn new() -> Self {
        Self {
            cycles: 0,
            div_counter: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    pub fn tick(&mut self) {
        self.cycles = self.cycles.wrapping_add(1);
        let old_div_counter = self.div_counter;
        let old_tma = self.tma;
        self.div_counter = self.div_counter.wrapping_add(1); // Increment DIV
        if falling_edge(old_div_counter, self.div_counter, TACClock::get_timer_bit_from_u8(self.tac & TACMask::TimerClock)) {
            // Change/Increment TMA as needed
            if (self.tima == 0xFF) {
                self.tima = self.tma;
            } else {
                self.tima += 1;
            }
        }
    }

    pub fn div(&self) -> u8 {
        (self.div_counter >> 8) as u8
    }

    pub fn reset_div(&mut self) {
        self.div_counter = 0;
    }

    pub fn is_tma_active(&self) -> bool {
        self.tac & TACMask::Enabled != 0
    }

    pub fn tima(&self) -> u8 {
        self.tima
    }

    pub fn tma(&self) -> u8 {
        self.tma
    }

    pub fn tac(&self) -> u8 {
        self.tac
    }

    pub fn set_tma(&mut self, val: u8) {
        self.tma = val;
    }

    pub fn set_tima(&mut self, val: u8) {
        self.tima = val;
    }

    pub fn set_tac_info(&mut self, enabled: bool, clock_mode: TACClock) {
        self.tac = clock_mode as u8 | (if enabled {0x01} else {0x00} << 2);
    }

    pub fn set_tac(&mut self, val: u8) {
        self.tac = val;
    }
}
