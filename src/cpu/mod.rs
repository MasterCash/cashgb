pub mod execute;
pub mod instructions;
pub mod registers;

#[cfg(test)]
mod tests;

use crate::{
    cpu::{instructions::Instruction::Nop, registers::TimerRegister},
    memory::{MemoryBus, cart::Cart, joypad::Button},
};
use instructions::*;
use log::{debug, trace};
use registers::Register;

pub struct Cpu {
    pub(crate) status: CpuStatus,
    pub(crate) register: Register,
    pub(crate) program_counter: u16,
    pub(crate) stack_pointer: u16,
    pub(crate) memory: MemoryBus,
    pub(crate) step_count: u8,
    pub(crate) instruction: Instruction,
    pub(crate) ime: bool,
    pub(crate) ime_next: bool,
    /// IME was turned on by EI as this instruction was fetched. On hardware IME is still off
    /// while this instruction runs, which matters for the HALT bug.
    pub(crate) ime_just_enabled: bool,
    /// The HALT bug: the next opcode fetch doesn't increment PC, so that byte is read twice.
    pub(crate) halt_bug: bool,
    pub(crate) timer_register: TimerRegister,
}

/// M-cycles in one frame: 70224 dots at 4 dots per M-cycle
pub const M_CYCLES_PER_FRAME: u32 = 70224 / 4;

impl Cpu {
    /// Run until the PPU finishes a frame or the CPU errors. Stops after one frame's worth of
    /// M-cycles even if no frame completes, for example while the LCD is off.
    pub fn run_frame(&mut self) {
        for _ in 0..M_CYCLES_PER_FRAME {
            self.step();
            if self.status == CpuStatus::Errored || self.is_frame_ready() {
                return;
            }
        }
    }

    /// Every byte the program has sent over the serial port.
    pub fn serial_output(&self) -> &[u8] {
        self.memory.serial_output()
    }

    pub fn step_instruction(&mut self) {
        self.step();
        while self.status != CpuStatus::Errored
            && self.status != CpuStatus::Stopped
            && self.status != CpuStatus::Halted
            && self.step_count > 0
        {
            self.step();
        }
    }
    pub fn step(&mut self) {
        if CpuStatus::Errored == self.status || self.status == CpuStatus::Stopped {
            return;
        }

        if let Some(interrupt) = self.timer_register.step() {
            self.request_interrupt(interrupt);
        }

        // Step PPU (assuming 4 cycles per CPU step for timing)
        let ppu_interrupts = self.memory.step_ppu(4);
        self.memory.request_interrupts(ppu_interrupts);

        self.memory.step_serial();

        // handle instructions
        if self.step_count == 0 {
            // handle interrupts (this can wake up from HALT)
            self.handle_interrupts();

            self.ime_just_enabled = self.ime_next;
            if self.ime_next {
                self.ime = true;
                self.ime_next = false;
            }

            // If CPU is halted and no interrupt woke it up, don't process instructions
            if self.status == CpuStatus::Halted {
                return;
            }

            if let Ok((instruction, step_count)) =
                Instruction::get_instruction(&self.memory.read(self.program_counter))
            {
                self.instruction = instruction;
                self.step_count = step_count;
                if self.halt_bug {
                    self.halt_bug = false;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(1);
                }
            } else {
                self.status = CpuStatus::Errored;
                return;
            }
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
            ime_just_enabled: false,
            halt_bug: false,
            instruction: Instruction::Nop,
            memory: MemoryBus::new(cart),
            program_counter: 0x000,
            stack_pointer: 0xFFFF,
            register: Register::new(),
            timer_register: TimerRegister::new(),
        };

        cpu.reset();
        cpu
    }
    pub fn empty() -> Self {
        Self {
            status: CpuStatus::Halted,
            register: Register::new(),
            program_counter: 0,
            stack_pointer: 0,
            memory: MemoryBus::empty(),
            step_count: 0,
            instruction: Nop,
            ime: false,
            ime_next: false,
            ime_just_enabled: false,
            halt_bug: false,
            timer_register: TimerRegister::new(),
        }
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.memory.request_interrupt(interrupt);
    }

    pub(crate) fn reset(&mut self) {
        use crate::memory::cart::cart_header::addresses;

        self.status = CpuStatus::Running;
        self.ime = false;
        self.ime_next = false;
        self.ime_just_enabled = false;
        self.halt_bug = false;
        self.program_counter = addresses::ENTRY_POINT;
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
        self.timer_register.write(*addr, value);

        if let Some(new_status) = self.memory.write(*addr, value) {
            self.status = new_status;
        }
    }

    pub(crate) fn read(&self, addr: &u16) -> u8 {
        if let Some(value) = self.timer_register.read(*addr) {
            return value;
        }

        self.memory.read(*addr)
    }

    fn process_instruction(&mut self) {
        trace!("Executing Instruction: {}", self.instruction);
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
                self.program_counter = self.program_counter.wrapping_add(1);
            }
            Instruction::Bit(bit, source) => self.bit(bit, source),
            Instruction::JumpRelative(condition) => self.jump_relative(condition),
            Instruction::EnableInterrupts => self.ime_next = true,
            Instruction::DisableInterrupts => self.ime = false,
            Instruction::LoadAccumulator(target, source) => self.load_accumulator(target, source),
            Instruction::Increment(target) => self.increment(target),
            Instruction::Call(condition) => self.call(condition),
            Instruction::Subtract(source) => self.sub(source),
            Instruction::Nop => (),
            Instruction::Push(target) => self.push(target),
            Instruction::RotateLeft(source, set_z) => self.rotate_left(source, set_z),
            Instruction::Pop(target) => self.pop(target),
            Instruction::Decrement(target) => self.decrement(target),
            Instruction::Add(target, source) => self.add(target, source),
            Instruction::AddCarry(source) => self.add_carry(source),
            Instruction::And(source) => self.and(source),
            Instruction::ComplementAccumulator => self.complement_accumulator(),
            Instruction::ComplementCarryFlag => self.complement_carry_flag(),
            Instruction::DecimalAdjustAccumulator => self.decimal_adjust_accumulator(),
            Instruction::Halt => self.halt(),
            Instruction::Stop => {
                self.program_counter = self.program_counter.wrapping_add(1);
                self.status = CpuStatus::Stopped;
            }
            Instruction::Restart(addr) => self.restart(addr),
            Instruction::Return(condition) => self.ret(condition),
            Instruction::JumpHL => self.jump_hl(),
            Instruction::ReturnInterrupt => {
                self.ret(JumpCondition::None);
                self.ime = true;
            }
            Instruction::RotateLeftCircular(source, set_z) => {
                self.rotate(source, |cpu: &mut Cpu, value: u8| {
                    let last = value >> 7;
                    let n = value.rotate_left(1);
                    cpu.update_flag(Flag::C, last == 0x01);
                    cpu.update_flag(Flag::Z, n == 0 && set_z);
                    cpu.clear_flag(Flag::H);
                    cpu.clear_flag(Flag::N);
                    n
                })
            }
            Instruction::RotateRight(source, set_z) => {
                self.rotate(source, |cpu: &mut Cpu, value: u8| {
                    let last = value & 0x01;
                    let mut n = value >> 1;
                    if cpu.get_flag(Flag::C) {
                        n |= 1u8 << 7;
                    }
                    cpu.update_flag(Flag::C, last == 0x01);
                    cpu.update_flag(Flag::Z, n == 0 && set_z);
                    cpu.clear_flag(Flag::H);
                    cpu.clear_flag(Flag::N);
                    n
                })
            }
            Instruction::RotateRightCircular(source, set_z) => {
                self.rotate(source, |cpu: &mut Cpu, value: u8| {
                    let last = value & 0x01;
                    let n = value.rotate_right(1);
                    cpu.update_flag(Flag::C, last == 0x01);
                    cpu.update_flag(Flag::Z, n == 0 && set_z);
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
                    let n = (value >> 1) | sign;
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
            Instruction::RES(pos, source) => self.res(pos, source),
            Instruction::SET(pos, source) => self.set(pos, source),
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

    pub(crate) fn add_u16(a: &u16, b: &u16) -> (u16, bool, bool) {
        let (results, overflow) = a.overflowing_add(*b);
        // Half-carry: set if carry from bit 11 to bit 12
        let half_carry = ((a & 0xFFF) + (b & 0xFFF)) >= 0x1000;
        // Carry: set if overflow occurred
        (results, overflow, half_carry)
    }
    // Helper math operations
    pub(crate) fn add_u8(a: &u8, b: &u8) -> (u8, bool, bool) {
        let (results, overflow) = a.overflowing_add(*b);
        // Half-carry: set if carry from bit 3 to bit 4
        let half_carry = (((a & 0xF) + (b & 0xF)) & 0x10) == 0x10;
        (results, overflow, half_carry)
    }

    pub(crate) fn sub_u8(a: &u8, b: &u8) -> (u8, bool, bool) {
        let (results, overflow) = a.overflowing_sub(*b);
        // Half-carry: set if borrow from bit 4 into bit 3 (lower nibble of a < lower nibble of b)
        let half_carry = (a & 0xF) < (b & 0xF);
        (results, overflow, half_carry)
    }

    /// HALT: wait for an interrupt. If one is already pending, the CPU doesn't halt. With IME
    /// off (or only just turned on by EI) that triggers the HALT bug.
    /// See <https://gbdev.io/pandocs/halt.html#halt-bug>.
    fn halt(&mut self) {
        let pending_interrupts =
            self.memory.get_interrupt_flags() & self.memory.get_interrupt_enable() & 0x1F;
        if pending_interrupts == 0 {
            self.status = CpuStatus::Halted;
        } else if !self.ime || self.ime_just_enabled {
            self.halt_bug = true;
        }
    }

    /// Handle pending interrupts
    fn handle_interrupts(&mut self) {
        let interrupt_flags = self.memory.get_interrupt_flags();
        let interrupt_enable = self.memory.get_interrupt_enable();
        let pending_interrupts = interrupt_flags & interrupt_enable;

        // Wake up from HALT if there are pending interrupts (regardless of IME)
        if pending_interrupts != 0 && self.status == CpuStatus::Halted {
            self.status = CpuStatus::Running;
            debug!(
                "CPU woken from HALT due to pending interrupt: {:#X}",
                pending_interrupts
            );
        }

        if !self.ime {
            return; // Interrupts globally disabled, but CPU is still awake
        }

        if pending_interrupts == 0 {
            return; // No pending interrupts
        }

        // Check interrupts in priority order
        for interrupt in [
            Interrupt::VBlank,
            Interrupt::LCD,
            Interrupt::Timer,
            Interrupt::Serial,
            Interrupt::Joypad,
        ] {
            if pending_interrupts & (interrupt as u8) != 0 {
                // Handle the interrupt
                self.handle_interrupt(interrupt);
                break; // Only handle one interrupt per cycle
            }
        }
    }

    /// Handle a specific interrupt
    fn handle_interrupt(&mut self, interrupt: Interrupt) {
        // Disable interrupts
        self.ime = false;

        // After EI; HALT the HALT bug makes the interrupt return to the HALT, not past it
        if self.halt_bug {
            self.halt_bug = false;
            self.program_counter = self.program_counter.wrapping_sub(1);
        }

        // Clear the interrupt flag
        let mut flags = self.memory.get_interrupt_flags();
        flags &= !(interrupt as u8);
        self.memory.write(0xFF0F, flags);

        // Push current PC to stack (high byte first at higher address)
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        let pc_high = (self.program_counter >> 8) as u8;
        self.memory.write(self.stack_pointer, pc_high);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        let pc_low = (self.program_counter & 0xFF) as u8;
        self.memory.write(self.stack_pointer, pc_low);

        // Jump to interrupt vector
        self.program_counter = match interrupt {
            Interrupt::VBlank => 0x0040,
            Interrupt::LCD => 0x0048,
            Interrupt::Timer => 0x0050,
            Interrupt::Serial => 0x0058,
            Interrupt::Joypad => 0x0060,
        };

        self.step_count = 5;

        debug!(
            "Handling interrupt: {:?} -> {:#06X}",
            interrupt, self.program_counter
        );
    }

    /// Add methods for PPU integration
    pub fn is_frame_ready(&self) -> bool {
        self.memory.is_frame_ready()
    }

    pub fn get_framebuffer(&mut self) -> &Vec<u8> {
        self.memory.get_framebuffer()
    }

    /// Debug methods for GUI access
    pub fn get_pc(&self) -> u16 {
        self.program_counter
    }

    pub fn get_sp(&self) -> u16 {
        self.stack_pointer
    }

    pub fn get_af(&self) -> u16 {
        self.register.get_af()
    }

    pub fn get_bc(&self) -> u16 {
        self.register.get_bc()
    }

    pub fn get_de(&self) -> u16 {
        self.register.get_de()
    }

    pub fn get_hl(&self) -> u16 {
        self.register.get_hl()
    }

    pub fn get_current_instruction(&self) -> &Instruction {
        &self.instruction
    }

    /// Read a byte for debugging. Sees the timer registers like `read` does, but ignores the
    /// PPU's VRAM/OAM access restrictions and has no side effects.
    pub fn read_memory(&self, addr: u16) -> u8 {
        if let Some(value) = self.timer_register.read(addr) {
            return value;
        }
        self.memory.peek(addr)
    }

    /// Press or release a joypad button. A new press of a selected input requests the
    /// Joypad interrupt, and any new press wakes the CPU from STOP.
    pub fn set_button(&mut self, button: Button, pressed: bool) {
        let was_pressed = self.memory.joypad.is_pressed(button);
        self.memory.set_button(button, pressed);
        if pressed && !was_pressed && self.status == CpuStatus::Stopped {
            self.status = CpuStatus::Running;
        }
    }

    pub fn get_flags(&self) -> (bool, bool, bool, bool) {
        (
            self.get_flag(Flag::Z),
            self.get_flag(Flag::N),
            self.get_flag(Flag::H),
            self.get_flag(Flag::C),
        )
    }
}
