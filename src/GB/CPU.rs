mod instructions;
pub mod registers;
pub mod cpu_mmio;

use crate::GB::bus::{Bus, MmioContext, BusDevice};
use crate::GB::types::{address::Address, Byte};
use crate::GB::CPU::instructions::microcode::{CheckCondition, MicroFlow};
use crate::GB::CPU::instructions::{Instruction, InstructionMicroOpIndex};
use crate::GB::CPU::registers::core_registers::Flags;
use crate::GB::{bus, GB};
use instructions::microcode::{AluOp, MCycleOp, MicroOp};
use registers::{core_registers::Registers, interrupt_registers::InterruptRegisters};
use crate::GB::CPU::cpu_mmio::CpuMmio;
use crate::GB::interrupt::Interrupt;
use crate::GB::memory::hram::HRAM;

pub const DIVIDER_FREQUENCY: u64 = 16384; // Divider Update Frequency in Hz
pub const CPU_INTERRUPT_CYCLES: u64 = 5; // Number of cycle to manage a requested Interrupt

#[cfg(test)]
mod test {
    use crate::GB::memory::{RAM, WRAM_ADDRESS, WRAM_SIZE};
    use crate::GB::CPU::CPU;

    #[test]
    fn cpu_new_8bit_registers() {
        let cpu = CPU::new();
        assert_eq!(cpu.registers.get_a(), 0);
        assert_eq!(cpu.registers.get_f(), 0);
        assert_eq!(cpu.registers.get_b(), 0);
        assert_eq!(cpu.registers.get_c(), 0);
        assert_eq!(cpu.registers.get_d(), 0);
        assert_eq!(cpu.registers.get_e(), 0);
        assert_eq!(cpu.registers.get_h(), 0);
        assert_eq!(cpu.registers.get_l(), 0);
    }

    #[test]
    fn cpu_new_16bit_registers() {
        let cpu = CPU::new();
        assert_eq!(cpu.registers.get_af(), 0);
        assert_eq!(cpu.registers.get_bc(), 0);
        assert_eq!(cpu.registers.get_de(), 0);
        assert_eq!(cpu.registers.get_hl(), 0);
        assert_eq!(
            cpu.registers.get_sp(),
            (WRAM_ADDRESS + WRAM_SIZE - 1) as u16
        );
        assert_eq!(cpu.registers.get_pc(), 0);
    }

    #[test]
    fn cpu_new_16_8bit_registers() {
        // 16 Bit register should be 0 as the compound of low register is 0 (and should not be altered by access of 8bit register)
        let cpu = CPU::new();
        assert_eq!(cpu.registers.get_a(), 0);
        assert_eq!(cpu.registers.get_f(), 0);
        assert_eq!(cpu.registers.get_b(), 0);
        assert_eq!(cpu.registers.get_c(), 0);
        assert_eq!(cpu.registers.get_d(), 0);
        assert_eq!(cpu.registers.get_e(), 0);
        assert_eq!(cpu.registers.get_h(), 0);
        assert_eq!(cpu.registers.get_l(), 0);
        assert_eq!(cpu.registers.get_af(), 0);
        assert_eq!(cpu.registers.get_bc(), 0);
        assert_eq!(cpu.registers.get_de(), 0);
        assert_eq!(cpu.registers.get_hl(), 0);
        assert_eq!(
            cpu.registers.get_sp(),
            (WRAM_ADDRESS + WRAM_SIZE - 1) as u16
        );
        assert_eq!(cpu.registers.get_pc(), 0);
    }

    #[test]
    fn cpu_push_n_pop() {
        let mut cpu = CPU::new();
        let start_sp = cpu.registers.get_sp();
        let test_value: u8 = 0x81;
        cpu.push(test_value);
        assert_eq!(cpu.registers.get_sp(), start_sp - 1);
        assert_eq!(cpu.read_memory(start_sp), test_value);

        let popped_val = cpu.pop();
        assert_eq!(cpu.registers.get_sp(), start_sp);
        assert_eq!(popped_val, test_value);
    }
}

#[derive(Debug, Copy, Clone)]
enum CpuStatus {
    Execute,
    Ready,
}

pub struct CPU {
    pub registers: Registers,
    pub ime: bool,  // Interrupt Master Enable - True if you want to enable and intercept interrupts
    pub opcode: u8, // Running Instruction Opcode - Known as IR (Instruction Register),
    pub instruction: Option<&'static Instruction>, // Instruction microcode to execute
    pub micro_code: MCycleOp, // Instruction microcode to execute
    pub micro_code_index: usize, // Index of Instruction's MicroOp
    pub micro_code_t_cycle: u8, // T-Cycles counting of a M-Cycle during instruction execution
}

impl CPU {
    pub const CPU_FREQUENCY_CLOCK: u32 = GB::SYSTEM_FREQUENCY_CLOCK / 4;

    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            ime: false,
            opcode: 0,
            instruction: None,
            micro_code: MCycleOp::None,
            micro_code_index: 0,
            micro_code_t_cycle: 0,
        }
    }

    /**
    Step 1 T-Cycle (4 T-Cycle = 1 M-Cycle)
    */
    pub fn tick(&mut self, bus: &mut bus::Bus, ctx: &mut bus::MmioContext) {
        self.micro_code_t_cycle = (self.micro_code_t_cycle + 1) & 0b0000_0011; // Just a Bit version of (value = value % 4)
        let cpu_status;
        if self.micro_code_t_cycle == 0 {
            cpu_status = self.m_cycle_tick(bus, ctx, self.micro_code);
        } else {
            cpu_status = CpuStatus::Execute;
        }

        match cpu_status {
            CpuStatus::Execute => { /* Wait 'till execution complete */ }
            CpuStatus::Ready => {
                let interrupt = self.interrupt(ctx.cpu_mmio.interrupt_registers());
                if interrupt.is_none() {
                    self.fetch_and_decode(bus, ctx, false);
                } else {
                    self.instruction = interrupt;
                    self.micro_code_index = 0;
                    self.micro_code = self.instruction.unwrap().micro_ops[0];
                }
            }
        }
        todo!()
    }

    pub fn fetch_next(&mut self, bus: &bus::Bus, ctx: &mut bus::MmioContext) -> Byte {
        let addr = self.registers.get_and_inc_pc();
        bus.read(ctx, Address(addr))
    }

    pub fn decode(opcode: u8, cb_optable: bool) -> Option<&'static Instruction> {
        let opcode_usize = opcode as usize;
        if cb_optable {
            return instructions::OPCODES_CB[opcode_usize];
        }
        instructions::OPCODES[opcode_usize]
    }

    pub fn fetch_and_decode(&mut self, bus: &bus::Bus, ctx: &mut bus::MmioContext, cb_optable: bool) -> (Option<&'static Instruction>, Byte) {
        let opcode = self.fetch_next(bus, ctx);
        (Self::decode(opcode, cb_optable), opcode)
    }

    pub fn load(&mut self, data: &Vec<u8>) {
        // let mut addr: u16 = 0;
        // for byte in data {
        //     self.write_memory(USER_PROGRAM_ADDRESS as u16 + addr, *byte);
        //     addr += 1;
        // }
        // self.registers.set_pc(USER_PROGRAM_ADDRESS as u16);
    }

    /// Check and jump to requested interrupt address after take a snapshot of status on stack.
    ///
    /// Interrupt bit has priority from lower bit to higher (bit 0 has the higher priority).
    pub fn interrupt(&mut self, interrupt_registers: &InterruptRegisters) -> Option<&'static Instruction> {
        let mut interrupt = None;
        if self.ime {
            if interrupt_registers.get_vblank_enabled() && interrupt_registers.get_vblank_interrupt() {
                // Bit 0
                interrupt = Some(&instructions::INTERRUPT_VBLANK);
            } else if interrupt_registers.get_lcd_enabled() && interrupt_registers.get_lcd_interrupt() {
                // Bit 1
                interrupt = Some(&instructions::INTERRUPT_LCD);
            } else if interrupt_registers.get_timer_enabled() && interrupt_registers.get_timer_interrupt() {
                // Bit 2
                interrupt = Some(&instructions::INTERRUPT_TIMER);
            } else if interrupt_registers.get_serial_enabled() && interrupt_registers.get_serial_interrupt() {
                // Bit 3
                interrupt = Some(&instructions::INTERRUPT_SERIAL);
            } else if interrupt_registers.get_joypad_enabled() && interrupt_registers.get_joypad_interrupt() {
                // Bit 4
                interrupt = Some(&instructions::INTERRUPT_JOYPAD);
            }
        }
        interrupt
    }

    /**
       CPU Push 1-byte using SP register (to not confuse with instruction PUSH r16, that PUSH in a 2-bytes value from a double-register)
    */
    #[inline]
    pub fn push(&mut self, bus: &mut Bus, ctx: &mut MmioContext, byte: u8) {
        bus.write(ctx, self.registers.get_sp_as_address(), byte);
        self.registers.set_sp(self.registers.get_sp() - 1);
    }

    /**
       CPU Pop 1-byte using SP register (to not confuse with instruction POP r16, that pop out a 2-bytes value to put in a double-register)
    */
    #[inline]
    pub fn pop(&mut self, bus: &mut Bus, ctx: &mut MmioContext) -> Byte {
        self.registers.set_sp(self.registers.get_sp() + 1);
        bus.read(ctx, self.registers.get_sp_as_address())
    }

    fn m_cycle_tick(
        &mut self,
        bus: &mut bus::Bus,
        ctx: &mut bus::MmioContext,
        m_cycle_op: MCycleOp,
    ) -> CpuStatus {
        let mut flow: MicroFlow;
        let cpu_status: CpuStatus;

        match m_cycle_op {
            MCycleOp::Main(micro_op) => {
                flow = self.micro_tick(bus, ctx, micro_op);
            }
            MCycleOp::Cc(micro_op, cc, idx) => {
                flow = self.micro_tick(bus, ctx, micro_op);
                match cc {
                    CheckCondition::Z => {
                        if self.registers.get_zero_flag() {
                            flow = MicroFlow::Jump(idx)
                        }
                    }
                    CheckCondition::N => {
                        if self.registers.get_negative_flag() {
                            flow = MicroFlow::Jump(idx)
                        }
                    }
                    CheckCondition::H => {
                        if self.registers.get_half_carry_flag() {
                            flow = MicroFlow::Jump(idx)
                        }
                    }
                    CheckCondition::C => {
                        if self.registers.get_carry_flag() {
                            flow = MicroFlow::Jump(idx)
                        }
                    }
                    CheckCondition::NZ => {
                        if !self.registers.get_zero_flag() {
                            flow = MicroFlow::Jump(idx)
                        }
                    }
                    CheckCondition::NN => {
                        if !self.registers.get_negative_flag() {
                            flow = MicroFlow::Jump(idx)
                        }
                    }
                    CheckCondition::NH => {
                        if !self.registers.get_half_carry_flag() {
                            flow = MicroFlow::Jump(idx)
                        }
                    }
                    CheckCondition::NC => {
                        if !self.registers.get_carry_flag() {
                            flow = MicroFlow::Jump(idx)
                        }
                    }
                }
            }
            MCycleOp::End(end_micro_op) => {
                self.micro_tick(bus, ctx, end_micro_op);
                // TODO: interrupt checks
                flow = MicroFlow::End;
            }
            MCycleOp::None => {
                flow = MicroFlow::End;
            }
        }

        // Check Micro-flow of micro-codes execution sequence
        match flow {
            MicroFlow::Next => {
                // Instruction is the same - go to next microOp
                self.micro_code_index += 1;
                self.micro_code = self.instruction.unwrap().micro_ops[self.micro_code_index];
                cpu_status = CpuStatus::Execute;
            }
            MicroFlow::Jump(to) => {
                // Same instruction - change flow of microOp (useful for conditional instructions like JP)
                self.micro_code_index = to;
                self.micro_code = self.instruction.unwrap().micro_ops[self.micro_code_index];
                cpu_status = CpuStatus::Execute;
            }
            MicroFlow::PrefixCB => {
                // CB Prefix Reset the microcode flow and fetch/decode next byte with CB optable
                self.micro_code_index = 0;
                (self.instruction, self.opcode) = self.fetch_and_decode(bus, ctx, true);
                self.micro_code = self.instruction.unwrap().micro_ops[self.micro_code_index];
                cpu_status = CpuStatus::Execute;
            }
            MicroFlow::End => {
                cpu_status = CpuStatus::Ready;
            }
        }
        cpu_status
    }

    fn micro_tick(
        &mut self,
        bus: &mut bus::Bus,
        ctx: &mut bus::MmioContext,
        micro_op: MicroOp,
    ) -> MicroFlow {
        let mut micro_flow = MicroFlow::Next;
        match micro_op {
            MicroOp::Fetch8(lhs) => {
                let fetched = self.fetch_next(bus, ctx);
                self.registers.set_byte(lhs, fetched);
            }
            MicroOp::Ld8(lhs, rhs) => {
                let moved = self.registers.get_byte(rhs);
                self.registers.set_byte(lhs, moved);
            }
            MicroOp::Ld16(lhs, rhs) => {
                let moved = self.registers.get_word(rhs);
                self.registers.set_word(lhs, moved);
            }
            MicroOp::Read8(lhs, rhs) => {
                let addr = Address(self.registers.get_word(rhs));
                let value = bus.read(ctx, addr);
                self.registers.set_byte(lhs, value);
            }
            MicroOp::Write8(lhs, rhs) => {
                let addr = Address(self.registers.get_word(lhs));
                let value = self.registers.get_byte(rhs);
                bus.write(ctx, addr, value);
            }
            MicroOp::Push16msb(rhs) => {
                let msb = self.registers.get_word_msb(rhs);
                self.push(bus, ctx, msb);
            }
            MicroOp::Push16lsb(rhs) => {
                let lsb = self.registers.get_word_lsb(rhs);
                self.push(bus, ctx, lsb);
            }
            MicroOp::Pop16msb(rhs) => {
                let msb = self.pop(bus, ctx);
                self.registers.set_word_msb(rhs, msb);
            }
            MicroOp::Pop16lsb(rhs) => {
                let lsb = self.pop(bus, ctx);
                self.registers.set_word_lsb(rhs, lsb);
            }
            MicroOp::Inc16(lhs) => {
                let word = self.registers.get_word(lhs);
                self.registers.set_word(lhs, word.wrapping_add(1));
            }
            MicroOp::Dec16(lhs) => {
                let word = self.registers.get_word(lhs);
                self.registers.set_word(lhs, word.wrapping_sub(1));
            }
            MicroOp::JumpVector(jump_addr) => {
                self.registers.set_pc(jump_addr as u16);
            }
            MicroOp::Alu(alu_op) => {
                self.alu_operation(alu_op);
            }
            MicroOp::ImeEnabled(enabled) => {
                self.ime = enabled;
            }
            MicroOp::PrefixCB => {
                micro_flow = MicroFlow::PrefixCB;
                (self.instruction, self.opcode) = self.fetch_and_decode(bus, ctx, false);
                self.micro_code = self.instruction.unwrap().micro_ops[self.micro_code_index];
            }
            MicroOp::Idle => {}
        }
        micro_flow
    }

    fn alu_operation(&mut self, op: AluOp) {
        let flags = self.registers.get_flags();
        match op {
            AluOp::Add(lhs, rhs) => {
                let old_lhs = self.registers.get_byte(lhs);
                let rhs = self.registers.get_byte(rhs);
                let new_lhs = old_lhs.wrapping_add(rhs);
                self.registers.set_byte(lhs, new_lhs);
                self.registers.set_flags(Flags::new(
                    new_lhs == 0,
                    false,
                    Flags::add_half_carry(old_lhs, rhs, false),
                    Flags::add_carry(old_lhs, rhs, false),
                ));
            }
            AluOp::Adc(lhs, rhs) => {
                let old_lhs = self.registers.get_byte(lhs);
                let rhs = self.registers.get_byte(rhs);
                let carry = self.registers.get_carry_flag();
                let new_lhs = old_lhs.wrapping_add(rhs).wrapping_add(carry.into());
                self.registers.set_byte(lhs, new_lhs);
                self.registers.set_flags(Flags::new(
                    new_lhs == 0,
                    false,
                    Flags::add_half_carry(old_lhs, rhs, carry),
                    Flags::add_carry(old_lhs, rhs, carry),
                ));
            }
            AluOp::Sub(lhs, rhs) => {
                let old_lhs = self.registers.get_byte(lhs);
                let rhs = self.registers.get_byte(rhs);
                let new_lhs = old_lhs.wrapping_sub(rhs);
                self.registers.set_byte(lhs, new_lhs);
                self.registers.set_flags(Flags::new(
                    new_lhs == 0,
                    true,
                    Flags::add_half_carry(old_lhs, rhs, false),
                    Flags::add_carry(old_lhs, rhs, false),
                ));
            }
            AluOp::Sbc(lhs, rhs) => {
                let old_lhs = self.registers.get_byte(lhs);
                let rhs = self.registers.get_byte(rhs);
                let carry = self.registers.get_carry_flag();
                let new_lhs = old_lhs.wrapping_sub(rhs).wrapping_sub(carry.into());
                self.registers.set_byte(lhs, new_lhs);
                self.registers.set_flags(Flags::new(
                    new_lhs == 0,
                    true,
                    Flags::add_half_carry(old_lhs, rhs, carry),
                    Flags::add_carry(old_lhs, rhs, carry),
                ));
            }
            AluOp::Cp(lhs, rhs) => {
                let old_lhs = self.registers.get_byte(lhs);
                let rhs = self.registers.get_byte(rhs);
                let new_lhs = old_lhs.wrapping_sub(rhs);
                // CP update only Flags
                self.registers.set_flags(Flags::new(
                    new_lhs == 0,
                    true,
                    Flags::add_half_carry(old_lhs, rhs, false),
                    Flags::add_carry(old_lhs, rhs, false),
                ));
            }
            AluOp::Inc(rhs) => {
                let old_lhs = self.registers.get_byte(rhs);
                let new_lhs = old_lhs.wrapping_add(1);
                self.registers.set_byte(rhs, new_lhs);
                self.registers.set_flags(Flags::new(
                    new_lhs == 0,
                    false,
                    flags.c(),
                    Flags::add_half_carry(old_lhs, 1, false),
                ));
            }
            AluOp::Dec(rhs) => {
                let old_lhs = self.registers.get_byte(rhs);
                let new_lhs = old_lhs.wrapping_add(1);
                self.registers.set_byte(rhs, new_lhs);
                self.registers.set_flags(Flags::new(
                    new_lhs == 0,
                    true,
                    flags.c(),
                    Flags::add_half_carry(old_lhs, 1, false),
                ));
            }
            AluOp::And(lhs, rhs) => {
                let old_lhs = self.registers.get_byte(lhs);
                let rhs = self.registers.get_byte(rhs);
                let new_lhs = old_lhs & rhs;
                // CP update only Flags
                self.registers
                    .set_flags(Flags::new(new_lhs == 0, false, true, false));
            }
            AluOp::Or(lhs, rhs) => {
                let old_lhs = self.registers.get_byte(lhs);
                let rhs = self.registers.get_byte(rhs);
                let new_lhs = old_lhs | rhs;
                // CP update only Flags
                self.registers
                    .set_flags(Flags::new(new_lhs == 0, false, false, false));
            }
            AluOp::Xor(lhs, rhs) => {
                let old_lhs = self.registers.get_byte(lhs);
                let rhs = self.registers.get_byte(rhs);
                let new_lhs = old_lhs ^ rhs;
                // CP update only Flags
                self.registers
                    .set_flags(Flags::new(new_lhs == 0, false, false, false));
            }
            AluOp::Ccf() => {
                self.registers
                    .set_flags(Flags::new(flags.z(), false, false, !flags.c()));
            }
            AluOp::Scf() => {
                self.registers
                    .set_flags(Flags::new(flags.z(), false, false, true));
            }
            AluOp::Daa() => {
                todo!();
                self.registers
                    .set_flags(Flags::new(todo!(), flags.n(), true, todo!()));
            }
            AluOp::Cpl(rhs) => {
                let old_lhs = self.registers.get_byte(rhs);
                let new_lhs = !old_lhs;
                self.registers.set_byte(rhs, new_lhs);
                self.registers
                    .set_flags(Flags::new(flags.z(), true, true, flags.c()));
            }
            AluOp::Rlca() => {
                let old_lhs = self.registers.get_a();
                let new_lhs = old_lhs.rotate_left(1);
                self.registers.set_a(new_lhs);
                self.registers
                    .set_flags(Flags::new(false, false, false, (new_lhs & 1) != 0));
            }
            AluOp::Rrca() => {
                let old_lhs = self.registers.get_a();
                let new_lhs = old_lhs.rotate_right(1);
                self.registers.set_a(new_lhs);
                self.registers.set_flags(Flags::new(
                    false,
                    false,
                    false,
                    (new_lhs & (1 << 7)) != 0,
                ));
            }
            AluOp::Rla() => {
                let old_lhs = self.registers.get_a();
                let carry = (old_lhs & (1 << 7)) != 0;
                let new_lhs = (old_lhs << 1) | flags.c() as u8;
                self.registers.set_a(new_lhs);
                self.registers
                    .set_flags(Flags::new(false, false, false, carry));
            }
            AluOp::Rra() => {
                let old_lhs = self.registers.get_a();
                let carry = (old_lhs & 1) != 0;
                let new_lhs = (old_lhs >> 1) | ((flags.c() as u8) << 7);
                self.registers.set_a(new_lhs);
                self.registers
                    .set_flags(Flags::new(false, false, false, carry));
            }
            AluOp::Rlc(rhs) => {
                let old_lhs = self.registers.get_byte(rhs);
                let new_lhs = old_lhs.rotate_left(1);
                self.registers.set_byte(rhs, new_lhs);
                self.registers.set_flags(Flags::new(
                    new_lhs == 0,
                    false,
                    false,
                    (new_lhs & 1) != 0,
                ));
            }
            AluOp::Rrc(rhs) => {
                let old_lhs = self.registers.get_byte(rhs);
                let new_lhs = old_lhs.rotate_right(1);
                self.registers.set_byte(rhs, new_lhs);
                self.registers.set_flags(Flags::new(
                    new_lhs == 0,
                    false,
                    false,
                    (new_lhs & (1 << 7)) != 0,
                ));
            }
            AluOp::Rl(rhs) => {
                let old_lhs = self.registers.get_byte(rhs);
                let carry = (old_lhs & (1 << 7)) != 0;
                let new_lhs = (old_lhs << 1) | flags.c() as u8;
                self.registers.set_byte(rhs, new_lhs);
                self.registers
                    .set_flags(Flags::new(new_lhs == 0, false, false, carry));
            }
            AluOp::Rr(rhs) => {
                let old_lhs = self.registers.get_byte(rhs);
                let carry = (old_lhs & 1) != 0;
                let new_lhs = (old_lhs >> 1) | ((flags.c() as u8) << 7);
                self.registers.set_byte(rhs, new_lhs);
                self.registers
                    .set_flags(Flags::new(new_lhs == 0, false, false, carry));
            }
            AluOp::Sla(rhs) => {
                let old_lhs = self.registers.get_byte(rhs);
                let carry = (old_lhs & (1 << 7)) != 0;
                let new_lhs = old_lhs << 1;
                self.registers.set_byte(rhs, new_lhs);
                self.registers
                    .set_flags(Flags::new(new_lhs == 0, false, false, carry));
            }
            AluOp::Sra(rhs) => {
                let old_lhs = self.registers.get_byte(rhs);
                let carry = (old_lhs & 1) != 0;
                let new_lhs = (old_lhs >> 1) | (old_lhs & (1 << 7));
                self.registers.set_byte(rhs, new_lhs);
                self.registers
                    .set_flags(Flags::new(new_lhs == 0, false, false, carry));
            }
            AluOp::Sll(rhs) => {
                let old_lhs = self.registers.get_byte(rhs);
                let carry = (old_lhs & (1 << 7)) != 0;
                let new_lhs = (old_lhs << 1) | (old_lhs & 1);
                self.registers.set_byte(rhs, new_lhs);
                self.registers
                    .set_flags(Flags::new(new_lhs == 0, false, false, carry));
            }
            AluOp::Srl(rhs) => {
                let old_lhs = self.registers.get_byte(rhs);
                let carry = (old_lhs & 1) != 0;
                let new_lhs = old_lhs >> 1;
                self.registers.set_byte(rhs, new_lhs);
                self.registers
                    .set_flags(Flags::new(new_lhs == 0, false, false, carry));
            }
            AluOp::Swap(rhs) => {
                let old_lhs = self.registers.get_byte(rhs);
                let new_lhs = ((old_lhs & 0x0F) << 4) | ((old_lhs & 0xF0) >> 4);
                self.registers.set_byte(rhs, new_lhs);
                self.registers
                    .set_flags(Flags::new(new_lhs == 0, false, false, false));
            }
            AluOp::Bit(bit, rhs) => {
                let bit_on = (self.registers.get_byte(rhs) & (1 << bit as u8)) != 0;
                self.registers
                    .set_flags(Flags::new(!bit_on, false, true, flags.c()));
            }
            AluOp::Res(bit, rhs) => {
                self.registers
                    .set_byte(rhs, self.registers.get_byte(rhs) & !(1 << bit as u8));
            }
            AluOp::Set(bit, rhs) => {
                self.registers
                    .set_byte(rhs, self.registers.get_byte(rhs) | (1 << bit as u8));
            }
        }
    }
}

pub struct CpuCtx {
    pub cpu: CPU,
    pub mmio: CpuMmio
}
