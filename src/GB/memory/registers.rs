use std::fmt;
use std::fmt::Formatter;
use crate::GB::memory::RAM;

// Port/Mode Registers
pub const P1: u16 = 0xFF00;
pub const SB: u16 = 0xFF01;
pub const SC: u16 = 0xFF02;
pub const DIV: u16 = 0xFF04;
pub const TIMA: u16 = 0xFF05;
pub const TMA: u16 = 0xFF06;
pub const TAC: u16 = 0xFF07;

// Interrupt Registers
pub const IF: u16 = 0xFF0F;
pub const IE: u16 = 0xFFFF;

// Display Registers
pub const LCDC: u16 = 0xFF40;
pub const STAT: u16 = 0xFF41;
pub const SCY: u16 = 0xFF42;
pub const SCX: u16 = 0xFF43;
pub const LY: u16 = 0xFF44;
pub const LYC: u16 = 0xFF45;
pub const DNA: u16 = 0xFF46;
pub const BGP: u16 = 0xFF47;
pub const OBP0: u16 = 0xFF48;
pub const OBP1: u16 = 0xFF49;
pub const WY: u16 = 0xFF4A;
pub const WX: u16 = 0xFF4B;

pub struct MemoryRegisters {
    pub p1: u8,
    pub sb: u8,
    pub sc: u8,
    pub div: u8,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
    pub interrupt_flag: u8,
    pub ie: u8,
    pub lcdc: u8,
    pub lcd_stat: u8,
    pub scy: u8,
    pub scx: u8,
    pub ly: u8,
    pub lyc: u8,
    pub dna: u8,
    pub bgp: u8,
    pub obp0: u8,
    pub obp1: u8,
    pub wy: u8,
    pub wx: u8,
}

impl MemoryRegisters {
    pub fn new(memory: &RAM) -> Self {
        Self {
            p1: memory.read(P1),
            sb: memory.read(SB),
            sc: memory.read(SC),
            div: memory.read(DIV),
            tima: memory.read(TIMA),
            tma: memory.read(TMA),
            tac: memory.read(TAC),
            interrupt_flag: memory.read(IF),
            ie: memory.read(IE),
            lcdc: memory.read(LCDC),
            lcd_stat: memory.read(STAT),
            scy: memory.read(SCY),
            scx: memory.read(SCX),
            ly: memory.read(LY),
            lyc: memory.read(LYC),
            dna: memory.read(DNA),
            bgp: memory.read(BGP),
            obp0: memory.read(OBP0),
            obp1: memory.read(OBP1),
            wy: memory.read(WY),
            wx: memory.read(WX),
        }
    }
}

impl fmt::Display for MemoryRegisters {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ LF: {:#04X}, LY: {:} , LYC: {} }}",
            self.p1, self.ly, self.lyc
        )
    }
}
