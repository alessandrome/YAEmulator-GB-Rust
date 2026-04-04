use crate::GB::bus::{Bus, MmioContextWrite};
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
    pub const LCD_OFF_FRAME: [GbColor; PPU::SCREEN_PIXELS as usize] = [GbColor::White; PPU::SCREEN_PIXELS as usize];

    pub fn new() -> Self {
        Self {
            screen: Box::new([GbColor::White; PPU::SCREEN_PIXELS as usize]),
            pixel: 0
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
    fn tick(&mut self, bus: &mut Bus, ctx: &mut MmioContextWrite) {
        match ctx.ppu_mmio.consume_pixel() {
            None => (),
            Some(color) => {
                self.screen[self.pixel] = color;
                self.pixel = (self.pixel + 1) % PPU::SCREEN_PIXELS as usize;
            }
        }
    }
}