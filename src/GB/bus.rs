mod bus_device;

use crate::GB::apu::APU;
use crate::GB::cartridge::ROM;
use crate::GB::memory::wram::WRAM;
use crate::GB::memory::hram::HRAM;
use crate::GB::cpu::CPU;
use crate::GB::cpu::registers::interrupt_registers::InterruptRegisters;
use crate::GB::ppu::PPU;
use super::timer::TimerRegisters;
pub(crate) use bus_device::{BusDevice, MmioDevice, MemoryDevice};
use crate::GB::cpu::cpu_mmio::CpuMmio;
use crate::GB::dma::DMA;
use crate::GB::dma::dma_mmio::DmaMmio;
use crate::GB::ppu::ppu_mmio::PpuMmio;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

pub struct MmioContext<'a> {
    pub cpu_mmio: &'a mut CpuMmio,
    pub ppu_mmio: &'a mut PpuMmio,
    // timer: &'a mut TimerRegisters,
    // pub apu: &'a mut APU,
    pub dma_mmio: &'a mut DmaMmio,
    pub wram: &'a mut WRAM,
}

pub struct Bus {}

impl Bus {
    pub fn new() -> Self {
        Self {}
    }
}

impl Bus {
    pub fn read(&self, ctx: &MmioContext, address: Address) -> Byte {
        match address {
            address if HRAM::HRAM_ADDRESS_RANGE.contains(&address) => {
                ctx.cpu_mmio.read(address)
            }
            InterruptRegisters::IE_ADDRESS | InterruptRegisters::IF_ADDRESS => {
                ctx.cpu_mmio.read(address)
            }
            DMA::DMA_SOURCE_ADDRESS => {
                ctx.dma_mmio.read(address)
            }
            address if WRAM::WRAM_ADDRESS_RANGE.contains(&address) => {
                ctx.wram.read(address)
            }
            _ => todo!("Implement all other ranges"),
        }
    }

    pub fn write(&mut self, ctx: &mut MmioContext, address: Address, data: Byte) {
        match address {
            address if HRAM::HRAM_ADDRESS_RANGE.contains(&address) => {
                ctx.cpu_mmio.write(address, data)
            }
            InterruptRegisters::IE_ADDRESS | InterruptRegisters::IF_ADDRESS => {
                ctx.cpu_mmio.write(address, data)
            }
            DMA::DMA_SOURCE_ADDRESS => {
                ctx.dma_mmio.write(address, data)
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
