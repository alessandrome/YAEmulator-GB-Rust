use crate::GB::CPU::CPU;
use crate::GB::debug_print;
use crate::GB::registers::{FlagBits, Flags};

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: u8,
    pub name: &'static str,
    pub cycles: u8,
    pub size: u8,
    pub flags: &'static [FlagBits],
    pub execute: fn(&Instruction, &mut CPU) -> u64, // Return number on M-Cycles needed to execute
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
        flags |= (FlagBits::C as u8);
    } else {
        flags &= !(FlagBits::C as u8);
    }

    if zero_flag {
        flags |= (FlagBits::Z as u8);
    } else {
        flags &= !(FlagBits::Z as u8);
    }

    flags &= !(FlagBits::H as u8);

    // Settare i nuovi valori dei flag
    // flags può essere un riferimento mutabile a un altro registro che contiene i flags
    // quindi è necessario modificarlo come desiderato
    (a, flags)
}

const fn create_opcodes() -> [Option<&'static Instruction>; 256] {
    let mut opcodes = [None; 256];
    opcodes[0x00] = Some(&Instruction {
        opcode: 0x00,
        name: "NOP",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // NOP Do nothing
            opcode.cycles as u64
        },
    });
    opcodes[0x01] = Some(&Instruction {
        opcode: 0x01,
        name: "LD BC, imm16",
        cycles: 3,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte_1 = cpu.fetch_next();
            let byte_2 = cpu.fetch_next();
            let mut dual_byte = byte_1 as u16 & 0xFF;
            dual_byte = dual_byte | (byte_2 as u16) << 8;
            cpu.registers.set_bc(dual_byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x02] = Some(&Instruction {
        opcode: 0x02,
        name: "LD [BC], A",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_bc(), cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x03] = Some(&Instruction {
        opcode: 0x03,
        name: "INC BC",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_bc(cpu.registers.get_bc() + 1);
            opcode.cycles as u64
        },
    });
    opcodes[0x04] = Some(&Instruction {
        opcode: 0x04,
        name: "INC B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_b = cpu.registers.get_b();
            cpu.registers.set_b(cpu.registers.get_b().wrapping_add(1));
            // Write flags (This could be calculated and place and just made a single functions call to set Flag register)
            cpu.registers.set_half_carry_flag((original_b & 0x0F) == 0x0F);
            cpu.registers.set_zero_flag(cpu.registers.get_b() == 0);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x05] = Some(&Instruction {
        opcode: 0x05,
        name: "DEC B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_b = cpu.registers.get_b();
            cpu.registers.set_b(cpu.registers.get_b().wrapping_sub(1));
            // Write flags
            cpu.registers.set_half_carry_flag((original_b & 0x0F) == 0);
            cpu.registers.set_zero_flag(cpu.registers.get_b() == 0);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x06] = Some(&Instruction {
        opcode: 0x06,
        name: "LD B, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.registers.set_b(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x07] = Some(&Instruction {
        opcode: 0x07,
        name: "RLCA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_carry_flag((cpu.registers.get_a() & 0b1000_0000) != 0);
            cpu.registers.set_a(cpu.registers.get_a().wrapping_shl(1) | cpu.registers.get_a().wrapping_shr(7));
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x08] = Some(&Instruction {
        opcode: 0x08,
        name: "LD [a16], SP",
        cycles: 5,
        size: 3,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte_low_addr = cpu.fetch_next();
            let byte_high_addr = cpu.fetch_next();
            let imm16_address: u16 = (byte_high_addr as u16) << 8 | (byte_low_addr as u16) & 0xFF;
            cpu.ram.write(imm16_address, (cpu.registers.get_sp() & 0xFF) as u8);
            cpu.ram.write(imm16_address + 1, (cpu.registers.get_sp() >> 8) as u8);
            opcode.cycles as u64
        },
    });
    opcodes[0x09] = Some(&Instruction {
        opcode: 0x09,
        name: "ADD HL, BC",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // Half carry is checked on bit 11 of HL (or bit 3 of H)
            let pre_h = cpu.registers.get_h();
            let pre_l = cpu.registers.get_l();
            cpu.registers.set_negative_flag(false);
            if (cpu.registers.get_bc() != 0) {
                cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(cpu.registers.get_bc()));
                cpu.registers.set_half_carry_flag((cpu.registers.get_h() & 0x0F) < (pre_h & 0x0F));
                cpu.registers.set_carry_flag(cpu.registers.get_h() < pre_h);
            } else {
                cpu.registers.set_half_carry_flag(false);
                cpu.registers.set_carry_flag(false);
            }
            opcode.cycles as u64
        },
    });
    opcodes[0x0A] = Some(&Instruction {
        opcode: 0x0A,
        name: "LD A, [BC]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.ram.read(cpu.registers.get_bc()));
            opcode.cycles as u64
        },
    });
    opcodes[0x0B] = Some(&Instruction {
        opcode: 0x0B,
        name: "DEC BC",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_bc(cpu.registers.get_bc().wrapping_sub(1));
            opcode.cycles as u64
        },
    });
    opcodes[0x0C] = Some(&Instruction {
        opcode: 0x0C,
        name: "INC C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_c = cpu.registers.get_c();
            cpu.registers.set_c(cpu.registers.get_c().wrapping_add(1));
            cpu.registers.set_zero_flag(cpu.registers.get_c() == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((original_c & 0x0F) == 0x0F);
            opcode.cycles as u64
        },
    });
    opcodes[0x0D] = Some(&Instruction {
        opcode: 0x0D,
        name: "DEC C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_c = cpu.registers.get_c();
            cpu.registers.set_c(cpu.registers.get_c().wrapping_sub(1));
            cpu.registers.set_zero_flag(cpu.registers.get_c() == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((original_c & 0x0F) == 0);
            opcode.cycles as u64
        },
    });
    opcodes[0x0E] = Some(&Instruction {
        opcode: 0x0E,
        name: "LD C, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.registers.set_c(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x0F] = Some(&Instruction {
        opcode: 0x0F,
        name: "RRCA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            cpu.registers.set_carry_flag(cpu.registers.get_a() & 1 != 0);
            cpu.registers.set_a((cpu.registers.get_a().wrapping_shr(1)) | cpu.registers.get_a().wrapping_shl(7));
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x10] = Some(&Instruction {
        opcode: 0x10,
        name: "STOP imm8",
        cycles: 1,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: implement
            let byte = cpu.fetch_next();
            opcode.cycles as u64
        },
    });
    opcodes[0x11] = Some(&Instruction {
        opcode: 0x11,
        name: "LD DE, imm16",
        cycles: 3,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte_1 = cpu.fetch_next();
            let byte_2 = cpu.fetch_next();
            let mut dual_byte = byte_1 as u16 & 0xFF;
            dual_byte = dual_byte | (byte_2 as u16) << 8;
            cpu.registers.set_de(dual_byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x12] = Some(&Instruction {
        opcode: 0x12,
        name: "LD [DE], A",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_de(), cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x13] = Some(&Instruction {
        opcode: 0x13,
        name: "INC DE",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_de(cpu.registers.get_de() + 1);
            opcode.cycles as u64
        },
    });
    opcodes[0x14] = Some(&Instruction {
        opcode: 0x14,
        name: "INC D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_d = cpu.registers.get_d();
            cpu.registers.set_d(cpu.registers.get_d().wrapping_add(1));
            // Write flags (This could be calculated and place and just made a single functions call to set Flag register)
            cpu.registers.set_half_carry_flag((cpu.registers.get_d() & 0x0F) < (original_d & 0x0F));
            cpu.registers.set_zero_flag(cpu.registers.get_d() == 0);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x15] = Some(&Instruction {
        opcode: 0x15,
        name: "DEC D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_d = cpu.registers.get_d();
            cpu.registers.set_d(cpu.registers.get_d().wrapping_sub(1));
            // Write flags
            cpu.registers.set_half_carry_flag((cpu.registers.get_d() & 0x0F) > (original_d & 0x0F));
            cpu.registers.set_zero_flag(cpu.registers.get_d() == 0);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x16] = Some(&Instruction {
        opcode: 0x16,
        name: "LD D, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.registers.set_d(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x17] = Some(&Instruction {
        opcode: 0x17,
        name: "RLA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let original_carry_flag: u8 = cpu.registers.get_carry_flag() as u8;
            cpu.registers.set_carry_flag((cpu.registers.get_a() & 0b1000_0000) != 0);
            cpu.registers.set_a(cpu.registers.get_a().wrapping_shl(1) | original_carry_flag);
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x18] = Some(&Instruction {
        opcode: 0x18,
        name: "JR e8",
        cycles: 3,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i8;
            cpu.registers.set_pc(cpu.registers.get_pc() -  if byte < 0 {byte.abs()} else {byte} as u16);
            opcode.cycles as u64
        },
    });
    opcodes[0x19] = Some(&Instruction {
        opcode: 0x19,
        name: "ADD HL, DE",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let pre_h = cpu.registers.get_h();
            let pre_l = cpu.registers.get_l();
            cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(cpu.registers.get_de()));
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((cpu.registers.get_h() & 0x0F) < (pre_h & 0x0F));
            cpu.registers.set_carry_flag(cpu.registers.get_h() < pre_h);
            opcode.cycles as u64
        },
    });
    opcodes[0x1A] = Some(&Instruction {
        opcode: 0x1A,
        name: "LD A, [DE]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.ram.read(cpu.registers.get_de()));
            opcode.cycles as u64
        },
    });
    opcodes[0x1B] = Some(&Instruction {
        opcode: 0x1B,
        name: "DEC DE",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_de(cpu.registers.get_de().wrapping_sub(1));
            opcode.cycles as u64
        },
    });
    opcodes[0x1C] = Some(&Instruction {
        opcode: 0x1C,
        name: "INC E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_e = cpu.registers.get_e();
            cpu.registers.set_e(cpu.registers.get_e().wrapping_add(1));
            cpu.registers.set_zero_flag(cpu.registers.get_e() == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((cpu.registers.get_e() & 0x0F) < (original_e & 0x0F));
            opcode.cycles as u64
        },
    });
    opcodes[0x1D] = Some(&Instruction {
        opcode: 0x1D,
        name: "DEC E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_e = cpu.registers.get_e();
            cpu.registers.set_e(cpu.registers.get_e().wrapping_sub(1));
            cpu.registers.set_zero_flag(cpu.registers.get_e() == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((cpu.registers.get_e() & 0x0F) > (original_e & 0x0F));
            opcode.cycles as u64
        },
    });
    opcodes[0x1E] = Some(&Instruction {
        opcode: 0x1E,
        name: "LD E, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.registers.set_e(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x1F] = Some(&Instruction {
        opcode: 0x1F,
        name: "RRA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let original_carry_flag: u8 = cpu.registers.get_carry_flag() as u8;
            cpu.registers.set_carry_flag(cpu.registers.get_a() & 1 != 0);
            cpu.registers.set_a((cpu.registers.get_a().wrapping_shr(1)) | original_carry_flag.wrapping_shl(7));
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x20] = Some(&Instruction {
        opcode: 0x20,
        name: "JR NZ, e8",
        cycles: 3, // 2 Cycles if condition doesn't match
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i8;
            if !cpu.registers.get_zero_flag() {
                cpu.registers.set_pc(cpu.registers.get_pc() - if byte < 0 { byte.abs() } else { byte } as u16);
                return opcode.cycles as u64;
            }
            2
        },
    });
    opcodes[0x21] = Some(&Instruction {
        opcode: 0x21,
        name: "LD HL, imm16",
        cycles: 3,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte_1 = cpu.fetch_next();
            let byte_2 = cpu.fetch_next();
            let mut dual_byte = byte_1 as u16 & 0xFF;
            dual_byte = dual_byte | (byte_2 as u16) << 8;
            cpu.registers.set_hl(dual_byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x22] = Some(&Instruction {
        opcode: 0x22,
        name: "LD [HL+], A", // Sometimes HL+ is named as HLI (HL Increment)
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_a());
            cpu.registers.set_hl(cpu.registers.get_hl() + 1);
            opcode.cycles as u64
        },
    });
    opcodes[0x23] = Some(&Instruction {
        opcode: 0x23,
        name: "INC HL",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_hl(cpu.registers.get_hl() + 1);
            opcode.cycles as u64
        },
    });
    opcodes[0x24] = Some(&Instruction {
        opcode: 0x24,
        name: "INC H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_h = cpu.registers.get_h();
            cpu.registers.set_h(cpu.registers.get_h().wrapping_add(1));
            cpu.registers.set_half_carry_flag((cpu.registers.get_h() & 0x0F) < (original_h & 0x0F));
            cpu.registers.set_zero_flag(cpu.registers.get_h() == 0);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x25] = Some(&Instruction {
        opcode: 0x25,
        name: "DEC H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_h = cpu.registers.get_h();
            cpu.registers.set_h(cpu.registers.get_h().wrapping_sub(1));
            // Write flags
            cpu.registers.set_half_carry_flag((original_h & 0x0F) == 0);
            cpu.registers.set_zero_flag(cpu.registers.get_h() == 0);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x26] = Some(&Instruction {
        opcode: 0x26,
        name: "LD H, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.registers.set_h(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x27] = Some(&Instruction {
        opcode: 0x27,
        name: "DAA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let (a, flags) = daa(cpu.registers.get_a(), cpu.registers.get_f());
            cpu.registers.set_a(a);
            cpu.registers.set_f(flags);
            opcode.cycles as u64
        },
    });
    opcodes[0x28] = Some(&Instruction {
        opcode: 0x28,
        name: "JR Z, e8",
        cycles: 3, // 2 Cycles if condition doesn't match
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i8;
            if cpu.registers.get_zero_flag() {
                cpu.registers.set_pc(cpu.registers.get_pc() - if byte < 0 { byte.abs() } else { byte } as u16);
                return opcode.cycles as u64;
            }
            2
        },
    });
    opcodes[0x29] = Some(&Instruction {
        opcode: 0x29,
        name: "ADD HL, HL",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let pre_h = cpu.registers.get_h();
            let pre_l = cpu.registers.get_l();
            cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(cpu.registers.get_hl()));
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((cpu.registers.get_h() & 0x0F) < (pre_h & 0x0F));
            cpu.registers.set_carry_flag(cpu.registers.get_h() < pre_h);
            opcode.cycles as u64
        },
    });
    opcodes[0x2A] = Some(&Instruction {
        opcode: 0x3A,
        name: "LD A, [HL+]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.ram.read(cpu.registers.get_hl()));
            cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(1));
            opcode.cycles as u64
        },
    });
    opcodes[0x2B] = Some(&Instruction {
        opcode: 0x2B,
        name: "DEC HL",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_hl(cpu.registers.get_hl().wrapping_sub(1));
            opcode.cycles as u64
        },
    });
    opcodes[0x2C] = Some(&Instruction {
        opcode: 0x2C,
        name: "INC L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_l = cpu.registers.get_l();
            cpu.registers.set_l(cpu.registers.get_l().wrapping_add(1));
            cpu.registers.set_half_carry_flag((cpu.registers.get_l() & 0x0F) < (original_l & 0x0F));
            cpu.registers.set_zero_flag(cpu.registers.get_l() == 0);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x2D] = Some(&Instruction {
        opcode: 0x2D,
        name: "DEC L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_l = cpu.registers.get_l();
            cpu.registers.set_l(cpu.registers.get_l().wrapping_sub(1));
            // Write flags
            cpu.registers.set_half_carry_flag((original_l & 0x0F) == 0);
            cpu.registers.set_zero_flag(cpu.registers.get_l() == 0);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x2E] = Some(&Instruction {
        opcode: 0x2E,
        name: "LD L, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.registers.set_l(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x2F] = Some(&Instruction {
        opcode: 0x2F,
        name: "CPL",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(!cpu.registers.get_a());
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x30] = Some(&Instruction {
        opcode: 0x20,
        name: "JR NC, e8",
        cycles: 3, // 2 Cycles if condition doesn't match
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i8;
            if !cpu.registers.get_carry_flag() {
                cpu.registers.set_pc(cpu.registers.get_pc() - if byte < 0 { byte.abs() } else { byte } as u16);
                return opcode.cycles as u64;
            }
            2
        },
    });
    opcodes[0x31] = Some(&Instruction {
        opcode: 0x31,
        name: "LD SP, imm16",
        cycles: 3,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte_1 = cpu.fetch_next();
            let byte_2 = cpu.fetch_next();
            let mut dual_byte = byte_1 as u16 & 0xFF;
            dual_byte = dual_byte | (byte_2 as u16) << 8;
            cpu.registers.set_sp(dual_byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x32] = Some(&Instruction {
        opcode: 0x32,
        name: "LD [HL-], A", // Sometimes HL- is named as HLD (HL Decrement)
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_a());
            cpu.registers.set_hl(cpu.registers.get_hl() - 1);
            opcode.cycles as u64
        },
    });
    opcodes[0x33] = Some(&Instruction {
        opcode: 0x33,
        name: "INC SP",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_sp(cpu.registers.get_sp() + 1);
            opcode.cycles as u64
        },
    });
    opcodes[0x34] = Some(&Instruction {
        opcode: 0x34,
        name: "INC [HL]",
        cycles: 3,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_hl_ram = cpu.ram.read(cpu.registers.get_hl());
            cpu.ram.write(cpu.registers.get_hl(), original_hl_ram.wrapping_add(1));
            cpu.registers.set_half_carry_flag((cpu.ram.read(cpu.registers.get_hl()) & 0x0F) < (original_hl_ram & 0x0F));
            cpu.registers.set_zero_flag(cpu.ram.read(cpu.registers.get_hl()) == 0);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x35] = Some(&Instruction {
        opcode: 0x35,
        name: "DEC [HL]",
        cycles: 3,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_byte = cpu.ram.read(cpu.registers.get_hl());
            cpu.ram.write(cpu.registers.get_hl(), original_byte.wrapping_sub(1));
            cpu.registers.set_half_carry_flag((cpu.ram.read(cpu.registers.get_hl()) & 0x0F) > (original_byte & 0x0F));
            cpu.registers.set_zero_flag(cpu.ram.read(cpu.registers.get_hl()) == 0);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x36] = Some(&Instruction {
        opcode: 0x36,
        name: "LD [HL], imm8",
        cycles: 3,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.ram.write(cpu.registers.get_hl(), byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x37] = Some(&Instruction {
        opcode: 0x37,
        name: "SCF",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_carry_flag(true);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x38] = Some(&Instruction {
        opcode: 0x38,
        name: "JR C, e8",
        cycles: 3, // 2 Cycles if condition doesn't match
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i8;
            if cpu.registers.get_carry_flag() {
                cpu.registers.set_pc(cpu.registers.get_pc() - if byte < 0 { byte.abs() } else { byte } as u16);
                return opcode.cycles as u64;
            }
            2
        },
    });
    opcodes[0x39] = Some(&Instruction {
        opcode: 0x39,
        name: "ADD HL, SP",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let pre_h = cpu.registers.get_h();
            let pre_l = cpu.registers.get_l();
            cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(cpu.registers.get_sp()));
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((cpu.registers.get_h() & 0x0F) < (pre_h & 0x0F));
            cpu.registers.set_carry_flag(cpu.registers.get_h() < pre_h);
            opcode.cycles as u64
        },
    });
    opcodes[0x3A] = Some(&Instruction {
        opcode: 0x3A,
        name: "LD A, [HL-]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.ram.read(cpu.registers.get_hl()));
            cpu.registers.set_hl(cpu.registers.get_hl().wrapping_sub(1));
            opcode.cycles as u64
        },
    });
    opcodes[0x3B] = Some(&Instruction {
        opcode: 0x3B,
        name: "DEC SP",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_sp(cpu.registers.get_sp().wrapping_sub(1));
            opcode.cycles as u64
        },
    });
    opcodes[0x3C] = Some(&Instruction {
        opcode: 0x3C,
        name: "INC A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_a = cpu.registers.get_a();
            cpu.registers.set_a(cpu.registers.get_a().wrapping_add(1));
            cpu.registers.set_half_carry_flag((cpu.registers.get_a() & 0x0F) < (original_a & 0x0F));
            cpu.registers.set_zero_flag(cpu.registers.get_a() == 0);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x3D] = Some(&Instruction {
        opcode: 0x3D,
        name: "DEC A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_a = cpu.registers.get_a();
            cpu.registers.set_a(cpu.registers.get_a().wrapping_sub(1));
            // Write flags
            cpu.registers.set_half_carry_flag((original_a & 0x0F) == 0);
            cpu.registers.set_zero_flag(cpu.registers.get_a() == 0);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x3E] = Some(&Instruction {
        opcode: 0x3E,
        name: "LD A, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.registers.set_a(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x3F] = Some(&Instruction {
        opcode: 0x3F,
        name: "CCF",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(!cpu.registers.get_carry_flag());
            opcode.cycles as u64
        },
    });
    opcodes[0x40] = Some(&Instruction {
        opcode: 0x40,
        name: "LD B, B",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x41] = Some(&Instruction {
        opcode: 0x41,
        name: "LD B, C",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x42] = Some(&Instruction {
        opcode: 0x42,
        name: "LD B, D",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x43] = Some(&Instruction {
        opcode: 0x43,
        name: "LD B, E",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x44] = Some(&Instruction {
        opcode: 0x44,
        name: "LD B, H",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x45] = Some(&Instruction {
        opcode: 0x45,
        name: "LD B, L",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x46] = Some(&Instruction {
        opcode: 0x46,
        name: "LD B, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.ram.read(cpu.registers.get_hl()));
            opcode.cycles as u64
        },
    });
    opcodes[0x47] = Some(&Instruction {
        opcode: 0x47,
        name: "LD B, A",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x48] = Some(&Instruction {
        opcode: 0x48,
        name: "LD C, B",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x49] = Some(&Instruction {
        opcode: 0x49,
        name: "LD C, C",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x4A] = Some(&Instruction {
        opcode: 0x4A,
        name: "LD C, D",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x4B] = Some(&Instruction {
        opcode: 0x4B,
        name: "LD C, E",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x4C] = Some(&Instruction {
        opcode: 0x4C,
        name: "LD C, H",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x4D] = Some(&Instruction {
        opcode: 0x4D,
        name: "LD C, L",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x4E] = Some(&Instruction {
        opcode: 0x4E,
        name: "LD C, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.ram.read(cpu.registers.get_hl()));
            opcode.cycles as u64
        },
    });
    opcodes[0x4F] = Some(&Instruction {
        opcode: 0x4F,
        name: "LD C, A",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x50] = Some(&Instruction {
        opcode: 0x50,
        name: "LD D, B",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x51] = Some(&Instruction {
        opcode: 0x51,
        name: "LD D, C",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x52] = Some(&Instruction {
        opcode: 0x52,
        name: "LD D, D",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x53] = Some(&Instruction {
        opcode: 0x53,
        name: "LD D, E",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x54] = Some(&Instruction {
        opcode: 0x54,
        name: "LD D, H",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x55] = Some(&Instruction {
        opcode: 0x55,
        name: "LD D, L",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x56] = Some(&Instruction {
        opcode: 0x56,
        name: "LD D, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.ram.read(cpu.registers.get_hl()));
            opcode.cycles as u64
        },
    });
    opcodes[0x57] = Some(&Instruction {
        opcode: 0x57,
        name: "LD D, A",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x58] = Some(&Instruction {
        opcode: 0x58,
        name: "LD E, B",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x59] = Some(&Instruction {
        opcode: 0x59,
        name: "LD E, C",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x5A] = Some(&Instruction {
        opcode: 0x5A,
        name: "LD E, D",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x5B] = Some(&Instruction {
        opcode: 0x5B,
        name: "LD D, E",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x5C] = Some(&Instruction {
        opcode: 0x5C,
        name: "LD E, H",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x5D] = Some(&Instruction {
        opcode: 0x5D,
        name: "LD E, L",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x5E] = Some(&Instruction {
        opcode: 0x5E,
        name: "LD E, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.ram.read(cpu.registers.get_hl()));
            opcode.cycles as u64
        },
    });
    opcodes[0x5F] = Some(&Instruction {
        opcode: 0x5F,
        name: "LD E, A",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x60] = Some(&Instruction {
        opcode: 0x60,
        name: "LD H, B",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x61] = Some(&Instruction {
        opcode: 0x61,
        name: "LD H, C",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x62] = Some(&Instruction {
        opcode: 0x62,
        name: "LD H, D",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x63] = Some(&Instruction {
        opcode: 0x63,
        name: "LD H, E",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x64] = Some(&Instruction {
        opcode: 0x64,
        name: "LD H, H",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x65] = Some(&Instruction {
        opcode: 0x65,
        name: "LD H, L",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x66] = Some(&Instruction {
        opcode: 0x66,
        name: "LD H, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.ram.read(cpu.registers.get_hl()));
            opcode.cycles as u64
        },
    });
    opcodes[0x67] = Some(&Instruction {
        opcode: 0x67,
        name: "LD H, A",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x68] = Some(&Instruction {
        opcode: 0x68,
        name: "LD L, B",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x69] = Some(&Instruction {
        opcode: 0x69,
        name: "LD L, C",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x6A] = Some(&Instruction {
        opcode: 0x6A,
        name: "LD L, D",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x6B] = Some(&Instruction {
        opcode: 0x6B,
        name: "LD L, E",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x6C] = Some(&Instruction {
        opcode: 0x6C,
        name: "LD L, H",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x6D] = Some(&Instruction {
        opcode: 0x6D,
        name: "LD L, L",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x6E] = Some(&Instruction {
        opcode: 0x6E,
        name: "LD L, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.ram.read(cpu.registers.get_hl()));
            opcode.cycles as u64
        },
    });
    opcodes[0x6F] = Some(&Instruction {
        opcode: 0x6F,
        name: "LD L, A",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x70] = Some(&Instruction {
        opcode: 0x70,
        name: "LD [HL], B",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x71] = Some(&Instruction {
        opcode: 0x71,
        name: "LD [HL], C",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x72] = Some(&Instruction {
        opcode: 0x72,
        name: "LD [HL], D",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x73] = Some(&Instruction {
        opcode: 0x73,
        name: "LD [HL], E",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x74] = Some(&Instruction {
        opcode: 0x74,
        name: "LD [HL], H",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x75] = Some(&Instruction {
        opcode: 0x75,
        name: "LD [HL], L",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x76] = Some(&Instruction {
        opcode: 0x76,
        name: "HALT",  // LD [HL], [HL] Decode as HALT instruction on DMG/GBC
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: implement
            opcode.cycles as u64
        },
    });
    opcodes[0x77] = Some(&Instruction {
        opcode: 0x77,
        name: "LD [HL], A",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x78] = Some(&Instruction {
        opcode: 0x78,
        name: "LD A, B",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x79] = Some(&Instruction {
        opcode: 0x79,
        name: "LD A, C",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x7A] = Some(&Instruction {
        opcode: 0x7A,
        name: "LD A, D",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x7B] = Some(&Instruction {
        opcode: 0x7B,
        name: "LD A, E",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x7C] = Some(&Instruction {
        opcode: 0x7C,
        name: "LD A, H",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x7D] = Some(&Instruction {
        opcode: 0x7D,
        name: "LD A, L",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x7E] = Some(&Instruction {
        opcode: 0x7E,
        name: "LD A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.ram.read(cpu.registers.get_hl()));
            opcode.cycles as u64
        },
    });
    opcodes[0x7F] = Some(&Instruction {
        opcode: 0x7F,
        name: "LD A, A",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0xCB] = Some(&Instruction {
        opcode: 0xCB,
        name: "CB SUBSET",
        cycles: 0,
        size: 0,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // Implement CB subset will be executed on next fetching & execute
            opcode.cycles as u64
        },
    });
    opcodes
}

const fn create_cb_opcodes() -> [Option<&'static Instruction>; 256] {
    let mut opcodes = [None; 256];
    // opcodes[0x00] = Some(&Instruction {
    //     opcode: 0x00,
    //     name: "NOP",
    //     cycles: 1,
    //     size: 1,
    //     flags: &[],
    //     execute: fn a(cpu: &CPU) -> u8,
    // });
    // opcodes[0x01] = Some(&Instruction {
    //     opcode: 0x01,
    //     name: "LD BC, d16",
    //     cycles: 3,
    //     size: 3,
    //     flags: &[],
    //     execute: fn a(cpu: &CPU) -> u8,
    // });
    // opcodes[0xCB] = Some(&Instruction {
    //     opcode: 0xCB,
    //     name: "LD BC, d16",
    //     cycles: 3,
    //     size: 3,
    //     flags: &[],
    //     execute: fn a(cpu: &CPU) -> u8{},
    // });
    opcodes
}

pub const OPCODES: [Option<&'static Instruction>; 256] = create_opcodes();

pub const OPCODES_CB: [Option<&'static Instruction>; 256] = create_cb_opcodes();


#[cfg(test)]
mod test {
    use crate::GB::CPU::CPU;
    use crate::GB::RAM;
    use crate::GB::RAM::WRAM_ADDRESS;

    #[test]
    fn test_0x00_nop() {
        let mut cpu = CPU::new();
        let registers_copy = cpu.registers;
        let program: Vec<u8> = vec![0x00, 0x00];
        cpu.load(&program);
        cpu.execute_next();
        cpu.execute_next();
        // NOP should not alter any register
        assert_eq!(registers_copy.get_a(), cpu.registers.get_a());
        assert_eq!(registers_copy.get_b(), cpu.registers.get_b());
        assert_eq!(registers_copy.get_c(), cpu.registers.get_c());
        assert_eq!(registers_copy.get_d(), cpu.registers.get_d());
        assert_eq!(registers_copy.get_e(), cpu.registers.get_e());
        assert_eq!(registers_copy.get_f(), cpu.registers.get_f());
        assert_eq!(registers_copy.get_h(), cpu.registers.get_h());
        assert_eq!(registers_copy.get_l(), cpu.registers.get_l());
    }

    #[test]
    fn test_0x01_ld_bc_imm16() {
        let test_value: u16 = 0xC05A;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x01, 0x5A, 0xC0];
        cpu.load(&program);
        cpu.execute_next();
        // NOP should not alter any register
        assert_eq!(cpu.registers.get_b(), 0xC0);
        assert_eq!(cpu.registers.get_c(), 0x5A);
        assert_eq!(cpu.registers.get_bc(), test_value);
    }

    #[test]
    fn test_0x02_ld__bc__a() {
        let test_value: u8 = 0xF4;
        let test_address: u16 = RAM::WRAM_ADDRESS as u16 + 0x0500;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x02];
        cpu.load(&program);
        cpu.registers.set_a(test_value);
        cpu.registers.set_bc(test_address);
        cpu.execute_next();
        // NOP should not alter any register
        assert_eq!(cpu.registers.get_a(), test_value);
        assert_eq!(cpu.ram.read(test_address), test_value);
    }

    #[test]
    fn test_0x03_inc_bc() {
        let test_value: u16 = 0xC0F4;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x03];
        cpu.load(&program);
        cpu.registers.set_bc(test_value - 1);
        cpu.execute_next();
        // NOP should not alter any register
        assert_eq!(cpu.registers.get_b(), 0xC0);
        assert_eq!(cpu.registers.get_c(), 0xF4);
        assert_eq!(cpu.registers.get_bc(), test_value);
    }

    #[test]
    fn test_0x04_inc_b() {
        //No Flags
        let test_value_1: u8 = 0b1111_0100;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x04];
        cpu_1.load(&program_1);
        cpu_1.registers.set_b(test_value_1 - 1);
        cpu_1.execute_next();
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z/H
        let test_value_2: u8 = 0xFF;
        let mut cpu_2 = CPU::new();
        let program_2: Vec<u8> = vec![0x04];
        cpu_2.load(&program_2);
        cpu_2.registers.set_b(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_b(), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags H
        let test_value_2: u8 = 0x0F;
        cpu_2 = CPU::new();
        cpu_2.load(&program_2);
        cpu_2.registers.set_b(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_b(), 0x10);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x05_dec_b() {
        //No Flags
        let test_value_1: u8 = 0xF4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x05];
        cpu_1.load(&program_1);
        cpu_1.registers.set_b(test_value_1 + 1);
        cpu_1.execute_next();
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_2: u8 = 0x00;
        let mut cpu_2 = CPU::new();
        let program_2: Vec<u8> = vec![0x05];
        cpu_2.load(&program_2);
        cpu_2.registers.set_b(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_b(), 0xFF);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags Z
        let test_value_3: u8 = 0x00;
        let mut cpu_3 = CPU::new();
        let program_3: Vec<u8> = vec![0x05];
        cpu_3.load(&program_3);
        cpu_3.registers.set_b(test_value_3 + 1);
        cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_b(), test_value_3);
        assert_eq!(cpu_3.registers.get_zero_flag(), true);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_4: u8 = 0xF0;
        cpu_3 = CPU::new();
        cpu_3.load(&program_3);
        cpu_3.registers.set_b(test_value_4);
        cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_b(), test_value_4 - 1);
        assert_eq!(cpu_3.registers.get_zero_flag(), false);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x06_ld_b_imm8() {
        //No Flags
        let test_value_1: u8 = 0xCD;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x06, test_value_1];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.execute_next();
        // Check load data and FLAGs should be untouched
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x07_rlca() {
        //No Flags
        let test_value_1: u8 = 0b1000_1000;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x07, 0x07];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.execute_next();
        // Check load data and FLAGs should be untouched
        assert_eq!(cpu_1.registers.get_a(), 0b0001_0001);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
        cpu_1.execute_next();
        assert_eq!(cpu_1.registers.get_a(), 0b0010_0010);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);
    }

    #[test]
    fn test_0x08_ld__a16__sp() {
        //No Flags
        let test_value_1: u16 = 0xBD89;
        let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x89;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x08, 0x89, (test_address_1 >> 8) as u8];
        cpu_1.load(&program_1);
        cpu_1.registers.set_sp(test_value_1);
        let cycles = cpu_1.execute_next();
        // Check address and data are correctly used
        assert_eq!(cycles, 5);
        assert_eq!(cpu_1.registers.get_sp(), test_value_1);
        assert_eq!(cpu_1.ram.read(test_address_1), 0x89);
        assert_eq!(cpu_1.ram.read(test_address_1 + 1), (test_value_1 >> 8) as u8);
    }

    #[test]
    fn test_0x09_add_hl_bc() {
        //No Flags
        let mut test_value_1: u16 = 0xBD89;
        let mut test_value_2: u16 = 0x1029;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x09];
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_bc(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_bc(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x7000;
        test_value_2 = 0x9000;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_bc(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_bc(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), 0);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        // H flags on ADD HL, rr should be on only carrying from bit 11 (check is made on H of HL)
        test_value_1 = 0x1070;
        test_value_2 = 0x1090;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_bc(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_bc(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x1700;
        test_value_2 = 0x1900;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_bc(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_bc(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x9700;
        test_value_2 = 0x7900;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_bc(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_bc(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1.wrapping_add(test_value_2));
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x0a_ld_a__bc_() {
        let mut test_value_1: u8 = 0xBD;
        let mut test_address_1: u16 = WRAM_ADDRESS as u16 + 0x0128;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x0A];
        cpu_1.load(&program);
        cpu_1.registers.set_bc(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        cpu_1.registers.set_a(0x11); // Sure different from expected value
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.ram.read(test_address_1), test_value_1);
        assert_eq!(cpu_1.registers.get_bc(), test_address_1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
    }

    #[test]
    fn test_0x0b_dec_bc() {
        //No Flags
        let mut test_value_1: u16 = 0xBD89;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x0B];
        cpu_1.load(&program);
        cpu_1.registers.set_bc(test_value_1 + 1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_bc(), test_value_1);
    }

    #[test]
    fn test_0x0c_inc_c() {
        //No Flags
        let test_value_1: u8 = 0b1111_0100;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x0C];
        cpu_1.load(&program_1);
        cpu_1.registers.set_c(test_value_1 - 1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z/H
        let test_value_2: u8 = 0xFF;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_c(test_value_2);
        cycles = cpu_2.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_2.registers.get_c(), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags H
        let test_value_2: u8 = 0x0F;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_c(test_value_2);
        cycles = cpu_2.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_2.registers.get_c(), 0x10);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x0d_dec_c() {
        //No Flags
        let test_value_1: u8 = 0xF4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x0D];
        cpu_1.load(&program_1);
        cpu_1.registers.set_c(test_value_1 + 1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_2: u8 = 0x00;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_c(test_value_2);
        cycles = cpu_2.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_2.registers.get_c(), 0xFF);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags Z
        let test_value_3: u8 = 0x00;
        let mut cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_c(test_value_3 + 1);
        cycles = cpu_3.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_3.registers.get_c(), test_value_3);
        assert_eq!(cpu_3.registers.get_zero_flag(), true);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_4: u8 = 0xF0;
        cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_c(test_value_4);
        cycles = cpu_3.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_3.registers.get_c(), test_value_4 - 1);
        assert_eq!(cpu_3.registers.get_zero_flag(), false);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x0e_ld_c_imm8() {
        //No Flags
        let test_value_1: u8 = 0xD4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x0E, test_value_1];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_c(0xAA);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x0f_rrca() {
        //No Flags
        let test_value_1: u8 = 0b0001_0001;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x0F, 0x0F];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.execute_next();
        // Check load data and FLAGs should be untouched
        assert_eq!(cpu_1.registers.get_a(), 0b1000_1000);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
        cpu_1.execute_next();
        assert_eq!(cpu_1.registers.get_a(), 0b0100_0100);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);
    }

    #[cfg(test)]
    fn test_0x10_stop() {
        // TODO: Study and implement STOP function
    }

    #[test]
    fn test_0x11_ld_de_imm16() {
        let test_value: u16 = 0xC05A;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x11, 0x5A, 0xC0];
        cpu.load(&program);
        let cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_d(), 0xC0);
        assert_eq!(cpu.registers.get_e(), 0x5A);
        assert_eq!(cpu.registers.get_de(), test_value);
    }

    #[test]
    fn test_0x12_ld__de__a() {
        let test_value: u8 = 0xF4;
        let test_address: u16 = RAM::WRAM_ADDRESS as u16 + 0x0500;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x12];
        cpu.load(&program);
        cpu.registers.set_a(test_value);
        cpu.registers.set_de(test_address);
        cpu.ram.write(test_address, 0x00);
        let cycles = cpu.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.get_a(), test_value);
        assert_eq!(cpu.registers.get_de(), test_address);
        assert_eq!(cpu.ram.read(test_address), test_value);
    }

    #[test]
    fn test_0x13_inc_de() {
        let test_value: u16 = 0xC0F4;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x13];
        cpu.load(&program);
        cpu.registers.set_de(test_value - 1);
        cpu.execute_next();
        // NOP should not alter any register
        assert_eq!(cpu.registers.get_d(), 0xC0);
        assert_eq!(cpu.registers.get_e(), 0xF4);
        assert_eq!(cpu.registers.get_de(), test_value);
    }

    #[test]
    fn test_0x14_inc_d() {
        //No Flags
        let test_value_1: u8 = 0b1111_0100;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x14];
        cpu_1.load(&program_1);
        cpu_1.registers.set_d(test_value_1 - 1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z/H
        let test_value_2: u8 = 0xFF;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_d(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_d(), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags H
        let test_value_2: u8 = 0x0F;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_d(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_d(), 0x10);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x15_dec_d() {
        //No Flags
        let test_value_1: u8 = 0xF4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x15];
        cpu_1.load(&program_1);
        cpu_1.registers.set_d(test_value_1 + 1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_2: u8 = 0x00;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_d(test_value_2);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_d(), 0xFF);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags Z
        let test_value_3: u8 = 0x00;
        let mut cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_d(test_value_3 + 1);
        cycle = cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_d(), test_value_3);
        assert_eq!(cpu_3.registers.get_zero_flag(), true);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_4: u8 = 0xF0;
        cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_d(test_value_4);
        cycle = cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_d(), test_value_4 - 1);
        assert_eq!(cpu_3.registers.get_zero_flag(), false);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x16_ld_d_imm8() {
        //No Flags
        let test_value_1: u8 = 0xCD;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x16, test_value_1];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        let cycles = cpu_1.execute_next();
        // Check load data and FLAGs should be untouched
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x17_rla() {
        //No Flags
        let test_value_1: u8 = 0b1000_1000;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x17, 0x17];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(false);
        cpu_1.execute_next();
        // The re-entrance Bit is given by the previous content of C Flag
        assert_eq!(cpu_1.registers.get_a(), 0b0001_0000);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
        cpu_1.execute_next();
        assert_eq!(cpu_1.registers.get_a(), 0b0010_0001);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);
    }

    #[test]
    fn test_0x18_jr_e8() {
        let mut test_value: i8 = -50;
        let mut start_address: i16 = 0x0350;
        let mut cpu = CPU::new();
        let mut program: Vec<u8> = vec![0x18, test_value as u8];
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        let mut cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);
    }

    #[test]
    fn test_0x19_add_hl_de() {
        //No Flags
        let mut test_value_1: u16 = 0xBD89;
        let mut test_value_2: u16 = 0x1029;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x19];
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_de(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_de(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x7000;
        test_value_2 = 0x9000;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_de(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_de(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), 0);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        // H flags on ADD HL, rr should be on only carrying from bit 11 (check is made on H of HL)
        test_value_1 = 0x1070;
        test_value_2 = 0x1090;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_de(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_de(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x1700;
        test_value_2 = 0x1900;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_de(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_de(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x9700;
        test_value_2 = 0x7900;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_de(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_de(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1.wrapping_add(test_value_2));
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x1a_ld_a__de_() {
        let mut test_value_1: u8 = 0xBD;
        let mut test_address_1: u16 = WRAM_ADDRESS as u16 + 0x0128;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x1A];
        cpu_1.load(&program);
        cpu_1.registers.set_de(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        cpu_1.registers.set_a(0x11); // Sure different from expected value
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.ram.read(test_address_1), test_value_1);
        assert_eq!(cpu_1.registers.get_de(), test_address_1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
    }

    #[test]
    fn test_0x1b_dec_bc() {
        //No Flags
        let mut test_value_1: u16 = 0xBD89;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x1B];
        cpu_1.load(&program);
        cpu_1.registers.set_de(test_value_1 + 1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_de(), test_value_1);
    }

    #[test]
    fn test_0x1c_inc_e() {
        //No Flags
        let test_value_1: u8 = 0b1111_0100;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x1C];
        cpu_1.load(&program_1);
        cpu_1.registers.set_e(test_value_1 - 1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 1);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z/H
        let test_value_2: u8 = 0xFF;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_e(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_e(), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags H
        let test_value_2: u8 = 0x0F;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_e(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_e(), 0x10);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x1d_dec_e() {
        //No Flags
        let test_value_1: u8 = 0xF4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x1D];
        cpu_1.load(&program_1);
        cpu_1.registers.set_e(test_value_1 + 1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 1);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_2: u8 = 0x00;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_e(test_value_2);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_e(), 0xFF);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags Z
        let test_value_3: u8 = 0x00;
        let mut cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_e(test_value_3 + 1);
        cycle = cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_e(), test_value_3);
        assert_eq!(cpu_3.registers.get_zero_flag(), true);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_4: u8 = 0xF0;
        cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_e(test_value_4);
        cycle = cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_e(), test_value_4 - 1);
        assert_eq!(cpu_3.registers.get_zero_flag(), false);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x1e_ld_e_imm8() {
        //No Flags
        let test_value_1: u8 = 0xD4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x1E, test_value_1];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_e(0xAA);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x1f_rra() {
        //No Flags
        let test_value_1: u8 = 0b0001_0001;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x1F, 0x1F];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(false);
        cpu_1.execute_next();
        // The re-entrance Bit is given by the previous content of C Flag
        assert_eq!(cpu_1.registers.get_a(), 0b0000_1000);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
        cpu_1.execute_next();
        assert_eq!(cpu_1.registers.get_a(), 0b1000_0100);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);
    }

    #[test]
    fn test_0x20_jr_nz_e8() {
        let mut test_value: i8 = -50;
        let mut start_address: i16 = 0x0350;
        let mut cpu = CPU::new();
        let mut program: Vec<u8> = vec![0x20, test_value as u8];
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_zero_flag(false);
        let mut cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);

        cpu = CPU::new();
        assert_eq!(cycles, 3);
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_zero_flag(true);
        cycles = cpu.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.get_pc(), 0x352);
    }

    #[test]
    fn test_0x21_ld_hl_imm16() {
        let test_value: u16 = 0xC05A;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x21, 0x5A, 0xC0];
        cpu.load(&program);
        let cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_h(), 0xC0);
        assert_eq!(cpu.registers.get_l(), 0x5A);
        assert_eq!(cpu.registers.get_hl(), test_value);
    }

    #[test]
    fn test_0x22_ld__hli__a() {
        let test_value: u8 = 0xF4;
        let test_address: u16 = RAM::WRAM_ADDRESS as u16 + 0x0500;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x22];
        cpu.load(&program);
        cpu.registers.set_a(test_value);
        cpu.registers.set_hl(test_address);
        cpu.ram.write(test_address, 0x00);
        let cycles = cpu.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.get_a(), test_value);
        assert_eq!(cpu.registers.get_hl(), test_address + 1);
        assert_eq!(cpu.ram.read(test_address), test_value);
    }

    #[test]
    fn test_0x23_inc_de() {
        let test_value: u16 = 0xC0F4;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x23];
        cpu.load(&program);
        cpu.registers.set_hl(test_value - 1);
        cpu.execute_next();
        assert_eq!(cpu.registers.get_h(), 0xC0);
        assert_eq!(cpu.registers.get_l(), 0xF4);
        assert_eq!(cpu.registers.get_hl(), test_value);
    }

    #[test]
    fn test_0x24_inc_h() {
        // No Flags
        let test_value_1: u8 = 0b1111_0100;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x24];
        cpu_1.load(&program_1);
        cpu_1.registers.set_h(test_value_1 - 1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 1);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z/H
        let test_value_2: u8 = 0xFF;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_h(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_h(), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags H
        let test_value_2: u8 = 0x0F;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_h(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_h(), 0x10);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x25_dec_h() {
        //No Flags
        let test_value_1: u8 = 0xF4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x25];
        cpu_1.load(&program_1);
        cpu_1.registers.set_h(test_value_1 + 1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 1);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_2: u8 = 0x00;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_h(test_value_2);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_h(), 0xFF);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags Z
        let test_value_3: u8 = 0x00;
        let mut cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_h(test_value_3 + 1);
        cycle = cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_h(), test_value_3);
        assert_eq!(cpu_3.registers.get_zero_flag(), true);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_4: u8 = 0xF0;
        cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_h(test_value_4);
        cycle = cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_h(), test_value_4 - 1);
        assert_eq!(cpu_3.registers.get_zero_flag(), false);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x26_ld_h_imm8() {
        //No Flags
        let test_value_1: u8 = 0xCD;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x26, test_value_1];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        let cycles = cpu_1.execute_next();
        // Check load data and FLAGs should be untouched
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x27_daa() {
        // TODO: Implement test for DAA and CBD values
    }

    #[test]
    fn test_0x28_jr_z_e8() {
        let mut test_value: i8 = -50;
        let mut start_address: i16 = 0x0350;
        let mut cpu = CPU::new();
        let mut program: Vec<u8> = vec![0x28, test_value as u8];
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_zero_flag(true);
        let mut cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);

        cpu = CPU::new();
        assert_eq!(cycles, 3);
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_zero_flag(false);
        cycles = cpu.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.get_pc(), 0x352);
    }

    #[test]
    fn test_0x29_add_hl_de() {
        let mut test_value: u16 = 0x1029;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x29];
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_hl(), test_value + test_value);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value = 0x8000;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_hl(), 0);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        // H flags on ADD HL, rr should be on only carrying from bit 11 (check is made on H of HL)
        test_value = 0x1080;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value);
        cpu_1.registers.set_de(test_value);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_hl(), test_value + test_value);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value = 0x1800;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value);
        cpu_1.registers.set_de(test_value);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_hl(), test_value + test_value);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value = 0x8800;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value);
        cpu_1.registers.set_de(test_value);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_hl(), test_value.wrapping_add(test_value));
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x2a_ld_a__hli_() {
        let mut test_value_1: u8 = 0xBD;
        let mut test_address_1: u16 = WRAM_ADDRESS as u16 + 0x0128;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x2A];
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        cpu_1.registers.set_a(0x11); // Sure different from expected value
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.ram.read(test_address_1), test_value_1);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1 + 1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
    }

    #[test]
    fn test_0x2b_dec_hl() {
        //No Flags
        let mut test_value_1: u16 = 0xBD89;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x2B];
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1 + 1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1);
    }

    #[test]
    fn test_0x2c_inc_l() {
        //No Flags
        let test_value_1: u8 = 0b1111_0100;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x2C];
        cpu_1.load(&program_1);
        cpu_1.registers.set_l(test_value_1 - 1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 1);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z/H
        let test_value_2: u8 = 0xFF;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_l(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_l(), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags H
        let test_value_2: u8 = 0x0F;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_l(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_l(), 0x10);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x2d_dec_l() {
        //No Flags
        let test_value_1: u8 = 0xF4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x2D];
        cpu_1.load(&program_1);
        cpu_1.registers.set_l(test_value_1 + 1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 1);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_2: u8 = 0x00;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_l(test_value_2);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_l(), 0xFF);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags Z
        let test_value_3: u8 = 0x00;
        let mut cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_l(test_value_3 + 1);
        cycle = cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_l(), test_value_3);
        assert_eq!(cpu_3.registers.get_zero_flag(), true);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_4: u8 = 0xF0;
        cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_l(test_value_4);
        cycle = cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_l(), test_value_4 - 1);
        assert_eq!(cpu_3.registers.get_zero_flag(), false);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x2e_ld_l_imm8() {
        //No Flags
        let test_value_1: u8 = 0xD4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x2E, test_value_1];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_l(0xAA);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x2f_cpl() {
        //No Flags
        let test_value_1: u8 = 0xD4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x2F];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_a(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), !test_value_1);
        // Z/C Flags untouched - N/H Flags on
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x30_jr_nc_e8() {
        let mut test_value: i8 = -50;
        let mut start_address: i16 = 0x0350;
        let mut cpu = CPU::new();
        let mut program: Vec<u8> = vec![0x30, test_value as u8];
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_carry_flag(false);
        let mut cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);

        cpu = CPU::new();
        assert_eq!(cycles, 3);
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_carry_flag(true);
        cycles = cpu.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.get_pc(), 0x352);
    }

    #[test]
    fn test_0x31_ld_sp_imm16() {
        let test_value: u16 = 0xC05A;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x31, 0x5A, 0xC0];
        cpu.load(&program);
        let cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_sp(), test_value);
    }

    #[test]
    fn test_0x32_ld__hld__a() {
        let test_value: u8 = 0xF4;
        let test_address: u16 = RAM::WRAM_ADDRESS as u16 + 0x0500;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x32];
        cpu.load(&program);
        cpu.registers.set_a(test_value);
        cpu.registers.set_hl(test_address);
        cpu.ram.write(test_address, 0x00);
        let cycles = cpu.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.get_a(), test_value);
        assert_eq!(cpu.registers.get_hl(), test_address - 1);
        assert_eq!(cpu.ram.read(test_address), test_value);
    }

    #[test]
    fn test_0x33_inc_de() {
        let test_value: u16 = 0xC0F4;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x33];
        cpu.load(&program);
        cpu.registers.set_sp(test_value - 1);
        cpu.execute_next();
        assert_eq!(cpu.registers.get_sp(), test_value);
    }

    #[test]
    fn test_0x34_inc__hl_() {
        // No Flags
        let test_value_1: u8 = 0b1111_0100;
        let test_address = WRAM_ADDRESS as u16 + 0x50;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x34];
        cpu_1.load(&program_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 3);
        assert_eq!(cpu_1.ram.read(test_address), test_value_1 + 1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z/H
        let test_value_2: u8 = 0xFF;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_hl(test_address);
        cpu_2.ram.write(test_address, test_value_2);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.ram.read(test_address), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags H
        let test_value_3: u8 = 0x0F;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_hl(test_address);
        cpu_2.ram.write(test_address, test_value_3);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.ram.read(test_address), 0x10);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x35_dec__hl_() {
        // No Flags
        let test_value_1: u8 = 0b1111_0100;
        let test_address = WRAM_ADDRESS as u16 + 0x50;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x35];
        cpu_1.load(&program_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 3);
        assert_eq!(cpu_1.ram.read(test_address), test_value_1 - 1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z
        let test_value_2: u8 = 0x01;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_hl(test_address);
        cpu_2.ram.write(test_address, test_value_2);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.ram.read(test_address), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_3: u8 = 0xF0;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_hl(test_address);
        cpu_2.ram.write(test_address, test_value_3);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.ram.read(test_address), test_value_3 - 1);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Test Underflow
        let test_value_4: u8 = 0x00;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_hl(test_address);
        cpu_2.ram.write(test_address, test_value_4);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.ram.read(test_address), 0xFF);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x36_ld__hl__imm8() {
        //No Flags
        let test_value_1: u8 = 0xCD;
        let test_address: u16 = WRAM_ADDRESS as u16 + 0x88;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x36, test_value_1];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, 0x00);
        let cycles = cpu_1.execute_next();
        // Check load data and FLAGs should be untouched
        assert_eq!(cycles, 3);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        assert_eq!(cpu_1.ram.read(test_address), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    // SCF = Set Carry Flag
    fn test_0x37_scf() {
        //No Flags
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x37];
        cpu_1.load(&program_1);
        cpu_1.registers.set_carry_flag(false);
        let cycles = cpu_1.execute_next();
        // Check load data and FLAGs should be untouched
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x38_jr_c_e8() {
        let mut test_value: i8 = -50;
        let mut start_address: i16 = 0x0350;
        let mut cpu = CPU::new();
        let mut program: Vec<u8> = vec![0x38, test_value as u8];
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_carry_flag(true);
        let mut cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);

        cpu = CPU::new();
        assert_eq!(cycles, 3);
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_carry_flag(false);
        cycles = cpu.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.get_pc(), 0x352);
    }

    #[test]
    fn test_0x39_add_hl_sp() {
        //No Flags
        let mut test_value_1: u16 = 0xBD89;
        let mut test_value_2: u16 = 0x1029;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x39];
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_sp(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_sp(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x7000;
        test_value_2 = 0x9000;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_sp(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_sp(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), 0);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        // H flags on ADD HL, rr should be on only carrying from bit 11 (check is made on H of HL)
        test_value_1 = 0x1070;
        test_value_2 = 0x1090;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_sp(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_sp(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x1700;
        test_value_2 = 0x1900;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_sp(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_sp(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x9700;
        test_value_2 = 0x7900;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_sp(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_sp(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1.wrapping_add(test_value_2));
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x3a_ld_a__hld_() {
        let mut test_value_1: u8 = 0xBD;
        let mut test_address_1: u16 = WRAM_ADDRESS as u16 + 0x0128;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x3A];
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        cpu_1.registers.set_a(0x11); // Sure different from expected value
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.ram.read(test_address_1), test_value_1);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1 - 1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
    }

    #[test]
    fn test_0x3b_dec_sp() {
        //No Flags
        let mut test_value_1: u16 = 0xBD89;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x3B];
        cpu_1.load(&program);
        cpu_1.registers.set_sp(test_value_1 + 1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_sp(), test_value_1);
    }

    #[test]
    fn test_0x3c_inc_a() {
        //No Flags
        let test_value_1: u8 = 0b1111_0100;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x3C];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1 - 1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z/H
        let test_value_2: u8 = 0xFF;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_a(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_a(), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags H
        let test_value_2: u8 = 0x0F;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_a(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_a(), 0x10);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x3d_dec_a() {
        //No Flags
        let test_value_1: u8 = 0xF4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x3D];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1 + 1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_2: u8 = 0x00;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_a(test_value_2);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_a(), 0xFF);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags Z
        let test_value_3: u8 = 0x00;
        let mut cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_a(test_value_3 + 1);
        cycle = cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_a(), test_value_3);
        assert_eq!(cpu_3.registers.get_zero_flag(), true);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_4: u8 = 0xF0;
        cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_a(test_value_4);
        cycle = cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_a(), test_value_4 - 1);
        assert_eq!(cpu_3.registers.get_zero_flag(), false);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x3e_ld_a_imm8() {
        //No Flags
        let test_value_1: u8 = 0xD4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x3E, test_value_1];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_a(0xAA);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x3f_ccf() {
        // CCF = Complement Carry Flag
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x3F, 0x3F];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_carry_flag(false);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        // Z Flag untouched - N/H Flags off
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        // Z Flag untouched - N/H Flags off
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);
    }

    #[test]
    fn test_0x40_ld_b_b() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x40];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_b(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x41_ld_b_c() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x41];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_b(0x00);
        cpu_1.registers.set_c(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x42_ld_b_d() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x42];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_b(0x00);
        cpu_1.registers.set_d(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x43_ld_b_e() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x43];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_b(0x00);
        cpu_1.registers.set_e(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x44_ld_b_h() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x44];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_b(0x00);
        cpu_1.registers.set_h(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x45_ld_b_l() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x45];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_b(0x00);
        cpu_1.registers.set_l(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x46_ld_b__hl_() {
        let test_value_1: u8 = 0xC4;
        let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x99;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x46];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_b(0x00);
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x47_ld_b_a() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x47];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_b(0x00);
        cpu_1.registers.set_a(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x48_ld_c_b() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x48];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_c(0x00);
        cpu_1.registers.set_b(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x49_ld_c_c() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x49];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_c(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x4a_ld_c_d() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x4A];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_c(0x00);
        cpu_1.registers.set_d(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x4b_ld_c_e() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x4B];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_c(0x00);
        cpu_1.registers.set_e(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x4c_ld_c_h() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x4C];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_c(0x00);
        cpu_1.registers.set_h(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x4d_ld_c_l() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x4D];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_c(0x00);
        cpu_1.registers.set_l(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x4e_ld_c__hl_() {
        let test_value_1: u8 = 0xC4;
        let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x99;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x4E];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_c(0x00);
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x4f_ld_c_a() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x4F];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_c(0x00);
        cpu_1.registers.set_a(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x50_ld_d_b() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x50];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_d(0x00);
        cpu_1.registers.set_b(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x51_ld_d_c() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x51];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_d(0x00);
        cpu_1.registers.set_c(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x52_ld_d_d() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x52];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_d(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x53_ld_d_e() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x53];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_d(0x00);
        cpu_1.registers.set_e(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x54_ld_d_h() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x54];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_d(0x00);
        cpu_1.registers.set_h(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x55_ld_d_l() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x55];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_d(0x00);
        cpu_1.registers.set_l(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x56_ld_d__hl_() {
        let test_value_1: u8 = 0xC4;
        let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x99;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x56];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_d(0x00);
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x57_ld_d_a() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x57];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_d(0x00);
        cpu_1.registers.set_a(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x58_ld_e_b() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x58];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_e(0x00);
        cpu_1.registers.set_b(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x59_ld_e_c() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x59];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_e(0x00);
        cpu_1.registers.set_c(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x5a_ld_e_d() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x5A];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_e(0x00);
        cpu_1.registers.set_d(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x5b_ld_e_e() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x5B];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_e(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x5c_ld_e_h() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x5C];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_e(0x000);
        cpu_1.registers.set_h(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x5d_ld_e_l() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x5D];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_e(0x000);
        cpu_1.registers.set_l(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x5e_ld_e__hl_() {
        let test_value_1: u8 = 0xC4;
        let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x99;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x5E];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_e(0x00);
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x5f_ld_e_a() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x5F];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_e(0x000);
        cpu_1.registers.set_a(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x60_ld_h_b() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x60];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_h(0x000);
        cpu_1.registers.set_b(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x61_ld_h_c() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x61];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_h(0x00);
        cpu_1.registers.set_c(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x62_ld_h_d() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x62];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_h(0x00);
        cpu_1.registers.set_d(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x63_ld_h_e() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x63];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_h(0x000);
        cpu_1.registers.set_e(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x64_ld_h_h() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x64];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_h(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x65_ld_h_l() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x65];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_h(0x00);
        cpu_1.registers.set_l(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x66_ld_h__hl_() {
        let test_value_1: u8 = 0xFF;
        let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x99;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x66];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1 & 0xFF | (test_value_1 as u16) << 8);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x67_ld_h_a() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x67];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_h(0x00);
        cpu_1.registers.set_a(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x68_ld_l_b() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x68];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_l(0x00);
        cpu_1.registers.set_b(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x69_ld_l_c() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x69];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_l(0x00);
        cpu_1.registers.set_c(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x6a_ld_l_d() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x6A];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_l(0x00);
        cpu_1.registers.set_d(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x6b_ld_l_e() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x6B];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_l(0x00);
        cpu_1.registers.set_e(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        assert_eq!(cpu_1.registers.get_e(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x6c_ld_l_h() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x6C];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_l(0x00);
        cpu_1.registers.set_h(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x6d_ld_l_l() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x6D];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_l(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x6e_ld_h__hl_() {
        let test_value_1: u8 = 0xFF;
        let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x99;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x6E];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1 & 0xFF00 | (test_value_1 as u16));
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x6f_ld_l_a() {
        let test_value_1: u8 = 0xC4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x6F];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_l(0x00);
        cpu_1.registers.set_a(test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x70_ld__hl__b() {
        let test_value_1: u8 = 0xC4;
        let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x99;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x70];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_b(test_value_1);
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, 0x00);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1);
        assert_eq!(cpu_1.ram.read(test_address_1), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }
}