mod bus_device;

use crate::GB::APU::APU;
use crate::GB::cartridge::ROM;
use crate::GB::memory::wram::WRAM;
use crate::GB::memory::hram::HRAM;
use crate::GB::CPU::CPU;
use crate::GB::CPU::registers::interrupt_registers::InterruptRegisters;
use crate::GB::PPU::PPU;
use super::timer::TimerRegisters;
pub(crate) use bus_device::{BusDevice, MmioDevice, MemoryDevice};
use crate::GB::DMA::DMA;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

pub struct BusContext<'a> {
    pub cpu: &'a mut CPU<'a>,
    // timer: &'a mut TimerRegisters,
    pub apu: &'a mut APU,
    pub dma: &'a mut DMA,
    pub wram: &'a mut WRAM,
}

pub struct Bus {}

impl Bus {
    pub fn new() -> Self {
        Self {}
    }
}

impl Bus {
    pub fn read(&self, ctx: &BusContext, address: Address) -> Byte {
        match address {
            address if HRAM::HRAM_ADDRESS_RANGE.contains(&address) => {
                ctx.cpu.read(address)
            }
            InterruptRegisters::IE_ADDRESS | InterruptRegisters::IF_ADDRESS => {
                ctx.cpu.read(address)
            }
            address if WRAM::WRAM_ADDRESS_RANGE.contains(&address) => {
                ctx.wram.read(address)
            }
            _ => todo!("Implement all other ranges"),
        }
    }

    pub fn write(&mut self, ctx: &mut BusContext, address: Address, data: Byte) {
        match address {
            address if HRAM::HRAM_ADDRESS_RANGE.contains(&address) => {
                ctx.cpu.write(address, data)
            }
            InterruptRegisters::IE_ADDRESS | InterruptRegisters::IF_ADDRESS => {
                ctx.cpu.write(address, data)
            }
            address if WRAM::WRAM_ADDRESS_RANGE.contains(&address) => {
                ctx.wram.write(address, data)
            }
            _ => todo!("Implement all other ranges"),
        }
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}
