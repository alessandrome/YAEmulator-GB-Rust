use crate::GB::bus;

/// Trait for component that must tick 1 T-Cycle at time (4 T-Cycle = 1 N-Cycle)
pub trait Tick {
    fn tick(&mut self, bus: &mut bus::Bus, ctx: &mut bus::MmioContext);
}
