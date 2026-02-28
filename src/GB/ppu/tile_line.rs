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

    pub fn line(&self) -> &[GbPaletteId; Tile::TILE_WIDTH as usize] {
        &self.line
    }

    pub fn line_mut(&mut self) -> &mut [GbPaletteId; Tile::TILE_WIDTH as usize] {
        &mut self.line
    }
}

impl Default for TileLine {
    fn default() -> Self {
        Self::new(0, 0)
    }
}
