#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum GbPalette {
    White = 0u8,
    LightGray = 1u8,
    DarkGray = 2u8,
    Black = 3u8,
}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum RGBPalette {
    White = 0xD1EE54,
    LightGray = 0x94B860,
    DarkGray = 0x87AB54,
    Black = 0x0C0F08,
}

#[derive(Debug, Default, Clone)]
pub struct Tile {
    pub data: [u8; 8 * 2],
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

    pub fn to_picture(&self) -> [GbPalette; 64] {
        let mut picture = [GbPalette::White; 8 * 8];
        for i in 0..8 {
            let byte1 = self.data[i * 2];
            let byte2 = self.data[i * 2 + 1];
            let byte1_expanded = expand_bits(byte1);
            let byte2_expanded = expand_bits(byte2) << 1;
            let resulting_byte = byte2_expanded | byte1_expanded;
            for j in 0..8 {
                let shift = (7 - j) * 2;
                picture[i * 8 + j] = Self::half_nibble_to_palette(((resulting_byte & (3 << shift)) >> shift) as u8);
            }
        }
        picture
    }

    pub fn half_nibble_to_palette(byte: u8) -> GbPalette {
        match byte & 3 {
            0 => GbPalette::White,
            1 => GbPalette::LightGray,
            2 => GbPalette::DarkGray,
            3 => GbPalette::Black,
            _ => GbPalette::White
        }
    }
}

#[cfg(test)]
mod test {
    use crate::GB::PPU::tile::expand_bits;

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
    fn test_tile_to_picture() {
        // TODO: Implement
        // let tile_data: [u8; 16] = [
        //
        // ];
    }
}