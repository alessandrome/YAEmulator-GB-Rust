#[cfg(test)]
mod test_default;
#[cfg(test)]
mod test_subset;

use crate::GB::CPU::CPU;
use crate::GB::debug_print;
use crate::GB::CPU::registers::core::{FlagBits, Flags};
use crate::GB::memory::{UseMemory};
use crate::GB::memory;

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
            cpu.write_memory(cpu.registers.get_bc(), cpu.registers.get_a());
            if cpu.registers.get_bc() == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
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
            cpu.write_memory(imm16_address, (cpu.registers.get_sp() & 0xFF) as u8);
            cpu.write_memory(imm16_address + 1, (cpu.registers.get_sp() >> 8) as u8);
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
            cpu.registers.set_a(cpu.read_memory(cpu.registers.get_bc()));
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
            cpu.write_memory(cpu.registers.get_de(), cpu.registers.get_a());
            if cpu.registers.get_de() == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
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
            let byte = cpu.fetch_next() as i8;
            if byte > 0 {
                cpu.registers.set_pc(cpu.registers.get_pc().wrapping_add(byte as u16));
            } else {
                cpu.registers.set_pc(cpu.registers.get_pc().wrapping_sub((byte as i16).abs() as u16));
            }
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
            cpu.registers.set_a(cpu.read_memory(cpu.registers.get_de()));
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
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next() as i8;
            if !cpu.registers.get_zero_flag() {
                if byte > 0 {
                    cpu.registers.set_pc(cpu.registers.get_pc().wrapping_add(byte as u16));
                } else {
                    cpu.registers.set_pc(cpu.registers.get_pc().wrapping_sub((byte as i16).abs() as u16));
                }
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
            cpu.write_memory(cpu.registers.get_hl(), cpu.registers.get_a());
            if cpu.registers.get_hl() == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
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
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i8;
            if cpu.registers.get_zero_flag() {
                if byte > 0 {
                    cpu.registers.set_pc(cpu.registers.get_pc().wrapping_add(byte as u16));
                } else {
                    cpu.registers.set_pc(cpu.registers.get_pc().wrapping_sub((byte as i16).abs() as u16));
                }
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
            cpu.registers.set_a(cpu.read_memory(cpu.registers.get_hl()));
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
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i8;
            if !cpu.registers.get_carry_flag() {
                if byte > 0 {
                    cpu.registers.set_pc(cpu.registers.get_pc().wrapping_add(byte as u16));
                } else {
                    cpu.registers.set_pc(cpu.registers.get_pc().wrapping_sub((byte as i16).abs() as u16));
                }
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
            cpu.write_memory(cpu.registers.get_hl(), cpu.registers.get_a());
            if cpu.registers.get_hl() == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
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
            let original_hl_ram = cpu.read_memory(cpu.registers.get_hl());
            if cpu.registers.get_hl() == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
            cpu.write_memory(cpu.registers.get_hl(), original_hl_ram.wrapping_add(1));
            cpu.registers.set_half_carry_flag((cpu.read_memory(cpu.registers.get_hl()) & 0x0F) < (original_hl_ram & 0x0F));
            cpu.registers.set_zero_flag(cpu.read_memory(cpu.registers.get_hl()) == 0);
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
            let original_byte = cpu.read_memory(cpu.registers.get_hl());
            if cpu.registers.get_hl() == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
            cpu.write_memory(cpu.registers.get_hl(), original_byte.wrapping_sub(1));
            cpu.registers.set_half_carry_flag((cpu.read_memory(cpu.registers.get_hl()) & 0x0F) > (original_byte & 0x0F));
            cpu.registers.set_zero_flag(cpu.read_memory(cpu.registers.get_hl()) == 0);
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
            cpu.write_memory(cpu.registers.get_hl(), byte);
            if cpu.registers.get_hl() == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
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
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i8;
            if cpu.registers.get_carry_flag() {
                if byte > 0 {
                    cpu.registers.set_pc(cpu.registers.get_pc().wrapping_add(byte as u16));
                } else {
                    cpu.registers.set_pc(cpu.registers.get_pc().wrapping_sub((byte as i16).abs() as u16));
                }
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
            cpu.registers.set_a(cpu.read_memory(cpu.registers.get_hl()));
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
            cpu.registers.set_b(cpu.read_memory(cpu.registers.get_hl()));
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
            cpu.registers.set_c(cpu.read_memory(cpu.registers.get_hl()));
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
            cpu.registers.set_d(cpu.read_memory(cpu.registers.get_hl()));
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
            cpu.registers.set_e(cpu.read_memory(cpu.registers.get_hl()));
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
            cpu.registers.set_h(cpu.read_memory(cpu.registers.get_hl()));
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
            cpu.registers.set_l(cpu.read_memory(cpu.registers.get_hl()));
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
            cpu.write_memory(cpu.registers.get_hl(), cpu.registers.get_b());
            if cpu.registers.get_hl() == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
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
            cpu.write_memory(cpu.registers.get_hl(), cpu.registers.get_c());
            if cpu.registers.get_hl() == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
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
            cpu.write_memory(cpu.registers.get_hl(), cpu.registers.get_d());
            if cpu.registers.get_hl() == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
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
            cpu.write_memory(cpu.registers.get_hl(), cpu.registers.get_e());
            if cpu.registers.get_hl() == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
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
            cpu.write_memory(cpu.registers.get_hl(), cpu.registers.get_h());
            if cpu.registers.get_hl() == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
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
            cpu.write_memory(cpu.registers.get_hl(), cpu.registers.get_l());
            if cpu.registers.get_hl() == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
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
            cpu.write_memory(cpu.registers.get_hl(), cpu.registers.get_a());
            if cpu.registers.get_hl() == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
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
            cpu.registers.set_a(cpu.read_memory(cpu.registers.get_hl()));
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
    opcodes[0x80] = Some(&Instruction {
        opcode: 0x80,
        name: "ADD A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(cpu.registers.get_b());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x81] = Some(&Instruction {
        opcode: 0x81,
        name: "ADD A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(cpu.registers.get_c());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x82] = Some(&Instruction {
        opcode: 0x82,
        name: "ADD A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(cpu.registers.get_d());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x83] = Some(&Instruction {
        opcode: 0x83,
        name: "ADD A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(cpu.registers.get_e());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x84] = Some(&Instruction {
        opcode: 0x84,
        name: "ADD A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(cpu.registers.get_h());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x85] = Some(&Instruction {
        opcode: 0x85,
        name: "ADD A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(cpu.registers.get_l());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x86] = Some(&Instruction {
        opcode: 0x86,
        name: "ADD A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(cpu.read_memory(cpu.registers.get_hl()));
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x87] = Some(&Instruction {
        opcode: 0x87,
        name: "ADD A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(old_value);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x88] = Some(&Instruction {
        opcode: 0x88,
        name: "ADC A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.registers.get_b());
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x89] = Some(&Instruction {
        opcode: 0x88,
        name: "ADC A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.registers.get_c());
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x8A] = Some(&Instruction {
        opcode: 0x8A,
        name: "ADC A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.registers.get_d());
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x8B] = Some(&Instruction {
        opcode: 0x8B,
        name: "ADC A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.registers.get_e());
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x8C] = Some(&Instruction {
        opcode: 0x8C,
        name: "ADC A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.registers.get_h());
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x8D] = Some(&Instruction {
        opcode: 0x8D,
        name: "ADC A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.registers.get_l());
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x8E] = Some(&Instruction {
        opcode: 0x8E,
        name: "ADC A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.read_memory(cpu.registers.get_hl()));
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x8F] = Some(&Instruction {
        opcode: 0x8F,
        name: "ADC A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.registers.get_a());
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x90] = Some(&Instruction {
        opcode: 0x90,
        name: "SUB A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_b());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x91] = Some(&Instruction {
        opcode: 0x91,
        name: "SUB A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_c());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x92] = Some(&Instruction {
        opcode: 0x92,
        name: "SUB A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_d());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x93] = Some(&Instruction {
        opcode: 0x93,
        name: "SUB A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_e());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x94] = Some(&Instruction {
        opcode: 0x94,
        name: "SUB A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_h());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x95] = Some(&Instruction {
        opcode: 0x95,
        name: "SUB A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_l());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x96] = Some(&Instruction {
        opcode: 0x96,
        name: "SUB A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.read_memory(cpu.registers.get_hl()));
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x97] = Some(&Instruction {
        opcode: 0x97,
        name: "SUB A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_a());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x98] = Some(&Instruction {
        opcode: 0x98,
        name: "SBC A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.registers.get_b());
            let new_value = intra_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x99] = Some(&Instruction {
        opcode: 0x99,
        name: "SBC A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.registers.get_c());
            let new_value = intra_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x9A] = Some(&Instruction {
        opcode: 0x9A,
        name: "SBC A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.registers.get_d());
            let new_value = intra_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x9B] = Some(&Instruction {
        opcode: 0x9B,
        name: "SBC A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.registers.get_e());
            let new_value = intra_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x9C] = Some(&Instruction {
        opcode: 0x9C,
        name: "SBC A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.registers.get_h());
            let new_value = intra_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x9D] = Some(&Instruction {
        opcode: 0x9D,
        name: "SBC A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.registers.get_l());
            let new_value = intra_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x9E] = Some(&Instruction {
        opcode: 0x9E,
        name: "SBC A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.read_memory(cpu.registers.get_hl()));
            let new_value = intra_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x9F] = Some(&Instruction {
        opcode: 0x9F,
        name: "SBC A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.registers.get_a());
            let new_value = intra_value.wrapping_sub(intra_value)
                .wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xA0] = Some(&Instruction {
        opcode: 0xA0,
        name: "AND A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.registers.get_b();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA1] = Some(&Instruction {
        opcode: 0xA1,
        name: "AND A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.registers.get_c();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA2] = Some(&Instruction {
        opcode: 0xA2,
        name: "AND A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.registers.get_d();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA3] = Some(&Instruction {
        opcode: 0xA3,
        name: "AND A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.registers.get_e();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA4] = Some(&Instruction {
        opcode: 0xA4,
        name: "AND A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.registers.get_h();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA5] = Some(&Instruction {
        opcode: 0xA5,
        name: "AND A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.registers.get_l();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA6] = Some(&Instruction {
        opcode: 0xA6,
        name: "AND A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.read_memory(cpu.registers.get_hl());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA7] = Some(&Instruction {
        opcode: 0xA7,
        name: "AND A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & old_value;
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA8] = Some(&Instruction {
        opcode: 0xA8,
        name: "XOR A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.registers.get_b();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA9] = Some(&Instruction {
        opcode: 0xA9,
        name: "XOR A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.registers.get_c();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xAA] = Some(&Instruction {
        opcode: 0xAA,
        name: "XOR A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.registers.get_d();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xAB] = Some(&Instruction {
        opcode: 0xAB,
        name: "XOR A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.registers.get_e();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xAC] = Some(&Instruction {
        opcode: 0xAC,
        name: "XOR A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.registers.get_h();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xAD] = Some(&Instruction {
        opcode: 0xAD,
        name: "XOR A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.registers.get_l();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xAE] = Some(&Instruction {
        opcode: 0xAE,
        name: "XOR A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.read_memory(cpu.registers.get_hl());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xAF] = Some(&Instruction {
        opcode: 0xAF,
        name: "XOR A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ old_value;
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB0] = Some(&Instruction {
        opcode: 0xB0,
        name: "OR A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.registers.get_b();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB1] = Some(&Instruction {
        opcode: 0xB1,
        name: "OR A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.registers.get_c();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB2] = Some(&Instruction {
        opcode: 0xB2,
        name: "OR A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.registers.get_d();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB3] = Some(&Instruction {
        opcode: 0xB3,
        name: "OR A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.registers.get_e();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB4] = Some(&Instruction {
        opcode: 0xB4,
        name: "OR A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.registers.get_h();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB5] = Some(&Instruction {
        opcode: 0xB5,
        name: "OR A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.registers.get_l();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB6] = Some(&Instruction {
        opcode: 0xB6,
        name: "OR A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.read_memory(cpu.registers.get_hl());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB7] = Some(&Instruction {
        opcode: 0xB7,
        name: "OR A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | old_value;
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB8] = Some(&Instruction {
        opcode: 0xB8,
        name: "CP A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_b());
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xB9] = Some(&Instruction {
        opcode: 0xB9,
        name: "CP A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_c());
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xBA] = Some(&Instruction {
        opcode: 0xBA,
        name: "CP A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_d());
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xBB] = Some(&Instruction {
        opcode: 0xBB,
        name: "CP A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_e());
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xBC] = Some(&Instruction {
        opcode: 0xBC,
        name: "CP A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_h());
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xBD] = Some(&Instruction {
        opcode: 0xBD,
        name: "CP A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_l());
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xBE] = Some(&Instruction {
        opcode: 0xBE,
        name: "CP A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.read_memory(cpu.registers.get_hl()));
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xBF] = Some(&Instruction {
        opcode: 0xBF,
        name: "CP A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_a());
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xC0] = Some(&Instruction {
        opcode: 0xC0,
        name: "RET NZ",
        cycles: 5,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            if !cpu.registers.get_zero_flag() {
                let mut return_address: u16 = 0x00;
                return_address |= cpu.pop() as u16;
                return_address |= (cpu.pop() as u16) << 8;
                cpu.registers.set_pc(return_address);
                return opcode.cycles as u64
            }
            2
        },
    });
    opcodes[0xC1] = Some(&Instruction {
        opcode: 0xC1,
        name: "POP BC",
        cycles: 3,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut byte = cpu.pop();
            cpu.registers.set_c(byte);
            byte = cpu.pop();
            cpu.registers.set_b(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0xC2] = Some(&Instruction {
        opcode: 0xC2,
        name: "JP NZ, imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut return_address: u16 = 0x00;
            return_address |= cpu.fetch_next() as u16;
            return_address |= (cpu.fetch_next() as u16) << 8;
            if !cpu.registers.get_zero_flag() {
                cpu.registers.set_pc(return_address);
                return opcode.cycles as u64
            }
            3
        },
    });
    opcodes[0xC3] = Some(&Instruction {
        opcode: 0xC3,
        name: "JP imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut return_address: u16 = 0x00;
            return_address |= cpu.fetch_next() as u16;
            return_address |= (cpu.fetch_next() as u16) << 8;
            cpu.registers.set_pc(return_address);
            opcode.cycles as u64
        },
    });
    opcodes[0xC4] = Some(&Instruction {
        opcode: 0xC4,
        name: "CALL NZ, imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut call_address: u16 = 0x00;
            call_address |= cpu.fetch_next() as u16;
            call_address |= (cpu.fetch_next() as u16) << 8;
            if !cpu.registers.get_zero_flag() {
                let return_address = cpu.registers.get_pc();
                cpu.push((return_address >> 8) as u8);
                cpu.push((return_address & 0xFF) as u8);
                cpu.registers.set_pc(call_address);
                return opcode.cycles as u64
            }
            3
        },
    });
    opcodes[0xC5] = Some(&Instruction {
        opcode: 0xC5,
        name: "PUSH BC",
        cycles: 3,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.push(cpu.registers.get_b());
            cpu.push(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0xC6] = Some(&Instruction {
        opcode: 0xC6,
        name: "ADD A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(byte);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xC7] = Some(&Instruction {
        opcode: 0xC7,
        name: "RST $00",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xC8] = Some(&Instruction {
        opcode: 0xC8,
        name: "RET Z",
        cycles: 5,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            if cpu.registers.get_zero_flag() {
                let mut return_addr = cpu.pop() as u16;
                return_addr |= (cpu.pop() as u16) << 8;
                cpu.registers.set_pc(return_addr);
                return opcode.cycles as u64
            }
            2
        },
    });
    opcodes[0xC9] = Some(&Instruction {
        opcode: 0xC9,
        name: "RET",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut return_addr = cpu.pop() as u16;
            return_addr |= (cpu.pop() as u16) << 8;
            cpu.registers.set_pc(return_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xCA] = Some(&Instruction {
        opcode: 0xCA,
        name: "JP Z, imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut return_address: u16 = 0x00;
            return_address |= cpu.fetch_next() as u16;
            return_address |= (cpu.fetch_next() as u16) << 8;
            if cpu.registers.get_zero_flag() {
                cpu.registers.set_pc(return_address);
                return opcode.cycles as u64
            }
            3
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
            cpu.execute_next()
        },
    });
    opcodes[0xCC] = Some(&Instruction {
        opcode: 0xCC,
        name: "CALL Z, imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut call_address: u16 = 0x00;
            call_address |= cpu.fetch_next() as u16;
            call_address |= (cpu.fetch_next() as u16) << 8;
            if cpu.registers.get_zero_flag() {
                cpu.push((cpu.registers.get_pc() >> 8) as u8);
                cpu.push((cpu.registers.get_pc() & 0xFF) as u8);
                cpu.registers.set_pc(call_address);
                return opcode.cycles as u64
            }
            3
        },
    });
    opcodes[0xCD] = Some(&Instruction {
        opcode: 0xCD,
        name: "CALL imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut call_address: u16 = 0x00;
            call_address |= cpu.fetch_next() as u16;
            call_address |= (cpu.fetch_next() as u16) << 8;
            cpu.push((cpu.registers.get_pc() >> 8) as u8);
            cpu.push((cpu.registers.get_pc() & 0xFF) as u8);
            cpu.registers.set_pc(call_address);
            return opcode.cycles as u64
        },
    });
    opcodes[0xCE] = Some(&Instruction {
        opcode: 0xCE,
        name: "ADC A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(byte);
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xCF] = Some(&Instruction {
        opcode: 0xCF,
        name: "RST $08",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xD0] = Some(&Instruction {
        opcode: 0xD0,
        name: "RET CZ",
        cycles: 5,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            if !cpu.registers.get_carry_flag() {
                let mut return_address: u16 = 0x00;
                return_address |= cpu.pop() as u16;
                return_address |= (cpu.pop() as u16) << 8;
                cpu.registers.set_pc(return_address);
                return opcode.cycles as u64
            }
            2
        },
    });
    opcodes[0xD1] = Some(&Instruction {
        opcode: 0xD1,
        name: "POP DE",
        cycles: 3,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut byte = cpu.pop();
            cpu.registers.set_e(byte);
            byte = cpu.pop();
            cpu.registers.set_d(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0xD2] = Some(&Instruction {
        opcode: 0xD2,
        name: "JP NC, imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut return_address: u16 = 0x00;
            return_address |= cpu.fetch_next() as u16;
            return_address |= (cpu.fetch_next() as u16) << 8;
            if !cpu.registers.get_carry_flag() {
                cpu.registers.set_pc(return_address);
                return opcode.cycles as u64
            }
            3
        },
    });
    opcodes[0xD4] = Some(&Instruction {
        opcode: 0xD4,
        name: "CALL NC, imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut call_address: u16 = 0x00;
            call_address |= cpu.fetch_next() as u16;
            call_address |= (cpu.fetch_next() as u16) << 8;
            if !cpu.registers.get_carry_flag() {
                let return_address = cpu.registers.get_pc();
                cpu.push((return_address >> 8) as u8);
                cpu.push((return_address & 0xFF) as u8);
                cpu.registers.set_pc(call_address);
                return opcode.cycles as u64
            }
            3
        },
    });
    opcodes[0xD5] = Some(&Instruction {
        opcode: 0xD5,
        name: "PUSH DE",
        cycles: 3,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.push(cpu.registers.get_d());
            cpu.push(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0xD6] = Some(&Instruction {
        opcode: 0xD6,
        name: "SUB A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(byte);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xD7] = Some(&Instruction {
        opcode: 0xD7,
        name: "RST $10",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xD8] = Some(&Instruction {
        opcode: 0xD8,
        name: "RET C",
        cycles: 5,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            if cpu.registers.get_carry_flag() {
                let mut return_addr = cpu.pop() as u16;
                return_addr |= (cpu.pop() as u16) << 8;
                cpu.registers.set_pc(return_addr);
                return opcode.cycles as u64
            }
            2
        },
    });
    opcodes[0xD9] = Some(&Instruction {
        opcode: 0xD9,
        name: "RETI",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut return_addr = cpu.pop() as u16;
            return_addr |= (cpu.pop() as u16) << 8;
            cpu.registers.set_pc(return_addr);
            cpu.ime = true;
            opcode.cycles as u64
        },
    });
    opcodes[0xDA] = Some(&Instruction {
        opcode: 0xDA,
        name: "JP C, imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut return_address: u16 = 0x00;
            return_address |= cpu.fetch_next() as u16;
            return_address |= (cpu.fetch_next() as u16) << 8;
            if cpu.registers.get_carry_flag() {
                cpu.registers.set_pc(return_address);
                return opcode.cycles as u64
            }
            3
        },
    });
    opcodes[0xDC] = Some(&Instruction {
        opcode: 0xDC,
        name: "CALL C, imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut call_address: u16 = 0x00;
            call_address |= cpu.fetch_next() as u16;
            call_address |= (cpu.fetch_next() as u16) << 8;
            if cpu.registers.get_carry_flag() {
                cpu.push((cpu.registers.get_pc() >> 8) as u8);
                cpu.push((cpu.registers.get_pc() & 0xFF) as u8);
                cpu.registers.set_pc(call_address);
                return opcode.cycles as u64
            }
            3
        },
    });
    opcodes[0xDE] = Some(&Instruction {
        opcode: 0xDE,
        name: "SBC A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_sub(byte);
            let new_value = intermediate_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) > (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xDF] = Some(&Instruction {
        opcode: 0xDF,
        name: "RST $18",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xE0] = Some(&Instruction {
        opcode: 0xE0,
        name: "LDH [imm8], A",
        cycles: 3,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = (cpu.fetch_next() as u16) & 0xFF;
            let mem_addr = 0xFF00 | byte;
            cpu.write_memory(mem_addr, cpu.registers.get_a());
            if mem_addr == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
            opcode.cycles as u64
        },
    });
    opcodes[0xE1] = Some(&Instruction {
        opcode: 0xE1,
        name: "POP HL",
        cycles: 3,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut byte = cpu.pop();
            cpu.registers.set_l(byte);
            byte = cpu.pop();
            cpu.registers.set_h(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0xE2] = Some(&Instruction {
        opcode: 0xE2,
        name: "LDH [C], A",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = (cpu.registers.get_c() as u16) & 0xFF;
            let mem_addr = 0xFF00 | byte;
            cpu.write_memory(mem_addr, cpu.registers.get_a());
            if mem_addr == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
            opcode.cycles as u64
        },
    });
    opcodes[0xE5] = Some(&Instruction {
        opcode: 0xE5,
        name: "PUSH HL",
        cycles: 3,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.push(cpu.registers.get_h());
            cpu.push(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0xE6] = Some(&Instruction {
        opcode: 0xE6,
        name: "AND A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.fetch_next();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xE7] = Some(&Instruction {
        opcode: 0xE7,
        name: "RST $20",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xE8] = Some(&Instruction {
        opcode: 0xE8,
        name: "ADD SP, e8",
        cycles: 4,
        size: 2,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = (cpu.fetch_next() as i8) as i16;
            let old_sp = cpu.registers.get_sp();
            if byte > 0 {
                cpu.registers.set_sp(old_sp.wrapping_add(byte as u16));
                cpu.registers.set_half_carry_flag((cpu.registers.get_sp() & 0x0F00) < (old_sp & 0x0F00));
                cpu.registers.set_carry_flag(cpu.registers.get_sp() < old_sp);
            } else {
                cpu.registers.set_sp(old_sp.wrapping_sub(byte.abs() as u16));
                cpu.registers.set_half_carry_flag((cpu.registers.get_sp() & 0x0F00) > (old_sp & 0x0F00));
                cpu.registers.set_carry_flag(cpu.registers.get_sp() > old_sp);
            }
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xE9] = Some(&Instruction {
        opcode: 0xE9,
        name: "JP HL",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_pc(cpu.registers.get_hl());
            opcode.cycles as u64
        },
    });
    opcodes[0xEA] = Some(&Instruction {
        opcode: 0xEA,
        name: "LD [imm16], A",
        cycles: 4,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut imm_address: u16 = 0x00;
            imm_address |= cpu.fetch_next() as u16 & 0xFF;
            imm_address |= (cpu.fetch_next() as u16) << 8;
            cpu.write_memory(imm_address, cpu.registers.get_a());
            if imm_address == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
            opcode.cycles as u64
        },
    });
    opcodes[0xEE] = Some(&Instruction {
        opcode: 0xEE,
        name: "XOR A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.fetch_next();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xEF] = Some(&Instruction {
        opcode: 0xEF,
        name: "RST $28",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xF0] = Some(&Instruction {
        opcode: 0xF0,
        name: "LDH A, [imm8]",
        cycles: 3,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = (cpu.fetch_next() as u16) & 0xFF;
            let mem_addr = 0xFF00 | byte;
            cpu.registers.set_a(cpu.read_memory(mem_addr));
            if mem_addr == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
            opcode.cycles as u64
        },
    });
    opcodes[0xF1] = Some(&Instruction {
        opcode: 0xF1,
        name: "POP AF",
        cycles: 3,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut byte = cpu.pop();
            cpu.registers.set_f(byte);
            byte = cpu.pop();
            cpu.registers.set_a(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0xF2] = Some(&Instruction {
        opcode: 0xF2,
        name: "LDH A, [C]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = (cpu.registers.get_c() as u16) & 0xFF;
            let mem_addr = 0xFF00 | byte;
            cpu.registers.set_a(cpu.read_memory(mem_addr));
            if mem_addr == memory::registers::DMA {
                cpu.dma_transfer = true;
            }
            opcode.cycles as u64
        },
    });
    opcodes[0xF3] = Some(&Instruction {
        opcode: 0xF3,
        name: "DI",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ime = false;
            opcode.cycles as u64
        },
    });
    opcodes[0xF5] = Some(&Instruction {
        opcode: 0xF5,
        name: "PUSH AF",
        cycles: 3,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.push(cpu.registers.get_a());
            cpu.push(cpu.registers.get_f());
            opcode.cycles as u64
        },
    });
    opcodes[0xF6] = Some(&Instruction {
        opcode: 0xF6,
        name: "OR A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.fetch_next();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xF7] = Some(&Instruction {
        opcode: 0xF7,
        name: "RST $30",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xF8] = Some(&Instruction {
        opcode: 0xF8,
        name: "LD HL, SP + e8",
        cycles: 3,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = (cpu.fetch_next() as i8) as i16;
            let mut temp_sp = cpu.registers.get_sp();
            if byte > 0 {
                temp_sp = temp_sp.wrapping_add(byte as u16);
                cpu.registers.set_half_carry_flag((temp_sp & 0x0F00) < (cpu.registers.get_sp() & 0x0F00));
                cpu.registers.set_carry_flag(temp_sp < cpu.registers.get_sp());
            } else {
                temp_sp = temp_sp.wrapping_sub(byte.abs() as u16);
                cpu.registers.set_half_carry_flag((temp_sp & 0x0F00) > (cpu.registers.get_sp() & 0x0F00));
                cpu.registers.set_carry_flag(temp_sp > cpu.registers.get_sp());
            }
            cpu.registers.set_hl(temp_sp);
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xF9] = Some(&Instruction {
        opcode: 0xF9,
        name: "LD SP, HL",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_sp(cpu.registers.get_hl());
            opcode.cycles as u64
        },
    });
    opcodes[0xFA] = Some(&Instruction {
        opcode: 0xFA,
        name: "LD A, [imm16]",
        cycles: 4,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut imm_address: u16 = 0x00;
            imm_address |= cpu.fetch_next() as u16 & 0xFF;
            imm_address |= (cpu.fetch_next() as u16) << 8;
            cpu.registers.set_a(cpu.read_memory(imm_address));
            opcode.cycles as u64
        },
    });
    opcodes[0xFB] = Some(&Instruction {
        opcode: 0xFB,
        name: "EI",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ime = true;
            opcode.cycles as u64
        },
    });
    opcodes[0xFE] = Some(&Instruction {
        opcode: 0xFE,
        name: "CP A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(byte);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xFF] = Some(&Instruction {
        opcode: 0xFF,
        name: "RST $30",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes
}

const fn create_cb_opcodes() -> [Option<&'static Instruction>; 256] {
    macro_rules! rlc {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    cpu.registers.set_carry_flag((old_val & 0b1000_0000) != 0);
                    let new_val = old_val.wrapping_shl(1) | cpu.registers.get_carry_flag() as u8;
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.read_memory(cpu.registers.$get_reg());
                    cpu.registers.set_carry_flag((old_val & 0b1000_0000) != 0);
                    let new_val = old_val.wrapping_shl(1) | cpu.registers.get_carry_flag() as u8;
                    cpu.write_memory(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }
    macro_rules! rl {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    let old_carry = cpu.registers.get_carry_flag() as u8;
                    cpu.registers.set_carry_flag((old_val & 0b1000_0000) != 0);
                    let new_val = old_val.wrapping_shl(1) | old_carry;
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.read_memory(cpu.registers.$get_reg());
                    let old_carry = cpu.registers.get_carry_flag() as u8;
                    cpu.registers.set_carry_flag((old_val & 0b1000_0000) != 0);
                    let new_val = old_val.wrapping_shl(1) | old_carry as u8;
                    cpu.write_memory(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }
    macro_rules! sla {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    cpu.registers.set_carry_flag((old_val & 0b1000_0000) != 0);
                    let new_val = old_val.wrapping_shl(1);
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.read_memory(cpu.registers.$get_reg());
                    cpu.registers.set_carry_flag((old_val & 0b1000_0000) != 0);
                    let new_val = old_val.wrapping_shl(1);
                    cpu.write_memory(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }

    macro_rules! rrc {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    cpu.registers.set_carry_flag((old_val & 0b0000_0001) != 0);
                    let new_val = old_val.wrapping_shr(1) | ((cpu.registers.get_carry_flag() as u8) << 7);
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.read_memory(cpu.registers.$get_reg());
                    cpu.registers.set_carry_flag((old_val & 0b0000_0001) != 0);
                    let new_val = old_val.wrapping_shr(1) | ((cpu.registers.get_carry_flag() as u8) << 7);
                    cpu.write_memory(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }
    macro_rules! rr {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    let old_carry = cpu.registers.get_carry_flag() as u8;
                    cpu.registers.set_carry_flag((old_val & 0b0000_0001) != 0);
                    let new_val = old_val.wrapping_shr(1) | (old_carry << 7);
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.read_memory(cpu.registers.$get_reg());
                    let old_carry = cpu.registers.get_carry_flag() as u8;
                    cpu.registers.set_carry_flag((old_val & 0b0000_0001) != 0);
                    let new_val = old_val.wrapping_shr(1) | (old_carry << 7);
                    cpu.write_memory(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }
    macro_rules! sra {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    cpu.registers.set_carry_flag((old_val & 0b0000_0001) != 0);
                    let new_val = old_val.wrapping_shr(1) | (old_val & 0b1000_0000);
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.read_memory(cpu.registers.$get_reg());
                    cpu.registers.set_carry_flag((old_val & 0b0000_0001) != 0);
                    let new_val = old_val.wrapping_shr(1) | (old_val & 0b1000_0000);
                    cpu.write_memory(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }
    macro_rules! srl {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    cpu.registers.set_carry_flag((old_val & 0b0000_0001) != 0);
                    let new_val = old_val.wrapping_shr(1);
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.read_memory(cpu.registers.$get_reg());
                    cpu.registers.set_carry_flag((old_val & 0b0000_0001) != 0);
                    let new_val = old_val.wrapping_shr(1);
                    cpu.write_memory(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }

    macro_rules! swap {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    let old_low_nibble =  old_val & 0x0F;
                    let old_high_nibble =  old_val & 0xF0;
                    let new_val = (old_low_nibble << 4) | (old_high_nibble >> 4);
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    cpu.registers.set_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.read_memory(cpu.registers.$get_reg());
                    let old_low_nibble =  old_val & 0x0F;
                    let old_high_nibble =  old_val & 0xF0;
                    let new_val = (old_low_nibble << 4) | (old_high_nibble >> 4);
                    cpu.write_memory(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    cpu.registers.set_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }

    macro_rules! bit {
        ($opcode:expr, $name:expr, r8, $bit:expr, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let val = cpu.registers.$get_reg() & (1 << $bit);
                    cpu.registers.set_zero_flag(val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(true);
                    cpu.registers.set_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $bit:expr, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let val = cpu.read_memory(cpu.registers.$get_reg()) & (1 << $bit);
                    cpu.registers.set_zero_flag(val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(true);
                    cpu.registers.set_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }

    macro_rules! res {
        ($opcode:expr, $name:expr, r8, $bit:expr, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let mask: u8 = !(1 << $bit);
                    cpu.registers.$set_reg(cpu.registers.$get_reg() & mask);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $bit:expr, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let mask: u8 = !(1 << $bit);
                    cpu.write_memory(cpu.registers.$get_reg(), cpu.read_memory(cpu.registers.$get_reg()) & mask);
                    opcode.cycles as u64
                }
            })
        };
    }

    macro_rules! set {
        ($opcode:expr, $name:expr, r8, $bit:expr, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let mask: u8 = (1 << $bit);
                    cpu.registers.$set_reg(cpu.registers.$get_reg() | mask);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $bit:expr, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let mask: u8 = (1 << $bit);
                    cpu.write_memory(cpu.registers.$get_reg(), cpu.read_memory(cpu.registers.$get_reg()) | mask);
                    opcode.cycles as u64
                }
            })
        };
    }

    let mut opcodes = [None; 256];
    opcodes[0x00] = rlc!(0x00, "RLC B", r8, set_b, get_b);
    opcodes[0x01] = rlc!(0x01, "RLC C", r8, set_c, get_c);
    opcodes[0x02] = rlc!(0x02, "RLC D", r8, set_d, get_d);
    opcodes[0x03] = rlc!(0x03, "RLC E", r8, set_e, get_e);
    opcodes[0x04] = rlc!(0x04, "RLC H", r8, set_h, get_h);
    opcodes[0x05] = rlc!(0x05, "RLC L", r8, set_l, get_l);
    opcodes[0x06] = rlc!(0x06, "RLC [HL]", ar16, set_hl, get_hl);
    opcodes[0x07] = rlc!(0x07, "RLC A", r8, set_a, get_a);

    opcodes[0x08] = rrc!(0x08, "RRC B", r8, set_b, get_b);
    opcodes[0x09] = rrc!(0x09, "RRC C", r8, set_c, get_c);
    opcodes[0x0A] = rrc!(0x0a, "RRC D", r8, set_d, get_d);
    opcodes[0x0B] = rrc!(0x0b, "RRC E", r8, set_e, get_e);
    opcodes[0x0C] = rrc!(0x0c, "RRC H", r8, set_h, get_h);
    opcodes[0x0D] = rrc!(0x0d, "RRC L", r8, set_l, get_l);
    opcodes[0x0E] = rrc!(0x0e, "RRC [HL]", ar16, set_hl, get_hl);
    opcodes[0x0F] = rrc!(0x0f, "RRC A", r8, set_a, get_a);

    opcodes[0x10] = rl!(0x10, "RL B", r8, set_b, get_b);
    opcodes[0x11] = rl!(0x11, "RL C", r8, set_c, get_c);
    opcodes[0x12] = rl!(0x12, "RL D", r8, set_d, get_d);
    opcodes[0x13] = rl!(0x13, "RL E", r8, set_e, get_e);
    opcodes[0x14] = rl!(0x14, "RL H", r8, set_h, get_h);
    opcodes[0x15] = rl!(0x15, "RL L", r8, set_l, get_l);
    opcodes[0x16] = rl!(0x16, "RL [HL]", ar16, set_hl, get_hl);
    opcodes[0x17] = rl!(0x17, "RL A", r8, set_a, get_a);

    opcodes[0x18] = rr!(0x18, "RR B", r8, set_b, get_b);
    opcodes[0x19] = rr!(0x19, "RR C", r8, set_c, get_c);
    opcodes[0x1A] = rr!(0x1a, "RR D", r8, set_d, get_d);
    opcodes[0x1B] = rr!(0x1b, "RR E", r8, set_e, get_e);
    opcodes[0x1C] = rr!(0x1c, "RR H", r8, set_h, get_h);
    opcodes[0x1D] = rr!(0x1d, "RR L", r8, set_l, get_l);
    opcodes[0x1E] = rr!(0x1e, "RR [HL]", ar16, set_hl, get_hl);
    opcodes[0x1F] = rr!(0x1f, "RR A", r8, set_a, get_a);

    opcodes[0x20] = sla!(0x20, "SLA B", r8, set_b, get_b);
    opcodes[0x21] = sla!(0x21, "SLA C", r8, set_c, get_c);
    opcodes[0x22] = sla!(0x22, "SLA D", r8, set_d, get_d);
    opcodes[0x23] = sla!(0x23, "SLA E", r8, set_e, get_e);
    opcodes[0x24] = sla!(0x24, "SLA H", r8, set_h, get_h);
    opcodes[0x25] = sla!(0x25, "SLA L", r8, set_l, get_l);
    opcodes[0x26] = sla!(0x26, "SLA [HL]", ar16, set_hl, get_hl);
    opcodes[0x27] = sla!(0x27, "SLA A", r8, set_a, get_a);

    opcodes[0x28] = sra!(0x28, "SRA B", r8, set_b, get_b);
    opcodes[0x29] = sra!(0x29, "SRA C", r8, set_c, get_c);
    opcodes[0x2A] = sra!(0x2a, "SRA D", r8, set_d, get_d);
    opcodes[0x2B] = sra!(0x2b, "SRA E", r8, set_e, get_e);
    opcodes[0x2C] = sra!(0x2c, "SRA H", r8, set_h, get_h);
    opcodes[0x2D] = sra!(0x2d, "SRA L", r8, set_l, get_l);
    opcodes[0x2E] = sra!(0x2e, "SRA [HL]", ar16, set_hl, get_hl);
    opcodes[0x2F] = sra!(0x2f, "SRA A", r8, set_a, get_a);

    opcodes[0x30] = swap!(0x30, "SWAP B", r8, set_b, get_b);
    opcodes[0x31] = swap!(0x31, "SWAP C", r8, set_c, get_c);
    opcodes[0x32] = swap!(0x32, "SWAP D", r8, set_d, get_d);
    opcodes[0x33] = swap!(0x33, "SWAP E", r8, set_e, get_e);
    opcodes[0x34] = swap!(0x34, "SWAP H", r8, set_h, get_h);
    opcodes[0x35] = swap!(0x35, "SWAP L", r8, set_l, get_l);
    opcodes[0x36] = swap!(0x36, "SWAP [HL]", ar16, set_hl, get_hl);
    opcodes[0x37] = swap!(0x37, "SWAP A", r8, set_a, get_a);

    opcodes[0x38] = srl!(0x38, "SRL B", r8, set_b, get_b);
    opcodes[0x39] = srl!(0x39, "SRL C", r8, set_c, get_c);
    opcodes[0x3A] = srl!(0x3a, "SRL D", r8, set_d, get_d);
    opcodes[0x3B] = srl!(0x3b, "SRL E", r8, set_e, get_e);
    opcodes[0x3C] = srl!(0x3c, "SRL H", r8, set_h, get_h);
    opcodes[0x3D] = srl!(0x3d, "SRL L", r8, set_l, get_l);
    opcodes[0x3E] = srl!(0x3e, "SRL [HL]", ar16, set_hl, get_hl);
    opcodes[0x3F] = srl!(0x3f, "SRL A", r8, set_a, get_a);

    opcodes[0x40] = bit!(0x40, "BIT 0, B", r8, 0, get_b);
    opcodes[0x41] = bit!(0x41, "BIT 0, C", r8, 0, get_c);
    opcodes[0x42] = bit!(0x42, "BIT 0, D", r8, 0, get_d);
    opcodes[0x43] = bit!(0x43, "BIT 0, E", r8, 0, get_e);
    opcodes[0x44] = bit!(0x44, "BIT 0, H", r8, 0, get_h);
    opcodes[0x45] = bit!(0x45, "BIT 0, L", r8, 0, get_l);
    opcodes[0x46] = bit!(0x46, "BIT 0, [HL]", ar16, 0, get_hl);
    opcodes[0x47] = bit!(0x47, "BIT 0, A", r8, 0, get_a);

    opcodes[0x48] = bit!(0x48, "BIT 1, B", r8, 1, get_b);
    opcodes[0x49] = bit!(0x49, "BIT 1, C", r8, 1, get_c);
    opcodes[0x4A] = bit!(0x4a, "BIT 1, D", r8, 1, get_d);
    opcodes[0x4B] = bit!(0x4b, "BIT 1, E", r8, 1, get_e);
    opcodes[0x4C] = bit!(0x4c, "BIT 1, H", r8, 1, get_h);
    opcodes[0x4D] = bit!(0x4d, "BIT 1, L", r8, 1, get_l);
    opcodes[0x4E] = bit!(0x4e, "BIT 1, [HL]", ar16, 1, get_hl);
    opcodes[0x4F] = bit!(0x4f, "BIT 1, A", r8, 1, get_a);

    opcodes[0x50] = bit!(0x50, "BIT 2, B", r8, 2, get_b);
    opcodes[0x51] = bit!(0x51, "BIT 2, C", r8, 2, get_c);
    opcodes[0x52] = bit!(0x52, "BIT 2, D", r8, 2, get_d);
    opcodes[0x53] = bit!(0x53, "BIT 2, E", r8, 2, get_e);
    opcodes[0x54] = bit!(0x54, "BIT 2, H", r8, 2, get_h);
    opcodes[0x55] = bit!(0x55, "BIT 2, L", r8, 2, get_l);
    opcodes[0x56] = bit!(0x56, "BIT 2, [HL]", ar16, 2, get_hl);
    opcodes[0x57] = bit!(0x57, "BIT 2, A", r8, 2, get_a);

    opcodes[0x58] = bit!(0x58, "BIT 3, B", r8, 3, get_b);
    opcodes[0x59] = bit!(0x59, "BIT 3, C", r8, 3, get_c);
    opcodes[0x5A] = bit!(0x5a, "BIT 3, D", r8, 3, get_d);
    opcodes[0x5B] = bit!(0x5b, "BIT 3, E", r8, 3, get_e);
    opcodes[0x5C] = bit!(0x5c, "BIT 3, H", r8, 3, get_h);
    opcodes[0x5D] = bit!(0x5d, "BIT 3, L", r8, 3, get_l);
    opcodes[0x5E] = bit!(0x5e, "BIT 3, [HL]", ar16, 3, get_hl);
    opcodes[0x5F] = bit!(0x5f, "BIT 3, A", r8, 3, get_a);

    opcodes[0x60] = bit!(0x60, "BIT 4, B", r8, 4, get_b);
    opcodes[0x61] = bit!(0x61, "BIT 4, C", r8, 4, get_c);
    opcodes[0x62] = bit!(0x62, "BIT 4, D", r8, 4, get_d);
    opcodes[0x63] = bit!(0x63, "BIT 4, E", r8, 4, get_e);
    opcodes[0x64] = bit!(0x64, "BIT 4, H", r8, 4, get_h);
    opcodes[0x65] = bit!(0x65, "BIT 4, L", r8, 4, get_l);
    opcodes[0x66] = bit!(0x66, "BIT 4, [HL]", ar16, 4, get_hl);
    opcodes[0x67] = bit!(0x67, "BIT 4, A", r8, 4, get_a);

    opcodes[0x68] = bit!(0x68, "BIT 5, B", r8, 5, get_b);
    opcodes[0x69] = bit!(0x69, "BIT 5, C", r8, 5, get_c);
    opcodes[0x6A] = bit!(0x6a, "BIT 5, D", r8, 5, get_d);
    opcodes[0x6B] = bit!(0x6b, "BIT 5, E", r8, 5, get_e);
    opcodes[0x6C] = bit!(0x6c, "BIT 5, H", r8, 5, get_h);
    opcodes[0x6D] = bit!(0x6d, "BIT 5, L", r8, 5, get_l);
    opcodes[0x6E] = bit!(0x6e, "BIT 5, [HL]", ar16, 5, get_hl);
    opcodes[0x6F] = bit!(0x6f, "BIT 5, A", r8, 5, get_a);

    opcodes[0x70] = bit!(0x70, "BIT 6, B", r8, 6, get_b);
    opcodes[0x71] = bit!(0x71, "BIT 6, C", r8, 6, get_c);
    opcodes[0x72] = bit!(0x72, "BIT 6, D", r8, 6, get_d);
    opcodes[0x73] = bit!(0x73, "BIT 6, E", r8, 6, get_e);
    opcodes[0x74] = bit!(0x74, "BIT 6, H", r8, 6, get_h);
    opcodes[0x75] = bit!(0x75, "BIT 6, L", r8, 6, get_l);
    opcodes[0x76] = bit!(0x76, "BIT 6, [HL]", ar16, 6, get_hl);
    opcodes[0x77] = bit!(0x77, "BIT 6, A", r8, 6, get_a);

    opcodes[0x78] = bit!(0x78, "BIT 7, B", r8, 7, get_b);
    opcodes[0x79] = bit!(0x79, "BIT 7, C", r8, 7, get_c);
    opcodes[0x7A] = bit!(0x7a, "BIT 7, D", r8, 7, get_d);
    opcodes[0x7B] = bit!(0x7b, "BIT 7, E", r8, 7, get_e);
    opcodes[0x7C] = bit!(0x7c, "BIT 7, H", r8, 7, get_h);
    opcodes[0x7D] = bit!(0x7d, "BIT 7, L", r8, 7, get_l);
    opcodes[0x7E] = bit!(0x7e, "BIT 7, [HL]", ar16, 7, get_hl);
    opcodes[0x7F] = bit!(0x7f, "BIT 7, A", r8, 7, get_a);

    opcodes[0x80] = res!(0x80, "RES 0, B", r8, 0, set_b, get_b);
    opcodes[0x81] = res!(0x81, "RES 0, C", r8, 0, set_c, get_c);
    opcodes[0x82] = res!(0x82, "RES 0, D", r8, 0, set_d, get_d);
    opcodes[0x83] = res!(0x83, "RES 0, E", r8, 0, set_e, get_e);
    opcodes[0x84] = res!(0x84, "RES 0, H", r8, 0, set_h, get_h);
    opcodes[0x85] = res!(0x85, "RES 0, L", r8, 0, set_l, get_l);
    opcodes[0x86] = res!(0x86, "RES 0, [HL]", ar16, 0, get_hl);
    opcodes[0x87] = res!(0x87, "RES 0, A", r8, 0, set_a, get_a);

    opcodes[0x88] = res!(0x88, "RES 1, B", r8, 1, set_b, get_b);
    opcodes[0x89] = res!(0x89, "RES 1, C", r8, 1, set_c, get_c);
    opcodes[0x8A] = res!(0x8a, "RES 1, D", r8, 1, set_d, get_d);
    opcodes[0x8B] = res!(0x8b, "RES 1, E", r8, 1, set_e, get_e);
    opcodes[0x8C] = res!(0x8c, "RES 1, H", r8, 1, set_h, get_h);
    opcodes[0x8D] = res!(0x8d, "RES 1, L", r8, 1, set_l, get_l);
    opcodes[0x8E] = res!(0x8e, "RES 1, [HL]", ar16, 1, get_hl);
    opcodes[0x8F] = res!(0x8f, "RES 1, A", r8, 1, set_a, get_a);

    opcodes[0x90] = res!(0x90, "RES 2, B", r8, 2, set_b, get_b);
    opcodes[0x91] = res!(0x91, "RES 2, C", r8, 2, set_c, get_c);
    opcodes[0x92] = res!(0x92, "RES 2, D", r8, 2, set_d, get_d);
    opcodes[0x93] = res!(0x93, "RES 2, E", r8, 2, set_e, get_e);
    opcodes[0x94] = res!(0x94, "RES 2, H", r8, 2, set_h, get_h);
    opcodes[0x95] = res!(0x95, "RES 2, L", r8, 2, set_l, get_l);
    opcodes[0x96] = res!(0x96, "RES 2, [HL]", ar16, 2, get_hl);
    opcodes[0x97] = res!(0x97, "RES 2, A", r8, 2, set_a, get_a);

    opcodes[0x98] = res!(0x98, "RES 3, B", r8, 3, set_b, get_b);
    opcodes[0x99] = res!(0x99, "RES 3, C", r8, 3, set_c, get_c);
    opcodes[0x9A] = res!(0x9a, "RES 3, D", r8, 3, set_d, get_d);
    opcodes[0x9B] = res!(0x9b, "RES 3, E", r8, 3, set_e, get_e);
    opcodes[0x9C] = res!(0x9c, "RES 3, H", r8, 3, set_h, get_h);
    opcodes[0x9D] = res!(0x9d, "RES 3, L", r8, 3, set_l, get_l);
    opcodes[0x9E] = res!(0x9e, "RES 3, [HL]", ar16, 3, get_hl);
    opcodes[0x9F] = res!(0x9f, "RES 3, A", r8, 3, set_a, get_a);

    opcodes[0xA0] = res!(0xA0, "RES 4, B", r8, 4, set_b, get_b);
    opcodes[0xA1] = res!(0xA1, "RES 4, C", r8, 4, set_c, get_c);
    opcodes[0xA2] = res!(0xA2, "RES 4, D", r8, 4, set_d, get_d);
    opcodes[0xA3] = res!(0xA3, "RES 4, E", r8, 4, set_e, get_e);
    opcodes[0xA4] = res!(0xA4, "RES 4, H", r8, 4, set_h, get_h);
    opcodes[0xA5] = res!(0xA5, "RES 4, L", r8, 4, set_l, get_l);
    opcodes[0xA6] = res!(0xA6, "RES 4, [HL]", ar16, 4, get_hl);
    opcodes[0xA7] = res!(0xA7, "RES 4, A", r8, 4, set_a, get_a);

    opcodes[0xA8] = res!(0xA8, "RES 5, B", r8, 5, set_b, get_b);
    opcodes[0xA9] = res!(0xA9, "RES 5, C", r8, 5, set_c, get_c);
    opcodes[0xAA] = res!(0xAa, "RES 5, D", r8, 5, set_d, get_d);
    opcodes[0xAB] = res!(0xAb, "RES 5, E", r8, 5, set_e, get_e);
    opcodes[0xAC] = res!(0xAc, "RES 5, H", r8, 5, set_h, get_h);
    opcodes[0xAD] = res!(0xAd, "RES 5, L", r8, 5, set_l, get_l);
    opcodes[0xAE] = res!(0xAe, "RES 5, [HL]", ar16, 5, get_hl);
    opcodes[0xAF] = res!(0xAf, "RES 5, A", r8, 5, set_a, get_a);

    opcodes[0xB0] = res!(0xB0, "RES 6, B", r8, 6, set_b, get_b);
    opcodes[0xB1] = res!(0xB1, "RES 6, C", r8, 6, set_c, get_c);
    opcodes[0xB2] = res!(0xB2, "RES 6, D", r8, 6, set_d, get_d);
    opcodes[0xB3] = res!(0xB3, "RES 6, E", r8, 6, set_e, get_e);
    opcodes[0xB4] = res!(0xB4, "RES 6, H", r8, 6, set_h, get_h);
    opcodes[0xB5] = res!(0xB5, "RES 6, L", r8, 6, set_l, get_l);
    opcodes[0xB6] = res!(0xB6, "RES 6, [HL]", ar16, 6, get_hl);
    opcodes[0xB7] = res!(0xB7, "RES 6, A", r8, 6, set_a, get_a);

    opcodes[0xB8] = res!(0xB8, "RES 7, B", r8, 7, set_b, get_b);
    opcodes[0xB9] = res!(0xB9, "RES 7, C", r8, 7, set_c, get_c);
    opcodes[0xBA] = res!(0xBa, "RES 7, D", r8, 7, set_d, get_d);
    opcodes[0xBB] = res!(0xBb, "RES 7, E", r8, 7, set_e, get_e);
    opcodes[0xBC] = res!(0xBc, "RES 7, H", r8, 7, set_h, get_h);
    opcodes[0xBD] = res!(0xBd, "RES 7, L", r8, 7, set_l, get_l);
    opcodes[0xBE] = res!(0xBe, "RES 7, [HL]", ar16, 7, get_hl);
    opcodes[0xBF] = res!(0xBf, "RES 7, A", r8, 7, set_a, get_a);

    opcodes[0xC0] = set!(0xC0, "SET 0, B", r8, 0, set_b, get_b);
    opcodes[0xC1] = set!(0xC1, "SET 0, C", r8, 0, set_c, get_c);
    opcodes[0xC2] = set!(0xC2, "SET 0, D", r8, 0, set_d, get_d);
    opcodes[0xC3] = set!(0xC3, "SET 0, E", r8, 0, set_e, get_e);
    opcodes[0xC4] = set!(0xC4, "SET 0, H", r8, 0, set_h, get_h);
    opcodes[0xC5] = set!(0xC5, "SET 0, L", r8, 0, set_l, get_l);
    opcodes[0xC6] = set!(0xC6, "SET 0, [HL]", ar16, 0, get_hl);
    opcodes[0xC7] = set!(0xC7, "SET 0, A", r8, 0, set_a, get_a);

    opcodes[0xC8] = set!(0xc8, "SET 1, B", r8, 1, set_b, get_b);
    opcodes[0xC9] = set!(0xc9, "SET 1, C", r8, 1, set_c, get_c);
    opcodes[0xCA] = set!(0xca, "SET 1, D", r8, 1, set_d, get_d);
    opcodes[0xCB] = set!(0xcb, "SET 1, E", r8, 1, set_e, get_e);
    opcodes[0xCC] = set!(0xcc, "SET 1, H", r8, 1, set_h, get_h);
    opcodes[0xCD] = set!(0xcd, "SET 1, L", r8, 1, set_l, get_l);
    opcodes[0xCE] = set!(0xce, "SET 1, [HL]", ar16, 1, get_hl);
    opcodes[0xCF] = set!(0xcf, "SET 1, A", r8, 1, set_a, get_a);

    opcodes[0xD0] = set!(0xd0, "SET 2, B", r8, 2, set_b, get_b);
    opcodes[0xD1] = set!(0xd1, "SET 2, C", r8, 2, set_c, get_c);
    opcodes[0xD2] = set!(0xd2, "SET 2, D", r8, 2, set_d, get_d);
    opcodes[0xD3] = set!(0xd3, "SET 2, E", r8, 2, set_e, get_e);
    opcodes[0xD4] = set!(0xd4, "SET 2, H", r8, 2, set_h, get_h);
    opcodes[0xD5] = set!(0xd5, "SET 2, L", r8, 2, set_l, get_l);
    opcodes[0xD6] = set!(0xd6, "SET 2, [HL]", ar16, 2, get_hl);
    opcodes[0xD7] = set!(0xd7, "SET 2, A", r8, 2, set_a, get_a);

    opcodes[0xD8] = set!(0xd8, "SET 3, B", r8, 3, set_b, get_b);
    opcodes[0xD9] = set!(0xd9, "SET 3, C", r8, 3, set_c, get_c);
    opcodes[0xDA] = set!(0xda, "SET 3, D", r8, 3, set_d, get_d);
    opcodes[0xDB] = set!(0xdb, "SET 3, E", r8, 3, set_e, get_e);
    opcodes[0xDC] = set!(0xdc, "SET 3, H", r8, 3, set_h, get_h);
    opcodes[0xDD] = set!(0xdd, "SET 3, L", r8, 3, set_l, get_l);
    opcodes[0xDE] = set!(0xde, "SET 3, [HL]", ar16, 3, get_hl);
    opcodes[0xDF] = set!(0xdf, "SET 3, A", r8, 3, set_a, get_a);

    opcodes[0xE0] = set!(0xe0, "SET 4, B", r8, 4, set_b, get_b);
    opcodes[0xE1] = set!(0xe1, "SET 4, C", r8, 4, set_c, get_c);
    opcodes[0xE2] = set!(0xe2, "SET 4, D", r8, 4, set_d, get_d);
    opcodes[0xE3] = set!(0xe3, "SET 4, E", r8, 4, set_e, get_e);
    opcodes[0xE4] = set!(0xe4, "SET 4, H", r8, 4, set_h, get_h);
    opcodes[0xE5] = set!(0xe5, "SET 4, L", r8, 4, set_l, get_l);
    opcodes[0xE6] = set!(0xe6, "SET 4, [HL]", ar16, 4, get_hl);
    opcodes[0xE7] = set!(0xe7, "SET 4, A", r8, 4, set_a, get_a);

    opcodes[0xE8] = set!(0xe8, "SET 5, B", r8, 5, set_b, get_b);
    opcodes[0xE9] = set!(0xe9, "SET 5, C", r8, 5, set_c, get_c);
    opcodes[0xEA] = set!(0xea, "SET 5, D", r8, 5, set_d, get_d);
    opcodes[0xEB] = set!(0xeb, "SET 5, E", r8, 5, set_e, get_e);
    opcodes[0xEC] = set!(0xec, "SET 5, H", r8, 5, set_h, get_h);
    opcodes[0xED] = set!(0xed, "SET 5, L", r8, 5, set_l, get_l);
    opcodes[0xEE] = set!(0xee, "SET 5, [HL]", ar16, 5, get_hl);
    opcodes[0xEF] = set!(0xef, "SET 5, A", r8, 5, set_a, get_a);

    opcodes[0xF0] = set!(0xF0, "SET 6, B", r8, 6, set_b, get_b);
    opcodes[0xF1] = set!(0xF1, "SET 6, C", r8, 6, set_c, get_c);
    opcodes[0xF2] = set!(0xF2, "SET 6, D", r8, 6, set_d, get_d);
    opcodes[0xF3] = set!(0xF3, "SET 6, E", r8, 6, set_e, get_e);
    opcodes[0xF4] = set!(0xF4, "SET 6, H", r8, 6, set_h, get_h);
    opcodes[0xF5] = set!(0xF5, "SET 6, L", r8, 6, set_l, get_l);
    opcodes[0xF6] = set!(0xF6, "SET 6, [HL]", ar16, 6, get_hl);
    opcodes[0xF7] = set!(0xF7, "SET 6, A", r8, 6, set_a, get_a);

    opcodes[0xF8] = set!(0xF8, "SET 7, B", r8, 7, set_b, get_b);
    opcodes[0xF9] = set!(0xF9, "SET 7, C", r8, 7, set_c, get_c);
    opcodes[0xFA] = set!(0xFa, "SET 7, D", r8, 7, set_d, get_d);
    opcodes[0xFB] = set!(0xFb, "SET 7, E", r8, 7, set_e, get_e);
    opcodes[0xFC] = set!(0xFc, "SET 7, H", r8, 7, set_h, get_h);
    opcodes[0xFD] = set!(0xFd, "SET 7, L", r8, 7, set_l, get_l);
    opcodes[0xFE] = set!(0xFe, "SET 7, [HL]", ar16, 7, get_hl);
    opcodes[0xFF] = set!(0xFf, "SET 7, A", r8, 7, set_a, get_a);
    opcodes
}

pub const OPCODES: [Option<&'static Instruction>; 256] = create_opcodes();

pub const OPCODES_CB: [Option<&'static Instruction>; 256] = create_cb_opcodes();
