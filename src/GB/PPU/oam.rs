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
        }
    }

    /// Return a tuple of 4 four bytes representing OAM in memory.
    ///
    /// Order of bytes is (Y, X, Tile ID, Attributes). Structure attributes are converted in its representing byte in memory.
    pub fn get_oam_bytes(&self) -> (u8, u8, u8, u8) {
        let attributes: u8 = (self.priority as u8) << (AttributesMasks::Priority as u8).leading_zeros()
            | (self.y_flip as u8) << (AttributesMasks::YFlip as u8).leading_zeros()
            | (self.x_flip as u8) << (AttributesMasks::XFlip as u8).leading_zeros()
            | (self.palette as u8) << (AttributesMasks::Palette as u8).leading_zeros();
        (self.y, self.x, self.tile_id, attributes)
    }
}
