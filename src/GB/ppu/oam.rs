use std::cmp::Ordering;
use crate::GB::ppu::oam::attributes_masks::AttributesMasks;
use crate::GB::ppu::tile::{TILE_HEIGHT, TILE_WIDTH};

pub mod attributes_masks;

pub const OAM_BYTE_SIZE: usize = 4;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OAM {
    id: Option<u8>, // Optional - Useful to manage as ID of OAM in GB Memory
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
    pub const OAM_BYTES: u8 = 4;

    pub fn new(y: u8, x: u8, tile_id: u8, attributes: u8, id: Option<u8>) -> Self {
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

    pub fn get_y_screen(&self) -> isize {
        self.y as isize - TILE_HEIGHT as isize * 2
    }

    pub fn get_x_screen(&self) -> isize {
        self.x as isize - TILE_WIDTH as isize
    }

    pub fn get_tile_id(&self) -> u8 {
        self.tile_id
    }
}

impl Ord for OAM {
    fn cmp(&self, other: &Self) -> Ordering {
        let x_cmp = self.x.cmp(&other.x);
        if x_cmp == std::cmp::Ordering::Equal {
            self.id.cmp(&other.id)
        } else {
            x_cmp
        }
    }
}


impl PartialOrd<Self> for OAM {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod test {
    use crate::GB::ppu::oam::attributes_masks::AttributesMasks;
    use crate::GB::ppu::oam::OAM;

    macro_rules! new_oam {
        ($oam: ident, $x_ident: ident, $y_ident: ident, $tile_ident: ident, $id_ident: ident, $attr_ident: ident, $x: expr, $y: expr, $tile_id: expr, $id: expr, $attributes: expr) => {
            let ($y_ident, $x_ident, $tile_ident, $id_ident) = ($y, $x, $tile_id, $id);
            let $attr_ident: u8 = $attributes;
            let $oam = OAM::new($y_ident, $x_ident, $tile_ident, $attr_ident, $id_ident);
        };
    }

    #[test]
    fn new_oam() {
        new_oam!(oam, test_x, test_y, test_tile_id, test_id, attributes, 56u8, 131u8, 33u8, Some(1usize), 0b1111_0000);
        assert_eq!(oam.y, test_y);
        assert_eq!(oam.x, test_x);
        assert_eq!(oam.tile_id, test_tile_id);
        assert_eq!(oam.id, test_id);
        assert_eq!(oam.priority, true);
        assert_eq!(oam.y_flip, true);
        assert_eq!(oam.x_flip, true);
        assert_eq!(oam.palette, true);
    }

    #[test]
    fn get_oam_bytes() {
        new_oam!(oam, test_x, test_y, test_tile_id, test_id, attributes, 56u8, 131u8, 33u8, None, 0b1001_0110);
        assert_eq!(oam.id, None);
        assert_eq!(oam.priority, true);
        assert_eq!(oam.y_flip, false);
        assert_eq!(oam.x_flip, false);
        assert_eq!(oam.palette, true);
        assert_eq!(oam.get_oam_bytes(), (test_y, test_x, test_tile_id, attributes));
    }

    #[test]
    fn test_order() {
        new_oam!(oam_1, test_x, test_y, test_tile_id, test_id, attributes, 56u8, 131u8, 33u8, Some(1), 0b1001_0110);
        new_oam!(oam_2, test_x, test_y, test_tile_id, test_id, attributes, 56u8, 131u8, 33u8, Some(2), 0b1001_0110);
        new_oam!(oam_3, test_x, test_y, test_tile_id, test_id, attributes, 39u8, 131u8, 33u8, Some(3), 0b1001_0110);
        assert_eq!(oam_1 < oam_2, true);
        assert_eq!(oam_2 < oam_3, false);
    }
}
