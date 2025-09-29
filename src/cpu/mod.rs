pub mod execute;
pub mod instructions;
pub mod registers;

#[cfg(test)]
mod tests;

use crate::memory::{cart::Cart, MemoryBus};
use instructions::*;
use registers::Register;

pub struct Cpu {
    #[cfg_attr(test, allow(dead_code))]
    pub(crate) status: CpuStatus,
    #[cfg_attr(test, allow(dead_code))]
    pub(crate) register: Register,
    #[cfg_attr(test, allow(dead_code))]
    pub(crate) program_counter: u16,
    #[cfg_attr(test, allow(dead_code))]
    pub(crate) stack_pointer: u16,
    #[cfg_attr(test, allow(dead_code))]
    pub(crate) memory: MemoryBus,
    #[cfg_attr(test, allow(dead_code))]
    pub(crate) step_count: u8,
    #[cfg_attr(test, allow(dead_code))]
    pub(crate) instruction: Instruction,
    #[cfg_attr(test, allow(dead_code))]
    pub(crate) ime: bool,
    #[cfg_attr(test, allow(dead_code))]
    pub(crate) ime_next: bool,
}

impl Cpu {
    pub fn step(&mut self) {
        if CpuStatus::Errored == self.status || self.status == CpuStatus::Stopped {
            return;
        }

        // handle interrupts

        if self.ime_next {
            self.ime = true;
            self.ime_next = false;
        }

        // handle instructions
        if self.step_count == 0 {
            (self.instruction, self.step_count) =
                Instruction::get_instruction(&self.memory.read(self.program_counter));
            self.program_counter += 1;
        }
        if self.step_count > 1 {
            self.step_count -= 1;
            return;
        }

        self.process_instruction();
    }

    pub fn new(cart: Cart) -> Self {
        let mut cpu = Self {
            status: CpuStatus::Running,
            ime_next: false,
            step_count: 0,
            ime: false,
            instruction: Instruction::Nop,
            memory: MemoryBus::new(cart),
            program_counter: 0x000,
            stack_pointer: 0xFFFF,
            register: Register::new(),
        };

        cpu.reset();
        cpu
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.memory.request_interrupt(interrupt);
    }

    pub(crate) fn reset(&mut self) {
        self.status = CpuStatus::Running;
        self.ime = false;
        self.ime_next = false;
        self.program_counter = 0x100;
        self.stack_pointer = 0xfffe;
        self.register.set_af(0x01b0);
        self.register.set_bc(0x0013);
        self.register.set_de(0x00d8);
        self.register.set_hl(0x014d);
        self.memory.write(0xFF05, 0x00);
        self.memory.write(0xFF06, 0x00);
        self.memory.write(0xFF07, 0x00);
        self.memory.write(0xFF10, 0x80);
        self.memory.write(0xFF11, 0xBF);
        self.memory.write(0xFF12, 0xF3);
        self.memory.write(0xFF14, 0xBF);
        self.memory.write(0xFF16, 0x3F);
        self.memory.write(0xFF17, 0x00);
        self.memory.write(0xFF19, 0xBF);
        self.memory.write(0xFF1A, 0x7F);
        self.memory.write(0xFF1B, 0xFF);
        self.memory.write(0xFF1C, 0x9F);
        self.memory.write(0xFF1E, 0xBF);
        self.memory.write(0xFF20, 0xFF);
        self.memory.write(0xFF21, 0x00);
        self.memory.write(0xFF22, 0x00);
        self.memory.write(0xFF23, 0xBF);
        self.memory.write(0xFF24, 0x77);
        self.memory.write(0xFF25, 0xF3);
        self.memory.write(0xFF26, 0xF1);
        self.memory.write(0xFF40, 0x91);
        self.memory.write(0xFF42, 0x00);
        self.memory.write(0xFF43, 0x00);
        self.memory.write(0xFF45, 0x00);
        self.memory.write(0xFF47, 0xFC);
        self.memory.write(0xFF48, 0xFF);
        self.memory.write(0xFF49, 0xFF);
        self.memory.write(0xFF4A, 0x00);
        self.memory.write(0xFF4B, 0x00);
        self.memory.write(0xFFFF, 0x00);
    }

    pub(crate) fn write(&mut self, addr: &u16, value: u8) {
        if let Some(new_status) = self.memory.write(*addr, value) {
            self.status = new_status;
        }
    }

    pub(crate) fn read(&self, addr: &u16) -> u8 {
        self.memory.read(*addr)
    }

    fn process_instruction(&mut self) {
        println!("Executing Instruction: {}", self.instruction);
        self.step_count = 0;
        match self.instruction {
            Instruction::Compare(source) => self.compare(source),
            Instruction::XOr(source) => self.xor(source),
            Instruction::Load(target, source) => self.load(target, source),
            Instruction::CB => {
                let (instruction, cycles) =
                    Instruction::get_cb_instruction(&self.memory.read(self.program_counter));
                self.instruction = instruction;
                self.step_count = cycles;
                self.program_counter += 1;
            }
            Instruction::Bit(bit, source) => self.bit(bit, source),
            Instruction::JumpRelative(condition) => self.jump_relative(condition),
            Instruction::EnableInterrupts => self.ime = true,
            Instruction::DisableInterrupts => self.ime = false,
            Instruction::LoadAccumulator(target, source) => self.load_accumulator(target, source),
            Instruction::Increment(target) => self.increment(target),
            Instruction::Call(condition) => self.call(condition),
            Instruction::Subtract(source) => self.sub(source),
            Instruction::Nop => (),
            Instruction::Push(target) => self.push(target),
            Instruction::RotateLeft(source) => self.rotate_left(source),
            Instruction::Pop(target) => self.pop(target),
            Instruction::Decrement(target) => self.decrement(target),
            Instruction::Add(target, source) => self.add(target, source),
            Instruction::AddCarry(source) => self.add_carry(source),
            Instruction::And(source) => self.and(source),
            Instruction::ComplementAccumulator => self.complement_accumulator(),
            Instruction::ComplementCarryFlag => self.complement_carry_flag(),
            Instruction::DecimalAdjustAccumulator => self.decimal_adjust_accumulator(),
            Instruction::Halt => self.status = CpuStatus::Halted,
            Instruction::Stop => {
                self.program_counter += 1;
                self.status = CpuStatus::Stopped;
            }
            Instruction::Restart(addr) => self.restart(addr),
            Instruction::Return(condition) => self.ret(condition),
            Instruction::JumpHL => self.jump_hl(),
            Instruction::ReturnInterrupt => {
                self.ret(JumpCondition::None);
                self.ime = true;
            }
            Instruction::RotateLeftCircular(source) => {
                self.rotate(source, |cpu: &mut Cpu, value: u8| {
                    let last = value >> 7;
                    let n = value.rotate_left(1);
                    cpu.update_flag(Flag::C, last == 0x01);
                    cpu.update_flag(Flag::Z, n == 0);
                    cpu.clear_flag(Flag::H);
                    cpu.clear_flag(Flag::N);
                    n
                })
            }
            Instruction::RotateRight(source) => self.rotate(source, |cpu: &mut Cpu, value: u8| {
                let last = value & 0x01;
                let mut n = value >> 1;
                if cpu.get_flag(Flag::C) {
                    n |= 1u8 << 7;
                }
                cpu.update_flag(Flag::C, last == 0x01);
                cpu.update_flag(Flag::Z, n == 0);
                cpu.clear_flag(Flag::H);
                cpu.clear_flag(Flag::N);
                n
            }),
            Instruction::RotateRightCircular(source) => {
                self.rotate(source, |cpu: &mut Cpu, value: u8| {
                    let last = value & 0x01;
                    let n = value.rotate_right(1);
                    cpu.update_flag(Flag::C, last == 0x01);
                    cpu.update_flag(Flag::Z, n == 0);
                    cpu.clear_flag(Flag::H);
                    cpu.clear_flag(Flag::N);
                    n
                })
            }
            Instruction::SetCarryFlag => {
                self.set_flag(Flag::C);
                self.clear_flag(Flag::N);
                self.clear_flag(Flag::H);
            }
            Instruction::ShiftLeftArithmetic(source) => {
                self.rotate(source, |cpu: &mut Cpu, value: u8| {
                    let last = value >> 7;
                    let n = value << 1;
                    cpu.update_flag(Flag::C, last == 0x01);
                    cpu.update_flag(Flag::Z, n == 0);
                    cpu.clear_flag(Flag::H);
                    cpu.clear_flag(Flag::N);
                    n
                })
            }
            Instruction::ShiftRightArithmetic(source) => {
                self.rotate(source, |cpu: &mut Cpu, value: u8| {
                    let sign = value & 0x80;
                    let c = value & 0x01;
                    let n = (value >> 1) & sign;
                    cpu.update_flag(Flag::C, c == 0x01);
                    cpu.update_flag(Flag::Z, n == 0);
                    cpu.clear_flag(Flag::H);
                    cpu.clear_flag(Flag::N);
                    n
                })
            }
            Instruction::ShiftRightLogical(source) => {
                self.rotate(source, |cpu: &mut Cpu, value: u8| {
                    let c = value & 0x01;
                    let n = value >> 1;
                    cpu.update_flag(Flag::C, c == 0x01);
                    cpu.update_flag(Flag::Z, n == 0);
                    cpu.clear_flag(Flag::H);
                    cpu.clear_flag(Flag::N);
                    n
                })
            }
            Instruction::SubtractCarry(source) => self.sub_carry(source),
            Instruction::Swap(source) => self.swap(source),
            Instruction::Or(source) => self.or(source),
            Instruction::Jump(condition) => self.jump(condition),
        }
    }

    // Helper methods for flag management
    pub(crate) fn update_flag(&mut self, flag: Flag, value: bool) {
        if value {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    pub(crate) fn set_flag(&mut self, flag: Flag) {
        self.register.set_f(self.register.get_f() | flag as u8);
    }

    pub(crate) fn clear_flag(&mut self, flag: Flag) {
        self.register.set_f(self.register.get_f() & !(flag as u8));
    }

    pub(crate) fn get_flag(&self, flag: Flag) -> bool {
        self.register.get_f() & flag as u8 == flag as u8
    }

    // Helper math operations
    pub(crate) fn addition(a: &u8, b: &u8) -> (u8, bool, bool) {
        let (results, overflow) = a.overflowing_add(*b);
        // Half-carry: set if carry from bit 3 to bit 4
        let half_carry = ((a & 0xF) + (b & 0xF)) & 0x10 == 0x10;
        (results, overflow, half_carry)
    }

    pub(crate) fn subtract(a: &u8, b: &u8) -> (u8, bool, bool) {
        let (results, overflow) = a.overflowing_sub(*b);
        // Half-carry: set if borrow from bit 4 to bit 3 (when lower nibble of a < lower nibble of b)
        let half_carry = (a & 0xF) < (b & 0xF);
        (results, overflow, half_carry)
    }
}
