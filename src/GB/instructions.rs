use crate::GB::registers::{FlagBits, Flags};

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: u8,
    pub name: &'static str,
    pub cycles: u8,
    pub size: u8,
    pub flags: &'static [FlagBits],
}

impl Instruction {
    pub fn new(opcode: u8) -> Option<&'static Self> {
        OPCODES[opcode as usize]
    }
}

const fn create_opcodes() -> [Option<&'static Instruction>; 256] {
    let mut opcodes = [None; 256];
    opcodes[0x00] = Some(&Instruction {
        opcode: 0x00,
        name: "NOP",
        cycles: 1,
        size: 1,
        flags: &[],
    });
    opcodes[0x01] = Some(&Instruction {
        opcode: 0x01,
        name: "LD BC, d16",
        cycles: 3,
        size: 3,
        flags: &[],
    });
    opcodes[0xCB] = Some(&Instruction {
        opcode: 0xCB,
        name: "LD BC, d16",
        cycles: 3,
        size: 3,
        flags: &[],
    });
    opcodes
}

const fn create_cb_opcodes() -> [Option<&'static Instruction>; 256] {
    let mut opcodes = [None; 256];
    opcodes[0x00] = Some(&Instruction {
        opcode: 0x00,
        name: "NOP",
        cycles: 1,
        size: 1,
        flags: &[],
    });
    opcodes[0x01] = Some(&Instruction {
        opcode: 0x01,
        name: "LD BC, d16",
        cycles: 3,
        size: 3,
        flags: &[],
    });
    opcodes[0xCB] = Some(&Instruction {
        opcode: 0xCB,
        name: "LD BC, d16",
        cycles: 3,
        size: 3,
        flags: &[],
    });
    opcodes
}

const OPCODES: [Option<&'static Instruction>; 256] = create_opcodes();

const OPCODES_CB: [Option<&'static Instruction>; 256] = create_cb_opcodes();
