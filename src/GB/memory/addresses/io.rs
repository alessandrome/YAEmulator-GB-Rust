use crate::GB::memory::registers;

pub const JOYP: u16 = 0xFF00;
pub const SB: u16 = 0xFF01;
pub const SC: u16 = 0xFF02;
pub const DIV: u16 = registers::DIV;
pub const TIMA: u16 = registers::TIMA;
pub const TMA: u16 = registers::TMA;

todo!("Implement all other I/O registers/addresses");
