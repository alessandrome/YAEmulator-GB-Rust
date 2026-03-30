mod bus_device;

pub(crate) use bus_device::{BusDevice, MmioDevice, MemoryDevice};
use crate::GB::memory::wram::WRAM;
use crate::GB::memory::hram::HRAM;
use crate::GB::cpu::registers::interrupt_registers::InterruptRegisters;
use crate::GB::apu::apu_mmio::ApuMmio;
use crate::GB::cartridge::Cartridge;
use crate::GB::cpu::cpu_mmio::CpuMmio;
use crate::GB::dma::DMA;
use crate::GB::dma::dma_mmio::DmaMmio;
use crate::GB::joypad::Joypad;
use crate::GB::memory::oam_memory::OamMemory;
use crate::GB::ppu::ppu_mmio::PpuMmio;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

pub struct MmioContextWrite<'a> {
    pub cpu_mmio: &'a mut CpuMmio,
    pub rom_mmio: &'a mut Option<Cartridge>,
    pub ppu_mmio: &'a mut PpuMmio,
    pub apu_mmio: &'a mut ApuMmio,
    pub dma_mmio: &'a mut DmaMmio,
    pub oam_mmio: &'a mut OamMemory,
    pub wram_mmio: &'a mut WRAM,
    pub joypad: &'a mut Joypad,
}

pub struct MmioContextRead<'a> {
    pub cpu_mmio: &'a CpuMmio,
    pub rom_mmio: &'a Option<Cartridge>,
    pub ppu_mmio: &'a PpuMmio,
    pub apu_mmio: &'a ApuMmio,
    pub dma_mmio: &'a DmaMmio,
    pub oam_mmio: &'a OamMemory,
    pub wram_mmio: &'a WRAM,
    pub joypad: &'a Joypad,
}

impl<'a> MmioContextWrite<'a> {
    fn as_read(&'a self) -> MmioContextRead<'a> {
        MmioContextRead {
            cpu_mmio: &self.cpu_mmio,
            rom_mmio: &self.rom_mmio,
            ppu_mmio: &self.ppu_mmio,
            apu_mmio: &self.apu_mmio,
            dma_mmio: &self.dma_mmio,
            oam_mmio: &self.oam_mmio,
            wram_mmio: &self.wram_mmio,
            joypad: &self.joypad,
        }
    }
}

impl<'a> From<&'a MmioContextWrite<'a>> for MmioContextRead<'a> {
    fn from(ctx: &'a MmioContextWrite<'a>) -> Self {
        ctx.as_read()
    }
}

pub struct Bus {}

impl Bus {
    pub fn new() -> Self {
        Self {}
    }
}

impl Bus {
    pub fn read(&self, ctx: &MmioContextRead, address: Address) -> Byte {
        match address {
            address if Cartridge::CART_ROM_RANGE_ADDRESS.contains(&address) => {
                match ctx.rom_mmio.as_ref() {
                    None => {
                        0xFF
                    }
                    Some(rom) => {
                        rom.read(address)
                    }
                }
            }
            address if Cartridge::CART_RAM_RANGE_ADDRESS.contains(&address) => {
                match ctx.rom_mmio.as_ref() {
                    None => {
                        0xFF
                    }
                    Some(rom) => {
                        rom.read(address)
                    }
                }
            }
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
            Joypad::JOYPAD_REGISTER_ADDRESS => {
                ctx.joypad.read(address)
            }
            _ => todo!("Implement all other ranges"),
        }
    }

    pub fn write(&mut self, ctx: &mut MmioContextWrite, address: Address, data: Byte) {
        match address {
            address if Cartridge::CART_ROM_RANGE_ADDRESS.contains(&address) => {
                match ctx.rom_mmio {
                    None => {}
                    Some(rom) => {
                        rom.write(address, data);
                    }
                }
            }
            address if Cartridge::CART_RAM_RANGE_ADDRESS.contains(&address) => {
                match ctx.rom_mmio {
                    None => {}
                    Some(rom) => {
                        rom.write(address, data);
                    }
                }
            }
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
            Joypad::JOYPAD_REGISTER_ADDRESS => {
                ctx.joypad.write(address, data)
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
