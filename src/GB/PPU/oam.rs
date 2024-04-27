pub struct OAM {
    id: Option<usize>, // Optional - Useful to manage as ID of OAM in GB Memory
    y: u8,
    x: u8,
    tile_id: u8,
    attributes: u8,
}

impl OAM {
    pub fn new(y: u8, x: u8, tile_id: u8, attributes: u8, id: Option<usize>) -> Self {
        Self {
            id,
            y,
            x,
            tile_id,
            attributes,
        }
    }
}
