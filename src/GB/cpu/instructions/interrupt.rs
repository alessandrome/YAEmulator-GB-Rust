use crate::GB::cpu::instructions::{self, Instruction};

#[derive(Copy, Clone, Debug)]
pub enum InterruptType {
    VBlank,
    LCD,
    Timer,
    Serial,
    Joypad,
}

impl InterruptType {
    pub fn interrupt_instruction(interrupt_type: InterruptType) -> &'static Instruction {
        match interrupt_type {
            InterruptType::VBlank => &instructions::INTERRUPT_VBLANK,
            InterruptType::LCD => &instructions::INTERRUPT_LCD,
            InterruptType::Timer => &instructions::INTERRUPT_TIMER,
            InterruptType::Serial => &instructions::INTERRUPT_SERIAL,
            InterruptType::Joypad => &instructions::INTERRUPT_JOYPAD,
        }
    }

    pub fn instruction(&self) -> &'static Instruction {
        Self::interrupt_instruction(*self)
    }
}
