use super::super::registers::core_registers::{Registers8Bit, Registers16Bit};

pub type Lhs8Bit = Registers8Bit;
pub type Rhs8Bit = Registers8Bit;

pub type Lhs16Bit = Registers16Bit;
pub type Rhs16Bit = Registers16Bit;
pub type AddressRegister = Registers16Bit;

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
    Ccf(Rhs8Bit),
    Scf(Rhs8Bit),
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

pub enum MicroOp {
    Fetch8(Registers8Bit),
    Ld8(Lhs8Bit, Rhs8Bit),
    Ld16(Lhs16Bit, Rhs16Bit),
    Read8(Lhs8Bit, AddressRegister),
    Write8(AddressRegister, Rhs8Bit),
    Inc8(Rhs8Bit),
    Dec8(Rhs8Bit),
    Inc16(Rhs16Bit),
    Dec16(Rhs16Bit),
    Alu(AluOp),
    Idle,
    End,
}
