#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod comprehensive_integration_tests;

#[cfg(test)]
mod cpu_tests {
    use crate::cpu::instructions::*;
    use crate::cpu::Cpu;
    use crate::memory::cart::Cart;

    /// Helper function to create a simple test cart with minimal ROM
    fn create_test_cart() -> Cart {
        // Create a minimal valid Game Boy ROM with proper header
        let mut rom_data = vec![0; 0x8000]; // 32KB ROM

        // Set up minimal Nintendo logo (required for cart validation)
        let nintendo_logo = [
            0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C,
            0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6,
            0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC,
            0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
        ];

        // Copy Nintendo logo to ROM
        for (i, &byte) in nintendo_logo.iter().enumerate() {
            rom_data[0x0104 + i] = byte;
        }

        // Set cart type (0x00 = ROM only)
        rom_data[0x0147] = 0x00;

        // Set ROM size (0x00 = 32KB)
        rom_data[0x0148] = 0x00;

        // Set RAM size (0x00 = No RAM)
        rom_data[0x0149] = 0x00;

        // Calculate and set header checksum
        let mut checksum = 0u8;
        for byte in &rom_data[0x0134..=0x014C] {
            checksum = checksum.wrapping_sub(*byte).wrapping_sub(1);
        }
        rom_data[0x014D] = checksum;

        Cart::new(&rom_data).expect("Failed to create test cart")
    }

    /// Helper function to create a CPU with test cart
    fn create_test_cpu() -> Cpu {
        let cart = create_test_cart();
        Cpu::new(cart)
    }

    #[test]
    fn test_cpu_new_initialization() {
        let cpu = create_test_cpu();

        // Test that CPU is created in Running state
        assert_eq!(cpu.status, CpuStatus::Running);

        // Test register initialization after reset
        assert_eq!(cpu.register.get_af(), 0x01b0);
        assert_eq!(cpu.register.get_bc(), 0x0013);
        assert_eq!(cpu.register.get_de(), 0x00d8);
        assert_eq!(cpu.register.get_hl(), 0x014d);

        // Test initial program counter and stack pointer
        assert_eq!(cpu.program_counter, 0x100);
        assert_eq!(cpu.stack_pointer, 0xfffe);

        // Test interrupt state
        assert!(!cpu.ime);
        assert!(!cpu.ime_next);

        // Test step count
        assert_eq!(cpu.step_count, 0);
    }

    #[test]
    fn test_cpu_reset_state() {
        let mut cpu = create_test_cpu();

        // Modify some state
        cpu.program_counter = 0x200;
        cpu.stack_pointer = 0x1000;
        cpu.ime = true;

        // Reset the CPU
        cpu.reset();

        // Verify reset state
        assert_eq!(cpu.status, CpuStatus::Running);
        assert_eq!(cpu.program_counter, 0x100);
        assert_eq!(cpu.stack_pointer, 0xfffe);
        assert!(!cpu.ime);
        assert!(!cpu.ime_next);

        // Verify registers are reset to expected values
        assert_eq!(cpu.register.get_af(), 0x01b0);
        assert_eq!(cpu.register.get_bc(), 0x0013);
        assert_eq!(cpu.register.get_de(), 0x00d8);
        assert_eq!(cpu.register.get_hl(), 0x014d);
    }

    #[test]
    fn test_cpu_memory_read_write() {
        let mut cpu = create_test_cpu();

        // Test writing and reading from memory
        let test_addr = 0xC000; // Work RAM
        let test_value = 0x42;

        cpu.write(&test_addr, test_value);
        let read_value = cpu.read(&test_addr);

        assert_eq!(read_value, test_value);
    }

    #[test]
    fn test_cpu_step_errored_state() {
        let mut cpu = create_test_cpu();
        cpu.status = CpuStatus::Errored;

        let initial_pc = cpu.program_counter;
        cpu.step();

        // CPU should not advance when in errored state
        assert_eq!(cpu.program_counter, initial_pc);
        assert_eq!(cpu.status, CpuStatus::Errored);
    }

    #[test]
    fn test_cpu_step_stopped_state() {
        let mut cpu = create_test_cpu();
        cpu.status = CpuStatus::Stopped;

        let initial_pc = cpu.program_counter;
        cpu.step();

        // CPU should not advance when stopped
        assert_eq!(cpu.program_counter, initial_pc);
        assert_eq!(cpu.status, CpuStatus::Stopped);
    }

    #[test]
    fn test_cpu_interrupt_handling() {
        let mut cpu = create_test_cpu();

        // Test interrupt request
        cpu.request_interrupt(Interrupt::VBlank);

        // Verify interrupt was requested in memory
        let interrupt_flags = cpu.memory.get_interrupt_flags();
        assert_eq!(
            interrupt_flags & (Interrupt::VBlank as u8),
            Interrupt::VBlank as u8
        );
    }

    #[test]
    fn test_flag_operations() {
        let mut cpu = create_test_cpu();

        // Test setting flags
        cpu.set_flag(Flag::Z);
        assert!(cpu.get_flag(Flag::Z));

        cpu.set_flag(Flag::C);
        assert!(cpu.get_flag(Flag::C));

        // Test clearing flags
        cpu.clear_flag(Flag::Z);
        assert!(!cpu.get_flag(Flag::Z));

        // Test update flag
        cpu.update_flag(Flag::H, true);
        assert!(cpu.get_flag(Flag::H));

        cpu.update_flag(Flag::H, false);
        assert!(!cpu.get_flag(Flag::H));
    }

    #[test]
    fn test_arithmetic_operations() {
        // Test addition helper
        let (result, carry, half_carry) = Cpu::addition(&0x0F, &0x01);
        assert_eq!(result, 0x10);
        assert!(!carry);
        assert!(half_carry);

        // Test overflow
        let (result, carry, half_carry) = Cpu::addition(&0xFF, &0x01);
        assert_eq!(result, 0x00);
        assert!(carry);
        assert!(half_carry);

        // Test subtraction helper
        let (result, borrow, half_borrow) = Cpu::subtract(&0x10, &0x01);
        assert_eq!(result, 0x0F);
        assert!(!borrow);
        assert!(half_borrow); // 0x0 < 0x1, so borrow needed

        // Test underflow
        let (result, borrow, half_borrow) = Cpu::subtract(&0x00, &0x01);
        assert_eq!(result, 0xFF);
        assert!(borrow);
        assert!(half_borrow);
    }

    #[test]
    fn test_nop_instruction() {
        let mut cpu = create_test_cpu();

        // Set up NOP instruction (0x00) at current PC
        let pc = cpu.program_counter;
        cpu.write(&pc, 0x00);

        let initial_pc = cpu.program_counter;
        let initial_registers = cpu.register;

        // Execute one step (should fetch and execute NOP)
        cpu.step();

        // NOP should only advance PC
        assert_eq!(cpu.program_counter, initial_pc + 1);
        assert_eq!(cpu.register.get_af(), initial_registers.get_af());
        assert_eq!(cpu.register.get_bc(), initial_registers.get_bc());
    }

    #[test]
    fn test_ime_flag_handling() {
        let mut cpu = create_test_cpu();

        // Test initial state
        assert!(!cpu.ime);
        assert!(!cpu.ime_next);

        // Test ime_next handling
        cpu.ime_next = true;
        cpu.step(); // This should set ime to true

        assert!(cpu.ime);
        assert!(!cpu.ime_next); // Should be reset after step
    }

    #[test]
    fn test_step_count_handling() {
        let mut cpu = create_test_cpu();

        // Set up an instruction that takes multiple cycles
        // We'll manually set step_count to test the timing
        cpu.step_count = 3;
        let initial_pc = cpu.program_counter;

        // First step should just decrement step_count
        cpu.step();
        assert_eq!(cpu.step_count, 2);
        assert_eq!(cpu.program_counter, initial_pc); // PC shouldn't advance

        // Second step
        cpu.step();
        assert_eq!(cpu.step_count, 1);
        assert_eq!(cpu.program_counter, initial_pc);

        // Third step should process instruction
        cpu.step();
        assert_eq!(cpu.step_count, 0);
    }
}
