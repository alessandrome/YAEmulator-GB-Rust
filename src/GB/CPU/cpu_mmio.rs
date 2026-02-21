use crate::GB::bus::BusDevice;
use crate::GB::memory::hram::HRAM;
use super::registers::interrupt_registers::InterruptRegisters;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

pub struct CpuMmio {
    interrupt_registers: InterruptRegisters,
    hram: HRAM,
}

impl CpuMmio {
    pub fn new() -> Self {
        Self {
            interrupt_registers: InterruptRegisters::new(),
            hram: HRAM::new(),
        }
    }

    pub fn interrupt_registers(&self) -> &InterruptRegisters {
        &self.interrupt_registers
    }

    pub fn hram(&self) -> &HRAM {
        &self.hram
    }
}

impl BusDevice for CpuMmio {
    fn read(&self, address: Address) -> Byte {
        match address {
            address if HRAM::HRAM_ADDRESS_RANGE.contains(&address) => self.hram.read(address),
            InterruptRegisters::IE_ADDRESS | InterruptRegisters::IF_ADDRESS => self.interrupt_registers.read(address),
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: Address, data: Byte) {
        match address {
            address if HRAM::HRAM_ADDRESS_RANGE.contains(&address) => self.hram.write(address, data),
            InterruptRegisters::IE_ADDRESS | InterruptRegisters::IF_ADDRESS => self.interrupt_registers.write(address, data),
            _ => unreachable!(),
        }
    }
}

impl Default for CpuMmio {
    fn default() -> Self {
        Self::new()
    }
}
