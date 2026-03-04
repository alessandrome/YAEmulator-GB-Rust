mod bg_fetcher;
mod sprite_fetcher;

pub use bg_fetcher::BackgroundFetcher;
pub use sprite_fetcher::SpriteFetcher;

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
