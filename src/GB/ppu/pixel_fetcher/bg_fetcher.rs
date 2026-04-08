use crate::GB::bus::{Bus, MmioContextWrite};
use crate::GB::ppu::pixel::{PixelFifo, PixelFifoPaletteRegister};
use crate::GB::ppu::ppu_mode::PpuMode;
use crate::GB::ppu::tile::{Tile, TileDataArea, TileMapArea};
use crate::GB::ppu::tile_line::TileLine;
use super::super::{PPU, oam::OAM, OAM_BUFFER};
use crate::GB::traits::Tick;
use crate::GB::types::Byte;
use super::PixelFetcherState;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BgFetchingMode {
    Bg,
    Window
}

pub struct BackgroundFetcher {
    state: PixelFetcherState,
    fetching_mode: BgFetchingMode,
    first_cycle: bool,
    bg_tile_x: u8,
    window_line: u8, // Internal Window Line
    pixel_shift: u8,
    tile_map_id: u16,
    tile_id: u8,
    line_high_byte: Byte,
    line_low_byte: Byte,
    tile_line: TileLine,
    window_drawn: bool,
    discarding_pixels: u8,
}

impl BackgroundFetcher {
    pub const OAM_BUFFER: u8 = PPU::OAM_BUFFER;

    pub fn new() -> Self {
        Self {
            state: PixelFetcherState::FetchTileT1,
            fetching_mode: BgFetchingMode::Bg,
            first_cycle: true,
            bg_tile_x: 0,
            window_line: 0,
            pixel_shift: 0,
            tile_map_id: 0,
            tile_id: 0,
            line_high_byte: 0,
            line_low_byte: 0,
            tile_line: TileLine::default(),
            window_drawn: false,
            discarding_pixels: 0,
        }
    }

    #[inline]
    pub fn fetching_mode(&self) -> BgFetchingMode {
        self.fetching_mode
    }

    #[inline]
    pub fn window_drawn(&self) -> bool {
        self.window_drawn
    }

    #[inline]
    pub fn set_window_mode(&mut self) {
        self.reset_cycle();
        self.state = PixelFetcherState::FetchTileT1;
    }

    #[inline]
    pub fn reset_line(&mut self) {
        self.bg_tile_x = 0;
        if self.window_drawn {
            self.window_line += 1;
        }
        self.window_drawn = false;
        self.discarding_pixels = 0;
        self.state = PixelFetcherState::FetchTileT1;
    }

    #[inline]
    pub fn reset_frame(&mut self) {
        self.bg_tile_x = 0;
        self.window_line = 0;
        self.window_drawn = false;
        self.discarding_pixels = 0;
        self.state = PixelFetcherState::FetchTileT1;
    }

    #[inline]
    pub fn reset_cycle(&mut self) {
        self.state = PixelFetcherState::FetchTileT1;
    }

    #[inline]
    pub fn set_pixel_shift(&mut self, pixel_shift: u8) {
        self.pixel_shift = pixel_shift;
    }
}

impl Tick for BackgroundFetcher {
    fn tick(&mut self, bus: &mut Bus, ctx: &mut MmioContextWrite) {
        let lcdc = ctx.ppu_mmio.lcdc_view();
        let wx = ctx.ppu_mmio.wx();
        let wy = ctx.ppu_mmio.wy();
        let lx = ctx.ppu_mmio.lx();
        let ly = ctx.ppu_mmio.ly();

        match self.state {
            PixelFetcherState::FetchTileT1 => {
                let scx = ctx.ppu_mmio.scx();
                let scy = ctx.ppu_mmio.scy();

                if !lcdc.bg_win_enabled || !lcdc.window_enabled || (lx < wx || ly < wy) {
                    // Background: Get tile map coordinates
                    let bg_map_x = (self.bg_tile_x + (scx / 8)) & 0x1F; // X of tile Map
                    let bg_map_y = ((ly as u16 + scy as u16) & 0xFF) as u8 / 8; // Y of Tile Map
                    self.tile_map_id = (32 * bg_map_y as u16 + bg_map_x as u16) & 0x3FF; // Idx of tile id given (X,Y) of the map
                    self.fetching_mode = BgFetchingMode::Bg;
                } else {
                    // Window: Get tile map coordinates
                    let win_map_x = (wx / 8) & 0x1F;
                    let win_map_y = (ly - wy) & 0x1F;
                    self.tile_map_id = (32 * win_map_y as u16 + win_map_x as u16) & 0x3FF;
                    self.fetching_mode = BgFetchingMode::Window;
                }
                self.state = PixelFetcherState::FetchTileT2;
            }
            PixelFetcherState::FetchTileT2 => {
                match self.fetching_mode {
                    BgFetchingMode::Bg => {
                        self.tile_id = ctx.ppu_mmio.vram().tile_id(self.tile_map_id, lcdc.bg_tile_map);
                    }
                    BgFetchingMode::Window => {
                        self.tile_id = ctx.ppu_mmio.vram().tile_id(self.tile_map_id, lcdc.window_tile_map);
                    }
                }
                self.state = PixelFetcherState::FetchTileDataHighT1;
            }
            PixelFetcherState::FetchTileDataLowT1 => {
                let tile_data_area = lcdc.bg_window_tile_area;
                match self.fetching_mode {
                    BgFetchingMode::Bg => {
                        // Background: Get tile line low data
                        let scy = ctx.ppu_mmio.scy();
                        let ly = ctx.ppu_mmio.ly();
                        self.line_low_byte = ctx.ppu_mmio.vram().tile_line_lsb_byte(
                            self.tile_id,
                            ((scy as u16 + ly as u16) & 7) as u8,
                            tile_data_area
                        );
                    }
                    BgFetchingMode::Window => {
                        // Window: Get tile line low data
                        self.line_low_byte = ctx.ppu_mmio.vram().tile_line_lsb_byte(
                            self.tile_id,
                            self.window_line,
                            tile_data_area
                        );
                    }
                }
                self.state = PixelFetcherState::FetchTileDataLowT2;
            }
            PixelFetcherState::FetchTileDataLowT2 => {
                self.state = PixelFetcherState::FetchTileDataHighT1;
            }
            PixelFetcherState::FetchTileDataHighT1 => {
                let tile_data_area = lcdc.bg_window_tile_area;
                match self.fetching_mode {
                    BgFetchingMode::Bg => {
                        // Background: Get tile line low data
                        let scy = ctx.ppu_mmio.scy();
                        let ly = ctx.ppu_mmio.ly();
                        self.line_high_byte = ctx.ppu_mmio.vram().tile_line_msb_byte(
                            self.tile_id,
                            ((scy as u16 + ly as u16) & 7) as u8,
                            tile_data_area
                        );
                    }
                    BgFetchingMode::Window => {
                        // Window: Get tile line low data
                        self.line_low_byte = ctx.ppu_mmio.vram().tile_line_msb_byte(
                            self.tile_id,
                            self.window_line,
                            tile_data_area
                        );
                    }
                }
                self.state = PixelFetcherState::FetchTileDataHighT2;
            }
            PixelFetcherState::FetchTileDataHighT2 => {
                if self.first_cycle {
                    // This has been a warmup cycle
                    self.first_cycle = false;
                    self.state = PixelFetcherState::FetchTileT1;
                } else {
                    self.state = PixelFetcherState::PushT1;
                }
            }
            PixelFetcherState::PushT1 => {
                self.tile_line = TileLine::new(self.line_high_byte, self.line_low_byte);
                if ctx.ppu_mmio.bg_fifo().is_empty() {
                    for pixel in 0..Tile::TILE_WIDTH {
                        ctx.ppu_mmio.push_bg_pixel(PixelFifo::new(
                            self.tile_line.line()[pixel as usize],
                            PixelFifoPaletteRegister::Bgp,
                            false
                        ));
                    }
                    self.state = PixelFetcherState::PushT2;
                }
            }
            PixelFetcherState::PushT2 => {
                self.bg_tile_x += 1;
                self.state = PixelFetcherState::FetchTileT1;
            }
        }
    }
}
