use crate::GB::bus::{Bus, MmioContext};
use crate::GB::ppu::pixel::{PixelFifo, PixelFifoPaletteRegister};
use crate::GB::ppu::ppu_mode::PpuMode;
use crate::GB::ppu::tile::{Tile, TileDataArea, TileMapArea};
use crate::GB::ppu::tile_line::TileLine;
use super::super::{PPU, oam::OAM, OAM_BUFFER};
use crate::GB::traits::Tick;
use crate::GB::types::Byte;
use super::PixelFetcherState;

pub struct SpriteFetcher {
    state: PixelFetcherState,
    oam: Option<OAM>,
    line_high_byte: Byte,
    line_low_byte: Byte,
    tile_line: TileLine,
}

impl SpriteFetcher {
    pub const OAM_BUFFER: u8 = PPU::OAM_BUFFER;

    pub fn new() -> Self {
        Self {
            state: PixelFetcherState::FetchTileT1,
            oam: None,
            line_high_byte: 0,
            line_low_byte: 0,
            tile_line: TileLine::default(),
        }
    }
}

impl Tick for SpriteFetcher {
    fn tick(&mut self, bus: &mut Bus, ctx: &mut MmioContext) {
        let lcdc = ctx.ppu_mmio.lcdc_view();

        // Reset here OAM buffer and X on starting phases
        if ctx.ppu_mmio.prev_ppu_mode() != ctx.ppu_mmio.ppu_mode() {
            match ctx.ppu_mmio.ppu_mode() {
                PpuMode::OAMScan => {}
                PpuMode::Drawing => {}
                PpuMode::HBlank => {}
                PpuMode::VBlank => {}
            }
        }

        match self.state {
            PixelFetcherState::FetchTileT1 => {
                todo!("Get tile ID from OAM to use and work with it!");
                self.state = PixelFetcherState::FetchTileT2;
            }
            PixelFetcherState::FetchTileT2 => {
                self.state = PixelFetcherState::FetchTileDataHighT1;
            }
            PixelFetcherState::FetchTileDataLowT1 => {
                todo!("Get line Low Byte");
                self.state = PixelFetcherState::FetchTileDataLowT2;
            }
            PixelFetcherState::FetchTileDataLowT2 => {
                self.state = PixelFetcherState::FetchTileDataHighT1;
            }
            PixelFetcherState::FetchTileDataHighT1 => {
                todo!("Get line High Byte");
                self.state = PixelFetcherState::FetchTileDataHighT2;
            }
            PixelFetcherState::FetchTileDataHighT2 => {
                self.state = PixelFetcherState::PushT1;
            }
            PixelFetcherState::PushT1 => {
                self.tile_line = TileLine::new(self.line_high_byte, self.line_low_byte);
                todo!("Push Sprite pixels");
                self.state = PixelFetcherState::PushT2;
            }
            PixelFetcherState::PushT2 => {
                self.state = PixelFetcherState::FetchTileT1;
            }
        }
    }
}
