use log::trace;

/// Regression tests from the adversarial code review.
///
/// Each test asserts correct Game Boy hardware behaviour for one reported bug, so a
/// failing test here means that bug is still present.
use crate::cpu::Cpu;
use crate::cpu::instructions::Interrupt;
use crate::memory::cart::Cart;
use crate::memory::cart::cart_header::{NINTENDO_LOGO, addresses};
use crate::ppu::LcdMode;

/// Build a ROM image with a valid logo and header checksum.
fn rom(cart_type: u8, rom_code: u8, ram_code: u8, size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    data[0x104..0x104 + NINTENDO_LOGO.len()].copy_from_slice(&NINTENDO_LOGO);
    data[addresses::CARTRIDGE_TYPE as usize] = cart_type;
    data[addresses::ROM_SIZE as usize] = rom_code;
    data[addresses::RAM_SIZE as usize] = ram_code;
    let mut checksum = 0u8;
    for byte in &data[0x134..=0x14C] {
        checksum = checksum.wrapping_sub(*byte).wrapping_sub(1);
    }
    data[addresses::HEADER_CHECKSUM as usize] = checksum;
    data
}

/// Create a CPU on a 32KB ROM-only cart with `code` placed at `at`, and PC set to `at`.
pub(super) fn cpu_with(code: &[u8], at: u16) -> Cpu {
    let mut data = rom(0x00, 0x00, 0x00, 0x8000);
    data[at as usize..at as usize + code.len()].copy_from_slice(code);
    let mut cpu = Cpu::new(Cart::new(&data).unwrap());
    cpu.program_counter = at;
    cpu
}

/// Execute `n` complete instructions.
pub(super) fn run(cpu: &mut Cpu, n: usize) {
    for _ in 0..n {
        cpu.step_instruction();
    }
}

/// Step until the PPU reaches `mode`, so VRAM/OAM access restrictions don't mask results.
fn step_until_mode(cpu: &mut Cpu, mode: LcdMode) {
    for _ in 0..20_000 {
        if cpu.memory.ppu.get_current_mode() == mode {
            return;
        }
        cpu.step();
    }
    panic!("PPU never reached {mode:?}");
}

// --- CPU registers -------------------------------------------------------------------

/// `set_b`/`set_d`/`set_h` must write the high byte without touching the low byte.
#[test]
fn ld_b_and_ld_c_are_independent() {
    let mut cpu = cpu_with(&[0x06, 0x12, 0x0E, 0x34], 0x150); // LD B,12; LD C,34
    run(&mut cpu, 2);
    assert_eq!(cpu.register.get_bc(), 0x1234);
}

#[test]
fn ld_d_and_ld_e_are_independent() {
    let mut cpu = cpu_with(&[0x16, 0x12, 0x1E, 0x34], 0x150); // LD D,12; LD E,34
    run(&mut cpu, 2);
    assert_eq!(cpu.register.get_de(), 0x1234);
}

#[test]
fn ld_h_and_ld_l_are_independent() {
    let mut cpu = cpu_with(&[0x26, 0x12, 0x2E, 0x34], 0x150); // LD H,12; LD L,34
    run(&mut cpu, 2);
    assert_eq!(cpu.register.get_hl(), 0x1234);
}

/// `POP AF` must mask the low nibble of F to zero.
#[test]
fn pop_af_masks_low_flag_bits() {
    // LD BC,12FF; PUSH BC; POP AF
    let mut cpu = cpu_with(&[0x01, 0xFF, 0x12, 0xC5, 0xF1], 0x150);
    run(&mut cpu, 3);
    assert_eq!(cpu.register.get_af(), 0x12F0);
}

// --- Instruction decoding ------------------------------------------------------------

/// 0xCD is an unconditional `CALL nn`, not `CALL C,nn`.
#[test]
fn call_nn_is_unconditional() {
    let mut cpu = cpu_with(&[0xA7, 0xCD, 0x00, 0x02], 0x150); // AND A (clears C); CALL 0200
    run(&mut cpu, 2);
    assert_eq!(cpu.program_counter, 0x0200);
}

/// A conditional `CALL` that is not taken must skip exactly its 2 operand bytes.
#[test]
fn call_cc_not_taken_skips_two_bytes() {
    // AND A; CALL C,0200 (not taken); LD A,77
    let mut cpu = cpu_with(&[0xA7, 0xDC, 0x00, 0x02, 0x3E, 0x77], 0x150);
    run(&mut cpu, 3);
    assert_eq!(cpu.register.get_a(), 0x77);
}

/// CB 0xC0-0xFF (`SET b,r`) must be decoded.
#[test]
fn cb_set_bit() {
    let mut cpu = cpu_with(&[0x3E, 0x00, 0xCB, 0xC7], 0x150); // LD A,0; SET 0,A
    run(&mut cpu, 2);
    assert_eq!(cpu.register.get_a(), 0x01);
}

/// CB 0x88-0xBF (`RES 1..7,r`) must be decoded.
#[test]
fn cb_res_high_bit() {
    let mut cpu = cpu_with(&[0x3E, 0xFF, 0xCB, 0xBF], 0x150); // LD A,FF; RES 7,A
    run(&mut cpu, 2);
    assert_eq!(cpu.register.get_a(), 0x7F);
}

/// 0x36 is `LD (HL),n`: it writes memory and leaves HL unchanged.
#[test]
fn ld_hl_addr_immediate() {
    // LD HL,C000; LD (HL),42
    let mut cpu = cpu_with(&[0x21, 0x00, 0xC0, 0x36, 0x42], 0x150);
    run(&mut cpu, 2);
    assert_eq!(cpu.read(&0xC000), 0x42);
    assert_eq!(cpu.register.get_hl(), 0xC000);
}

/// 0xE8 is `ADD SP,e8` with a signed immediate, not `ADD SP,E`.
#[test]
fn add_sp_uses_immediate() {
    // LD SP,D000; LD E,05; ADD SP,+2
    let mut cpu = cpu_with(&[0x31, 0x00, 0xD0, 0x1E, 0x05, 0xE8, 0x02], 0x150);
    run(&mut cpu, 3);
    assert_eq!(cpu.stack_pointer, 0xD002);
}

/// `ADD SP,e8` sets H and C from the unsigned low byte addition and clears Z and N.
#[test]
fn add_sp_negative_e_flags_use_unsigned_low_byte() {
    // XOR A; LD SP,0001; ADD SP,-1  =>  0x01 + 0xFF carries out of bits 3 and 7
    let mut cpu = cpu_with(&[0xAF, 0x31, 0x01, 0x00, 0xE8, 0xFF], 0x150);
    run(&mut cpu, 3);
    assert_eq!(cpu.stack_pointer, 0x0000);
    assert_eq!(cpu.get_flags(), (false, false, true, true));
}

/// An illegal opcode must stop the CPU in `Errored`, not panic the process.
#[test]
fn illegal_opcode_does_not_panic() {
    let mut cpu = cpu_with(&[0xD3], 0x150);
    run(&mut cpu, 1);
    assert!(cpu.status == crate::cpu::instructions::CpuStatus::Errored);
}

// --- Instruction semantics -----------------------------------------------------------

/// `RL` rotates the value left through carry.
#[test]
fn rl_rotates_through_carry() {
    // SCF; LD A,01; RL A
    let mut cpu = cpu_with(&[0x37, 0x3E, 0x01, 0xCB, 0x17], 0x150);
    run(&mut cpu, 3);
    assert_eq!(cpu.register.get_a(), 0x03);
}

/// `SRA` shifts right while preserving the sign bit.
#[test]
fn sra_preserves_sign() {
    let mut cpu = cpu_with(&[0x3E, 0x81, 0xCB, 0x2F], 0x150); // LD A,81; SRA A
    run(&mut cpu, 2);
    assert_eq!(cpu.register.get_a(), 0xC0);
    assert_eq!(cpu.get_flags(), (false, false, false, true)); // C = old bit 0
}

/// `SRA` on a positive value shifts in 0 and sets Z when the result is 0.
#[test]
fn sra_positive_to_zero() {
    let mut cpu = cpu_with(&[0x3E, 0x01, 0xCB, 0x2F], 0x150); // LD A,01; SRA A
    run(&mut cpu, 2);
    assert_eq!(cpu.register.get_a(), 0x00);
    assert_eq!(cpu.get_flags(), (true, false, false, true));
}

/// `RL` sets Z from the result and clears N and H.
#[test]
fn rl_sets_z_and_clears_n_h() {
    // AND A (A=01, H=1); CB RL C where C=00 and carry clear => C=00, Z=1, H=0
    let mut cpu = cpu_with(&[0x0E, 0x00, 0xA7, 0xCB, 0x11], 0x150); // LD C,0; AND A; RL C
    run(&mut cpu, 3);
    assert_eq!(cpu.register.get_c(), 0x00);
    assert_eq!(cpu.get_flags(), (true, false, false, false));
}

/// RLCA/RRCA/RLA/RRA always clear Z, even when the result is 0.
#[test]
fn accumulator_rotates_always_clear_z() {
    // Each case: XOR A (A=0, Z=1, C=0) then the rotate; result is 0 but Z must be 0.
    for opcode in [0x07, 0x0F, 0x17, 0x1F] {
        let mut cpu = cpu_with(&[0xAF, opcode], 0x150);
        run(&mut cpu, 2);
        assert_eq!(cpu.register.get_a(), 0x00, "opcode {opcode:#04x}");
        assert_eq!(
            cpu.get_flags(),
            (false, false, false, false),
            "opcode {opcode:#04x}"
        );
    }
}

/// The CB-prefixed rotates on A (RLC/RRC/RL/RR A) set Z from the result.
#[test]
fn cb_rotates_on_a_set_z() {
    for opcode in [0x07, 0x0F, 0x17, 0x1F] {
        let mut cpu = cpu_with(&[0xAF, 0xCB, opcode], 0x150); // XOR A; CB <rotate> A
        run(&mut cpu, 2);
        assert_eq!(cpu.register.get_a(), 0x00, "CB {opcode:#04x}");
        assert_eq!(
            cpu.get_flags(),
            (true, false, false, false),
            "CB {opcode:#04x}"
        );
    }
}

/// `ADD HL,rr` leaves Z unchanged.
#[test]
fn add_hl_preserves_z() {
    // XOR A (Z=1); LD HL,0001; LD BC,0001; ADD HL,BC
    let mut cpu = cpu_with(&[0xAF, 0x21, 0x01, 0x00, 0x01, 0x01, 0x00, 0x09], 0x150);
    run(&mut cpu, 4);
    assert_eq!(cpu.register.get_hl(), 0x0002);
    let (z, n, _, _) = cpu.get_flags();
    assert!(z, "Z must be preserved");
    assert!(!n);
}

/// `SBC` includes the incoming carry in the half-borrow check.
#[test]
fn sbc_half_borrow_includes_carry() {
    // LD A,10; SCF; SBC A,0F  =>  0x10 - 0x0F - 1 = 0x00; low nibble 0 < F + 1 => H=1
    let mut cpu = cpu_with(&[0x3E, 0x10, 0x37, 0xDE, 0x0F], 0x150);
    run(&mut cpu, 3);
    assert_eq!(cpu.register.get_a(), 0x00);
    assert_eq!(cpu.get_flags(), (true, true, true, false));
}

/// `SBC` sets H when only the incoming carry causes the half-borrow.
#[test]
fn sbc_half_borrow_from_carry_only() {
    // LD A,10; SCF; SBC A,00  =>  0x10 - 0x00 - 1 = 0x0F; low nibble 0 < 0 + 1 => H=1, C=0
    let mut cpu = cpu_with(&[0x3E, 0x10, 0x37, 0xDE, 0x00], 0x150);
    run(&mut cpu, 3);
    assert_eq!(cpu.register.get_a(), 0x0F);
    assert_eq!(cpu.get_flags(), (false, true, true, false));
}

/// `ADC A,FF` with carry set must wrap to 0 with Z and C set, without overflowing.
#[test]
fn adc_ff_with_carry() {
    // LD A,00; SCF; ADC A,FF
    let mut cpu = cpu_with(&[0x3E, 0x00, 0x37, 0xCE, 0xFF], 0x150);
    run(&mut cpu, 3);
    assert_eq!(cpu.register.get_a(), 0x00);
    let (z, _, _, c) = cpu.get_flags();
    assert!(z && c);
}

/// `SBC A,FF` with carry set must wrap to 0 with C set, without overflowing.
#[test]
fn sbc_ff_with_carry() {
    // LD A,00; SCF; SBC A,FF
    let mut cpu = cpu_with(&[0x3E, 0x00, 0x37, 0xDE, 0xFF], 0x150);
    run(&mut cpu, 3);
    assert_eq!(cpu.register.get_a(), 0x00);
    let (_, _, _, c) = cpu.get_flags();
    assert!(c);
}

/// `JR` must use wrapping 16-bit arithmetic across 0x7FFF/0x8000.
#[test]
fn jr_across_0x8000_boundary() {
    let mut cpu = cpu_with(&[0x18, 0x01], 0x7FFD); // JR +1 at 7FFD -> 8000
    run(&mut cpu, 1);
    trace!("PC after JR: {:#06X}", cpu.program_counter);
    assert_eq!(cpu.program_counter, 0x8000);
}

/// `LD HL,SP+e8` must set H and C from the low byte addition and clear Z and N.
#[test]
fn ld_hl_sp_e_sets_flags() {
    // XOR A (Z=1, N=H=C=0, so H/C below must be computed); LD SP,00FF; LD HL,SP+1
    let mut cpu = cpu_with(&[0xAF, 0x31, 0xFF, 0x00, 0xF8, 0x01], 0x150);
    run(&mut cpu, 3);
    assert_eq!(cpu.register.get_hl(), 0x0100);
    assert_eq!(cpu.get_flags(), (false, false, true, true));
}

/// `LD HL,SP+e8` clears H and C when the low byte addition doesn't carry.
#[test]
fn ld_hl_sp_e_clears_flags_without_carry() {
    // SCF (C=1); LD SP,0000; LD HL,SP+1
    let mut cpu = cpu_with(&[0x37, 0x31, 0x00, 0x00, 0xF8, 0x01], 0x150);
    run(&mut cpu, 3);
    assert_eq!(cpu.register.get_hl(), 0x0001);
    assert_eq!(cpu.get_flags(), (false, false, false, false));
}

/// With a negative offset, H and C still come from the unsigned low byte addition.
#[test]
fn ld_hl_sp_negative_e_flags_use_unsigned_low_byte() {
    // XOR A; LD SP,0001; LD HL,SP-1  =>  0x01 + 0xFF carries out of bits 3 and 7
    let mut cpu = cpu_with(&[0xAF, 0x31, 0x01, 0x00, 0xF8, 0xFF], 0x150);
    run(&mut cpu, 3);
    assert_eq!(cpu.register.get_hl(), 0x0000);
    assert_eq!(cpu.get_flags(), (false, false, true, true));
}

// --- Interrupts ----------------------------------------------------------------------

/// An interrupt must not be dispatched while an instruction is still in flight.
#[test]
fn interrupt_does_not_split_instruction() {
    let mut cpu = cpu_with(&[0xC3, 0x00, 0x02], 0x150); // JP 0200
    cpu.ime = true;
    cpu.memory.ie = 0x01;
    cpu.step(); // fetch JP; instruction now in flight
    cpu.memory.request_interrupt(Interrupt::VBlank);
    for _ in 0..10 {
        cpu.step();
    }
    let ret = cpu.read(&0xFFFC) as u16 | ((cpu.read(&0xFFFD) as u16) << 8);
    assert_eq!(ret, 0x0200, "return address pushed by interrupt");
}

// --- Memory bus and I/O --------------------------------------------------------------

/// With no buttons pressed the joypad's low nibble reads 1s (active-low).
#[test]
fn joypad_idle_reads_unpressed() {
    let mut cpu = cpu_with(&[], 0x150);
    cpu.write(&0xFF00, 0x20);
    assert_eq!(cpu.read(&0xFF00) & 0x0F, 0x0F);
}

/// Writes to the unusable region 0xFEA0-0xFEFF are ignored, not fatal.
#[test]
fn write_unusable_region_does_not_panic() {
    let mut cpu = cpu_with(&[], 0x150);
    cpu.write(&0xFEA0, 0x00);
}

/// Echo RAM writes are mirrored into work RAM.
#[test]
fn echo_ram_write_mirrors_work_ram() {
    let mut cpu = cpu_with(&[], 0x150);
    cpu.write(&0xE000, 0x5A);
    assert_eq!(cpu.read(&0xC000), 0x5A);
}

/// Writing 0xFF50 (boot ROM disable) must not stop the CPU.
#[test]
fn boot_rom_disable_does_not_stop_cpu() {
    let mut cpu = cpu_with(&[], 0x150);
    cpu.write(&0xFF50, 0x01);
    assert!(cpu.status == crate::cpu::instructions::CpuStatus::Running);
}

/// The timer divider (0xFF04) must advance as the CPU runs.
#[test]
fn div_register_advances() {
    let mut cpu = cpu_with(&[], 0x150);
    let before = cpu.read(&0xFF04);
    for _ in 0..1000 {
        cpu.step();
    }
    assert_ne!(cpu.read(&0xFF04), before);
}

/// Writing 0xFF46 copies 160 bytes from `XX00` into OAM.
#[test]
fn oam_dma_copies_to_oam() {
    let mut cpu = cpu_with(&[], 0x150);
    cpu.write(&0xC000, 0x99);
    cpu.write(&0xC09F, 0x77);
    cpu.write(&0xFF46, 0xC0);
    for _ in 0..200 {
        cpu.step();
    }
    step_until_mode(&mut cpu, LcdMode::HBlank);
    assert_eq!(cpu.read(&0xFE00), 0x99);
    assert_eq!(cpu.read(&0xFE9F), 0x77);
}

// --- Cartridge / MBC -----------------------------------------------------------------

/// ROM-area writes reach the MBC so bank switching works through the bus.
#[test]
fn mbc1_bank_switch_via_bus() {
    let mut data = rom(0x01, 0x01, 0x00, 0x10000); // MBC1, 64KB
    data[2 * 0x4000] = 0xAB;
    let mut cpu = Cpu::new(Cart::new(&data).unwrap());
    cpu.write(&0x2000, 0x02);
    assert_eq!(cpu.read(&0x4000), 0xAB);
}

/// MBC1 bank numbers beyond the ROM size are masked, not fatal.
#[test]
fn mbc1_out_of_range_bank_does_not_panic() {
    let mut data = rom(0x01, 0x01, 0x00, 0x10000); // MBC1, 64KB = 4 banks
    data[2 * 0x4000] = 0xCD; // bank 2
    let mut cpu = Cpu::new(Cart::new(&data).unwrap());
    cpu.write(&0x2000, 0x06); // bank 6 masks to bank 2
    assert_eq!(cpu.read(&0x4000), 0xCD);
}

/// MBC1 RAM enable requires exactly 0x0A in the low nibble.
#[test]
fn mbc1_ram_enable_requires_exact_nibble() {
    let data = rom(0x03, 0x01, 0x02, 0x10000); // MBC1+RAM+BATTERY, 8KB RAM
    let mut cpu = Cpu::new(Cart::new(&data).unwrap());
    cpu.write(&0x0000, 0x0A);
    cpu.write(&0xA000, 0x42);
    assert_eq!(cpu.read(&0xA000), 0x42, "RAM must work once enabled");
    cpu.write(&0x0000, 0x0B); // not 0x0A: RAM must be disabled
    assert_eq!(cpu.read(&0xA000), 0xFF);
}

/// MBC1 carts without RAM must not divide by zero on a RAM write.
#[test]
fn mbc1_without_ram_write_does_not_panic() {
    let mut data = rom(0x01, 0x01, 0x00, 0x10000); // MBC1, no RAM
    data[2 * 0x4000] = 0xCD;
    let mut cpu = Cpu::new(Cart::new(&data).unwrap());
    // Guard: prove ROM-area writes reach the MBC, otherwise RAM enable below is a no-op.
    cpu.write(&0x2000, 0x02);
    assert_eq!(
        cpu.read(&0x4000),
        0xCD,
        "ROM-area writes must reach the MBC"
    );
    cpu.write(&0x0000, 0x0A);
    cpu.write(&0xA000, 0x42);
    assert_eq!(cpu.read(&0xA000), 0xFF);
}

/// ROM+RAM carts with no MBC must allocate usable RAM.
#[test]
fn nombc_ram_cart_write() {
    let data = rom(0x08, 0x00, 0x02, 0x8000); // ROM+RAM, 8KB RAM
    let mut cpu = Cpu::new(Cart::new(&data).unwrap());
    cpu.write(&0xA000, 0x5A);
    assert_eq!(cpu.read(&0xA000), 0x5A);
}

/// Reading cart RAM on a cart without RAM returns open bus, not a panic.
#[test]
fn nombc_without_ram_read_does_not_panic() {
    let cpu = cpu_with(&[], 0x150);
    assert_eq!(cpu.read(&0xA000), 0xFF);
}

// --- PPU -----------------------------------------------------------------------------

/// Turning the LCD off resets LY to 0 and frees VRAM/OAM for the CPU.
#[test]
fn lcd_off_resets_ly_and_unlocks_memory() {
    let mut cpu = cpu_with(&[], 0x150);
    while cpu.read(&0xFF44) < 5 {
        cpu.step();
    }
    step_until_mode(&mut cpu, LcdMode::Drawing);
    cpu.write(&0xFF40, 0x00);
    assert_eq!(cpu.read(&0xFF44), 0);
    cpu.write(&0x8000, 0x11);
    cpu.write(&0xFE00, 0x22);
    assert_eq!(cpu.read(&0x8000), 0x11);
    assert_eq!(cpu.read(&0xFE00), 0x22);
}

/// The LYC=LY coincidence flag (STAT bit 2) is set when LY reaches LYC, including in VBlank.
#[test]
fn lyc_coincidence_flag_set_in_vblank() {
    let mut cpu = cpu_with(&[], 0x150);
    cpu.write(&0xFF45, 150);
    for _ in 0..20_000 {
        if cpu.read(&0xFF44) == 150 {
            break;
        }
        cpu.step();
    }
    assert_eq!(cpu.read(&0xFF44), 150);
    assert_ne!(cpu.read(&0xFF41) & 0x04, 0);
}
