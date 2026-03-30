use crate::GB::ppu::tile::{GbColor, GbPaletteId};
use crate::GB::types::Byte;
use crate::{default_enum_u8, default_enum_u8_bit_ops};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GbPaletteMask {
    Id0 = 0b0000_0011,
    Id1 = 0b0000_1100,
    Id2 = 0b0011_0000,
    Id3 = 0b1100_0000,
}
default_enum_u8!(GbPaletteMask {Id0 = 0b0000_0011, Id1 = 0b0000_1100, Id2 = 0b0011_0000, Id3 = 0b1100_0000});

pub struct GbPalette {
    id0: GbColor,
    id1: GbColor,
    id2: GbColor,
    id3: GbColor,
}

impl GbPalette {
    pub fn new(id0: GbColor, id1: GbColor, id2: GbColor, id3: GbColor) -> Self {
        Self { id0, id1, id2, id3 }
    }

    #[inline]
    pub fn byte_repr(&self) -> Byte {
        (self.id0 as u8) | (self.id1 as u8) << 2 | (self.id2 as u8) << 4 | (self.id3 as u8) << 6
    }

    pub fn from_byte(byte: Byte) -> Self {
        Self {
            id0: GbColor::from(byte & GbPaletteMask::Id0 as u8),
            id1: GbColor::from((byte & GbPaletteMask::Id1 as u8) << (GbPaletteMask::Id1 as u8).trailing_zeros()),
            id2: GbColor::from((byte & GbPaletteMask::Id2 as u8) << (GbPaletteMask::Id2 as u8).trailing_zeros()),
            id3: GbColor::from((byte & GbPaletteMask::Id3 as u8) << (GbPaletteMask::Id3 as u8).trailing_zeros()),
        }
    }

    #[inline]
    pub fn color(&self, id: GbPaletteId) -> GbColor {
        match id {
            GbPaletteId::Id0 => self.id0,
            GbPaletteId::Id1 => self.id1,
            GbPaletteId::Id2 => self.id2,
            GbPaletteId::Id3 => self.id3,
        }
    }
}

impl Default for GbPalette {
    fn default() -> Self {
        Self::new(
            GbColor::White,
            GbColor::LightGray,
            GbColor::DarkGray,
            GbColor::Black,
        )
    }
}
