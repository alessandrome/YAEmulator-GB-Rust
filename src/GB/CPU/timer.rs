use crate::define_enum_u8;


pub define_enum_u8! {
    TACMask {
        Enabled = 0b0000_0100,
        TimerClock = 0b0000_0011
    }
}

pub define_enum_u8! {
    TACClock {
        M256 =  0b0000_0000,
        M4 =    0b0000_0001,
        M16 =   0b0000_0010,
        M64 =   0b0000_0011
    }
}
