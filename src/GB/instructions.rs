use crate::GB::CPU::CPU;
use crate::GB::registers::{FlagBits, Flags};

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: u8,
    pub name: &'static str,
    pub cycles: u8,
    pub size: u8,
    pub flags: &'static [FlagBits],
    pub execute: fn(&Instruction, &mut CPU) -> u8,
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
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u8 {
            // NOP Do nothing
            opcode.cycles
        },
    });
    opcodes[0x01] = Some(&Instruction {
        opcode: 0x01,
        name: "LD BC, imm16",
        cycles: 3,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u8 {
            let byte_1 = cpu.fetch_next();
            let byte_2 = cpu.fetch_next();
            let mut dual_byte = byte_1 as u16 & 0xFF;
            dual_byte = dual_byte | (byte_2 as u16) << 8;
            cpu.registers.set_bc(dual_byte);
            opcode.cycles
        },
    });
    opcodes[0x02] = Some(&Instruction {
        opcode: 0x02,
        name: "LD [BC], A",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u8 {
            cpu.ram.write(cpu.registers.get_bc(), cpu.registers.get_a());
            opcode.cycles
        },
    });
    opcodes[0xCB] = Some(&Instruction {
        opcode: 0xCB,
        name: "CB SUBSET",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: fn a(cpu: &CPU) -> u8,
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
        execute: fn a(cpu: &CPU) -> u8,
    });
    opcodes[0x01] = Some(&Instruction {
        opcode: 0x01,
        name: "LD BC, d16",
        cycles: 3,
        size: 3,
        flags: &[],
        execute: fn a(cpu: &CPU) -> u8,
    });
    opcodes[0xCB] = Some(&Instruction {
        opcode: 0xCB,
        name: "LD BC, d16",
        cycles: 3,
        size: 3,
        flags: &[],
        execute: fn a(cpu: &CPU) -> u8{},
    });
    opcodes
}

pub const OPCODES: [Option<&'static Instruction>; 256] = create_opcodes();

pub const OPCODES_CB: [Option<&'static Instruction>; 256] = create_cb_opcodes();
