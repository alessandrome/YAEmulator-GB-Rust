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
        OPCODES[opcode]
    }
}

const fn create_opcodes() -> [Option<&'static Instruction>; 256] {
    let mut opcodes = [None; 256];
    for i in 0..256 {
        opcodes[i] = match i {
            0x00 => Some(&Instruction {
                opcode: 0x00,
                name: "NOP",
                cycles: 1,
                size: 1,
                flags: Flags::None,
            }),
            0x01 => Some(&Instruction {
                opcode: 0x01,
                name: "LD BC, d16",
                cycles: 3,
                size: 3,
                flags: Flags::None,
            }),
            0xCB => Some(&Instruction {
                opcode: 0xCB,
                name: "0xCB SUB-SET",
                cycles: 1,
                size: 1,
                flags: Flags::None,
            }),
            // ... Define instructions for other opcodes based on information
            _ => None,
        }
    }
    opcodes
}

const fn create_cb_opcodes() -> [Option<&'static Instruction>; 256] {
    let mut opcodes = [None; 256];
    for i in 0..256 {
        opcodes[i] = match i {
            0x00 => Some(&Instruction {
                opcode: 0x00,
                name: "NOP",
                cycles: 1,
                size: 1,
                flags: Flags::None,
            }),
            0x01 => Some(&Instruction {
                opcode: 0x01,
                name: "LD BC, d16",
                cycles: 3,
                size: 3,
                flags: Flags::None,
            }),
            0xCB => Some(&Instruction {
                opcode: 0xCB,
                name: "0xCB SUB-SET",
                cycles: 1,
                size: 1,
                flags: Flags::None,
            }),
            // ... Define instructions for other opcodes based on information
            _ => None,
        }
    }
    opcodes
}

const OPCODES: [Option<&'static Instruction>; 256] = create_opcodes();

const OPCODES_CB: [Option<&'static Instruction>; 256] = create_cb_opcodes();
