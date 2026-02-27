use crate::GB::bus::{Bus, MmioContext};
use crate::GB::ppu::palette::GbPalette;
use crate::GB::ppu::pixel::{PixelFifo, PixelFifoPaletteRegister};
use crate::GB::ppu::PPU;
use crate::GB::ppu::tile::{GbColor, GbPaletteId};
use crate::GB::traits::Tick;

pub struct LCD {
    screen: Box<[GbColor; PPU::SCREEN_PIXELS as usize]>,
    pixel: usize,
}

impl LCD {
    pub fn new() -> Self {
        Self {
            screen: Box::new([GbColor::White; PPU::SCREEN_PIXELS as usize]),
            pixel: 0
        }
    }

    pub fn pixel_mixer(obj_pixel: Option<&PixelFifo>, bg_pixel: &PixelFifo) -> PixelFifo {
        match obj_pixel {
            None => bg_pixel.clone(),
            Some(obj_pixel) => {
                if obj_pixel.color_id() == GbPaletteId::Id0 {
                    return bg_pixel.clone();
                }
                if obj_pixel.priority() && bg_pixel.color_id() != GbPaletteId::Id0 {
                    return bg_pixel.clone();
                }
                obj_pixel.clone()
            }
        }
    }

    #[inline]
    pub fn drawing_pixel(&self) -> usize {
        self.pixel
    }

    #[inline]
    pub fn screen(&self) -> &[GbColor; PPU::SCREEN_PIXELS as usize] {
        &self.screen
    }
}

impl Tick for LCD {
    fn tick(&mut self, bus: &mut Bus, ctx: &mut MmioContext) {
        let ready: bool = !ctx.ppu_mmio.bg_fifo().is_empty();
        if ready {
            let obj_pixel;
            match ctx.ppu_mmio.pop_obj_pixel() {
                None => {
                    obj_pixel = None;
                }
                Some(o) => {
                    obj_pixel = Some(o);
                }
            }
            let mixed_pixel_fifo = Self::pixel_mixer(
                obj_pixel.as_ref(),
                &ctx.ppu_mmio.pop_bg_pixel().unwrap(),
            );
            let color: GbColor;
            match mixed_pixel_fifo.palette() {
                PixelFifoPaletteRegister::Bgp => {
                    color = ctx.ppu_mmio.bgp_view().color(mixed_pixel_fifo.color_id());
                }
                PixelFifoPaletteRegister::Obp0 => {
                    color = ctx.ppu_mmio.obp0_view().color(mixed_pixel_fifo.color_id());
                }
                PixelFifoPaletteRegister::Obp1 => {
                    color = ctx.ppu_mmio.obp1_view().color(mixed_pixel_fifo.color_id());
                }
            }
            self.screen[self.pixel] = color;
            self.pixel = (self.pixel + 1) % PPU::SCREEN_PIXELS as usize;
        }
    }
}