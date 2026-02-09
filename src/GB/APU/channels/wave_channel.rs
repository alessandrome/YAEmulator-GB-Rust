pub struct WaveChannel {
    nr30: u8,  // DAC Enabled
    nr31: u8,  // Length Timer
    nr32: u8,  // Output Level
    nr33: u8,  // Period Low
    nr34: u8,  // Period High & Control
    ram: [u8; 16]
}

impl WaveChannel {
    pub fn new() -> WaveChannel {
        WaveChannel {
            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,
            ram: [0; 16],
        }
    }

    #[inline]
    pub fn nr30(&self) -> u8 {
        self.nr30
    }

    #[inline]
    pub fn nr31(&self) -> u8 {
        self.nr31
    }

    #[inline]
    pub fn nr32(&self) -> u8 {
        self.nr32
    }

    #[inline]
    pub fn nr33(&self) -> u8 {
        self.nr33
    }

    #[inline]
    pub fn nr34(&self) -> u8 {
        self.nr34
    }

    #[inline]
    pub fn set_nr30(&mut self, value: u8) {
        self.nr34 = value;
    }

    #[inline]
    pub fn set_nr31(&mut self, value: u8) {
        self.nr34 = value;
    }

    #[inline]
    pub fn set_nr32(&mut self, value: u8) {
        self.nr34 = value;
    }

    #[inline]
    pub fn set_nr33(&mut self, value: u8) {
        self.nr34 = value;
    }

    #[inline]
    pub fn set_nr34(&mut self, value: u8) {
        self.nr34 = value;
    }
}
