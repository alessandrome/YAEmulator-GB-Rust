use crate::GB::PPU::oam::attributes_masks::AttributesMasks;

pub mod attributes_masks;

pub struct OAM {
    id: Option<usize>, // Optional - Useful to manage as ID of OAM in GB Memory
    y: u8,
    x: u8,
    tile_id: u8,
    priority: bool, // False = No | True = BG and Window colors 1–3 are drawn over this OBJ
    y_flip: bool,
    x_flip: bool,
    palette: bool, // False = OBP0 | True = OBP1
    original_attributes: u8,
}

impl OAM {
    pub fn new(y: u8, x: u8, tile_id: u8, attributes: u8, id: Option<usize>) -> Self {
        let priority = (attributes & AttributesMasks::Priority) != 0;
        let y_flip = (attributes & AttributesMasks::YFlip) != 0;
        let x_flip = (attributes & AttributesMasks::XFlip) != 0;
        let palette = (attributes & AttributesMasks::Palette) != 0;
        Self {
            id,
            y,
            x,
            tile_id,
            priority,
            y_flip,
            x_flip,
            palette,
            original_attributes: attributes,
        }
    }

    /// Return a tuple of 4 four bytes representing OAM in memory.
    ///
    /// Order of bytes is (Y, X, Tile ID, Attributes). Structure attributes are converted in its representing byte in memory.
    pub fn get_oam_bytes(&self) -> (u8, u8, u8, u8) {
        let attributes: u8 = (self.priority as u8) << (AttributesMasks::Priority as u8).trailing_zeros()
            | (self.y_flip as u8) << (AttributesMasks::YFlip as u8).trailing_zeros()
            | (self.x_flip as u8) << (AttributesMasks::XFlip as u8).trailing_zeros()
            | (self.palette as u8) << (AttributesMasks::Palette as u8).trailing_zeros()
            | (self.original_attributes & 0x0F);
        (self.y, self.x, self.tile_id, attributes)
    }
}

#[cfg(test)]
mod test {
    use crate::GB::PPU::oam::attributes_masks::AttributesMasks;
    use crate::GB::PPU::oam::OAM;

    #[test]
    fn new_oam() {
        let (test_y, test_x, test_tile_id, test_id) = (56u8, 131u8, 33u8, 1usize);
        let attributes: u8 = 0b1111_0000;
        let oam = OAM::new(test_y, test_x, test_tile_id, attributes, Option::from(test_id));
        assert_eq!(oam.y, test_y);
        assert_eq!(oam.x, test_x);
        assert_eq!(oam.tile_id, test_tile_id);
        assert_eq!(oam.id, Some(test_id));
        assert_eq!(oam.priority, true);
        assert_eq!(oam.y_flip, true);
        assert_eq!(oam.x_flip, true);
        assert_eq!(oam.palette, true);
    }

    #[test]
    fn get_oam_bytes() {
        let (test_y, test_x, test_tile_id, test_id) = (56u8, 131u8, 33u8, None);
        let attributes: u8 = 0b1001_0110;
        let oam = OAM::new(test_y, test_x, test_tile_id, attributes, test_id);
        assert_eq!(oam.id, None);
        assert_eq!(oam.priority, true);
        assert_eq!(oam.y_flip, false);
        assert_eq!(oam.x_flip, false);
        assert_eq!(oam.palette, true);
        assert_eq!(oam.get_oam_bytes(), (test_y, test_x, test_tile_id, attributes));
    }
}
