mod bus_device;

pub(crate) use bus_device::{BusDevice, MmioDevice, MemoryDevice};
use crate::GB::memory::wram::WRAM;
use crate::GB::memory::hram::HRAM;
use crate::GB::cpu::registers::interrupt_registers::InterruptRegisters;
use crate::GB::apu::apu_mmio::ApuMmio;
use crate::GB::cpu::cpu_mmio::CpuMmio;
use crate::GB::dma::DMA;
use crate::GB::dma::dma_mmio::DmaMmio;
use crate::GB::memory::oam_memory::OamMemory;
use crate::GB::ppu::ppu_mmio::PpuMmio;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

pub struct MmioContext<'a> {
    pub cpu_mmio: &'a mut CpuMmio,
    pub ppu_mmio: &'a mut PpuMmio,
    pub apu_mmio: &'a mut ApuMmio,
    pub dma_mmio: &'a mut DmaMmio,
    pub oam_mmio: &'a mut OamMemory,
    pub wram_mmio: &'a mut WRAM,
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
                ctx.wram_mmio.read(address)
            }
            address if OamMemory::OAM_ADDRESS_RANGE.contains(&address) => {
                ctx.oam_mmio.read(address)
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
                ctx.wram_mmio.write(address, data)
            }
            address if OamMemory::OAM_ADDRESS_RANGE.contains(&address) => {
                ctx.oam_mmio.write(address, data)
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
