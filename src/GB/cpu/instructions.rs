#[cfg(test)]
mod test;
pub mod microcode;
pub mod interrupt;

pub use interrupt::InterruptType;
use super::registers::core_registers::{FlagBits};
use microcode::{*};

pub type InstructionMicroOpIndex = usize;

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: u8,
    pub name: &'static str,
    pub cycles: u8,
    pub size: u8,
    pub flags: &'static [FlagBits],
    pub micro_ops: &'static [MCycleOp], // List of micro-ops to execute in 1 M-Cycle each
}

impl Instruction {
    pub fn new(opcode: u8) -> Option<&'static Self> {
        OPCODES[opcode as usize]
    }
}

const fn daa(mut a: u8, mut flags: u8) -> (u8, u8) {
    // Code recovered inspired by other gits, but not sure if precise as expected from GB Docs
    let mut adjust = 0;
    let mut carry_flag = (FlagBits::C as u8) & flags != 0;
    let mut half_carry_flag = (FlagBits::H as u8) & flags != 0;

    if half_carry_flag {
        adjust |= 0x06;
    }
    if carry_flag || a > 0x99 {
        adjust |= 0x60;
    }

    // Edit register A for CDB representation
    if (FlagBits::N as u8) & flags == 0 {
        if (a & 0x0F )> 9 {
            adjust |= 0x06;
        }

        if a > 0x99 {
            adjust |= 0x60;
        }

        a = a.wrapping_add(adjust);
    } else {
        a = a.wrapping_sub(adjust);
    }

    carry_flag = (adjust & 0x60) != 0;
    let zero_flag = a == 0;

    // Impostare i flag appropriati
    if carry_flag {
        flags |= FlagBits::C as u8;
    } else {
        flags &= !(FlagBits::C as u8);
    }

    if zero_flag {
        flags |= FlagBits::Z as u8;
    } else {
        flags &= !(FlagBits::Z as u8);
    }

    flags &= !(FlagBits::H as u8);

    // Settare i nuovi valori dei flag
    // flags può essere un riferimento mutabile a un altro registro che contiene i flags
    // quindi è necessario modificarlo come desiderato
    (a, flags)
}

pub const INTERRUPT_VBLANK: Instruction = Instruction {
    opcode: 0x00, // Not important, interrupt routine hasn't an opcode
    name: "Interrupt - VBlank",
    cycles: 5,
    size: 0,
    flags: &[],
    micro_ops: &[
        MCycleOp::Main(MicroOp::Idle),
        MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
        MCycleOp::Main(MicroOp::Write16msbDec(Rhs16Bit::SP, Rhs16Bit::PC)),
        MCycleOp::Main(MicroOp::Write16lsb(Rhs16Bit::SP, Rhs16Bit::PC)),
        MCycleOp::End(MicroOp::JumpVector(VectorAddress::VBlank)),
    ],
};

pub const INTERRUPT_LCD: Instruction = Instruction {
    opcode: 0x00, // Not important, interrupt routine hasn't an opcode
    name: "Interrupt - LCD",
    cycles: 5,
    size: 0,
    flags: &[],
    micro_ops: &[
        MCycleOp::Main(MicroOp::Idle),
        MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
        MCycleOp::Main(MicroOp::Write16msbDec(Rhs16Bit::SP, Rhs16Bit::PC)),
        MCycleOp::Main(MicroOp::Write16lsb(Rhs16Bit::SP, Rhs16Bit::PC)),
        MCycleOp::End(MicroOp::JumpVector(VectorAddress::STAT)),
    ],
};

pub const INTERRUPT_TIMER: Instruction = Instruction {
    opcode: 0x00, // Not important, interrupt routine hasn't an opcode
    name: "Interrupt - Timer",
    cycles: 5,
    size: 0,
    flags: &[],
    micro_ops: &[
        MCycleOp::Main(MicroOp::Idle),
        MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
        MCycleOp::Main(MicroOp::Write16msbDec(Rhs16Bit::SP, Rhs16Bit::PC)),
        MCycleOp::Main(MicroOp::Write16lsb(Rhs16Bit::SP, Rhs16Bit::PC)),
        MCycleOp::End(MicroOp::JumpVector(VectorAddress::Timer)),
    ],
};

pub const INTERRUPT_SERIAL: Instruction = Instruction {
    opcode: 0x00, // Not important, interrupt routine hasn't an opcode
    name: "Interrupt - Serial",
    cycles: 5,
    size: 0,
    flags: &[],
    micro_ops: &[
        MCycleOp::Main(MicroOp::Idle),
        MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
        MCycleOp::Main(MicroOp::Write16msbDec(Rhs16Bit::SP, Rhs16Bit::PC)),
        MCycleOp::Main(MicroOp::Write16lsb(Rhs16Bit::SP, Rhs16Bit::PC)),
        MCycleOp::End(MicroOp::JumpVector(VectorAddress::Serial)),
    ],
};

pub const INTERRUPT_JOYPAD: Instruction = Instruction {
    opcode: 0x00, // Not important, interrupt routine hasn't an opcode
    name: "Interrupt - Joypad",
    cycles: 5,
    size: 0,
    flags: &[],
    micro_ops: &[
        MCycleOp::Main(MicroOp::Idle),
        MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
        MCycleOp::Main(MicroOp::Write16msbDec(Rhs16Bit::SP, Rhs16Bit::PC)),
        MCycleOp::Main(MicroOp::Write16lsb(Rhs16Bit::SP, Rhs16Bit::PC)),
        MCycleOp::End(MicroOp::JumpVector(VectorAddress::Joypad)),
    ],
};

const fn create_opcodes() -> [Option<&'static Instruction>; 256] {
    let mut opcodes = [None; 256];
    opcodes[0x00] = Some(&Instruction {
        opcode: 0x00,
        name: "NOP",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0x01] = Some(&Instruction {
        opcode: 0x01,
        name: "LD BC, imm16",
        cycles: 3,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Lhs8Bit::Z)), // lsb
            MCycleOp::Main(MicroOp::Fetch8(Lhs8Bit::W)), // MSB
            MCycleOp::End(MicroOp::Ld16(Lhs16Bit::BC, Rhs16Bit::WZ)),
        ],
    });
    opcodes[0x02] = Some(&Instruction {
        opcode: 0x02,
        name: "LD [BC], A",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Write8(AddressRegister::BC, Rhs8Bit::A)),
            MCycleOp::End(MicroOp::Idle), // Simulating M-Cycle to reset Address Buffer to PC
        ],
    });
    opcodes[0x03] = Some(&Instruction {
        opcode: 0x03,
        name: "INC BC",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Inc16(AddressRegister::BC)),
            MCycleOp::End(MicroOp::Idle), // Simulating M-Cycle to stabilize bit signals after Inc/Dec in the 16-bit register
        ],
    });
    opcodes[0x04] = Some(&Instruction {
        opcode: 0x04,
        name: "INC B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Inc(Rhs8Bit::B)))
        ],
    });
    opcodes[0x05] = Some(&Instruction {
        opcode: 0x05,
        name: "DEC B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Dec(Rhs8Bit::B))),
        ],
    });
    opcodes[0x06] = Some(&Instruction {
        opcode: 0x06,
        name: "LD B, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::B, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x07] = Some(&Instruction {
        opcode: 0x07,
        name: "RLCA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Rlca())),
        ],
    });
    opcodes[0x08] = Some(&Instruction {
        opcode: 0x08,
        name: "LD [a16], SP",
        cycles: 5,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::W)),
            MCycleOp::Main(MicroOp::Write16lsbInc(AddressRegister::WZ, Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write16msb(AddressRegister::WZ, Rhs16Bit::SP)),
            MCycleOp::End(MicroOp::Idle), // Need to reload PC as buffer address
        ],
    });
    opcodes[0x09] = Some(&Instruction {
        opcode: 0x09,
        name: "ADD HL, BC",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Alu(AluOp::AddLsb(Lhs16Bit::HL, Lhs16Bit::BC))),
            MCycleOp::End(MicroOp::Alu(AluOp::AddMsb(Lhs16Bit::HL, Lhs16Bit::BC))),
        ],
    });
    opcodes[0x0A] = Some(&Instruction {
        opcode: 0x0A,
        name: "LD A, [BC]",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8(Lhs8Bit::Z, AddressRegister::BC)),
            MCycleOp::End(MicroOp::Ld8(Lhs8Bit::A, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x0B] = Some(&Instruction {
        opcode: 0x0B,
        name: "DEC BC",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::BC)), // This need to put BC on Address Buffer
            MCycleOp::End(MicroOp::Idle), // Need to re-latch PC in Address Buffer
        ],
    });
    opcodes[0x0C] = Some(&Instruction {
        opcode: 0x0C,
        name: "INC C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Inc(Rhs8Bit::C))),
        ],
    });
    opcodes[0x0D] = Some(&Instruction {
        opcode: 0x0D,
        name: "DEC C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Dec(Rhs8Bit::C))),
        ],
    });
    opcodes[0x0E] = Some(&Instruction {
        opcode: 0x0E,
        name: "LD C, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Lhs8Bit::Z)),
            MCycleOp::End(MicroOp::Ld8(Lhs8Bit::C, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x0F] = Some(&Instruction {
        opcode: 0x0F,
        name: "RRCA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Rrca())),
        ],
    });
    opcodes[0x10] = Some(&Instruction {
        opcode: 0x10,
        name: "STOP imm8",
        cycles: 1,
        size: 2,
        flags: &[],
        micro_ops: &[
            // TODO: implement STOP
            MCycleOp::End(MicroOp::Alu(AluOp::Rrca())),
        ],
    });
    opcodes[0x11] = Some(&Instruction {
        opcode: 0x11,
        name: "LD DE, imm16",
        cycles: 3,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::W)),
            MCycleOp::End(MicroOp::Ld16(Lhs16Bit::DE, Rhs16Bit::WZ)),
        ],
    });
    opcodes[0x12] = Some(&Instruction {
        opcode: 0x12,
        name: "LD [DE], A",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Write8(AddressRegister::DE, Rhs8Bit::A)),
            MCycleOp::End(MicroOp::Idle), // Simulating M-Cycle to reset Address Buffer to PC
        ],
    });
    opcodes[0x13] = Some(&Instruction {
        opcode: 0x13,
        name: "INC DE",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Inc16(AddressRegister::DE)),
            MCycleOp::End(MicroOp::Idle), // Simulating M-Cycle to stabilize bit signals after Inc/Dec in the 16-bit register
        ],
    });
    opcodes[0x14] = Some(&Instruction {
        opcode: 0x14,
        name: "INC D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Inc(Rhs8Bit::D)))
        ],
    });
    opcodes[0x15] = Some(&Instruction {
        opcode: 0x15,
        name: "DEC D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Dec(Rhs8Bit::D)))
        ],
    });
    opcodes[0x16] = Some(&Instruction {
        opcode: 0x16,
        name: "LD D, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Lhs8Bit::Z)),
            MCycleOp::End(MicroOp::Ld8(Lhs8Bit::D, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x17] = Some(&Instruction {
        opcode: 0x17,
        name: "RLA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Rla())),
        ],
    });
    opcodes[0x18] = Some(&Instruction {
        opcode: 0x18,
        name: "JR e8",
        cycles: 3,
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Lhs8Bit::Z)),
            MCycleOp::Main(MicroOp::SumSignedByte16(Lhs16Bit::PC, Rhs16Bit::WZ, false)),
            MCycleOp::End(MicroOp::Idu(IduOp::None(Rhs16Bit::PC, Lhs16Bit::WZ))),
        ],
    });
    opcodes[0x19] = Some(&Instruction {
        opcode: 0x19,
        name: "ADD HL, DE",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Alu(AluOp::AddLsb(Lhs16Bit::HL, Lhs16Bit::DE))),
            MCycleOp::End(MicroOp::Alu(AluOp::AddMsb(Lhs16Bit::HL, Lhs16Bit::DE))),
        ],
    });
    opcodes[0x1A] = Some(&Instruction {
        opcode: 0x1A,
        name: "LD A, [DE]",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8(Lhs8Bit::Z, AddressRegister::DE)),
            MCycleOp::End(MicroOp::Ld8(Lhs8Bit::A, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x1B] = Some(&Instruction {
        opcode: 0x1B,
        name: "DEC DE",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::DE)), // This need to put DE on Address Buffer
            MCycleOp::End(MicroOp::Idle), // Need to re-latch PC in Address Buffer
        ],
    });
    opcodes[0x1C] = Some(&Instruction {
        opcode: 0x1C,
        name: "INC E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Inc(Rhs8Bit::E)))
        ],
    });
    opcodes[0x1D] = Some(&Instruction {
        opcode: 0x1D,
        name: "DEC E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Dec(Rhs8Bit::E)))
        ],
    });
    opcodes[0x1E] = Some(&Instruction {
        opcode: 0x1E,
        name: "LD E, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Lhs8Bit::Z)),
            MCycleOp::End(MicroOp::Ld8(Lhs8Bit::E, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x1F] = Some(&Instruction {
        opcode: 0x1F,
        name: "RRA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Rra())),
        ],
    });
    opcodes[0x20] = Some(&Instruction {
        opcode: 0x20,
        name: "JR NZ, e8",
        cycles: 3, // 2 Cycles if condition doesn't match
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Cc(MicroOp::Fetch8(Lhs8Bit::Z), CheckCondition::NZ, 3),
            MCycleOp::End(MicroOp::Idle),
            MCycleOp::Main(MicroOp::SumSignedByte16(Lhs16Bit::PC, Rhs16Bit::WZ, false)),
            MCycleOp::End(MicroOp::Idu(IduOp::None(Rhs16Bit::PC, Lhs16Bit::WZ))),
        ],
    });
    opcodes[0x21] = Some(&Instruction {
        opcode: 0x21,
        name: "LD HL, imm16",
        cycles: 3,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::W)),
            MCycleOp::End(MicroOp::Ld16(Lhs16Bit::HL, Rhs16Bit::WZ)),
        ],
    });
    opcodes[0x22] = Some(&Instruction {
        opcode: 0x22,
        name: "LD [HL+], A", // Sometimes HL+ is named as HLI (HL Increment)
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Write8Inc(AddressRegister::HL, Rhs8Bit::A)),
            MCycleOp::End(MicroOp::Idle), // Simulating M-Cycle to stabilize bit signals after Inc/Dec in the 16-bit register
        ],
    });
    opcodes[0x23] = Some(&Instruction {
        opcode: 0x23,
        name: "INC HL",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Inc16(AddressRegister::HL)),
            MCycleOp::End(MicroOp::Idle), // Simulating M-Cycle to stabilize bit signals after Inc/Dec in the 16-bit register
        ],
    });
    opcodes[0x24] = Some(&Instruction {
        opcode: 0x24,
        name: "INC H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Inc(Rhs8Bit::H)))
        ],
    });
    opcodes[0x25] = Some(&Instruction {
        opcode: 0x25,
        name: "DEC H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Dec(Rhs8Bit::H)))
        ],
    });
    opcodes[0x26] = Some(&Instruction {
        opcode: 0x26,
        name: "LD H, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Ld8(Lhs8Bit::H, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x27] = Some(&Instruction {
        opcode: 0x27,
        name: "DAA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::H, FlagBits::C],
        micro_ops: &[
            //TODO: Implement
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0x28] = Some(&Instruction {
        opcode: 0x28,
        name: "JR Z, e8",
        cycles: 3, // 2 Cycles if condition doesn't match
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Cc(MicroOp::Fetch8(Lhs8Bit::Z), CheckCondition::Z, 3),
            MCycleOp::End(MicroOp::Idle),
            MCycleOp::Main(MicroOp::SumSignedByte16(Lhs16Bit::PC, Rhs16Bit::WZ, false)),
            MCycleOp::End(MicroOp::Idu(IduOp::None(Rhs16Bit::PC, Lhs16Bit::WZ))),
        ],
    });
    opcodes[0x29] = Some(&Instruction {
        opcode: 0x29,
        name: "ADD HL, HL",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Alu(AluOp::AddLsb(Lhs16Bit::HL, Lhs16Bit::HL))),
            MCycleOp::End(MicroOp::Alu(AluOp::AddMsb(Lhs16Bit::HL, Lhs16Bit::HL))),
        ],
    });
    opcodes[0x2A] = Some(&Instruction {
        opcode: 0x3A,
        name: "LD A, [HL+]",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8Inc(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::A, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x2B] = Some(&Instruction {
        opcode: 0x2B,
        name: "DEC HL",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::HL)), // This need to push HL on Address Buffer
            MCycleOp::End(MicroOp::Idle), // Need to re-latch PC in Address Buffer
        ],
    });
    opcodes[0x2C] = Some(&Instruction {
        opcode: 0x2C,
        name: "INC L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Inc(Rhs8Bit::L)))
        ],
    });
    opcodes[0x2D] = Some(&Instruction {
        opcode: 0x2D,
        name: "DEC L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Dec(Rhs8Bit::E)))
        ],
    });
    opcodes[0x2E] = Some(&Instruction {
        opcode: 0x2E,
        name: "LD L, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Ld8(Lhs8Bit::L, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x2F] = Some(&Instruction {
        opcode: 0x2F,
        name: "CPL",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Cpl(Rhs8Bit::A))),
        ],
    });
    opcodes[0x30] = Some(&Instruction {
        opcode: 0x20,
        name: "JR NC, e8",
        cycles: 3, // 2 Cycles if condition doesn't match
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Cc(MicroOp::Fetch8(Lhs8Bit::Z), CheckCondition::NC, 3),
            MCycleOp::End(MicroOp::Idle),
            MCycleOp::Main(MicroOp::SumSignedByte16(Lhs16Bit::PC, Rhs16Bit::WZ, false)),
            MCycleOp::End(MicroOp::Idu(IduOp::None(Rhs16Bit::PC, Lhs16Bit::WZ))),
        ],
    });
    opcodes[0x31] = Some(&Instruction {
        opcode: 0x31,
        name: "LD SP, imm16",
        cycles: 3,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::W)),
            MCycleOp::End(MicroOp::Ld16(Lhs16Bit::SP, Rhs16Bit::WZ)),
        ],
    });
    opcodes[0x32] = Some(&Instruction {
        opcode: 0x32,
        name: "LD [HL-], A", // Sometimes HL- is named as HLD (HL Decrement)
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Write8Dec(AddressRegister::HL, Rhs8Bit::A)),
            MCycleOp::End(MicroOp::Idle), // Simulating M-Cycle to stabilize bit signals after Inc/Dec in the 16-bit register
        ],
    });
    opcodes[0x33] = Some(&Instruction {
        opcode: 0x33,
        name: "INC SP",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Inc16(AddressRegister::SP)),
            MCycleOp::End(MicroOp::Idle), // Simulating M-Cycle to stabilize bit signals after Inc/Dec in the 16-bit register
        ],
    });
    opcodes[0x34] = Some(&Instruction {
        opcode: 0x34,
        name: "INC [HL]",
        cycles: 3,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::Main(MicroOp::AluAndWrite8(AluOp::Inc(Rhs8Bit::Z), AddressRegister::HL, Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Idle), // Simulating M-Cycle to restore Buffer Address to PC
        ],
    });
    opcodes[0x35] = Some(&Instruction {
        opcode: 0x35,
        name: "DEC [HL]",
        cycles: 3,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::Main(MicroOp::AluAndWrite8(AluOp::Dec(Rhs8Bit::Z), AddressRegister::HL, Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Idle), // Simulating M-Cycle to restore Buffer Address to PC
        ],
    });
    opcodes[0x36] = Some(&Instruction {
        opcode: 0x36,
        name: "LD [HL], imm8",
        cycles: 3,
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Main(MicroOp::Write8(AddressRegister::HL, Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Idle), // Re-latching of Address Buffer to PC
        ],
    });
    opcodes[0x37] = Some(&Instruction {
        opcode: 0x37,
        name: "SCF",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::SetFlags(SetFlagZ::Same, SetFlagN::Off, SetFlagH::Off, SetFlagC::On))),
        ],
    });
    opcodes[0x38] = Some(&Instruction {
        opcode: 0x38,
        name: "JR C, e8",
        cycles: 3, // 2 Cycles if condition doesn't match
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Cc(MicroOp::Fetch8(Lhs8Bit::Z), CheckCondition::C, 3),
            MCycleOp::End(MicroOp::Idle),
            MCycleOp::Main(MicroOp::SumSignedByte16(Lhs16Bit::PC, Rhs16Bit::WZ, false)),
            MCycleOp::End(MicroOp::Idu(IduOp::None(Rhs16Bit::PC, Lhs16Bit::WZ))),
        ],
    });
    opcodes[0x39] = Some(&Instruction {
        opcode: 0x39,
        name: "ADD HL, SP",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Alu(AluOp::AddLsb(Lhs16Bit::HL, Rhs16Bit::SP))),
            MCycleOp::End(MicroOp::Alu(AluOp::AddMsb(Lhs16Bit::HL, Rhs16Bit::SP))),
        ],
    });
    opcodes[0x3A] = Some(&Instruction {
        opcode: 0x3A,
        name: "LD A, [HL-]",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8Dec(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::A, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x3B] = Some(&Instruction {
        opcode: 0x3B,
        name: "DEC SP",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(AddressRegister::HL)),
            MCycleOp::End(MicroOp::Idle), // Simulating M-Cycle to stabilize bit signals after Inc/Dec in the 16-bit register
        ],
    });
    opcodes[0x3C] = Some(&Instruction {
        opcode: 0x3C,
        name: "INC A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Inc(Rhs8Bit::A))),
        ],
    });
    opcodes[0x3D] = Some(&Instruction {
        opcode: 0x3D,
        name: "DEC A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Dec(Rhs8Bit::A))),
        ],
    });
    opcodes[0x3E] = Some(&Instruction {
        opcode: 0x3E,
        name: "LD A, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::A, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x3F] = Some(&Instruction {
        opcode: 0x3F,
        name: "CCF",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::SetFlags(SetFlagZ::Same, SetFlagN::Off, SetFlagH::Off, SetFlagC::On))),
        ],
    });
    opcodes[0x40] = Some(&Instruction {
        opcode: 0x40,
        name: "LD B, B",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::B, Rhs8Bit::B)),
        ],
    });
    opcodes[0x41] = Some(&Instruction {
        opcode: 0x41,
        name: "LD B, C",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::B, Rhs8Bit::C)),
        ],
    });
    opcodes[0x42] = Some(&Instruction {
        opcode: 0x42,
        name: "LD B, D",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::B, Rhs8Bit::D)),
        ],
    });
    opcodes[0x43] = Some(&Instruction {
        opcode: 0x43,
        name: "LD B, E",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::B, Rhs8Bit::E)),
        ],
    });
    opcodes[0x44] = Some(&Instruction {
        opcode: 0x44,
        name: "LD B, H",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::B, Rhs8Bit::H)),
        ],
    });
    opcodes[0x45] = Some(&Instruction {
        opcode: 0x45,
        name: "LD B, L",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::B, Rhs8Bit::L)),
        ],
    });
    opcodes[0x46] = Some(&Instruction {
        opcode: 0x46,
        name: "LD B, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::B, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x47] = Some(&Instruction {
        opcode: 0x47,
        name: "LD B, A",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::B, Rhs8Bit::A)),
        ],
    });
    opcodes[0x48] = Some(&Instruction {
        opcode: 0x48,
        name: "LD C, B",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::C, Rhs8Bit::B)),
        ],
    });
    opcodes[0x49] = Some(&Instruction {
        opcode: 0x49,
        name: "LD C, C",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::C, Rhs8Bit::C)),
        ],
    });
    opcodes[0x4A] = Some(&Instruction {
        opcode: 0x4A,
        name: "LD C, D",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::C, Rhs8Bit::D)),
        ],
    });
    opcodes[0x4B] = Some(&Instruction {
        opcode: 0x4B,
        name: "LD C, E",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::C, Rhs8Bit::E)),
        ],
    });
    opcodes[0x4C] = Some(&Instruction {
        opcode: 0x4C,
        name: "LD C, H",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::C, Rhs8Bit::H)),
        ],
    });
    opcodes[0x4D] = Some(&Instruction {
        opcode: 0x4D,
        name: "LD C, L",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::C, Rhs8Bit::L)),
        ],
    });
    opcodes[0x4E] = Some(&Instruction {
        opcode: 0x4E,
        name: "LD C, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::C, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x4F] = Some(&Instruction {
        opcode: 0x4F,
        name: "LD C, A",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::C, Rhs8Bit::A)),
        ],
    });
    opcodes[0x50] = Some(&Instruction {
        opcode: 0x50,
        name: "LD D, B",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::D, Rhs8Bit::B)),
        ],
    });
    opcodes[0x51] = Some(&Instruction {
        opcode: 0x51,
        name: "LD D, C",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::D, Rhs8Bit::C)),
        ],
    });
    opcodes[0x52] = Some(&Instruction {
        opcode: 0x52,
        name: "LD D, D",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::D, Rhs8Bit::D)),
        ],
    });
    opcodes[0x53] = Some(&Instruction {
        opcode: 0x53,
        name: "LD D, E",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::D, Rhs8Bit::E)),
        ],
    });
    opcodes[0x54] = Some(&Instruction {
        opcode: 0x54,
        name: "LD D, H",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::D, Rhs8Bit::H)),
        ],
    });
    opcodes[0x55] = Some(&Instruction {
        opcode: 0x55,
        name: "LD D, L",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::D, Rhs8Bit::L)),
        ],
    });
    opcodes[0x56] = Some(&Instruction {
        opcode: 0x56,
        name: "LD D, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::D, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x57] = Some(&Instruction {
        opcode: 0x57,
        name: "LD D, A",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::D, Rhs8Bit::A)),
        ],
    });
    opcodes[0x58] = Some(&Instruction {
        opcode: 0x58,
        name: "LD E, B",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::E, Rhs8Bit::B)),
        ],
    });
    opcodes[0x59] = Some(&Instruction {
        opcode: 0x59,
        name: "LD E, C",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::E, Rhs8Bit::C)),
        ],
    });
    opcodes[0x5A] = Some(&Instruction {
        opcode: 0x5A,
        name: "LD E, D",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::E, Rhs8Bit::D)),
        ],
    });
    opcodes[0x5B] = Some(&Instruction {
        opcode: 0x5B,
        name: "LD E, E",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::E, Rhs8Bit::E)),
        ],
    });
    opcodes[0x5C] = Some(&Instruction {
        opcode: 0x5C,
        name: "LD E, H",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::E, Rhs8Bit::H)),
        ],
    });
    opcodes[0x5D] = Some(&Instruction {
        opcode: 0x5D,
        name: "LD E, L",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::E, Rhs8Bit::L)),
        ],
    });
    opcodes[0x5E] = Some(&Instruction {
        opcode: 0x5E,
        name: "LD E, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::E, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x5F] = Some(&Instruction {
        opcode: 0x5F,
        name: "LD E, A",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::E, Rhs8Bit::A)),
        ],
    });
    opcodes[0x60] = Some(&Instruction {
        opcode: 0x60,
        name: "LD H, B",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::H, Rhs8Bit::B)),
        ],
    });
    opcodes[0x61] = Some(&Instruction {
        opcode: 0x61,
        name: "LD H, C",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::H, Rhs8Bit::C)),
        ],
    });
    opcodes[0x62] = Some(&Instruction {
        opcode: 0x62,
        name: "LD H, D",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::H, Rhs8Bit::D)),
        ],
    });
    opcodes[0x63] = Some(&Instruction {
        opcode: 0x63,
        name: "LD H, E",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::H, Rhs8Bit::E)),
        ],
    });
    opcodes[0x64] = Some(&Instruction {
        opcode: 0x64,
        name: "LD H, H",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::H, Rhs8Bit::H)),
        ],
    });
    opcodes[0x65] = Some(&Instruction {
        opcode: 0x65,
        name: "LD H, L",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::H, Rhs8Bit::L)),
        ],
    });
    opcodes[0x66] = Some(&Instruction {
        opcode: 0x66,
        name: "LD H, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::H, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x67] = Some(&Instruction {
        opcode: 0x67,
        name: "LD H, A",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::H, Rhs8Bit::A)),
        ],
    });
    opcodes[0x68] = Some(&Instruction {
        opcode: 0x68,
        name: "LD L, B",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::L, Rhs8Bit::B)),
        ],
    });
    opcodes[0x69] = Some(&Instruction {
        opcode: 0x69,
        name: "LD L, C",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::L, Rhs8Bit::C)),
        ],
    });
    opcodes[0x6A] = Some(&Instruction {
        opcode: 0x6A,
        name: "LD L, D",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::L, Rhs8Bit::D)),
        ],
    });
    opcodes[0x6B] = Some(&Instruction {
        opcode: 0x6B,
        name: "LD L, E",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::L, Rhs8Bit::E)),
        ],
    });
    opcodes[0x6C] = Some(&Instruction {
        opcode: 0x6C,
        name: "LD L, H",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::L, Rhs8Bit::H)),
        ],
    });
    opcodes[0x6D] = Some(&Instruction {
        opcode: 0x6D,
        name: "LD L, L",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::L, Rhs8Bit::L)),
        ],
    });
    opcodes[0x6E] = Some(&Instruction {
        opcode: 0x6E,
        name: "LD L, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::L, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x6F] = Some(&Instruction {
        opcode: 0x6F,
        name: "LD L, A",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::L, Rhs8Bit::A)),
        ],
    });
    opcodes[0x70] = Some(&Instruction {
        opcode: 0x70,
        name: "LD [HL], B",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Write8(AddressRegister::HL, Rhs8Bit::B)),
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0x71] = Some(&Instruction {
        opcode: 0x71,
        name: "LD [HL], C",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Write8(AddressRegister::HL, Rhs8Bit::C)),
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0x72] = Some(&Instruction {
        opcode: 0x72,
        name: "LD [HL], D",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Write8(AddressRegister::HL, Rhs8Bit::D)),
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0x73] = Some(&Instruction {
        opcode: 0x73,
        name: "LD [HL], E",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Write8(AddressRegister::HL, Rhs8Bit::E)),
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0x74] = Some(&Instruction {
        opcode: 0x74,
        name: "LD [HL], H",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Write8(AddressRegister::HL, Rhs8Bit::H)),
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0x75] = Some(&Instruction {
        opcode: 0x75,
        name: "LD [HL], L",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Write8(AddressRegister::HL, Rhs8Bit::L)),
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0x76] = Some(&Instruction {
        opcode: 0x76,
        name: "HALT",  // LD [HL], [HL] Decode as HALT instruction on DMG/GBC
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            // Todo: implement HALT microcode
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0x77] = Some(&Instruction {
        opcode: 0x77,
        name: "LD [HL], A",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Write8(AddressRegister::HL, Rhs8Bit::A)),
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0x78] = Some(&Instruction {
        opcode: 0x78,
        name: "LD A, B",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::A, Rhs8Bit::B)),
        ],
    });
    opcodes[0x79] = Some(&Instruction {
        opcode: 0x79,
        name: "LD A, C",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::A, Rhs8Bit::C)),
        ],
    });
    opcodes[0x7A] = Some(&Instruction {
        opcode: 0x7A,
        name: "LD A, D",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::A, Rhs8Bit::D)),
        ],
    });
    opcodes[0x7B] = Some(&Instruction {
        opcode: 0x7B,
        name: "LD A, E",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::A, Rhs8Bit::E)),
        ],
    });
    opcodes[0x7C] = Some(&Instruction {
        opcode: 0x7C,
        name: "LD A, H",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::A, Rhs8Bit::H)),
        ],
    });
    opcodes[0x7D] = Some(&Instruction {
        opcode: 0x7D,
        name: "LD A, L",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::A, Rhs8Bit::L)),
        ],
    });
    opcodes[0x7E] = Some(&Instruction {
        opcode: 0x7E,
        name: "LD A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::A, Rhs8Bit::Z)),
        ],
    });
    opcodes[0x7F] = Some(&Instruction {
        opcode: 0x7F,
        name: "LD A, A",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::A, Rhs8Bit::A)),
        ],
    });
    opcodes[0x80] = Some(&Instruction {
        opcode: 0x80,
        name: "ADD A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Add(Rhs8Bit::A, Rhs8Bit::B))),
        ],
    });
    opcodes[0x81] = Some(&Instruction {
        opcode: 0x81,
        name: "ADD A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Add(Rhs8Bit::A, Rhs8Bit::C))),
        ],
    });
    opcodes[0x82] = Some(&Instruction {
        opcode: 0x82,
        name: "ADD A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Add(Rhs8Bit::A, Rhs8Bit::D))),
        ],
    });
    opcodes[0x83] = Some(&Instruction {
        opcode: 0x83,
        name: "ADD A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Add(Rhs8Bit::A, Rhs8Bit::E))),
        ],
    });
    opcodes[0x84] = Some(&Instruction {
        opcode: 0x84,
        name: "ADD A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Add(Rhs8Bit::A, Rhs8Bit::H))),
        ],
    });
    opcodes[0x85] = Some(&Instruction {
        opcode: 0x85,
        name: "ADD A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Add(Rhs8Bit::A, Rhs8Bit::L))),
        ],
    });
    opcodes[0x86] = Some(&Instruction {
        opcode: 0x86,
        name: "ADD A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Alu(AluOp::Add(Rhs8Bit::A, Rhs8Bit::Z))),
        ],
    });
    opcodes[0x87] = Some(&Instruction {
        opcode: 0x87,
        name: "ADD A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Add(Rhs8Bit::A, Rhs8Bit::B))),
        ],
    });
    opcodes[0x88] = Some(&Instruction {
        opcode: 0x88,
        name: "ADC A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Adc(Rhs8Bit::A, Rhs8Bit::B))),
        ],
    });
    opcodes[0x89] = Some(&Instruction {
        opcode: 0x88,
        name: "ADC A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Adc(Rhs8Bit::A, Rhs8Bit::C))),
        ],
    });
    opcodes[0x8A] = Some(&Instruction {
        opcode: 0x8A,
        name: "ADC A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Adc(Rhs8Bit::A, Rhs8Bit::D))),
        ],
    });
    opcodes[0x8B] = Some(&Instruction {
        opcode: 0x8B,
        name: "ADC A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Adc(Rhs8Bit::A, Rhs8Bit::E))),
        ],
    });
    opcodes[0x8C] = Some(&Instruction {
        opcode: 0x8C,
        name: "ADC A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Adc(Rhs8Bit::A, Rhs8Bit::H))),
        ],
    });
    opcodes[0x8D] = Some(&Instruction {
        opcode: 0x8D,
        name: "ADC A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Adc(Rhs8Bit::A, Rhs8Bit::L))),
        ],
    });
    opcodes[0x8E] = Some(&Instruction {
        opcode: 0x8E,
        name: "ADC A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Alu(AluOp::Adc(Rhs8Bit::A, Rhs8Bit::Z))),
        ],
    });
    opcodes[0x8F] = Some(&Instruction {
        opcode: 0x8F,
        name: "ADC A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Adc(Rhs8Bit::A, Rhs8Bit::A))),
        ],
    });
    opcodes[0x90] = Some(&Instruction {
        opcode: 0x90,
        name: "SUB A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Sub(Rhs8Bit::A, Rhs8Bit::B))),
        ],
    });
    opcodes[0x91] = Some(&Instruction {
        opcode: 0x91,
        name: "SUB A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Sub(Rhs8Bit::A, Rhs8Bit::C))),
        ],
    });
    opcodes[0x92] = Some(&Instruction {
        opcode: 0x92,
        name: "SUB A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Sub(Rhs8Bit::A, Rhs8Bit::D))),
        ],
    });
    opcodes[0x93] = Some(&Instruction {
        opcode: 0x93,
        name: "SUB A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Sub(Rhs8Bit::A, Rhs8Bit::E))),
        ],
    });
    opcodes[0x94] = Some(&Instruction {
        opcode: 0x94,
        name: "SUB A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Sub(Rhs8Bit::A, Rhs8Bit::H))),
        ],
    });
    opcodes[0x95] = Some(&Instruction {
        opcode: 0x95,
        name: "SUB A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Sub(Rhs8Bit::A, Rhs8Bit::L))),
        ],
    });
    opcodes[0x96] = Some(&Instruction {
        opcode: 0x96,
        name: "SUB A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Alu(AluOp::Sub(Rhs8Bit::A, Rhs8Bit::Z))),
        ],
    });
    opcodes[0x97] = Some(&Instruction {
        opcode: 0x97,
        name: "SUB A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Sub(Rhs8Bit::A, Rhs8Bit::A))),
        ],
    });
    opcodes[0x98] = Some(&Instruction {
        opcode: 0x98,
        name: "SBC A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Sbc(Rhs8Bit::A, Rhs8Bit::B))),
        ],
    });
    opcodes[0x99] = Some(&Instruction {
        opcode: 0x99,
        name: "SBC A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Sbc(Rhs8Bit::A, Rhs8Bit::C))),
        ],
    });
    opcodes[0x9A] = Some(&Instruction {
        opcode: 0x9A,
        name: "SBC A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Sbc(Rhs8Bit::A, Rhs8Bit::D))),
        ],
    });
    opcodes[0x9B] = Some(&Instruction {
        opcode: 0x9B,
        name: "SBC A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Sbc(Rhs8Bit::A, Rhs8Bit::E))),
        ],
    });
    opcodes[0x9C] = Some(&Instruction {
        opcode: 0x9C,
        name: "SBC A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Sbc(Rhs8Bit::A, Rhs8Bit::H))),
        ],
    });
    opcodes[0x9D] = Some(&Instruction {
        opcode: 0x9D,
        name: "SBC A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Sbc(Rhs8Bit::A, Rhs8Bit::L))),
        ],
    });
    opcodes[0x9E] = Some(&Instruction {
        opcode: 0x9E,
        name: "SBC A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Alu(AluOp::Sbc(Rhs8Bit::A, Rhs8Bit::Z))),
        ],
    });
    opcodes[0x9F] = Some(&Instruction {
        opcode: 0x9F,
        name: "SBC A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Sbc(Rhs8Bit::A, Rhs8Bit::A))),
        ],
    });
    opcodes[0xA0] = Some(&Instruction {
        opcode: 0xA0,
        name: "AND A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::And(Rhs8Bit::A, Rhs8Bit::B))),
        ],
    });
    opcodes[0xA1] = Some(&Instruction {
        opcode: 0xA1,
        name: "AND A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::And(Rhs8Bit::A, Rhs8Bit::C))),
        ],
    });
    opcodes[0xA2] = Some(&Instruction {
        opcode: 0xA2,
        name: "AND A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::And(Rhs8Bit::A, Rhs8Bit::D))),
        ],
    });
    opcodes[0xA3] = Some(&Instruction {
        opcode: 0xA3,
        name: "AND A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::And(Rhs8Bit::A, Rhs8Bit::E))),
        ],
    });
    opcodes[0xA4] = Some(&Instruction {
        opcode: 0xA4,
        name: "AND A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::And(Rhs8Bit::A, Rhs8Bit::H))),
        ],
    });
    opcodes[0xA5] = Some(&Instruction {
        opcode: 0xA5,
        name: "AND A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::And(Rhs8Bit::A, Rhs8Bit::L))),
        ],
    });
    opcodes[0xA6] = Some(&Instruction {
        opcode: 0xA6,
        name: "AND A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Alu(AluOp::And(Rhs8Bit::A, Rhs8Bit::Z))),
        ],
    });
    opcodes[0xA7] = Some(&Instruction {
        opcode: 0xA7,
        name: "AND A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::And(Rhs8Bit::A, Rhs8Bit::A))),
        ],
    });
    opcodes[0xA8] = Some(&Instruction {
        opcode: 0xA8,
        name: "XOR A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Xor(Rhs8Bit::A, Rhs8Bit::B))),
        ],
    });
    opcodes[0xA9] = Some(&Instruction {
        opcode: 0xA9,
        name: "XOR A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Xor(Rhs8Bit::A, Rhs8Bit::C))),
        ],
    });
    opcodes[0xAA] = Some(&Instruction {
        opcode: 0xAA,
        name: "XOR A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Xor(Rhs8Bit::A, Rhs8Bit::D))),
        ],
    });
    opcodes[0xAB] = Some(&Instruction {
        opcode: 0xAB,
        name: "XOR A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Xor(Rhs8Bit::A, Rhs8Bit::E))),
        ],
    });
    opcodes[0xAC] = Some(&Instruction {
        opcode: 0xAC,
        name: "XOR A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Xor(Rhs8Bit::A, Rhs8Bit::H))),
        ],
    });
    opcodes[0xAD] = Some(&Instruction {
        opcode: 0xAD,
        name: "XOR A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Xor(Rhs8Bit::A, Rhs8Bit::L))),
        ],
    });
    opcodes[0xAE] = Some(&Instruction {
        opcode: 0xAE,
        name: "XOR A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Alu(AluOp::Xor(Rhs8Bit::A, Rhs8Bit::Z))),
        ],
    });
    opcodes[0xAF] = Some(&Instruction {
        opcode: 0xAF,
        name: "XOR A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Xor(Rhs8Bit::A, Rhs8Bit::A))),
        ],
    });
    opcodes[0xB0] = Some(&Instruction {
        opcode: 0xB0,
        name: "OR A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Or(Rhs8Bit::A, Rhs8Bit::B))),
        ],
    });
    opcodes[0xB1] = Some(&Instruction {
        opcode: 0xB1,
        name: "OR A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Or(Rhs8Bit::A, Rhs8Bit::C))),
        ],
    });
    opcodes[0xB2] = Some(&Instruction {
        opcode: 0xB2,
        name: "OR A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Or(Rhs8Bit::A, Rhs8Bit::D))),
        ],
    });
    opcodes[0xB3] = Some(&Instruction {
        opcode: 0xB3,
        name: "OR A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Or(Rhs8Bit::A, Rhs8Bit::E))),
        ],
    });
    opcodes[0xB4] = Some(&Instruction {
        opcode: 0xB4,
        name: "OR A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Or(Rhs8Bit::A, Rhs8Bit::H))),
        ],
    });
    opcodes[0xB5] = Some(&Instruction {
        opcode: 0xB5,
        name: "OR A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Or(Rhs8Bit::A, Rhs8Bit::L))),
        ],
    });
    opcodes[0xB6] = Some(&Instruction {
        opcode: 0xB6,
        name: "OR A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Alu(AluOp::Or(Rhs8Bit::A, Rhs8Bit::Z))),
        ],
    });
    opcodes[0xB7] = Some(&Instruction {
        opcode: 0xB7,
        name: "OR A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Or(Rhs8Bit::A, Rhs8Bit::A))),
        ],
    });
    opcodes[0xB8] = Some(&Instruction {
        opcode: 0xB8,
        name: "CP A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Cp(Rhs8Bit::A, Rhs8Bit::B))),
        ],
    });
    opcodes[0xB9] = Some(&Instruction {
        opcode: 0xB9,
        name: "CP A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Cp(Rhs8Bit::A, Rhs8Bit::C))),
        ],
    });
    opcodes[0xBA] = Some(&Instruction {
        opcode: 0xBA,
        name: "CP A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Cp(Rhs8Bit::A, Rhs8Bit::D))),
        ],
    });
    opcodes[0xBB] = Some(&Instruction {
        opcode: 0xBB,
        name: "CP A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Cp(Rhs8Bit::A, Rhs8Bit::E))),
        ],
    });
    opcodes[0xBC] = Some(&Instruction {
        opcode: 0xBC,
        name: "CP A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Cp(Rhs8Bit::A, Rhs8Bit::H))),
        ],
    });
    opcodes[0xBD] = Some(&Instruction {
        opcode: 0xBD,
        name: "CP A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Cp(Rhs8Bit::A, Rhs8Bit::L))),
        ],
    });
    opcodes[0xBE] = Some(&Instruction {
        opcode: 0xBE,
        name: "CP A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Read8(Rhs8Bit::Z, AddressRegister::HL)),
            MCycleOp::End(MicroOp::Alu(AluOp::Cp(Rhs8Bit::A, Rhs8Bit::Z))),
        ],
    });
    opcodes[0xBF] = Some(&Instruction {
        opcode: 0xBF,
        name: "CP A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Alu(AluOp::Cp(Rhs8Bit::A, Rhs8Bit::A))),
        ],
    });
    opcodes[0xC0] = Some(&Instruction {
        opcode: 0xC0,
        name: "RET NZ",
        cycles: 5,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Cc(MicroOp::Idle, CheckCondition::NZ, 2),
            MCycleOp::End(MicroOp::Idle),
            MCycleOp::Main(MicroOp::Read16lsbInc(Rhs16Bit::WZ, AddressRegister::SP)),
            MCycleOp::Main(MicroOp::Read16msbInc(Rhs16Bit::WZ, AddressRegister::SP)),
            MCycleOp::Main(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)),
            MCycleOp::End(MicroOp::Idle), // Restoring address buffer to PC
        ],
    });
    opcodes[0xC1] = Some(&Instruction {
        opcode: 0xC1,
        name: "POP BC",
        cycles: 3,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8Inc(Rhs8Bit::Z, AddressRegister::SP)), // lsb
            MCycleOp::Main(MicroOp::Read8Inc(Rhs8Bit::W, AddressRegister::SP)), // MSB
            MCycleOp::End(MicroOp::Ld16(Rhs16Bit::BC, Rhs16Bit::WZ)),
        ],
    });
    opcodes[0xC2] = Some(&Instruction {
        opcode: 0xC2,
        name: "JP NZ, imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Cc(MicroOp::Fetch8(Rhs8Bit::W), CheckCondition::NZ, 3),
            MCycleOp::End(MicroOp::Idle), // Restoring address buffer to PC
            MCycleOp::Main(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)),
            MCycleOp::End(MicroOp::Idle), // Restoring address buffer to PC
        ],
    });
    opcodes[0xC3] = Some(&Instruction {
        opcode: 0xC3,
        name: "JP imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::W)),
            MCycleOp::Main(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)),
            MCycleOp::End(MicroOp::Idle), // Restoring address buffer to PC
        ],
    });
    opcodes[0xC4] = Some(&Instruction {
        opcode: 0xC4,
        name: "CALL NZ, imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Cc(MicroOp::Fetch8(Rhs8Bit::W), CheckCondition::NZ, 3),
            MCycleOp::End(MicroOp::Idle),
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write16msbDec(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::Main(MicroOp::Write16lsb(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::End(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)), // TODO Refactor: This action is in the previous M-Cycle in parallel with the PC lsb push
        ],
    });
    opcodes[0xC5] = Some(&Instruction {
        opcode: 0xC5,
        name: "PUSH BC",
        cycles: 4,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write8Inc(AddressRegister::SP, Rhs8Bit::B)), // MSB
            MCycleOp::Main(MicroOp::Write8(AddressRegister::SP, Rhs8Bit::C)), // lsb
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0xC6] = Some(&Instruction {
        opcode: 0xC6,
        name: "ADD A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Alu(AluOp::Add(Rhs8Bit::A, Rhs8Bit::Z))),
        ],
    });
    opcodes[0xC7] = Some(&Instruction {
        opcode: 0xC7,
        name: "RST $00",
        cycles: 4,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write16msbDec(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::Main(MicroOp::Write16lsb(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::End(MicroOp::JumpVector(VectorAddress::V0)), // TODO Refactor: This action is in the previous M-Cycle in parallel with the PC lsb push
        ],
    });
    opcodes[0xC8] = Some(&Instruction {
        opcode: 0xC8,
        name: "RET Z",
        cycles: 5,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Cc(MicroOp::Idle, CheckCondition::Z, 2),
            MCycleOp::End(MicroOp::Idle),
            MCycleOp::Main(MicroOp::Read16lsbInc(Rhs16Bit::WZ, AddressRegister::SP)),
            MCycleOp::Main(MicroOp::Read16msbInc(Rhs16Bit::WZ, AddressRegister::SP)),
            MCycleOp::Main(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)),
            MCycleOp::End(MicroOp::Idle), // Restoring address buffer to PC
        ],
    });
    opcodes[0xC9] = Some(&Instruction {
        opcode: 0xC9,
        name: "RET",
        cycles: 4,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read16lsbInc(Rhs16Bit::WZ, AddressRegister::SP)),
            MCycleOp::Main(MicroOp::Read16msbInc(Rhs16Bit::WZ, AddressRegister::SP)),
            MCycleOp::Main(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)),
            MCycleOp::End(MicroOp::Idle), // Restoring address buffer to PC
        ],
    });
    opcodes[0xCA] = Some(&Instruction {
        opcode: 0xCA,
        name: "JP Z, imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Cc(MicroOp::Fetch8(Rhs8Bit::W), CheckCondition::Z, 3),
            MCycleOp::End(MicroOp::Idle), // Restoring address buffer to PC
            MCycleOp::Main(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)),
            MCycleOp::End(MicroOp::Idle), // Restoring address buffer to PC
        ],
    });
    opcodes[0xCB] = Some(&Instruction {
        opcode: 0xCB,
        name: "CB SUBSET",
        cycles: 0,
        size: 0,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::PrefixCB), // Restoring address buffer to PC
        ],
    });
    opcodes[0xCC] = Some(&Instruction {
        opcode: 0xCC,
        name: "CALL Z, imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Cc(MicroOp::Fetch8(Rhs8Bit::W), CheckCondition::Z, 3),
            MCycleOp::End(MicroOp::Idle),
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write16msbDec(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::Main(MicroOp::Write16lsb(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::End(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)), // TODO Refactor: This action is in the previous M-Cycle in parallel with the PC lsb push
        ],
    });
    opcodes[0xCD] = Some(&Instruction {
        opcode: 0xCD,
        name: "CALL imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::W)),
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write16msbDec(AddressRegister::SP, Rhs16Bit::PC)), // MSB
            MCycleOp::Main(MicroOp::Write16lsb(AddressRegister::SP, Rhs16Bit::PC)), // lsb
            MCycleOp::End(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)), // TODO Future-Refactor: not the exact timing, this happen in the previous M-Cycle after stored PC lsb
        ],
    });
    opcodes[0xCE] = Some(&Instruction {
        opcode: 0xCE,
        name: "ADC A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Alu(AluOp::Adc(Rhs8Bit::A, Rhs8Bit::Z))),
        ],
    });
    opcodes[0xCF] = Some(&Instruction {
        opcode: 0xCF,
        name: "RST $08",
        cycles: 4,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write16msbDec(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::Main(MicroOp::Write16lsb(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::End(MicroOp::JumpVector(VectorAddress::V1)), // TODO Refactor: This action is in the previous M-Cycle in parallel with the PC lsb push
        ],
    });
    opcodes[0xD0] = Some(&Instruction {
        opcode: 0xD0,
        name: "RET NC",
        cycles: 5,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Cc(MicroOp::Idle, CheckCondition::NC, 2),
            MCycleOp::End(MicroOp::Idle),
            MCycleOp::Main(MicroOp::Read16lsbInc(Rhs16Bit::WZ, AddressRegister::SP)),
            MCycleOp::Main(MicroOp::Read16msbInc(Rhs16Bit::WZ, AddressRegister::SP)),
            MCycleOp::Main(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)),
            MCycleOp::End(MicroOp::Idle), // Restoring address buffer to PC
        ],
    });
    opcodes[0xD1] = Some(&Instruction {
        opcode: 0xD1,
        name: "POP DE",
        cycles: 3,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8Inc(Rhs8Bit::Z, AddressRegister::SP)), // lsb
            MCycleOp::Main(MicroOp::Read8Inc(Rhs8Bit::W, AddressRegister::SP)), // MSB
            MCycleOp::End(MicroOp::Ld16(Rhs16Bit::DE, Rhs16Bit::WZ)),
        ],
    });
    opcodes[0xD2] = Some(&Instruction {
        opcode: 0xD2,
        name: "JP NC, imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Cc(MicroOp::Fetch8(Rhs8Bit::W), CheckCondition::NC, 3),
            MCycleOp::End(MicroOp::Idle), // Restoring address buffer to PC
            MCycleOp::Main(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)),
            MCycleOp::End(MicroOp::Idle), // Restoring address buffer to PC
        ],
    });
    opcodes[0xD4] = Some(&Instruction {
        opcode: 0xD4,
        name: "CALL NC, imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Cc(MicroOp::Fetch8(Rhs8Bit::W), CheckCondition::NC, 3),
            MCycleOp::End(MicroOp::Idle),
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write16msbDec(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::Main(MicroOp::Write16lsb(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::End(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)), // TODO Refactor: This action is in the previous M-Cycle in parallel with the PC lsb push
        ],
    });
    opcodes[0xD5] = Some(&Instruction {
        opcode: 0xD5,
        name: "PUSH DE",
        cycles: 4,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write8Inc(AddressRegister::SP, Rhs8Bit::D)), // MSB
            MCycleOp::Main(MicroOp::Write8(AddressRegister::SP, Rhs8Bit::E)), // lsb
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0xD6] = Some(&Instruction {
        opcode: 0xD6,
        name: "SUB A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Alu(AluOp::Sub(Rhs8Bit::A, Rhs8Bit::Z))),
        ],
    });
    opcodes[0xD7] = Some(&Instruction {
        opcode: 0xD7,
        name: "RST $10",
        cycles: 4,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write16msbDec(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::Main(MicroOp::Write16lsb(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::End(MicroOp::JumpVector(VectorAddress::V2)), // TODO Refactor: This action is in the previous M-Cycle in parallel with the PC lsb push
        ],
    });
    opcodes[0xD8] = Some(&Instruction {
        opcode: 0xD8,
        name: "RET C",
        cycles: 5,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Cc(MicroOp::Idle, CheckCondition::C, 2),
            MCycleOp::End(MicroOp::Idle),
            MCycleOp::Main(MicroOp::Read16lsbInc(Rhs16Bit::WZ, AddressRegister::SP)),
            MCycleOp::Main(MicroOp::Read16msbInc(Rhs16Bit::WZ, AddressRegister::SP)),
            MCycleOp::Main(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)),
            MCycleOp::End(MicroOp::Idle), // Restoring address buffer to PC
        ],
    });
    opcodes[0xD9] = Some(&Instruction {
        opcode: 0xD9,
        name: "RETI",
        cycles: 4,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read16lsbInc(Rhs16Bit::WZ, AddressRegister::SP)),
            MCycleOp::Main(MicroOp::Read16msbInc(Rhs16Bit::WZ, AddressRegister::SP)),
            MCycleOp::Main(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)),
            MCycleOp::End(MicroOp::ImeEnabled(true)), // TODO Refactor: This action is in the previous M-Cycle in parallel with Ld16
        ],
    });
    opcodes[0xDA] = Some(&Instruction {
        opcode: 0xDA,
        name: "JP C, imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Cc(MicroOp::Fetch8(Rhs8Bit::W), CheckCondition::C, 3),
            MCycleOp::End(MicroOp::Idle), // Restoring address buffer to PC
            MCycleOp::Main(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)),
            MCycleOp::End(MicroOp::Idle), // Restoring address buffer to PC
        ],
    });
    opcodes[0xDC] = Some(&Instruction {
        opcode: 0xDC,
        name: "CALL C, imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Cc(MicroOp::Fetch8(Rhs8Bit::W), CheckCondition::C, 3),
            MCycleOp::End(MicroOp::Idle),
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write16msbDec(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::Main(MicroOp::Write16lsb(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::End(MicroOp::Ld16(Rhs16Bit::PC, Rhs16Bit::WZ)), // TODO Refactor: This action is in the previous M-Cycle in parallel with the PC lsb push
        ],
    });
    opcodes[0xDE] = Some(&Instruction {
        opcode: 0xDE,
        name: "SBC A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Alu(AluOp::Sbc(Rhs8Bit::A, Rhs8Bit::Z))),
        ],
    });
    opcodes[0xDF] = Some(&Instruction {
        opcode: 0xDF,
        name: "RST $18",
        cycles: 4,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write16msbDec(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::Main(MicroOp::Write16lsb(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::End(MicroOp::JumpVector(VectorAddress::V3)), // TODO Refactor: This action is in the previous M-Cycle in parallel with the PC lsb push
        ],
    });
    opcodes[0xE0] = Some(&Instruction {
        opcode: 0xE0,
        name: "LDH [imm8], A",
        cycles: 3,
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Main(MicroOp::Write8H(Rhs8Bit::Z, Rhs8Bit::A)),
            MCycleOp::End(MicroOp::Idle), // Restore Address Buffer with PC
        ],
    });
    opcodes[0xE1] = Some(&Instruction {
        opcode: 0xE1,
        name: "POP HL",
        cycles: 3,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8Inc(Rhs8Bit::Z, AddressRegister::SP)), // lsb
            MCycleOp::Main(MicroOp::Read8Inc(Rhs8Bit::W, AddressRegister::SP)), // MSB
            MCycleOp::End(MicroOp::Ld16(Rhs16Bit::HL, Rhs16Bit::WZ)),
        ],
    });
    opcodes[0xE2] = Some(&Instruction {
        opcode: 0xE2,
        name: "LDH [C], A",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Write8H(Rhs8Bit::C, Rhs8Bit::A)),
            MCycleOp::End(MicroOp::Idle), // Restore Address Buffer with PC
        ],
    });
    opcodes[0xE5] = Some(&Instruction {
        opcode: 0xE5,
        name: "PUSH HL",
        cycles: 4,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write8Inc(AddressRegister::SP, Rhs8Bit::H)), // MSB
            MCycleOp::Main(MicroOp::Write8(AddressRegister::SP, Rhs8Bit::L)), // lsb
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0xE6] = Some(&Instruction {
        opcode: 0xE6,
        name: "AND A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Alu(AluOp::And(Lhs8Bit::A, Lhs8Bit::Z))),
        ],
    });
    opcodes[0xE7] = Some(&Instruction {
        opcode: 0xE7,
        name: "RST $20",
        cycles: 4,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write16msbDec(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::Main(MicroOp::Write16lsb(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::End(MicroOp::JumpVector(VectorAddress::V4)), // TODO Refactor: This action is in the previous M-Cycle in parallel with the PC lsb push
        ],
    });
    opcodes[0xE8] = Some(&Instruction {
        opcode: 0xE8,
        name: "ADD SP, e8",
        cycles: 4,
        size: 2,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            // Todo Refactor: Well, this is not too accurate, but it should work by now
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Main(MicroOp::SumSignedByte16(Lhs16Bit::SP, Rhs16Bit::WZ, true)),
            MCycleOp::Main(MicroOp::Idle), // Yeah, I know this is not accurate
            MCycleOp::End(MicroOp::Idu(IduOp::None(Lhs16Bit::SP, Lhs16Bit::WZ))),
        ],
    });
    opcodes[0xE9] = Some(&Instruction {
        opcode: 0xE9,
        name: "JP HL",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::Idu(IduOp::None(Lhs16Bit::PC, Lhs16Bit::HL))),
        ],
    });
    opcodes[0xEA] = Some(&Instruction {
        opcode: 0xEA,
        name: "LD [imm16], A",
        cycles: 4,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::W)),
            MCycleOp::Main(MicroOp::Write8(Lhs16Bit::WZ, Rhs8Bit::A)),
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0xEE] = Some(&Instruction {
        opcode: 0xEE,
        name: "XOR A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Alu(AluOp::Xor(Lhs8Bit::A, Lhs8Bit::Z))),
        ],
    });
    opcodes[0xEF] = Some(&Instruction {
        opcode: 0xEF,
        name: "RST $28",
        cycles: 4,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write16msbDec(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::Main(MicroOp::Write16lsb(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::End(MicroOp::JumpVector(VectorAddress::V5)), // TODO Refactor: This action is in the previous M-Cycle in parallel with the PC lsb push
        ],
    });
    opcodes[0xF0] = Some(&Instruction {
        opcode: 0xF0,
        name: "LDH A, [imm8]",
        cycles: 3,
        size: 2,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Main(MicroOp::Read8H(Rhs8Bit::A, Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Idle), // Restore Address Buffer with PC
        ],
    });
    opcodes[0xF1] = Some(&Instruction {
        opcode: 0xF1,
        name: "POP AF",
        cycles: 3,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8Inc(Rhs8Bit::Z, AddressRegister::SP)), // lsb
            MCycleOp::Main(MicroOp::Read8Inc(Rhs8Bit::W, AddressRegister::SP)), // MSB
            MCycleOp::End(MicroOp::Ld16(Rhs16Bit::AF, Rhs16Bit::WZ)),
        ],
    });
    opcodes[0xF2] = Some(&Instruction {
        opcode: 0xF2,
        name: "LDH A, [C]",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Read8H(Rhs8Bit::A, Rhs8Bit::C)),
            MCycleOp::End(MicroOp::Idle), // Restore Address Buffer with PC
        ],
    });
    opcodes[0xF3] = Some(&Instruction {
        opcode: 0xF3,
        name: "DI",
        cycles: 1,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::End(MicroOp::ImeEnabled(false)),
        ],
    });
    opcodes[0xF5] = Some(&Instruction {
        opcode: 0xF5,
        name: "PUSH AF",
        cycles: 4,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write8Inc(AddressRegister::SP, Rhs8Bit::A)), // MSB
            MCycleOp::Main(MicroOp::Write8(AddressRegister::SP, Rhs8Bit::F)), // lsb
            MCycleOp::End(MicroOp::Idle),
        ],
    });
    opcodes[0xF6] = Some(&Instruction {
        opcode: 0xF6,
        name: "OR A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Alu(AluOp::Or(Rhs8Bit::A, Rhs8Bit::Z))),
        ],
    });
    opcodes[0xF7] = Some(&Instruction {
        opcode: 0xF7,
        name: "RST $30",
        cycles: 4,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write16msbDec(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::Main(MicroOp::Write16lsb(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::End(MicroOp::JumpVector(VectorAddress::V6)), // TODO Refactor: This action is in the previous M-Cycle in parallel with the PC lsb push
        ],
    });
    opcodes[0xF8] = Some(&Instruction {
        opcode: 0xF8,
        name: "LD HL, SP + e8",
        cycles: 3,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Main(MicroOp::SumSignedByte16(Lhs16Bit::SP, Rhs16Bit::WZ, true)),
            MCycleOp::End(MicroOp::Idu(IduOp::None(Lhs16Bit::HL, Lhs16Bit::WZ))),  // Todo Refactor: Well, this is not too accurate, but it should work by now
        ],
    });
    opcodes[0xF9] = Some(&Instruction {
        opcode: 0xF9,
        name: "LD SP, HL",
        cycles: 2,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Ld16(Rhs16Bit::SP, Rhs16Bit::HL)), // This latch address buffer with HL
            MCycleOp::End(MicroOp::Idle), // Restore Address Buffer with PC
        ],
    });
    opcodes[0xFA] = Some(&Instruction {
        opcode: 0xFA,
        name: "LD A, [imm16]",
        cycles: 4,
        size: 3,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::Main(MicroOp::Fetch8(Rhs8Bit::W)),
            MCycleOp::Main(MicroOp::Read8(Rhs8Bit::Z, Rhs16Bit::WZ)),
            MCycleOp::End(MicroOp::Ld8(Rhs8Bit::A, Rhs8Bit::Z)),
        ],
    });
    opcodes[0xFB] = Some(&Instruction {
        opcode: 0xFB,
        name: "EI",
        cycles: 1,
        size: 1,
        micro_ops: &[
            MCycleOp::End(MicroOp::ImeEnabled(true)),
        ],
        flags: &[],
    });
    opcodes[0xFE] = Some(&Instruction {
        opcode: 0xFE,
        name: "CP A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        micro_ops: &[
            MCycleOp::End(MicroOp::Fetch8(Rhs8Bit::Z)),
            MCycleOp::End(MicroOp::Alu(AluOp::Cp(Rhs8Bit::A, Rhs8Bit::Z))),
        ],
    });
    opcodes[0xFF] = Some(&Instruction {
        opcode: 0xFF,
        name: "RST $38",
        cycles: 4,
        size: 1,
        flags: &[],
        micro_ops: &[
            MCycleOp::Main(MicroOp::Dec16(Rhs16Bit::SP)),
            MCycleOp::Main(MicroOp::Write16msbDec(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::Main(MicroOp::Write16lsb(AddressRegister::SP, Rhs16Bit::PC)),
            MCycleOp::End(MicroOp::JumpVector(VectorAddress::V7)), // TODO Refactor: This action is in the previous M-Cycle in parallel with the PC lsb push
        ],
    });
    opcodes
}

const fn create_cb_opcodes() -> [Option<&'static Instruction>; 256] {
    macro_rules! rlc {
        ($opcode:expr, $name:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::End(MicroOp::Alu(AluOp::Rlc($rhs8bit))),
                ],
            })
        };
        ($opcode:expr, $name:expr, $address_register:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::Main(MicroOp::Read8($rhs8bit, $address_register)),
                    MCycleOp::Main(MicroOp::Alu(AluOp::Rlc($rhs8bit))),
                    MCycleOp::End(MicroOp::Write8($address_register, $rhs8bit)), // TODO refactor: not 100% M-Cycle accurate. This is done in the previous and this just reset PC as buffer address
                ],
            })
        };
    }

    macro_rules! rrc {
        ($opcode:expr, $name:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::End(MicroOp::Alu(AluOp::Rrc($rhs8bit))),
                ],
            })
        };
        ($opcode:expr, $name:expr, $address_register:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::Main(MicroOp::Read8($rhs8bit, $address_register)),
                    MCycleOp::Main(MicroOp::Alu(AluOp::Rlc($rhs8bit))),
                    MCycleOp::End(MicroOp::Write8($address_register, $rhs8bit)), // TODO refactor: not 100% M-Cycle accurate. This is done in the previous and this just reset PC as buffer address
                ],
            })
        };
    }

    macro_rules! rl {
        ($opcode:expr, $name:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::End(MicroOp::Alu(AluOp::Rl($rhs8bit))),
                ],
            })
        };
        ($opcode:expr, $name:expr, $address_register:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::Main(MicroOp::Read8($rhs8bit, $address_register)),
                    MCycleOp::Main(MicroOp::Alu(AluOp::Rl($rhs8bit))),
                    MCycleOp::End(MicroOp::Write8($address_register, $rhs8bit)), // TODO refactor: not 100% M-Cycle accurate. This is done in the previous and this just reset PC as buffer address
                ],
            })
        };
    }
    macro_rules! sla {
        ($opcode:expr, $name:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::End(MicroOp::Alu(AluOp::Sla($rhs8bit))),
                ],
            })
        };
        ($opcode:expr, $name:expr, $address_register:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::Main(MicroOp::Read8($rhs8bit, $address_register)),
                    MCycleOp::Main(MicroOp::Alu(AluOp::Sla($rhs8bit))),
                    MCycleOp::End(MicroOp::Write8($address_register, $rhs8bit)), // TODO refactor: not 100% M-Cycle accurate. This is done in the previous and this just reset PC as buffer address
                ],
            })
        };
    }

    macro_rules! rr {
        ($opcode:expr, $name:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::End(MicroOp::Alu(AluOp::Rr($rhs8bit))),
                ],
            })
        };
        ($opcode:expr, $name:expr, $address_register:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::Main(MicroOp::Read8($rhs8bit, $address_register)),
                    MCycleOp::Main(MicroOp::Alu(AluOp::Rr($rhs8bit))),
                    MCycleOp::End(MicroOp::Write8($address_register, $rhs8bit)), // TODO refactor: not 100% M-Cycle accurate. This is done in the previous and this just reset PC as buffer address
                ],
            })
        };
    }
    macro_rules! sra {
        ($opcode:expr, $name:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::End(MicroOp::Alu(AluOp::Sra($rhs8bit))),
                ],
            })
        };
        ($opcode:expr, $name:expr, $address_register:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::Main(MicroOp::Read8($rhs8bit, $address_register)),
                    MCycleOp::Main(MicroOp::Alu(AluOp::Sra($rhs8bit))),
                    MCycleOp::End(MicroOp::Write8($address_register, $rhs8bit)), // TODO refactor: not 100% M-Cycle accurate. This is done in the previous and this just reset PC as buffer address
                ],
            })
        };
    }

    macro_rules! srl {
        ($opcode:expr, $name:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::End(MicroOp::Alu(AluOp::Srl($rhs8bit))),
                ],
            })
        };
        ($opcode:expr, $name:expr, $address_register:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::Main(MicroOp::Read8($rhs8bit, $address_register)),
                    MCycleOp::Main(MicroOp::Alu(AluOp::Srl($rhs8bit))),
                    MCycleOp::End(MicroOp::Write8($address_register, $rhs8bit)), // TODO refactor: not 100% M-Cycle accurate. This is done in the previous and this just reset PC as buffer address
                ],
            })
        };
    }

    macro_rules! swap {
        ($opcode:expr, $name:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::End(MicroOp::Alu(AluOp::Swap($rhs8bit))),
                ],
            })
        };
        ($opcode:expr, $name:expr, $address_register:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                micro_ops: &[
                    MCycleOp::Main(MicroOp::Read8($rhs8bit, $address_register)),
                    MCycleOp::Main(MicroOp::Alu(AluOp::Swap($rhs8bit))),
                    MCycleOp::End(MicroOp::Write8($address_register, $rhs8bit)), // TODO refactor: not 100% M-Cycle accurate. This is done in the previous and this just reset PC as buffer address
                ],
            })
        };
    }

    macro_rules! bit {
        ($opcode:expr, $name:expr, $bit:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
                micro_ops: &[
                    MCycleOp::End(MicroOp::Alu(AluOp::Bit($bit, $rhs8bit))),
                ],
            })
        };
        ($opcode:expr, $name:expr, $bit:expr, $address_register:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
                micro_ops: &[
                    MCycleOp::Main(MicroOp::Read8($rhs8bit, $address_register)),
                    MCycleOp::Main(MicroOp::Alu(AluOp::Bit($bit, $rhs8bit))),
                    MCycleOp::End(MicroOp::Write8($address_register, $rhs8bit)), // TODO refactor: not 100% M-Cycle accurate. This is done in the previous and this just reset PC as buffer address
                ],
            })
        };
    }

    macro_rules! res {
        ($opcode:expr, $name:expr, $bit:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[],
                micro_ops: &[
                    MCycleOp::End(MicroOp::Alu(AluOp::Res($bit, $rhs8bit))),
                ],
            })
        };
        ($opcode:expr, $name:expr, $bit:expr, $address_register:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[],
                micro_ops: &[
                    MCycleOp::Main(MicroOp::Read8($rhs8bit, $address_register)),
                    MCycleOp::Main(MicroOp::Alu(AluOp::Res($bit, $rhs8bit))),
                    MCycleOp::End(MicroOp::Write8($address_register, $rhs8bit)), // TODO refactor: not 100% M-Cycle accurate. This is done in the previous and this just reset PC as buffer address
                ],
            })
        };
    }

    macro_rules! set {
        ($opcode:expr, $name:expr, $bit:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[],
                micro_ops: &[
                    MCycleOp::End(MicroOp::Alu(AluOp::Set($bit, $rhs8bit))),
                ],
            })
        };
        ($opcode:expr, $name:expr, $bit:expr, $address_register:expr, $rhs8bit:expr) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[],
                micro_ops: &[
                    MCycleOp::Main(MicroOp::Read8($rhs8bit, $address_register)),
                    MCycleOp::Main(MicroOp::Alu(AluOp::Set($bit, $rhs8bit))),
                    MCycleOp::End(MicroOp::Write8($address_register, $rhs8bit)), // TODO refactor: not 100% M-Cycle accurate. This is done in the previous and this just reset PC as buffer address
                ],
            })
        };
    }

    let mut opcodes = [None; 256];
    opcodes[0x00] = rlc!(0x00, "RLC B", Rhs8Bit::B);
    opcodes[0x01] = rlc!(0x01, "RLC C", Rhs8Bit::C);
    opcodes[0x02] = rlc!(0x02, "RLC D", Rhs8Bit::D);
    opcodes[0x03] = rlc!(0x03, "RLC E", Rhs8Bit::E);
    opcodes[0x04] = rlc!(0x04, "RLC H", Rhs8Bit::H);
    opcodes[0x05] = rlc!(0x05, "RLC L", Rhs8Bit::L);
    opcodes[0x06] = rlc!(0x06, "RLC [HL]", AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x07] = rlc!(0x07, "RLC A", Rhs8Bit::A);

    opcodes[0x08] = rrc!(0x08, "RRC B", Rhs8Bit::B);
    opcodes[0x09] = rrc!(0x09, "RRC C", Rhs8Bit::C);
    opcodes[0x0A] = rrc!(0x0a, "RRC D", Rhs8Bit::D);
    opcodes[0x0B] = rrc!(0x0b, "RRC E", Rhs8Bit::E);
    opcodes[0x0C] = rrc!(0x0c, "RRC H", Rhs8Bit::H);
    opcodes[0x0D] = rrc!(0x0d, "RRC L", Rhs8Bit::L);
    opcodes[0x0E] = rrc!(0x0e, "RRC [HL]", AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x0F] = rrc!(0x0f, "RRC A", Rhs8Bit::A);

    opcodes[0x10] = rl!(0x10, "RL B", Rhs8Bit::B);
    opcodes[0x11] = rl!(0x11, "RL C", Rhs8Bit::C);
    opcodes[0x12] = rl!(0x12, "RL D", Rhs8Bit::D);
    opcodes[0x13] = rl!(0x13, "RL E", Rhs8Bit::E);
    opcodes[0x14] = rl!(0x14, "RL H", Rhs8Bit::H);
    opcodes[0x15] = rl!(0x15, "RL L", Rhs8Bit::L);
    opcodes[0x16] = rl!(0x16, "RL [HL]", AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x17] = rl!(0x17, "RL A", Rhs8Bit::A);

    opcodes[0x18] = rr!(0x18, "RR B",Rhs8Bit::B);
    opcodes[0x19] = rr!(0x19, "RR C",Rhs8Bit::C);
    opcodes[0x1A] = rr!(0x1a, "RR D",Rhs8Bit::D);
    opcodes[0x1B] = rr!(0x1b, "RR E",Rhs8Bit::E);
    opcodes[0x1C] = rr!(0x1c, "RR H",Rhs8Bit::H);
    opcodes[0x1D] = rr!(0x1d, "RR L",Rhs8Bit::L);
    opcodes[0x1E] = rr!(0x1e, "RR [HL]", AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x1F] = rr!(0x1f, "RR A", Rhs8Bit::A);

    opcodes[0x20] = sla!(0x20, "SLA B", Rhs8Bit::B);
    opcodes[0x21] = sla!(0x21, "SLA C", Rhs8Bit::C);
    opcodes[0x22] = sla!(0x22, "SLA D", Rhs8Bit::D);
    opcodes[0x23] = sla!(0x23, "SLA E", Rhs8Bit::E);
    opcodes[0x24] = sla!(0x24, "SLA H", Rhs8Bit::H);
    opcodes[0x25] = sla!(0x25, "SLA L", Rhs8Bit::L);
    opcodes[0x26] = sla!(0x26, "SLA [HL]", AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x27] = sla!(0x27, "SLA A", Rhs8Bit::A);

    opcodes[0x28] = sra!(0x28, "SRA B", Rhs8Bit::B);
    opcodes[0x29] = sra!(0x29, "SRA C", Rhs8Bit::C);
    opcodes[0x2A] = sra!(0x2a, "SRA D", Rhs8Bit::D);
    opcodes[0x2B] = sra!(0x2b, "SRA E", Rhs8Bit::E);
    opcodes[0x2C] = sra!(0x2c, "SRA H", Rhs8Bit::H);
    opcodes[0x2D] = sra!(0x2d, "SRA L", Rhs8Bit::L);
    opcodes[0x2E] = sra!(0x2e, "SRA [HL]", AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x2F] = sra!(0x2f, "SRA A", Rhs8Bit::A);

    opcodes[0x30] = swap!(0x30, "SWAP B", Rhs8Bit::B);
    opcodes[0x31] = swap!(0x31, "SWAP C", Rhs8Bit::C);
    opcodes[0x32] = swap!(0x32, "SWAP D", Rhs8Bit::D);
    opcodes[0x33] = swap!(0x33, "SWAP E", Rhs8Bit::E);
    opcodes[0x34] = swap!(0x34, "SWAP H", Rhs8Bit::H);
    opcodes[0x35] = swap!(0x35, "SWAP L", Rhs8Bit::L);
    opcodes[0x36] = swap!(0x36, "SWAP [HL]", AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x37] = swap!(0x37, "SWAP A", Rhs8Bit::A);

    opcodes[0x38] = srl!(0x38, "SRL B", Rhs8Bit::B);
    opcodes[0x39] = srl!(0x39, "SRL C", Rhs8Bit::C);
    opcodes[0x3A] = srl!(0x3a, "SRL D", Rhs8Bit::D);
    opcodes[0x3B] = srl!(0x3b, "SRL E", Rhs8Bit::E);
    opcodes[0x3C] = srl!(0x3c, "SRL H", Rhs8Bit::H);
    opcodes[0x3D] = srl!(0x3d, "SRL L", Rhs8Bit::L);
    opcodes[0x3E] = srl!(0x3e, "SRL [HL]", AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x3F] = srl!(0x3f, "SRL A", Rhs8Bit::A);

    opcodes[0x40] = bit!(0x40, "BIT 0, B", ByteBit::Zero, Rhs8Bit::B);
    opcodes[0x41] = bit!(0x41, "BIT 0, C", ByteBit::Zero, Rhs8Bit::C);
    opcodes[0x42] = bit!(0x42, "BIT 0, D", ByteBit::Zero, Rhs8Bit::D);
    opcodes[0x43] = bit!(0x43, "BIT 0, E", ByteBit::Zero, Rhs8Bit::E);
    opcodes[0x44] = bit!(0x44, "BIT 0, H", ByteBit::Zero, Rhs8Bit::H);
    opcodes[0x45] = bit!(0x45, "BIT 0, L", ByteBit::Zero, Rhs8Bit::L);
    opcodes[0x46] = bit!(0x46, "BIT 0, [HL]", ByteBit::Zero, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x47] = bit!(0x47, "BIT 0, A", ByteBit::Zero, Rhs8Bit::C);

    opcodes[0x48] = bit!(0x48, "BIT 1, B", ByteBit::One, Rhs8Bit::B);
    opcodes[0x49] = bit!(0x49, "BIT 1, C", ByteBit::One, Rhs8Bit::C);
    opcodes[0x4A] = bit!(0x4a, "BIT 1, D", ByteBit::One, Rhs8Bit::D);
    opcodes[0x4B] = bit!(0x4b, "BIT 1, E", ByteBit::One, Rhs8Bit::E);
    opcodes[0x4C] = bit!(0x4c, "BIT 1, H", ByteBit::One, Rhs8Bit::H);
    opcodes[0x4D] = bit!(0x4d, "BIT 1, L", ByteBit::One, Rhs8Bit::L);
    opcodes[0x4E] = bit!(0x4e, "BIT 1, [HL]", ByteBit::One, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x4F] = bit!(0x4f, "BIT 1, A", ByteBit::One, Rhs8Bit::C);

    opcodes[0x50] = bit!(0x50, "BIT 2, B", ByteBit::Two, Rhs8Bit::B);
    opcodes[0x51] = bit!(0x51, "BIT 2, C", ByteBit::Two, Rhs8Bit::C);
    opcodes[0x52] = bit!(0x52, "BIT 2, D", ByteBit::Two, Rhs8Bit::D);
    opcodes[0x53] = bit!(0x53, "BIT 2, E", ByteBit::Two, Rhs8Bit::E);
    opcodes[0x54] = bit!(0x54, "BIT 2, H", ByteBit::Two, Rhs8Bit::H);
    opcodes[0x55] = bit!(0x55, "BIT 2, L", ByteBit::Two, Rhs8Bit::L);
    opcodes[0x56] = bit!(0x56, "BIT 2, [HL]", ByteBit::Two, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x57] = bit!(0x57, "BIT 2, A", ByteBit::Two, Rhs8Bit::C);

    opcodes[0x58] = bit!(0x58, "BIT 3, B", ByteBit::Three, Rhs8Bit::B);
    opcodes[0x59] = bit!(0x59, "BIT 3, C", ByteBit::Three, Rhs8Bit::C);
    opcodes[0x5A] = bit!(0x5a, "BIT 3, D", ByteBit::Three, Rhs8Bit::D);
    opcodes[0x5B] = bit!(0x5b, "BIT 3, E", ByteBit::Three, Rhs8Bit::E);
    opcodes[0x5C] = bit!(0x5c, "BIT 3, H", ByteBit::Three, Rhs8Bit::H);
    opcodes[0x5D] = bit!(0x5d, "BIT 3, L", ByteBit::Three, Rhs8Bit::L);
    opcodes[0x5E] = bit!(0x5e, "BIT 3, [HL]", ByteBit::Three, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x5F] = bit!(0x5f, "BIT 3, A", ByteBit::Three, Rhs8Bit::C);

    opcodes[0x60] = bit!(0x60, "BIT 4, B", ByteBit::Four, Rhs8Bit::B);
    opcodes[0x61] = bit!(0x61, "BIT 4, C", ByteBit::Four, Rhs8Bit::C);
    opcodes[0x62] = bit!(0x62, "BIT 4, D", ByteBit::Four, Rhs8Bit::D);
    opcodes[0x63] = bit!(0x63, "BIT 4, E", ByteBit::Four, Rhs8Bit::E);
    opcodes[0x64] = bit!(0x64, "BIT 4, H", ByteBit::Four, Rhs8Bit::H);
    opcodes[0x65] = bit!(0x65, "BIT 4, L", ByteBit::Four, Rhs8Bit::L);
    opcodes[0x66] = bit!(0x66, "BIT 4, [HL]", ByteBit::Four, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x67] = bit!(0x67, "BIT 4, A", ByteBit::Four, Rhs8Bit::C);

    opcodes[0x68] = bit!(0x68, "BIT 5, B", ByteBit::Five, Rhs8Bit::B);
    opcodes[0x69] = bit!(0x69, "BIT 5, C", ByteBit::Five, Rhs8Bit::C);
    opcodes[0x6A] = bit!(0x6a, "BIT 5, D", ByteBit::Five, Rhs8Bit::D);
    opcodes[0x6B] = bit!(0x6b, "BIT 5, E", ByteBit::Five, Rhs8Bit::E);
    opcodes[0x6C] = bit!(0x6c, "BIT 5, H", ByteBit::Five, Rhs8Bit::H);
    opcodes[0x6D] = bit!(0x6d, "BIT 5, L", ByteBit::Five, Rhs8Bit::L);
    opcodes[0x6E] = bit!(0x6e, "BIT 5, [HL]", ByteBit::Five, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x6F] = bit!(0x6f, "BIT 5, A", ByteBit::Five, Rhs8Bit::C);

    opcodes[0x70] = bit!(0x70, "BIT 6, B", ByteBit::Six, Rhs8Bit::B);
    opcodes[0x71] = bit!(0x71, "BIT 6, C", ByteBit::Six, Rhs8Bit::C);
    opcodes[0x72] = bit!(0x72, "BIT 6, D", ByteBit::Six, Rhs8Bit::D);
    opcodes[0x73] = bit!(0x73, "BIT 6, E", ByteBit::Six, Rhs8Bit::E);
    opcodes[0x74] = bit!(0x74, "BIT 6, H", ByteBit::Six, Rhs8Bit::H);
    opcodes[0x75] = bit!(0x75, "BIT 6, L", ByteBit::Six, Rhs8Bit::L);
    opcodes[0x76] = bit!(0x76, "BIT 6, [HL]", ByteBit::Six, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x77] = bit!(0x77, "BIT 6, A", ByteBit::Six, Rhs8Bit::C);

    opcodes[0x78] = bit!(0x78, "BIT 7, B", ByteBit::Seven, Rhs8Bit::B);
    opcodes[0x79] = bit!(0x79, "BIT 7, C", ByteBit::Seven, Rhs8Bit::C);
    opcodes[0x7A] = bit!(0x7a, "BIT 7, D", ByteBit::Seven, Rhs8Bit::D);
    opcodes[0x7B] = bit!(0x7b, "BIT 7, E", ByteBit::Seven, Rhs8Bit::E);
    opcodes[0x7C] = bit!(0x7c, "BIT 7, H", ByteBit::Seven, Rhs8Bit::H);
    opcodes[0x7D] = bit!(0x7d, "BIT 7, L", ByteBit::Seven, Rhs8Bit::L);
    opcodes[0x7E] = bit!(0x7e, "BIT 7, [HL]", ByteBit::Seven, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x7F] = bit!(0x7f, "BIT 7, A", ByteBit::Seven, Rhs8Bit::C);

    opcodes[0x80] = res!(0x80, "RES 0, B", ByteBit::Zero, Rhs8Bit::B);
    opcodes[0x81] = res!(0x81, "RES 0, C", ByteBit::Zero, Rhs8Bit::C);
    opcodes[0x82] = res!(0x82, "RES 0, D", ByteBit::Zero, Rhs8Bit::D);
    opcodes[0x83] = res!(0x83, "RES 0, E", ByteBit::Zero, Rhs8Bit::E);
    opcodes[0x84] = res!(0x84, "RES 0, H", ByteBit::Zero, Rhs8Bit::H);
    opcodes[0x85] = res!(0x85, "RES 0, L", ByteBit::Zero, Rhs8Bit::L);
    opcodes[0x86] = res!(0x86, "RES 0, [HL]", ByteBit::Zero, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x87] = res!(0x87, "RES 0, A", ByteBit::Zero, Rhs8Bit::C);

    opcodes[0x88] = res!(0x88, "RES 1, B", ByteBit::One, Rhs8Bit::B);
    opcodes[0x89] = res!(0x89, "RES 1, C", ByteBit::One, Rhs8Bit::C);
    opcodes[0x8A] = res!(0x8a, "RES 1, D", ByteBit::One, Rhs8Bit::D);
    opcodes[0x8B] = res!(0x8b, "RES 1, E", ByteBit::One, Rhs8Bit::E);
    opcodes[0x8C] = res!(0x8c, "RES 1, H", ByteBit::One, Rhs8Bit::H);
    opcodes[0x8D] = res!(0x8d, "RES 1, L", ByteBit::One, Rhs8Bit::L);
    opcodes[0x8E] = res!(0x8e, "RES 1, [HL]", ByteBit::One, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x8F] = res!(0x8f, "RES 1, A", ByteBit::One, Rhs8Bit::C);

    opcodes[0x90] = res!(0x90, "RES 2, B", ByteBit::Two, Rhs8Bit::B);
    opcodes[0x91] = res!(0x91, "RES 2, C", ByteBit::Two, Rhs8Bit::C);
    opcodes[0x92] = res!(0x92, "RES 2, D", ByteBit::Two, Rhs8Bit::D);
    opcodes[0x93] = res!(0x93, "RES 2, E", ByteBit::Two, Rhs8Bit::E);
    opcodes[0x94] = res!(0x94, "RES 2, H", ByteBit::Two, Rhs8Bit::H);
    opcodes[0x95] = res!(0x95, "RES 2, L", ByteBit::Two, Rhs8Bit::L);
    opcodes[0x96] = res!(0x96, "RES 2, [HL]", ByteBit::Two, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x97] = res!(0x97, "RES 2, A", ByteBit::Two, Rhs8Bit::C);

    opcodes[0x98] = res!(0x98, "RES 3, B", ByteBit::Three, Rhs8Bit::B);
    opcodes[0x99] = res!(0x99, "RES 3, C", ByteBit::Three, Rhs8Bit::C);
    opcodes[0x9A] = res!(0x9a, "RES 3, D", ByteBit::Three, Rhs8Bit::D);
    opcodes[0x9B] = res!(0x9b, "RES 3, E", ByteBit::Three, Rhs8Bit::E);
    opcodes[0x9C] = res!(0x9c, "RES 3, H", ByteBit::Three, Rhs8Bit::H);
    opcodes[0x9D] = res!(0x9d, "RES 3, L", ByteBit::Three, Rhs8Bit::L);
    opcodes[0x9E] = res!(0x9e, "RES 3, [HL]", ByteBit::Three, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0x9F] = res!(0x9f, "RES 3, A", ByteBit::Three, Rhs8Bit::C);

    opcodes[0xA0] = res!(0xA0, "RES 4, B", ByteBit::Four, Rhs8Bit::B);
    opcodes[0xA1] = res!(0xA1, "RES 4, C", ByteBit::Four, Rhs8Bit::C);
    opcodes[0xA2] = res!(0xA2, "RES 4, D", ByteBit::Four, Rhs8Bit::D);
    opcodes[0xA3] = res!(0xA3, "RES 4, E", ByteBit::Four, Rhs8Bit::E);
    opcodes[0xA4] = res!(0xA4, "RES 4, H", ByteBit::Four, Rhs8Bit::H);
    opcodes[0xA5] = res!(0xA5, "RES 4, L", ByteBit::Four, Rhs8Bit::L);
    opcodes[0xA6] = res!(0xA6, "RES 4, [HL]", ByteBit::Four, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0xA7] = res!(0xA7, "RES 4, A", ByteBit::Four, Rhs8Bit::C);

    opcodes[0xA8] = res!(0xA8, "RES 5, B", ByteBit::Five, Rhs8Bit::B);
    opcodes[0xA9] = res!(0xA9, "RES 5, C", ByteBit::Five, Rhs8Bit::C);
    opcodes[0xAA] = res!(0xAa, "RES 5, D", ByteBit::Five, Rhs8Bit::D);
    opcodes[0xAB] = res!(0xAb, "RES 5, E", ByteBit::Five, Rhs8Bit::E);
    opcodes[0xAC] = res!(0xAc, "RES 5, H", ByteBit::Five, Rhs8Bit::H);
    opcodes[0xAD] = res!(0xAd, "RES 5, L", ByteBit::Five, Rhs8Bit::L);
    opcodes[0xAE] = res!(0xAe, "RES 5, [HL]", ByteBit::Five, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0xAF] = res!(0xAf, "RES 5, A", ByteBit::Five, Rhs8Bit::C);

    opcodes[0xB0] = res!(0xB0, "RES 6, B", ByteBit::Six, Rhs8Bit::B);
    opcodes[0xB1] = res!(0xB1, "RES 6, C", ByteBit::Six, Rhs8Bit::C);
    opcodes[0xB2] = res!(0xB2, "RES 6, D", ByteBit::Six, Rhs8Bit::D);
    opcodes[0xB3] = res!(0xB3, "RES 6, E", ByteBit::Six, Rhs8Bit::E);
    opcodes[0xB4] = res!(0xB4, "RES 6, H", ByteBit::Six, Rhs8Bit::H);
    opcodes[0xB5] = res!(0xB5, "RES 6, L", ByteBit::Six, Rhs8Bit::L);
    opcodes[0xB6] = res!(0xB6, "RES 6, [HL]", ByteBit::Six, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0xB7] = res!(0xB7, "RES 6, A", ByteBit::Six, Rhs8Bit::C);

    opcodes[0xB8] = res!(0xB8, "RES 7, B", ByteBit::Seven, Rhs8Bit::B);
    opcodes[0xB9] = res!(0xB9, "RES 7, C", ByteBit::Seven, Rhs8Bit::C);
    opcodes[0xBA] = res!(0xBa, "RES 7, D", ByteBit::Seven, Rhs8Bit::D);
    opcodes[0xBB] = res!(0xBb, "RES 7, E", ByteBit::Seven, Rhs8Bit::E);
    opcodes[0xBC] = res!(0xBc, "RES 7, H", ByteBit::Seven, Rhs8Bit::H);
    opcodes[0xBD] = res!(0xBd, "RES 7, L", ByteBit::Seven, Rhs8Bit::L);
    opcodes[0xBE] = res!(0xBe, "RES 7, [HL]", ByteBit::Seven, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0xBF] = res!(0xBf, "RES 7, A", ByteBit::Seven, Rhs8Bit::C);

    opcodes[0xC0] = set!(0xC0, "SET 0, B", ByteBit::Zero, Rhs8Bit::B);
    opcodes[0xC1] = set!(0xC1, "SET 0, C", ByteBit::Zero, Rhs8Bit::C);
    opcodes[0xC2] = set!(0xC2, "SET 0, D", ByteBit::Zero, Rhs8Bit::D);
    opcodes[0xC3] = set!(0xC3, "SET 0, E", ByteBit::Zero, Rhs8Bit::E);
    opcodes[0xC4] = set!(0xC4, "SET 0, H", ByteBit::Zero, Rhs8Bit::H);
    opcodes[0xC5] = set!(0xC5, "SET 0, L", ByteBit::Zero, Rhs8Bit::L);
    opcodes[0xC6] = set!(0xC6, "SET 0, [HL]", ByteBit::Zero, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0xC7] = set!(0xC7, "SET 0, A", ByteBit::Zero, Rhs8Bit::C);

    opcodes[0xC8] = set!(0xc8, "SET 1, B", ByteBit::One, Rhs8Bit::B);
    opcodes[0xC9] = set!(0xc9, "SET 1, C", ByteBit::One, Rhs8Bit::C);
    opcodes[0xCA] = set!(0xca, "SET 1, D", ByteBit::One, Rhs8Bit::D);
    opcodes[0xCB] = set!(0xcb, "SET 1, E", ByteBit::One, Rhs8Bit::E);
    opcodes[0xCC] = set!(0xcc, "SET 1, H", ByteBit::One, Rhs8Bit::H);
    opcodes[0xCD] = set!(0xcd, "SET 1, L", ByteBit::One, Rhs8Bit::L);
    opcodes[0xCE] = set!(0xce, "SET 1, [HL ", ByteBit::One, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0xCF] = set!(0xcf, "SET 1, A", ByteBit::One, Rhs8Bit::C);

    opcodes[0xD0] = set!(0xd0, "SET 2, B", ByteBit::Two, Rhs8Bit::B);
    opcodes[0xD1] = set!(0xd1, "SET 2, C", ByteBit::Two, Rhs8Bit::C);
    opcodes[0xD2] = set!(0xd2, "SET 2, D", ByteBit::Two, Rhs8Bit::D);
    opcodes[0xD3] = set!(0xd3, "SET 2, E", ByteBit::Two, Rhs8Bit::E);
    opcodes[0xD4] = set!(0xd4, "SET 2, H", ByteBit::Two, Rhs8Bit::H);
    opcodes[0xD5] = set!(0xd5, "SET 2, L", ByteBit::Two, Rhs8Bit::L);
    opcodes[0xD6] = set!(0xd6, "SET 2, [HL]", ByteBit::Two, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0xD7] = set!(0xd7, "SET 2, A", ByteBit::Two, Rhs8Bit::C);

    opcodes[0xD8] = set!(0xd8, "SET 3, B", ByteBit::Three, Rhs8Bit::B);
    opcodes[0xD9] = set!(0xd9, "SET 3, C", ByteBit::Three, Rhs8Bit::C);
    opcodes[0xDA] = set!(0xda, "SET 3, D", ByteBit::Three, Rhs8Bit::D);
    opcodes[0xDB] = set!(0xdb, "SET 3, E", ByteBit::Three, Rhs8Bit::E);
    opcodes[0xDC] = set!(0xdc, "SET 3, H", ByteBit::Three, Rhs8Bit::H);
    opcodes[0xDD] = set!(0xdd, "SET 3, L", ByteBit::Three, Rhs8Bit::L);
    opcodes[0xDE] = set!(0xde, "SET 3, [HL]", ByteBit::Three, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0xDF] = set!(0xdf, "SET 3, A", ByteBit::Three, Rhs8Bit::C);

    opcodes[0xE0] = set!(0xe0, "SET 4, B", ByteBit::Four, Rhs8Bit::B);
    opcodes[0xE1] = set!(0xe1, "SET 4, C", ByteBit::Four, Rhs8Bit::C);
    opcodes[0xE2] = set!(0xe2, "SET 4, D", ByteBit::Four, Rhs8Bit::D);
    opcodes[0xE3] = set!(0xe3, "SET 4, E", ByteBit::Four, Rhs8Bit::E);
    opcodes[0xE4] = set!(0xe4, "SET 4, H", ByteBit::Four, Rhs8Bit::H);
    opcodes[0xE5] = set!(0xe5, "SET 4, L", ByteBit::Four, Rhs8Bit::L);
    opcodes[0xE6] = set!(0xe6, "SET 4, [HL]", ByteBit::Four, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0xE7] = set!(0xe7, "SET 4, A", ByteBit::Four, Rhs8Bit::C);

    opcodes[0xE8] = set!(0xe8, "SET 5, B", ByteBit::Five, Rhs8Bit::B);
    opcodes[0xE9] = set!(0xe9, "SET 5, C", ByteBit::Five, Rhs8Bit::C);
    opcodes[0xEA] = set!(0xea, "SET 5, D", ByteBit::Five, Rhs8Bit::D);
    opcodes[0xEB] = set!(0xeb, "SET 5, E", ByteBit::Five, Rhs8Bit::E);
    opcodes[0xEC] = set!(0xec, "SET 5, H", ByteBit::Five, Rhs8Bit::H);
    opcodes[0xED] = set!(0xed, "SET 5, L", ByteBit::Five, Rhs8Bit::L);
    opcodes[0xEE] = set!(0xee, "SET 5, [HL]", ByteBit::Five, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0xEF] = set!(0xef, "SET 5, A", ByteBit::Five, Rhs8Bit::C);

    opcodes[0xF0] = set!(0xF0, "SET 6, B", ByteBit::Six, Rhs8Bit::B);
    opcodes[0xF1] = set!(0xF1, "SET 6, C", ByteBit::Six, Rhs8Bit::C);
    opcodes[0xF2] = set!(0xF2, "SET 6, D", ByteBit::Six, Rhs8Bit::D);
    opcodes[0xF3] = set!(0xF3, "SET 6, E", ByteBit::Six, Rhs8Bit::E);
    opcodes[0xF4] = set!(0xF4, "SET 6, H", ByteBit::Six, Rhs8Bit::H);
    opcodes[0xF5] = set!(0xF5, "SET 6, L", ByteBit::Six, Rhs8Bit::L);
    opcodes[0xF6] = set!(0xF6, "SET 6, [HL]", ByteBit::Six, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0xF7] = set!(0xF7, "SET 6, A", ByteBit::Six, Rhs8Bit::C);

    opcodes[0xF8] = set!(0xF8, "SET 7, B", ByteBit::Seven, Rhs8Bit::B);
    opcodes[0xF9] = set!(0xF9, "SET 7, C", ByteBit::Seven, Rhs8Bit::C);
    opcodes[0xFA] = set!(0xFa, "SET 7, D", ByteBit::Seven, Rhs8Bit::D);
    opcodes[0xFB] = set!(0xFb, "SET 7, E", ByteBit::Seven, Rhs8Bit::E);
    opcodes[0xFC] = set!(0xFc, "SET 7, H", ByteBit::Seven, Rhs8Bit::H);
    opcodes[0xFD] = set!(0xFd, "SET 7, L", ByteBit::Seven, Rhs8Bit::L);
    opcodes[0xFE] = set!(0xFe, "SET 7, [HL]", ByteBit::Seven, AddressRegister::HL, Rhs8Bit::Z);
    opcodes[0xFF] = set!(0xFf, "SET 7, A", ByteBit::Seven, Rhs8Bit::C);
    opcodes
}

pub const OPCODES: [Option<&'static Instruction>; 256] = create_opcodes();

pub const OPCODES_CB: [Option<&'static Instruction>; 256] = create_cb_opcodes();
