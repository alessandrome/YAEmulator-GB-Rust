use crate::GB::ppu::tile::{GbColor, GbPaletteId};
use crate::GB::types::Byte;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PixelFifoPaletteRegister {
    Bgp,
    Obp0,
    Obp1,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PixelFifo {
    color_id: GbPaletteId,
    palette: PixelFifoPaletteRegister,
    priority: bool
}

impl PixelFifo {
    pub fn new(color: GbPaletteId, palette: PixelFifoPaletteRegister, priority: bool) -> Self {
        Self {
            color_id: color,
            palette,
            priority,
        }
    }

    #[inline]
    pub fn color_id(&self) -> GbPaletteId {
        self.color_id
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
