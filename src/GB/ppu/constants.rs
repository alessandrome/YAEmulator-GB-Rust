use crate::GB::ppu::tile::{TILE_HEIGHT, TILE_WIDTH};

pub const LINE_DOTS: usize = 456;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;
pub const SCREEN_DOTS: usize = LINE_DOTS * SCREEN_HEIGHT;

pub const SCAN_OAM_DOTS: usize = 80;
pub const DRAW_LINE_MIN_DOTS: usize = 168;
pub const DRAW_LINE_MAX_DOTS: usize = 291;
pub const HBLANK_MIN_DOTS: usize = LINE_DOTS - SCAN_OAM_DOTS - DRAW_LINE_MAX_DOTS;
pub const HBLANK_MAX_DOTS: usize = LINE_DOTS - SCAN_OAM_DOTS - DRAW_LINE_MIN_DOTS;
pub const VBLANK_LINES: usize = 10;
pub const VBLANK_DOTS: usize = LINE_DOTS * VBLANK_LINES;

pub const FRAME_LINES: usize = SCREEN_HEIGHT + VBLANK_LINES;
pub const FRAME_DOTS: usize = SCREEN_DOTS + VBLANK_DOTS;

pub const OAM_NUMBERS: usize = 40;
pub const MAX_SPRITE_PER_LINE: usize = 10;
pub const MAX_SPRITE_ON_SCREEN: usize = 40;

pub const MAP_ROW_TILES: usize = 32;
pub const MAP_ROW_PIXELS: usize = MAP_ROW_TILES * TILE_WIDTH;
pub const MAP_LINES: usize = 32;
pub const MAP_HEIGHT_PIXELS: usize = MAP_LINES * TILE_HEIGHT;
pub const MAP_TILES: usize = MAP_ROW_TILES * MAP_LINES;
pub const MAP_PIXELS: usize = MAP_TILES * TILE_WIDTH * TILE_HEIGHT;
