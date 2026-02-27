use crate::GB::bus::{Bus, MmioContext};
use super::{PPU, oam::OAM, OAM_BUFFER};
use crate::GB::traits::Tick;

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
    x: u8,
}

impl PixelFetcher {
    pub const OAM_BUFFER: u8 = PPU::OAM_BUFFER;

    pub fn new() -> Self {
        Self {
            state: PixelFetcherState::FetchTileT1,
            oam_buffer: Vec::with_capacity(Self::OAM_BUFFER as usize),
            x: 0
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

        match self.state {
            PixelFetcherState::FetchTileT1 => {

            }
            _ => {}
        }
        todo!()
    }
}
