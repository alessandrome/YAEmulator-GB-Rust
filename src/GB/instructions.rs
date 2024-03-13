

#[derive(Debug, Clone, Copy)]
pub enum Flags {
    None,
    Zero,
    Negative,
    HalfCarry,
    Carry,
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub opcode: u8,
    pub name: &'static str,
    pub cycles: u8,
    pub size: u8,
    pub flags: Flags,
}

impl Instruction {
    pub fn new(opcode: u8) -> Self {
        INSTRUCTIONS[opcode]
    }
}

const INSTRUCTIONS: [Instruction; 256] = [
    for i in 0..256 {
        INSTRUCTIONS[i] = match i {
            0x00 => Instruction {
                opcode: 0x00,
                name: "NOP",
                cycles: 1,
                size: 1,
                flags: Flags::None,
            },
            0x01 => Instruction {
                opcode: 0x01,
                name: "LD BC, d16",
                cycles: 3,
                size: 3,
                flags: Flags::None,
            },
            0xCB => Instruction {
                opcode: 0xCB,
                name: "0xCB SUB-SET",
                cycles: 1,
                size: 1,
                flags: Flags::None,
            },
            // ... Define instructions for other opcodes based on information
            _ => Instruction {
                opcode: i as u8,
                name: "UNKNOWN",
                cycles: 0,
                size: 0,
                flags: Flags::None,
            },
        }
    }
];

const INSTRUCTIONS_CB: [Instruction; 256] = [
    for i in 0..256 {
        INSTRUCTIONS[i] = match i {
            0x00 => Instruction {
                opcode: 0x00,
                name: "NOP",
                cycles: 1,
                size: 1,
                flags: Flags::None,
            },
            0x01 => Instruction {
                opcode: 0x01,
                name: "LD BC, d16",
                cycles: 3,
                size: 3,
                flags: Flags::None,
            },
            // ... Define instructions for other opcodes based on information
            _ => Instruction {
                opcode: i as u8,
                name: "UNKNOWN",
                cycles: 0,
                size: 0,
                flags: Flags::None,
            },
        };
    }
];

