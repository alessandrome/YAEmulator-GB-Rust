use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

pub const TILE_SIZE: usize = 16; // In Bytes

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GbPalette {
    White = 0u8,
    LightGray = 1u8,
    DarkGray = 2u8,
    Black = 3u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GbPaletteId {
    Id0 = 0u8,
    Id1 = 1u8,
    Id2 = 2u8,
    Id3 = 3u8,
}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum RGBPalette {
    White = 0xD1EE54,
    LightGray = 0x94B860,
    DarkGray = 0x87AB54,
    Black = 0x0C0F08,
}

lazy_static! {
pub static ref CONSOLE_PALETTE: HashMap<GbPalette, char> = HashMap::from([
        (GbPalette::White, '█'),
        (GbPalette::LightGray, '▓'),
        (GbPalette::DarkGray, '▒'),
        (GbPalette::Black, '░'),
    ]);
}

lazy_static! {
    pub static ref PALETTE_ID_REPR: HashMap<GbPaletteId, &'static str> = HashMap::from([
        (GbPaletteId::Id0, "█"),
        (GbPaletteId::Id1, "▓"),
        (GbPaletteId::Id2, "▒"),
        (GbPaletteId::Id3, "░"),
    ]);
}

#[derive(Debug, Default, Clone)]
pub struct Tile {
    pub data: [u8; TILE_SIZE],
}

pub fn expand_bits(byte: u8) -> u16 {
    let mut result: u16 = 0;
    for i in 0..8 {
        result |= ((byte & (1 << i)) as u16) << i
    }
    result
}

impl Tile {
    pub fn new(tile: [u8; 8 * 2]) -> Self {
        Self { data: tile }
    }

    pub fn get_tile_map(&self) -> [GbPaletteId; 64] {
        self.to_picture_map()
    }

    pub fn to_picture_map(&self) -> [GbPaletteId; 64] {
        let mut picture = [GbPaletteId::Id0; 8 * 8];
        for i in 0..8 {
            let byte1 = self.data[i * 2];
            let byte2 = self.data[i * 2 + 1];
            let byte1_expanded = expand_bits(byte1);
            let byte2_expanded = expand_bits(byte2) << 1;
            let resulting_byte = byte2_expanded | byte1_expanded;
            for j in 0..8 {
                let shift = (7 - j) * 2;
                picture[i * 8 + j] = Self::half_nibble_to_palette_map(((resulting_byte & (3 << shift)) >> shift) as u8);
            }
        }
        picture
    }

    pub fn half_nibble_to_palette_map(byte: u8) -> GbPaletteId {
        match byte & 3 {
            0 => GbPaletteId::Id0,
            1 => GbPaletteId::Id1,
            2 => GbPaletteId::Id2,
            3 => GbPaletteId::Id3,
            _ => GbPaletteId::Id0
        }
    }

    pub fn get_printable_id_map(&self, doubled: bool) -> String {
        Self::palette_id_map_to_printable_id_map(&self.get_tile_map(), doubled)
    }

    pub fn palette_id_map_to_printable_id_map(array_map: &[GbPaletteId; 8 * 8], doubled: bool) -> String {
        let mut to_print = "".to_string();
        for i in 0..8 {
            for j in 0..8 {
                let to_push = PALETTE_ID_REPR[&array_map[i * 8 + j]];
                to_print.push_str(to_push);
                if doubled {
                    to_print.push_str(to_push);
                }
            }
            to_print.push('\n')
        }
        to_print
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut data_s = "".to_string();
        for i in 0..self.data.len() {
            data_s.push_str(format!("{:02X}", self.data[i]).as_str());
            if i != self.data.len() - 1 {
                data_s.push(' ');
            }
        }
        write!(
            f,
            "Tile {{ Data: [{}] }}",
            data_s,
        )
    }
}

#[cfg(test)]
mod test {
    use crate::GB::PPU::tile::{expand_bits, GbPaletteId, Tile};

    #[test]
    fn test_bit_expander() {
        let mut byte = 0b1111_1111;
        let mut expanded = expand_bits(byte);
        let mut expected: u16 = 0b01010101_01010101;
        assert_eq!(expanded, expected);

        byte = 0b1001_0110;
        expanded = expand_bits(byte);
        expected = 0b01000001_00010100;
        assert_eq!(expanded, expected);

        byte = 0b0000_0000;
        expanded = expand_bits(byte);
        expected = 0b00000000_00000000;
        assert_eq!(expanded, expected);
    }

    #[test]
    fn test_tile_to_picture_map() {
        // Each bit of the first byte is mixed with bit of second byte making a 2-bit color ID
        let tile_data: [u8; 16] = [
            0x3C, 0x7E,     // ██▓▓▓▓██   █▒▒▒▒▒▒█   ██▒▒░░░░░░░░▒▒██
            0x42, 0x42,     // █▓████▓█   █▒████▒█   ██░░████████░░██
            0x42, 0x42,     // █▓████▓█   █▒████▒█   ██░░████████░░██
            0x42, 0x42,     // █▓████▓█   █▒████▒█   ██░░████████░░██
            0x7E, 0x5E,     // █▓▓▓▓▓▓█ + █▒█▒▒▒▒█ = ██░░▓▓░░░░░░░░██
            0x7E, 0x0A,     // █▓▓▓▓▓▓█   ████▒█▒█   ██▓▓▓▓▓▓░░▓▓░░██
            0x7C, 0x56,     // █▓▓▓▓▓██   █▒█▒█▒▒█   ██░░▓▓░░▓▓░░▒▒██
            0x38, 0x7C,     // ██▓▓▓███   █▒▒▒▒▒██   ██▒▒░░░░░░▒▒████
        ];
        let tile = Tile { data: tile_data };
        let (c0, c1, c2, c3) =
            (GbPaletteId::Id0, GbPaletteId::Id1, GbPaletteId::Id2,GbPaletteId::Id3);
        let expected_id_map = [
          c0, c2, c3, c3, c3, c3, c2, c0,
          c0, c3, c0, c0, c0, c0, c3, c0,
          c0, c3, c0, c0, c0, c0, c3, c0,
          c0, c3, c0, c0, c0, c0, c3, c0,
          c0, c3, c1, c3, c3, c3, c3, c0,
          c0, c1, c1, c1, c3, c1, c3, c0,
          c0, c3, c1, c3, c1, c3, c2, c0,
          c0, c2, c3, c3, c3, c2, c0, c0,
        ];
        let result = tile.to_picture_map();
        let printable_test = Tile::palette_id_map_to_printable_id_map(&expected_id_map, true);
        let printable_result = Tile::palette_id_map_to_printable_id_map(&result, true);
        println!("{}", printable_test);
        println!("{}", printable_result);
        assert_eq!(result, expected_id_map);
    }
}