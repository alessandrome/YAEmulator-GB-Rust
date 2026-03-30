#[derive(Copy, Clone, Debug)]
pub enum InterruptType {
    VBlank,
    LCD,
    Timer,
    Serial,
    Joypad,
}
