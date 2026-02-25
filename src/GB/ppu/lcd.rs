use crate::GB::bus::{Bus, MmioContext};
use crate::GB::ppu::pixel::PixelFifo;
use crate::GB::ppu::PPU;
use crate::GB::ppu::tile::GbPalette;
use crate::GB::traits::Tick;

pub struct LCD {
    screen: Box<[GbPalette; PPU::SCREEN_PIXELS as usize]>,
    pixel: usize,
}

impl LCD {
    pub fn new() -> Self {
        Self {
            screen: Box::new([GbPalette::White; PPU::SCREEN_PIXELS as usize]),
            pixel: 0
        }
    }

    pub fn pixel_mixer(obj_pixel: &PixelFifo, bg_pixel: &PixelFifo) -> PixelFifo {
        todo!("Must return PixelFifo to output")
    }
}

impl Tick for LCD {
    fn tick(&mut self, bus: &mut Bus, ctx: &mut MmioContext) {
        let ready: bool = !ctx.ppu_mmio.obj_fifo().is_empty() && !ctx.ppu_mmio.bg_fifo().is_empty();
        if ready {
        }
        todo!()
    }
}