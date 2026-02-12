pub struct Sweep {
    period: u8,
    negate: bool,
    shift: u8,
    timer: u8,
    shadow_freq: u16,
}

impl Sweep {
    pub fn new() -> Self {
        Self {
            period: 0,
            negate: false,
            shift: 0,
            timer: 0,
            shadow_freq: 0,
        }
    }
}
