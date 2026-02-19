use crate::GB::types::address::Address;
use super::super::registers::core_registers::{Registers8Bit, Registers16Bit};

pub type Lhs8Bit = Registers8Bit;
pub type Rhs8Bit = Registers8Bit;

pub type Lhs16Bit = Registers16Bit;
pub type Rhs16Bit = Registers16Bit;
pub type AddressRegister = Registers16Bit;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ByteBit {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum VectorAddress {
    V0 = 0x00,
    V1 = 0x08,
    V2 = 0x10,
    V3 = 0x18,
    V4 = 0x20,
    V5 = 0x28,
    V6 = 0x30,
    V7 = 0x38,
    VBlank = 0x40,
    STAT = 0x48,
    Timer = 0x50,
    Serial = 0x58,
    Joypad = 0x60,
}

#[derive(Debug, Clone, Copy)]
pub enum AluOp {
    Add(Lhs8Bit, Rhs8Bit),
    Adc(Lhs8Bit, Rhs8Bit),
    Sub(Lhs8Bit, Rhs8Bit),
    Sbc(Lhs8Bit, Rhs8Bit),
    Cp(Lhs8Bit, Rhs8Bit),
    Inc(Rhs8Bit),
    Dec(Rhs8Bit),
    And(Lhs8Bit, Rhs8Bit),
    Or(Lhs8Bit, Rhs8Bit),
    Xor(Lhs8Bit, Rhs8Bit),
    Ccf(),
    Scf(),
    Daa(),
    Cpl(Rhs8Bit),
    Rlca(),  // RL and RR for Accumulator register - Zero flags is always reset
    Rrca(),  // RL and RR for Accumulator register - Zero flags is always reset
    Rla(),   // RL and RR for Accumulator register - Zero flags is always reset
    Rra(),   // RL and RR for Accumulator register - Zero flags is always reset
    Rlc(Rhs8Bit),
    Rrc(Rhs8Bit),
    Rl(Rhs8Bit),
    Rr(Rhs8Bit),
    Sla(Rhs8Bit),
    Swap(Rhs8Bit),
    Sra(Rhs8Bit),
    Sll(Rhs8Bit),
    Srl(Rhs8Bit),
    Bit(ByteBit, Rhs8Bit),
    Res(ByteBit, Rhs8Bit),
    Set(ByteBit, Rhs8Bit),
}

#[derive(Debug, Clone, Copy)]
pub enum MicroOp {
    Fetch8(Registers8Bit),
    Ld8(Lhs8Bit, Rhs8Bit),
    Ld16(Lhs16Bit, Rhs16Bit),
    Read8(Lhs8Bit, AddressRegister),
    Write8(AddressRegister, Rhs8Bit),
    Push16msb(Rhs16Bit),
    Push16lsb(Rhs16Bit),
    Pop16msb(Rhs16Bit),
    Pop16lsb(Rhs16Bit),
    Inc16(Rhs16Bit),
    Dec16(Rhs16Bit),
    JumpVector(VectorAddress), // To immediate set PC during interrupts and RST
    Alu(AluOp),
    ImeEnabled(bool),
    PrefixCB,
    Idle,
}


#[derive(Debug, Clone, Copy)]
pub enum MCycleOp {
    Main(MicroOp),
    End(MicroOp),
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum MicroFlow {
    Next,
    Jump(usize),
    PrefixCB,
}
