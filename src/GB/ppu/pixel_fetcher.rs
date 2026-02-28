use crate::GB::bus::{Bus, MmioContext};
use crate::GB::ppu::pixel::{PixelFifo, PixelFifoPaletteRegister};
use crate::GB::ppu::ppu_mode::PpuMode;
use crate::GB::ppu::tile::{Tile, TileDataArea, TileMapArea};
use crate::GB::ppu::tile_line::TileLine;
use super::{PPU, oam::OAM, OAM_BUFFER};
use crate::GB::traits::Tick;
use crate::GB::types::Byte;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PixelFetcherState {
    FetchTileT1,
    FetchTileT2,
    FetchTileDataHighT1,
    FetchTileDataHighT2,
    FetchTileDataLowT1,
    FetchTileDataLowT2,
    PushT1,
    PushT2,
}

pub struct PixelFetcher {
    state: PixelFetcherState,
    oam_buffer: Vec<OAM>,
    screen_tile_x: u8,
    pixel_shift: u8,
    tile_map_id: u16,
    tile_id: u8,
    line_high_byte: Byte,
    line_low_byte: Byte,
    tile_line: TileLine,
}

impl PixelFetcher {
    pub const OAM_BUFFER: u8 = PPU::OAM_BUFFER;

    pub fn new() -> Self {
        Self {
            state: PixelFetcherState::FetchTileT1,
            oam_buffer: Vec::with_capacity(Self::OAM_BUFFER as usize),
            screen_tile_x: 0,
            pixel_shift: 0,
            tile_map_id: 0,
            tile_id: 0,
            line_high_byte: 0,
            line_low_byte: 0,
            tile_line: TileLine::default(),
        }
    }

    #[inline]
    pub fn oam_buffer(&self) -> &Vec<OAM> {
        &self.oam_buffer
    }

    #[inline]
    pub fn clear_oam_buffer(&mut self) {
        self.oam_buffer.clear();
    }

    #[inline]
    pub fn push_oam_buffer(&mut self, oam: OAM) {
        self.oam_buffer.push(oam);
    }

    #[inline]
    pub fn order_oam_buffer(&mut self) {
        self.oam_buffer.sort();
    }
}

impl Tick for PixelFetcher {
    fn tick(&mut self, bus: &mut Bus, ctx: &mut MmioContext) {
        let lcdc = ctx.ppu_mmio.lcdc_view();


        // Reset here OAM buffer and X on starting phases
        if ctx.ppu_mmio.prev_ppu_mode() != ctx.ppu_mmio.ppu_mode() {
            match ctx.ppu_mmio.ppu_mode() {
                PpuMode::OAMScan => {
                    self.clear_oam_buffer();
                    self.screen_tile_x = 0;
                }
                PpuMode::Drawing => {
                    self.order_oam_buffer();
                    self.pixel_shift = ctx.ppu_mmio.scx() & 0x07;
                }
                PpuMode::HBlank => {}
                PpuMode::VBlank => {}
            }
        }

        match self.state {
            PixelFetcherState::FetchTileT1 => {
                // TODO: add for code to switch from bg to windows calculus mode
                let scx = ctx.ppu_mmio.scx();
                let scy = ctx.ppu_mmio.scy();
                let ly = ctx.ppu_mmio.ly();

                // Get tile map coordinates
                let bg_map_x = (self.screen_tile_x + (scx / 8)) & 0x1F; // X of tile Map
                let bg_map_y = ((ly as u16 + scy as u16) & 0xFF) as u8 / 8; // Y of Tile Map
                self.tile_map_id = (32 * bg_map_y as u16 + bg_map_x as u16) & 0x3FF; // Idx of tile id given (X,Y) of the map
                self.state = PixelFetcherState::FetchTileT2;
            }
            PixelFetcherState::FetchTileT2 => {
                self.tile_id = ctx.ppu_mmio.vram().tile_id(self.tile_map_id, lcdc.bg_tile_map);
                self.state = PixelFetcherState::FetchTileDataHighT1;
            }
            PixelFetcherState::FetchTileDataLowT1 => {
                let scy = ctx.ppu_mmio.scy();
                let ly = ctx.ppu_mmio.ly();
                self.line_low_byte = ctx.ppu_mmio.vram().tile_line_lsb_byte(
                    self.tile_id,
                    ((scy as u16 + ly as u16) & 7) as u8,
                    TileDataArea::DataBlock12
                );
                self.state = PixelFetcherState::FetchTileDataLowT2;
            }
            PixelFetcherState::FetchTileDataLowT2 => {
                self.state = PixelFetcherState::FetchTileDataHighT1;
            }
            PixelFetcherState::FetchTileDataHighT1 => {
                let scy = ctx.ppu_mmio.scy();
                let ly = ctx.ppu_mmio.ly();
                self.line_high_byte = ctx.ppu_mmio.vram().tile_line_msb_byte(
                    self.tile_id,
                    ((scy as u16 + ly as u16) & 7) as u8,
                    TileDataArea::DataBlock12
                );
                self.state = PixelFetcherState::FetchTileDataHighT2;
            }
            PixelFetcherState::FetchTileDataHighT2 => {
                self.state = PixelFetcherState::PushT1;
            }
            PixelFetcherState::PushT1 => {
                self.tile_line = TileLine::new(self.line_high_byte, self.line_low_byte);
                self.state = PixelFetcherState::PushT2;
            }
            PixelFetcherState::PushT2 => {
                if ctx.ppu_mmio.bg_fifo().is_empty() {
                    // Todo: add shifting of pixel
                    for pixel in 0..Tile::TILE_WIDTH {
                        ctx.ppu_mmio.push_bg_pixel(PixelFifo::new(
                            self.tile_line.line()[pixel as usize],
                            PixelFifoPaletteRegister::Bgp,
                            false
                        ));
                    }
                    self.screen_tile_x += 1;
                    self.state = PixelFetcherState::FetchTileT1;
                }
            }
        }
        todo!()
    }
}
