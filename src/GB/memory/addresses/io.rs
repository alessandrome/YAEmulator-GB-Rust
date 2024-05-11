use crate::GB::memory::registers;

pub const JOYP: usize = 0xFF00;
pub const SB: usize = 0xFF01;
pub const SC: usize = 0xFF02;
pub const DIV: usize = registers::DIV as usize;
pub const TIMA: usize = registers::TIMA as usize;
pub const TMA: usize = registers::TMA as usize;

// TODO: "Implement all other I/O registers/addresses"
