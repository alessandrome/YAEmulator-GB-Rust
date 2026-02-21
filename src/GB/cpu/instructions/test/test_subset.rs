use std::rc::Rc;
use std::cell::RefCell;
use crate::GB::cpu::CPU;
use crate::GB::memory;
use crate::GB::memory::{RAM, USER_PROGRAM_ADDRESS, WRAM_ADDRESS};
use crate::GB::input::GBInput as GBInput;

macro_rules! create_memory {
    () => {
        Rc::new(RefCell::new(RAM::new(Rc::new(RefCell::new(GBInput::default())))))
    };
}

macro_rules! test_flags {
    ($cpu:ident, $zero:expr, $negative:expr, $half:expr, $carry:expr) => {
        assert_eq!($cpu.registers.get_zero_flag(), $zero);
        assert_eq!($cpu.registers.get_negative_flag(), $negative);
        assert_eq!($cpu.registers.get_half_carry_flag(), $half);
        assert_eq!($cpu.registers.get_carry_flag(), $carry);
    };
}

macro_rules! test_rlc {
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b1000_1000;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
            let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.registers.$set_reg_src(test_value_1);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b0001_0001);
            test_flags!(cpu, false, false, false, true);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b0010_0010);
            test_flags!(cpu, false, false, false, false);
        }
    };
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b1000_1000;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
            let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.write_memory(test_addr, test_value_1);
            cpu.registers.set_hl(test_addr);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b0001_0001);
            test_flags!(cpu, false, false, false, true);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b0010_0010);
            test_flags!(cpu, false, false, false, false);
        }
    };
}
macro_rules! test_rl {
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b1000_1000;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.registers.$set_reg_src(test_value_1);
            cpu.registers.set_carry_flag(false);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b0001_0000);
            test_flags!(cpu, false, false, false, true);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b0010_0001);
            test_flags!(cpu, false, false, false, false);
        }
    };
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b1000_1000;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.write_memory(test_addr, test_value_1);
            cpu.registers.set_hl(test_addr);
            cpu.registers.set_carry_flag(false);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b0001_0000);
            test_flags!(cpu, false, false, false, true);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b0010_0001);
            test_flags!(cpu, false, false, false, false);
        }
    };
}
macro_rules! test_sla {
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b1000_1000;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.registers.$set_reg_src(test_value_1);
            cpu.registers.set_carry_flag(false);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b0001_0000);
            test_flags!(cpu, false, false, false, true);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b0010_0000);
            test_flags!(cpu, false, false, false, false);
        }
    };
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b1000_1000;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.write_memory(test_addr, test_value_1);
            cpu.registers.set_hl(test_addr);
            cpu.registers.set_carry_flag(false);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b0001_0000);
            test_flags!(cpu, false, false, false, true);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b0010_0000);
            test_flags!(cpu, false, false, false, false);
        }
    };
}

macro_rules! test_rrc {
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b0001_0001;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.registers.$set_reg_src(test_value_1);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b1000_1000);
            test_flags!(cpu, false, false, false, true);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b0100_0100);
            test_flags!(cpu, false, false, false, false);
        }
    };
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b0001_0001;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.write_memory(test_addr, test_value_1);
            cpu.registers.set_hl(test_addr);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b1000_1000);
            test_flags!(cpu, false, false, false, true);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b0100_0100);
            test_flags!(cpu, false, false, false, false);
        }
    };
}
macro_rules! test_rr {
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b0001_0001;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.registers.$set_reg_src(test_value_1);
            cpu.registers.set_carry_flag(false);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b0000_1000);
            test_flags!(cpu, false, false, false, true);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b1000_0100);
            test_flags!(cpu, false, false, false, false);
        }
    };
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b0001_0001;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.write_memory(test_addr, test_value_1);
            cpu.registers.set_hl(test_addr);
            cpu.registers.set_carry_flag(false);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b0000_1000);
            test_flags!(cpu, false, false, false, true);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b1000_0100);
            test_flags!(cpu, false, false, false, false);
        }
    };
}
macro_rules! test_sra {
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b1001_0001;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.registers.$set_reg_src(test_value_1);
            cpu.registers.set_carry_flag(false);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b1100_1000);
            test_flags!(cpu, false, false, false, true);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b1110_0100);
            test_flags!(cpu, false, false, false, false);
        }
    };
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b1001_0001;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.write_memory(test_addr, test_value_1);
            cpu.registers.set_hl(test_addr);
            cpu.registers.set_carry_flag(false);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b1100_1000);
            test_flags!(cpu, false, false, false, true);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b1110_0100);
            test_flags!(cpu, false, false, false, false);
        }
    };
}
macro_rules! test_srl {
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b0001_0001;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.registers.$set_reg_src(test_value_1);
            cpu.registers.set_carry_flag(false);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b0000_1000);
            test_flags!(cpu, false, false, false, true);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b0000_0100);
            test_flags!(cpu, false, false, false, false);
        }
    };
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b0001_0001;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.write_memory(test_addr, test_value_1);
            cpu.registers.set_hl(test_addr);
            cpu.registers.set_carry_flag(false);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b0000_1000);
            test_flags!(cpu, false, false, false, true);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b0000_0100);
            test_flags!(cpu, false, false, false, false);
        }
    };
}

macro_rules! test_swap {
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b0001_1000;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.registers.$set_reg_src(test_value_1);
            cpu.registers.set_carry_flag(false);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), 0b1000_0001);
            test_flags!(cpu, false, false, false, false);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), test_value_1);
            test_flags!(cpu, false, false, false, false);
        }
    };
    ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0b0001_1000;
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.write_memory(test_addr, test_value_1);
            cpu.registers.set_hl(test_addr);
            cpu.registers.set_carry_flag(false);
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), 0b1000_0001);
            test_flags!(cpu, false, false, false, false);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), test_value_1);
            test_flags!(cpu, false, false, false, false);
        }
    };
}

macro_rules! test_bit {
    ($opcode:expr, $func:ident, $bit:expr, $set_reg_src:ident, $get_reg_src:ident) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = !(1 << $bit);
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.registers.$set_reg_src(test_value_1);
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            let registers_copy = cpu.registers;
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), test_value_1);
            test_flags!(cpu, true, false, true, registers_copy.get_carry_flag());
            cpu.registers.$set_reg_src(!test_value_1);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), !test_value_1);
            test_flags!(cpu, false, false, true, registers_copy.get_carry_flag());
        }
    };
    ($opcode:expr, $func:ident, $bit:expr, $set_reg_src:ident, $get_reg_src:ident, memory) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = !(1 << $bit);
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
            cpu.load(&program_1);
            cpu.write_memory(test_addr, test_value_1);
            cpu.registers.$set_reg_src(test_addr);
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            let registers_copy = cpu.registers;
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), test_value_1);
            assert_eq!(cpu.registers.$get_reg_src(), test_addr);
            test_flags!(cpu, true, false, true, registers_copy.get_carry_flag());
            cpu.write_memory(test_addr, !test_value_1);
            cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), !test_value_1);
            assert_eq!(cpu.registers.$get_reg_src(), test_addr);
            test_flags!(cpu, false, false, true, registers_copy.get_carry_flag());
        }
    };
}

macro_rules! test_res {
    ($opcode:expr, $func:ident, $bit:expr, $set_reg_src:ident, $get_reg_src:ident) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0xFF;
            let test_mask: u8 = !(1 << $bit);
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode];
            cpu.load(&program_1);
            cpu.registers.$set_reg_src(test_value_1);
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            let registers_copy = cpu.registers;
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), test_value_1 & test_mask);
            test_flags!(
                cpu,
                registers_copy.get_zero_flag(),
                registers_copy.get_negative_flag(),
                registers_copy.get_half_carry_flag(),
                registers_copy.get_carry_flag());
        }
    };
    ($opcode:expr, $func:ident, $bit:expr, $set_reg_src:ident, $get_reg_src:ident, memory) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0xFF;
            let test_mask: u8 = !(1 << $bit);
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode];
            cpu.load(&program_1);
            cpu.write_memory(test_addr, test_value_1);
            cpu.registers.$set_reg_src(test_addr);
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            let registers_copy = cpu.registers;
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), test_value_1 & test_mask);
            assert_eq!(cpu.registers.$get_reg_src(), test_addr);
            test_flags!(
                cpu,
                registers_copy.get_zero_flag(),
                registers_copy.get_negative_flag(),
                registers_copy.get_half_carry_flag(),
                registers_copy.get_carry_flag());
        }
    };
}

macro_rules! test_set {
    ($opcode:expr, $func:ident, $bit:expr, $set_reg_src:ident, $get_reg_src:ident) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0x0;
            let test_mask: u8 = (1 << $bit);
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode];
            cpu.load(&program_1);
            cpu.registers.$set_reg_src(test_value_1);
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            let registers_copy = cpu.registers;
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 2);
            assert_eq!(cpu.registers.$get_reg_src(), test_value_1 | test_mask);
            test_flags!(
                cpu,
                registers_copy.get_zero_flag(),
                registers_copy.get_negative_flag(),
                registers_copy.get_half_carry_flag(),
                registers_copy.get_carry_flag());
        }
    };
    ($opcode:expr, $func:ident, $bit:expr, $set_reg_src:ident, $get_reg_src:ident, memory) => {
        #[test]
        fn $func() {
            let test_value_1: u8 = 0x0;
            let test_mask: u8 = (1 << $bit);
            let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
            let mut memory_ref = create_memory!();
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
            let program_1: Vec<u8> = vec![0xCB, $opcode];
            cpu.load(&program_1);
            cpu.write_memory(test_addr, test_value_1);
            cpu.registers.$set_reg_src(test_addr);
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            let registers_copy = cpu.registers;
            let mut cycles = cpu.execute_next();
            assert_eq!(cycles, 4);
            assert_eq!(cpu.read_memory(test_addr), test_value_1 | test_mask);
            assert_eq!(cpu.registers.$get_reg_src(), test_addr);
            test_flags!(
                cpu,
                registers_copy.get_zero_flag(),
                registers_copy.get_negative_flag(),
                registers_copy.get_half_carry_flag(),
                registers_copy.get_carry_flag());
        }
    };
}

test_rlc!(0x00, test_0x00_rlc_b, set_b, get_b);
test_rlc!(0x01, test_0x01_rlc_c, set_c, get_c);
test_rlc!(0x02, test_0x02_rlc_d, set_d, get_d);
test_rlc!(0x03, test_0x03_rlc_e, set_e, get_e);
test_rlc!(0x04, test_0x04_rlc_h, set_h, get_h);
test_rlc!(0x05, test_0x05_rlc_l, set_l, get_l);
test_rlc!(0x06, test_0x06_rlc__hl_, set_hl, get_hl, memory);
test_rlc!(0x07, test_0x07_rlc_a, set_a, get_a);

test_rrc!(0x08, test_0x08_rrc_b, set_b, get_b);
test_rrc!(0x09, test_0x09_rrc_c, set_c, get_c);
test_rrc!(0x0A, test_0x0a_rrc_d, set_d, get_d);
test_rrc!(0x0B, test_0x0b_rrc_e, set_e, get_e);
test_rrc!(0x0C, test_0x0c_rrc_h, set_h, get_h);
test_rrc!(0x0D, test_0x0d_rrc_l, set_l, get_l);
test_rrc!(0x0E, test_0x0e_rrc__hl_, set_hl, get_hl, memory);
test_rrc!(0x0F, test_0x0f_rrc_a, set_a, get_a);

test_rl!(0x10, test_0x10_rl_b, set_b, get_b);
test_rl!(0x11, test_0x11_rl_c, set_c, get_c);
test_rl!(0x12, test_0x12_rl_d, set_d, get_d);
test_rl!(0x13, test_0x13_rl_e, set_e, get_e);
test_rl!(0x14, test_0x14_rl_h, set_h, get_h);
test_rl!(0x15, test_0x15_rl_l, set_l, get_l);
test_rl!(0x16, test_0x16_rl__hl_, set_hl, get_hl, memory);
test_rl!(0x17, test_0x17_rl_a, set_a, get_a);

test_rr!(0x18, test_0x18_rr_b, set_b, get_b);
test_rr!(0x19, test_0x19_rr_c, set_c, get_c);
test_rr!(0x1A, test_0x1a_rr_d, set_d, get_d);
test_rr!(0x1B, test_0x1b_rr_e, set_e, get_e);
test_rr!(0x1C, test_0x1c_rr_h, set_h, get_h);
test_rr!(0x1D, test_0x1d_rr_l, set_l, get_l);
test_rr!(0x1E, test_0x1e_rr__hl_, set_hl, get_hl, memory);
test_rr!(0x1F, test_0x1f_rr_a, set_a, get_a);

test_sla!(0x20, test_0x20_sla_b, set_b, get_b);
test_sla!(0x21, test_0x21_sla_c, set_c, get_c);
test_sla!(0x22, test_0x22_sla_d, set_d, get_d);
test_sla!(0x23, test_0x23_sla_e, set_e, get_e);
test_sla!(0x24, test_0x24_sla_h, set_h, get_h);
test_sla!(0x25, test_0x25_sla_l, set_l, get_l);
test_sla!(0x26, test_0x26_sla__hl_, set_hl, get_hl, memory);
test_sla!(0x27, test_0x27_sla_a, set_a, get_a);

test_sra!(0x28, test_0x28_sra_b, set_b, get_b);
test_sra!(0x29, test_0x29_sra_c, set_c, get_c);
test_sra!(0x2A, test_0x2a_sra_d, set_d, get_d);
test_sra!(0x2B, test_0x2b_sra_e, set_e, get_e);
test_sra!(0x2C, test_0x2c_sra_h, set_h, get_h);
test_sra!(0x2D, test_0x2d_sra_l, set_l, get_l);
test_sra!(0x2E, test_0x2e_sra__hl_, set_hl, get_hl, memory);
test_sra!(0x2F, test_0x2f_sra_a, set_a, get_a);

test_swap!(0x30, test_0x30_swap_b, set_b, get_b);
test_swap!(0x31, test_0x31_swap_c, set_c, get_c);
test_swap!(0x32, test_0x32_swap_d, set_d, get_d);
test_swap!(0x33, test_0x33_swap_e, set_e, get_e);
test_swap!(0x34, test_0x34_swap_h, set_h, get_h);
test_swap!(0x35, test_0x35_swap_l, set_l, get_l);
test_swap!(0x36, test_0x36_swap__hl_, set_hl, get_hl, memory);
test_swap!(0x37, test_0x37_swap_a, set_a, get_a);

test_srl!(0x38, test_0x38_srl_b, set_b, get_b);
test_srl!(0x39, test_0x39_srl_c, set_c, get_c);
test_srl!(0x3A, test_0x3a_srl_d, set_d, get_d);
test_srl!(0x3B, test_0x3b_srl_e, set_e, get_e);
test_srl!(0x3C, test_0x3c_srl_h, set_h, get_h);
test_srl!(0x3D, test_0x3d_srl_l, set_l, get_l);
test_srl!(0x3E, test_0x3e_srl__hl_, set_hl, get_hl, memory);
test_srl!(0x3F, test_0x3f_srl_a, set_a, get_a);

// BIT Instructions tests
test_bit!(0x40, test_0x40_bit_0_b, 0, set_b, get_b);
test_bit!(0x41, test_0x41_bit_0_c, 0, set_c, get_c);
test_bit!(0x42, test_0x42_bit_0_d, 0, set_d, get_d);
test_bit!(0x43, test_0x43_bit_0_e, 0, set_e, get_e);
test_bit!(0x44, test_0x44_bit_0_h, 0, set_h, get_h);
test_bit!(0x45, test_0x45_bit_0_l, 0, set_l, get_l);
test_bit!(0x46, test_0x46_bit_0__hl_, 0, set_hl, get_hl, memory);
test_bit!(0x47, test_0x47_bit_0_a, 0, set_a, get_a);

test_bit!(0x48, test_0x48_bit_1_b, 1, set_b, get_b);
test_bit!(0x49, test_0x49_bit_1_c, 1, set_c, get_c);
test_bit!(0x4A, test_0x4a_bit_1_d, 1, set_d, get_d);
test_bit!(0x4B, test_0x4b_bit_1_e, 1, set_e, get_e);
test_bit!(0x4C, test_0x4c_bit_1_h, 1, set_h, get_h);
test_bit!(0x4D, test_0x4d_bit_1_l, 1, set_l, get_l);
test_bit!(0x4E, test_0x4e_bit_1__hl_, 1, set_hl, get_hl, memory);
test_bit!(0x4F, test_0x4f_bit_1_a, 1, set_a, get_a);

test_bit!(0x50, test_0x50_bit_2_b, 2, set_b, get_b);
test_bit!(0x51, test_0x51_bit_2_c, 2, set_c, get_c);
test_bit!(0x52, test_0x52_bit_2_d, 2, set_d, get_d);
test_bit!(0x53, test_0x53_bit_2_e, 2, set_e, get_e);
test_bit!(0x54, test_0x54_bit_2_h, 2, set_h, get_h);
test_bit!(0x55, test_0x55_bit_2_l, 2, set_l, get_l);
test_bit!(0x56, test_0x56_bit_2__hl_, 2, set_hl, get_hl, memory);
test_bit!(0x57, test_0x57_bit_2_a, 2, set_a, get_a);

test_bit!(0x58, test_0x58_bit_3_b, 3, set_b, get_b);
test_bit!(0x59, test_0x59_bit_3_c, 3, set_c, get_c);
test_bit!(0x5A, test_0x5a_bit_3_d, 3, set_d, get_d);
test_bit!(0x5B, test_0x5b_bit_3_e, 3, set_e, get_e);
test_bit!(0x5C, test_0x5c_bit_3_h, 3, set_h, get_h);
test_bit!(0x5D, test_0x5d_bit_3_l, 3, set_l, get_l);
test_bit!(0x5E, test_0x5e_bit_3__hl_, 3, set_hl, get_hl, memory);
test_bit!(0x5F, test_0x5f_bit_3_a, 3, set_a, get_a);

test_bit!(0x60, test_0x60_bit_4_b, 4, set_b, get_b);
test_bit!(0x61, test_0x61_bit_4_c, 4, set_c, get_c);
test_bit!(0x62, test_0x62_bit_4_d, 4, set_d, get_d);
test_bit!(0x63, test_0x63_bit_4_e, 4, set_e, get_e);
test_bit!(0x64, test_0x64_bit_4_h, 4, set_h, get_h);
test_bit!(0x65, test_0x65_bit_4_l, 4, set_l, get_l);
test_bit!(0x66, test_0x66_bit_4__hl_, 4, set_hl, get_hl, memory);
test_bit!(0x67, test_0x67_bit_4_a, 4, set_a, get_a);

test_bit!(0x68, test_0x68_bit_5_b, 5, set_b, get_b);
test_bit!(0x69, test_0x69_bit_5_c, 5, set_c, get_c);
test_bit!(0x6A, test_0x6a_bit_5_d, 5, set_d, get_d);
test_bit!(0x6B, test_0x6b_bit_5_e, 5, set_e, get_e);
test_bit!(0x6C, test_0x6c_bit_5_h, 5, set_h, get_h);
test_bit!(0x6D, test_0x6d_bit_5_l, 5, set_l, get_l);
test_bit!(0x6E, test_0x6e_bit_5__hl_, 5, set_hl, get_hl, memory);
test_bit!(0x6F, test_0x6f_bit_5_a, 5, set_a, get_a);

test_bit!(0x70, test_0x70_bit_6_b, 6, set_b, get_b);
test_bit!(0x71, test_0x71_bit_6_c, 6, set_c, get_c);
test_bit!(0x72, test_0x72_bit_6_d, 6, set_d, get_d);
test_bit!(0x73, test_0x73_bit_6_e, 6, set_e, get_e);
test_bit!(0x74, test_0x74_bit_6_h, 6, set_h, get_h);
test_bit!(0x75, test_0x75_bit_6_l, 6, set_l, get_l);
test_bit!(0x76, test_0x76_bit_6__hl_, 6, set_hl, get_hl, memory);
test_bit!(0x77, test_0x77_bit_6_a, 6, set_a, get_a);

test_bit!(0x78, test_0x78_bit_7_b, 7, set_b, get_b);
test_bit!(0x79, test_0x79_bit_7_c, 7, set_c, get_c);
test_bit!(0x7A, test_0x7a_bit_7_d, 7, set_d, get_d);
test_bit!(0x7B, test_0x7b_bit_7_e, 7, set_e, get_e);
test_bit!(0x7C, test_0x7c_bit_7_h, 7, set_h, get_h);
test_bit!(0x7D, test_0x7d_bit_7_l, 7, set_l, get_l);
test_bit!(0x7E, test_0x7e_bit_7__hl_, 7, set_hl, get_hl, memory);
test_bit!(0x7F, test_0x7f_bit_7_a, 7, set_a, get_a);

// RES instruction tests
test_res!(0x80, test_0x80_res_0_b, 0, set_b, get_b);
test_res!(0x81, test_0x81_res_0_c, 0, set_c, get_c);
test_res!(0x82, test_0x82_res_0_d, 0, set_d, get_d);
test_res!(0x83, test_0x83_res_0_e, 0, set_e, get_e);
test_res!(0x84, test_0x84_res_0_h, 0, set_h, get_h);
test_res!(0x85, test_0x85_res_0_l, 0, set_l, get_l);
test_res!(0x86, test_0x86_res_0__hl_, 0, set_hl, get_hl, memory);
test_res!(0x87, test_0x87_res_0_a, 0, set_a, get_a);

test_res!(0x88, test_0x88_res_1_b, 1, set_b, get_b);
test_res!(0x89, test_0x89_res_1_c, 1, set_c, get_c);
test_res!(0x8A, test_0x8a_res_1_d, 1, set_d, get_d);
test_res!(0x8B, test_0x8b_res_1_e, 1, set_e, get_e);
test_res!(0x8C, test_0x8c_res_1_h, 1, set_h, get_h);
test_res!(0x8D, test_0x8d_res_1_l, 1, set_l, get_l);
test_res!(0x8E, test_0x8e_res_1__hl_, 1, set_hl, get_hl, memory);
test_res!(0x8F, test_0x8f_res_1_a, 1, set_a, get_a);

test_res!(0x90, test_0x90_res_2_b, 2, set_b, get_b);
test_res!(0x91, test_0x91_res_2_c, 2, set_c, get_c);
test_res!(0x92, test_0x92_res_2_d, 2, set_d, get_d);
test_res!(0x93, test_0x93_res_2_e, 2, set_e, get_e);
test_res!(0x94, test_0x94_res_2_h, 2, set_h, get_h);
test_res!(0x95, test_0x95_res_2_l, 2, set_l, get_l);
test_res!(0x96, test_0x96_res_2__hl_, 2, set_hl, get_hl, memory);
test_res!(0x97, test_0x97_res_2_a, 2, set_a, get_a);

test_res!(0x98, test_0x98_res_3_b, 3, set_b, get_b);
test_res!(0x99, test_0x99_res_3_c, 3, set_c, get_c);
test_res!(0x9A, test_0x9a_res_3_d, 3, set_d, get_d);
test_res!(0x9B, test_0x9b_res_3_e, 3, set_e, get_e);
test_res!(0x9C, test_0x9c_res_3_h, 3, set_h, get_h);
test_res!(0x9D, test_0x9d_res_3_l, 3, set_l, get_l);
test_res!(0x9E, test_0x9e_res_3__hl_, 3, set_hl, get_hl, memory);
test_res!(0x9F, test_0x9f_res_3_a, 3, set_a, get_a);

test_res!(0xA0, test_0xa0_res_4_b, 4, set_b, get_b);
test_res!(0xA1, test_0xa1_res_4_c, 4, set_c, get_c);
test_res!(0xA2, test_0xa2_res_4_d, 4, set_d, get_d);
test_res!(0xA3, test_0xa3_res_4_e, 4, set_e, get_e);
test_res!(0xA4, test_0xa4_res_4_h, 4, set_h, get_h);
test_res!(0xA5, test_0xa5_res_4_l, 4, set_l, get_l);
test_res!(0xA6, test_0xa6_res_4__hl_, 4, set_hl, get_hl, memory);
test_res!(0xA7, test_0xa7_res_4_a, 4, set_a, get_a);

test_res!(0xA8, test_0xa8_res_5_b, 5, set_b, get_b);
test_res!(0xA9, test_0xa9_res_5_c, 5, set_c, get_c);
test_res!(0xAA, test_0xaa_res_5_d, 5, set_d, get_d);
test_res!(0xAB, test_0xab_res_5_e, 5, set_e, get_e);
test_res!(0xAC, test_0xac_res_5_h, 5, set_h, get_h);
test_res!(0xAD, test_0xad_res_5_l, 5, set_l, get_l);
test_res!(0xAE, test_0xae_res_5__hl_, 5, set_hl, get_hl, memory);
test_res!(0xAF, test_0xaf_res_5_a, 5, set_a, get_a);

test_res!(0xB0, test_0xb0_res_6_b, 6, set_b, get_b);
test_res!(0xB1, test_0xb1_res_6_c, 6, set_c, get_c);
test_res!(0xB2, test_0xb2_res_6_d, 6, set_d, get_d);
test_res!(0xB3, test_0xb3_res_6_e, 6, set_e, get_e);
test_res!(0xB4, test_0xb4_res_6_h, 6, set_h, get_h);
test_res!(0xB5, test_0xb5_res_6_l, 6, set_l, get_l);
test_res!(0xB6, test_0xb6_res_6__hl_, 6, set_hl, get_hl, memory);
test_res!(0xB7, test_0xb7_res_6_a, 6, set_a, get_a);

test_res!(0xB8, test_0xb8_res_7_b, 7, set_b, get_b);
test_res!(0xB9, test_0xb9_res_7_c, 7, set_c, get_c);
test_res!(0xBA, test_0xba_res_7_d, 7, set_d, get_d);
test_res!(0xBB, test_0xbb_res_7_e, 7, set_e, get_e);
test_res!(0xBC, test_0xbc_res_7_h, 7, set_h, get_h);
test_res!(0xBD, test_0xbd_res_7_l, 7, set_l, get_l);
test_res!(0xBE, test_0xbe_res_7__hl_, 7, set_hl, get_hl, memory);
test_res!(0xBF, test_0xbf_res_7_a, 7, set_a, get_a);

// SET instruction tests
test_set!(0xC0, test_0xc0_set_0_b, 0, set_b, get_b);
test_set!(0xC1, test_0xc1_set_0_c, 0, set_c, get_c);
test_set!(0xC2, test_0xc2_set_0_d, 0, set_d, get_d);
test_set!(0xC3, test_0xc3_set_0_e, 0, set_e, get_e);
test_set!(0xC4, test_0xc4_set_0_h, 0, set_h, get_h);
test_set!(0xC5, test_0xc5_set_0_l, 0, set_l, get_l);
test_set!(0xC6, test_0xc6_set_0__hl_, 0, set_hl, get_hl, memory);
test_set!(0xC7, test_0xc7_set_0_a, 0, set_a, get_a);

test_set!(0xC8, test_0xc8_set_1_b, 1, set_b, get_b);
test_set!(0xC9, test_0xc9_set_1_c, 1, set_c, get_c);
test_set!(0xCA, test_0xca_set_1_d, 1, set_d, get_d);
test_set!(0xCB, test_0xcb_set_1_e, 1, set_e, get_e);
test_set!(0xCC, test_0xcc_set_1_h, 1, set_h, get_h);
test_set!(0xCD, test_0xcd_set_1_l, 1, set_l, get_l);
test_set!(0xCE, test_0xce_set_1__hl_, 1, set_hl, get_hl, memory);
test_set!(0xCF, test_0xcf_set_1_a, 1, set_a, get_a);

test_set!(0xD0, test_0xd0_set_2_b, 2, set_b, get_b);
test_set!(0xD1, test_0xd1_set_2_c, 2, set_c, get_c);
test_set!(0xD2, test_0xd2_set_2_d, 2, set_d, get_d);
test_set!(0xD3, test_0xd3_set_2_e, 2, set_e, get_e);
test_set!(0xD4, test_0xd4_set_2_h, 2, set_h, get_h);
test_set!(0xD5, test_0xd5_set_2_l, 2, set_l, get_l);
test_set!(0xD6, test_0xd6_set_2__hl_, 2, set_hl, get_hl, memory);
test_set!(0xD7, test_0xd7_set_2_a, 2, set_a, get_a);

test_set!(0xD8, test_0xd8_set_3_b, 3, set_b, get_b);
test_set!(0xD9, test_0xd9_set_3_c, 3, set_c, get_c);
test_set!(0xDA, test_0xda_set_3_d, 3, set_d, get_d);
test_set!(0xDB, test_0xdb_set_3_e, 3, set_e, get_e);
test_set!(0xDC, test_0xdc_set_3_h, 3, set_h, get_h);
test_set!(0xDD, test_0xdd_set_3_l, 3, set_l, get_l);
test_set!(0xDE, test_0xde_set_3__hl_, 3, set_hl, get_hl, memory);
test_set!(0xDF, test_0xdf_set_3_a, 3, set_a, get_a);

test_set!(0xE0, test_0xe0_set_4_b, 4, set_b, get_b);
test_set!(0xE1, test_0xe1_set_4_c, 4, set_c, get_c);
test_set!(0xE2, test_0xe2_set_4_d, 4, set_d, get_d);
test_set!(0xE3, test_0xe3_set_4_e, 4, set_e, get_e);
test_set!(0xE4, test_0xe4_set_4_h, 4, set_h, get_h);
test_set!(0xE5, test_0xe5_set_4_l, 4, set_l, get_l);
test_set!(0xE6, test_0xe6_set_4__hl_, 4, set_hl, get_hl, memory);
test_set!(0xE7, test_0xe7_set_4_a, 4, set_a, get_a);

test_set!(0xE8, test_0xe8_set_5_b, 5, set_b, get_b);
test_set!(0xE9, test_0xe9_set_5_c, 5, set_c, get_c);
test_set!(0xEA, test_0xea_set_5_d, 5, set_d, get_d);
test_set!(0xEB, test_0xeb_set_5_e, 5, set_e, get_e);
test_set!(0xEC, test_0xec_set_5_h, 5, set_h, get_h);
test_set!(0xED, test_0xed_set_5_l, 5, set_l, get_l);
test_set!(0xEE, test_0xee_set_5__hl_, 5, set_hl, get_hl, memory);
test_set!(0xEF, test_0xef_set_5_a, 5, set_a, get_a);

test_set!(0xF0, test_0xf0_set_6_b, 6, set_b, get_b);
test_set!(0xF1, test_0xf1_set_6_c, 6, set_c, get_c);
test_set!(0xF2, test_0xf2_set_6_d, 6, set_d, get_d);
test_set!(0xF3, test_0xf3_set_6_e, 6, set_e, get_e);
test_set!(0xF4, test_0xf4_set_6_h, 6, set_h, get_h);
test_set!(0xF5, test_0xf5_set_6_l, 6, set_l, get_l);
test_set!(0xF6, test_0xf6_set_6__hl_, 6, set_hl, get_hl, memory);
test_set!(0xF7, test_0xf7_set_6_a, 6, set_a, get_a);

test_set!(0xF8, test_0xf8_set_7_b, 7, set_b, get_b);
test_set!(0xF9, test_0xf9_set_7_c, 7, set_c, get_c);
test_set!(0xFA, test_0xfa_set_7_d, 7, set_d, get_d);
test_set!(0xFB, test_0xfb_set_7_e, 7, set_e, get_e);
test_set!(0xFC, test_0xfc_set_7_h, 7, set_h, get_h);
test_set!(0xFD, test_0xfd_set_7_l, 7, set_l, get_l);
test_set!(0xFE, test_0xfe_set_7__hl_, 7, set_hl, get_hl, memory);
test_set!(0xFF, test_0xff_set_7_a, 7, set_a, get_a);
