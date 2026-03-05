use crate::GB::bus::{Bus, MmioContext};
use crate::GB::ppu::lcd_control::ObjSize;
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
    oam: OAM,
    line_y: u8,
    line_high_byte: Byte,
    line_low_byte: Byte,
    tile_line: TileLine,
}

impl SpriteFetcher {
    pub const OAM_BUFFER: u8 = PPU::OAM_BUFFER;

    pub fn new() -> Self {
        Self {
            state: PixelFetcherState::FetchTileT1,
            oam: OAM::default(),
            line_y: 0,
            line_high_byte: 0,
            line_low_byte: 0,
            tile_line: TileLine::default(),
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.state = PixelFetcherState::FetchTileT1;
    }

    #[inline]
    pub fn state(&self) -> PixelFetcherState {
        self.state
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
                // Pop OAM and store it
                self.oam = ctx.ppu_mmio.pop_oam_buffer().unwrap();

                self.state = PixelFetcherState::FetchTileT2;
            }
            PixelFetcherState::FetchTileT2 => {
                let tile_height: u8;
                if lcdc.obj_size == ObjSize::Single {
                    tile_height = 8;
                } else {
                    tile_height = 16;
                }

                let ly = ctx.ppu_mmio.ly();
                if self.oam.y_flip() {
                    self.line_y = tile_height - (self.oam.y() - ly);
                } else {
                    self.line_y = self.oam.y() - ly - 1;
                }

                self.state = PixelFetcherState::FetchTileDataHighT1;
            }
            PixelFetcherState::FetchTileDataLowT1 => {
                self.line_low_byte = ctx.ppu_mmio.vram().tile_line_lsb_byte(
                    self.oam.tile_id(),
                    self.line_y,
                    TileDataArea::DataBlock01
                );
                self.state = PixelFetcherState::FetchTileDataLowT2;
            }
            PixelFetcherState::FetchTileDataLowT2 => {
                self.state = PixelFetcherState::FetchTileDataHighT1;
            }
            PixelFetcherState::FetchTileDataHighT1 => {
                self.line_high_byte = ctx.ppu_mmio.vram().tile_line_msb_byte(
                    self.oam.tile_id(),
                    self.line_y,
                    TileDataArea::DataBlock01
                );
                self.state = PixelFetcherState::FetchTileDataHighT2;
            }
            PixelFetcherState::FetchTileDataHighT2 => {
                self.state = PixelFetcherState::PushT1;
            }
            PixelFetcherState::PushT1 => {
                self.tile_line = TileLine::new(self.line_high_byte, self.line_low_byte);
                if self.oam.x_flip() {
                    self.tile_line = self.tile_line.reverse();
                }

                let obj_fifo_len = ctx.ppu_mmio.oam_buffer().len();
                for pixel in 0..Tile::TILE_WIDTH {
                    // Push only pixels that don't overlap with already pushed ones
                    if pixel as usize >= obj_fifo_len {
                        let palette;
                        if self.oam.palette() {
                            palette = PixelFifoPaletteRegister::Obp1;
                        } else {
                            palette = PixelFifoPaletteRegister::Obp0;
                        }
                        ctx.ppu_mmio.push_obj_pixel(PixelFifo::new(
                            self.tile_line.line()[pixel as usize],
                            palette,
                            self.oam.priority()
                        ));
                    }
                }

                self.state = PixelFetcherState::PushT2;
            }
            PixelFetcherState::PushT2 => {
                self.state = PixelFetcherState::FetchTileT1;
            }
        }
    }
}
