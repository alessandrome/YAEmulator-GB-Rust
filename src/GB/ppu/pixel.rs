use crate::GB::ppu::tile::{GbPalette, GbPaletteId};
use crate::GB::types::Byte;

#[derive(Copy, Clone, Debug)]
pub enum PixelFifoPaletteRegister {
    Obp0,
    Obp1,
}

#[derive(Copy, Clone, Debug)]
pub struct PixelFifo {
    color: GbPaletteId,
    palette: PixelFifoPaletteRegister,
    priority: bool
}

impl PixelFifo {
    pub fn new(color: GbPaletteId, palette: PixelFifoPaletteRegister, priority: bool) -> Self {
        Self {
            color,
            palette,
            priority,
        }
    }

    #[inline]
    pub fn color(&self) -> GbPaletteId {
        self.color
    }

    #[inline]
    pub fn palette(&self) -> PixelFifoPaletteRegister {
        self.palette
    }

    #[inline]
    pub fn priority(&self) -> bool {
        self.priority
    }

    #[inline]
    /// I've not found exact representation of bit order
    pub fn byte_repr(&self) -> Byte {
        todo!()
    }
}
