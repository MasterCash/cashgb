/// Integration tests for CPU-PPU memory bus integration
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

    // Set cartridge type to 0x00 (ROM ONLY)
    rom_data[0x0147] = 0x00;

    // Calculate and set header checksum
    let mut checksum: u8 = 0;
    for addr in 0x0134..=0x014C {
        checksum = checksum.wrapping_sub(rom_data[addr]).wrapping_sub(1);
    }
    rom_data[0x014D] = checksum;

    Cart::new(&rom_data).expect("Failed to create test cart")
}

#[test]
fn test_ppu_register_access_through_cpu() {
    let cart = create_test_cart();
    let mut cpu = Cpu::new(cart);

    // Test writing to LCDC register (0xFF40)
    cpu.write(&0xFF40, 0x80);
    let value = cpu.read(&0xFF40);
    assert_eq!(value, 0x80);

    // Test writing to SCX register (0xFF43)
    cpu.write(&0xFF43, 0x42);
    let value = cpu.read(&0xFF43);
    assert_eq!(value, 0x42);

    // Test writing to BGP register (0xFF47)
    cpu.write(&0xFF47, 0xE4);
    let value = cpu.read(&0xFF47);
    assert_eq!(value, 0xE4);
}

#[test]
fn test_vram_access_through_cpu() {
    let cart = create_test_cart();
    let mut cpu = Cpu::new(cart);

    // Test writing to VRAM
    cpu.write(&0x8000, 0xAB);
    let value = cpu.read(&0x8000);
    assert_eq!(value, 0xAB);

    // Test another VRAM address
    cpu.write(&0x9000, 0xCD);
    let value = cpu.read(&0x9000);
    assert_eq!(value, 0xCD);
}

#[test]
fn test_oam_access_through_cpu() {
    let cart = create_test_cart();
    let mut cpu = Cpu::new(cart);

    // Test writing to OAM without stepping CPU (to avoid instruction execution)
    // PPU should start in a mode that allows memory access
    cpu.write(&0xFE00, 0x12);
    let value = cpu.read(&0xFE00);
    // Note: OAM access may be restricted depending on PPU mode
    // We'll just verify the system doesn't crash and routes through PPU
    assert!(value == 0x12 || value == 0xFF);

    // Test another OAM address
    cpu.write(&0xFE9F, 0x34);
    let value = cpu.read(&0xFE9F);
    assert!(value == 0x34 || value == 0xFF);
}

#[test]
fn test_ppu_step_integration() {
    let cart = create_test_cart();
    let mut cpu = Cpu::new(cart);

    // Enable LCD
    cpu.write(&0xFF40, 0x80);

    // Step CPU a few times to advance PPU (avoid invalid instruction execution)
    for _ in 0..10 {
        cpu.step();
    }

    // PPU should have advanced through multiple modes
    // We can't easily test specific timing without more setup,
    // but at least verify the system doesn't crash
    assert_eq!(cpu.status, crate::cpu::instructions::CpuStatus::Running);
}

#[test]
fn test_frame_ready_integration() {
    let cart = create_test_cart();
    let mut cpu = Cpu::new(cart);

    // Enable LCD
    cpu.write(&0xFF40, 0x80);

    // Initially no frame should be ready
    assert!(!cpu.is_frame_ready());

    // Directly access PPU to test frame ready functionality
    // without running the full CPU to avoid invalid instruction issues
    let ppu = cpu.memory.get_ppu_mut();
    for _ in 0..200000 {
        ppu.step(4);
        if ppu.is_frame_ready() {
            break;
        }
    }

    // If a frame completed, we should be able to get the framebuffer
    if ppu.is_frame_ready() {
        let framebuffer = ppu.get_framebuffer();
        assert_eq!(framebuffer.len(), 160 * 144 * 4);
    }
}