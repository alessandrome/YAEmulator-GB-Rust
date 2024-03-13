use crate::GB::RAM as RAM;
use crate::GB::registers as registers;
use crate::GB::instructions as instructions;

pub struct CPU {
    registers: registers::Registers,
    ram: RAM::RAM,
}
