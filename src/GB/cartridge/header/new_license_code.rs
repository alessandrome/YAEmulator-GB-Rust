#[derive(Copy, Clone, Debug)]
#[repr(u16)]
pub enum NewLicenseCode {
    None = 0x3030,
    NintendoRnD = 0x3031,
    Capcom = 0x3038,
    EA = 0x3133,
    HudsonSoft = 0x3138,
    BAI = 0x3139,
    KSS = 0x3230,
    PlanningOfficeWADA = 0x3232,
    PCMComplete = 0x3234,
    SanX = 0x3235,
    Kemco = 0x3238,
    SETRACorporation = 0x3239,
    Viacom = 0x3330,
    Nintendo = 0x3331,
    Bandai = 0x3332,
    OceanSoftware = 0x3333,
    //todo
}
