pub mod dma_mmio;

use crate::GB::bus::{Bus, MmioContext, BusDevice};
use crate::GB::DMA::dma_mmio::DmaMmio;
use crate::GB::traits::Tick;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

pub struct DMA {
    t_cycle: u8,
    m_cycle: u8,
    to_disable: bool,
}

impl DMA {
    pub const DMA_SOURCE_ADDRESS: Address = Address(0xFF46);

    pub fn new() -> Self {
        Self {
            t_cycle: 0,
            m_cycle: 0,
            to_disable: false,
        }
    }
}

impl Tick for DMA {
    fn tick(&mut self, bus: &mut Bus, ctx: &mut MmioContext) {
        self.t_cycle = (self.t_cycle + 1) & 0b11;  // (T-Cycle + 1) % 4

        if self.to_disable {
            ctx.dma_mmio.reset();
            self.to_disable = false;
        }

        if ctx.dma_mmio.enabled() && self.t_cycle == 0 {
            let lsb_address = self.m_cycle as u16;
            let from_address = Address(((ctx.dma_mmio.value() as u16) << 8) | lsb_address);
            let to_address = Address(0xFE00 | lsb_address);
            let transfer_byte = bus.read(ctx, from_address);
            bus.write(ctx, to_address, transfer_byte);

            self.m_cycle = (self.m_cycle + 1) % 160;
            if self.m_cycle == 0 {
                self.to_disable = true;
            }
        }
    }
}

impl Default for DMA {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DmaCtx {
    pub dma: DMA,
    pub mmio: DmaMmio,
}
