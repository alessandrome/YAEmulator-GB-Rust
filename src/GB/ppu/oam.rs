use std::cmp::Ordering;
use crate::GB::ppu::tile::{TILE_HEIGHT, TILE_WIDTH};
use crate::{default_enum_u8, default_enum_u8_bit_ops};
use crate::GB::types::Byte;

pub mod test;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum AttributesMasks {
    Priority = 0b1000_0000,
    YFlip = 0b0100_0000,
    XFlip = 0b0010_0000,
    Palette = 0b0001_0000,
}

default_enum_u8!(AttributesMasks {Priority = 128, YFlip = 64, XFlip = 32, Palette = 16});


const OAM_BYTE_SIZE: usize = 4;

/// 4-bytes of a OAM item. Order of byte in memory is the following: y, x, tile_id, attributes
pub struct OamBytes {
    pub y: Byte,
    pub x: Byte,
    pub tile_id: Byte,
    pub attributes: Byte,
}

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
    original_attributes: Byte,
}

/// High-Level View of a OAM item
impl OAM {
    pub const OAM_BYTES: u8 = 4;

    pub fn new(y: u8, x: u8, tile_id: u8, attributes: Byte, id: Option<u8>) -> Self {
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

    #[inline]
    pub fn id(&self) -> Option<u8> {
        self.id
    }

    #[inline]
    pub fn y(&self) -> u8 {
        self.y
    }

    #[inline]
    pub fn x(&self) -> u8 {
        self.x
    }

    #[inline]
    pub fn tile_id(&self) -> u8 {
        self.tile_id
    }

    #[inline]
    pub fn priority(&self) -> bool {
        self.priority
    }

    #[inline]
    pub fn y_flip(&self) -> bool {
        self.y_flip
    }

    #[inline]
    pub fn x_flip(&self) -> bool {
        self.x_flip
    }

    #[inline]
    pub fn palette(&self) -> bool {
        self.palette
    }

    #[inline]
    pub fn attributes_byte(&self) -> Byte {
        (self.priority as u8) << (AttributesMasks::Priority as u8).trailing_zeros()
            | (self.y_flip as u8) << (AttributesMasks::YFlip as u8).trailing_zeros()
            | (self.x_flip as u8) << (AttributesMasks::XFlip as u8).trailing_zeros()
            | (self.palette as u8) << (AttributesMasks::Palette as u8).trailing_zeros()
            | (self.original_attributes & 0x0F)
    }

    /// Return a OamBytes structure with u8 representation of OAM data.
    pub fn get_oam_bytes(&self) -> OamBytes {
        let attributes: Byte = self.attributes_byte();

        OamBytes {
            y: self.y,
            x: self.x,
            tile_id: self.tile_id,
            attributes
        }
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
        (self.x, self.id).cmp(&(other.x, other.id))
    }
}


impl PartialOrd<Self> for OAM {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
