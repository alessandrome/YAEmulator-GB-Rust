use crate::GB::types::Byte;
use crate::utils::expand_byte_bits;
use super::tile::{GbPaletteId, Tile};

pub struct TileLine {
    line: [GbPaletteId; Tile::TILE_WIDTH as usize],
}

impl TileLine {
    /// Create a new Tile Line. Remember that byte_1 contains lsb values and byte_2 MSB ones.
    pub fn new(byte_1: Byte, byte_2: Byte) -> Self {
        let mut line = [GbPaletteId::Id0; Tile::TILE_WIDTH as usize];
        let word = (expand_byte_bits(byte_2) << 1) | expand_byte_bits(byte_1);
        for col in 0_usize..8 {
            line[col] = GbPaletteId::half_nibble_to_palette_map(((word >> ((7 - col) * 2)) as Byte) & 0b11);
        }
        Self {
            line
        }
    }

    #[inline]
    pub fn line(&self) -> &[GbPaletteId; Tile::TILE_WIDTH as usize] {
        &self.line
    }

    #[inline]
    pub fn line_mut(&mut self) -> &mut [GbPaletteId; Tile::TILE_WIDTH as usize] {
        &mut self.line
    }

    #[inline]
    pub fn reverse(&self) -> TileLine {
        TileLine {
            line: [
                self.line[7].clone(),
                self.line[6].clone(),
                self.line[5].clone(),
                self.line[4].clone(),
                self.line[3].clone(),
                self.line[2].clone(),
                self.line[1].clone(),
                self.line[0].clone(),
            ]
        }
    }
}

impl Default for TileLine {
    fn default() -> Self {
        Self::new(0, 0)
    }
}
