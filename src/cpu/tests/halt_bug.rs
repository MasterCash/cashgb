//! HALT with an interrupt already pending. See <https://gbdev.io/pandocs/halt.html#halt-bug>.

use super::review_regressions::{cpu_with, run};
use crate::cpu::Cpu;
use crate::cpu::instructions::{CpuStatus, Interrupt};

/// Enable the Timer interrupt in IE and request it in IF.
fn request_timer_interrupt(cpu: &mut Cpu) {
    cpu.write(&0xFFFF, Interrupt::Timer as u8);
    cpu.request_interrupt(Interrupt::Timer);
}

#[test]
fn halt_with_nothing_pending_halts() {
    let mut cpu = cpu_with(&[0x76], 0x0200); // HALT

    run(&mut cpu, 1);

    assert_eq!(cpu.status, CpuStatus::Halted);
}

#[test]
fn halt_bug_reads_the_next_byte_twice() {
    // HALT; LD A,$3E is read as HALT; LD A,$3E; INC D (0x14)
    let mut cpu = cpu_with(&[0x76, 0x3E, 0x14], 0x0200);
    request_timer_interrupt(&mut cpu);
    let d = cpu.register.get_d();

    run(&mut cpu, 3);

    assert_eq!(cpu.status, CpuStatus::Running, "HALT doesn't halt");
    assert_eq!(
        cpu.register.get_a(),
        0x3E,
        "the LD opcode is read again as its operand"
    );
    assert_eq!(
        cpu.register.get_d(),
        d.wrapping_add(1),
        "the operand runs as INC D"
    );
    assert_eq!(cpu.program_counter, 0x0203);
}

#[test]
fn halt_bug_on_a_one_byte_instruction_runs_it_twice() {
    // HALT; INC A runs INC A twice
    let mut cpu = cpu_with(&[0x76, 0x3C, 0x00], 0x0200);
    request_timer_interrupt(&mut cpu);
    cpu.register.set_a(0);

    run(&mut cpu, 3);

    assert_eq!(cpu.register.get_a(), 2);
    assert_eq!(cpu.program_counter, 0x0202);
}

#[test]
fn interrupt_during_halt_returns_past_the_halt() {
    let mut cpu = cpu_with(&[0x76, 0x00], 0x0200); // HALT; NOP
    cpu.ime = true;
    cpu.write(&0xFFFF, Interrupt::Timer as u8);

    run(&mut cpu, 1);
    assert_eq!(cpu.status, CpuStatus::Halted);
    cpu.request_interrupt(Interrupt::Timer);
    cpu.step(); // wakes and dispatches the interrupt

    assert_eq!(cpu.stack_pointer, 0xFFFC, "the interrupt was dispatched");
    assert_eq!(
        cpu.read(&cpu.stack_pointer),
        0x01,
        "returns to 0x0201, past the HALT"
    );
    assert!(!cpu.halt_bug);
}

#[test]
fn ei_then_halt_returns_to_the_halt() {
    let mut cpu = cpu_with(&[0xFB, 0x76, 0x00], 0x0200); // EI; HALT; NOP
    request_timer_interrupt(&mut cpu);

    run(&mut cpu, 2);
    cpu.step(); // dispatches the interrupt

    assert_eq!(cpu.stack_pointer, 0xFFFC, "the interrupt was dispatched");
    assert_eq!(
        cpu.read(&cpu.stack_pointer),
        0x01,
        "returns to 0x0201, the HALT"
    );
    assert!(!cpu.halt_bug);
}
