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
