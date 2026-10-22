use crate::cpu::instructions::Interrupt;
use crate::memory::{MemoryBus, joypad::Button};

/// Turn the LCD off so OAM reads aren't blocked by the PPU mode.
fn bus_with_lcd_off() -> MemoryBus {
    let mut bus = MemoryBus::empty();
    bus.write(0xFF40, 0x00);
    bus
}

#[test]
fn test_oam_dma_copies_160_bytes() {
    let mut bus = bus_with_lcd_off();
    for i in 0..0xA0u16 {
        bus.write(0xC000 + i, i as u8);
    }

    bus.write(0xFF46, 0xC0);

    for i in 0..0xA0u16 {
        assert_eq!(bus.read(0xFE00 + i), i as u8, "OAM byte {i:#04x}");
    }
}

#[test]
fn test_oam_dma_register_reads_back_last_value() {
    let mut bus = bus_with_lcd_off();

    bus.write(0xFF46, 0xC1);

    assert_eq!(bus.read(0xFF46), 0xC1);
}

#[test]
fn test_oam_dma_writes_while_oam_is_locked() {
    let mut bus = MemoryBus::empty(); // LCD on, PPU starts in OAM Scan
    bus.write(0xC000, 0x99);

    bus.write(0xFF46, 0xC0);
    bus.write(0xFF40, 0x00); // unlock OAM to read it back

    assert_eq!(bus.read(0xFE00), 0x99);
}

#[test]
fn test_joypad_register_routed_through_bus() {
    let mut bus = MemoryBus::empty();
    bus.write(0xFF00, 0x10); // select buttons
    bus.set_button(Button::A, true);

    assert_eq!(bus.read(0xFF00), 0xD0 | 0b1110);
}

#[test]
fn test_joypad_press_requests_interrupt() {
    let mut bus = MemoryBus::empty();
    bus.write(0xFF00, 0x20); // select D-pad

    assert!(bus.set_button(Button::Up, true));
    assert_ne!(bus.get_interrupt_flags() & Interrupt::Joypad as u8, 0);
}

#[test]
fn test_joypad_press_in_unselected_group_does_not_request_interrupt() {
    let mut bus = MemoryBus::empty();
    bus.write(0xFF00, 0x20); // select D-pad

    assert!(!bus.set_button(Button::Start, true));
    assert_eq!(bus.get_interrupt_flags() & Interrupt::Joypad as u8, 0);
}

#[test]
fn test_peek_sees_vram_and_oam_while_locked() {
    let mut bus = bus_with_lcd_off();
    bus.write(0x8000, 0x12);
    bus.write(0xFE00, 0x34);
    bus.write(0xFF40, 0x91); // LCD on: PPU starts in OAM Scan, which locks OAM

    assert_eq!(bus.read(0xFE00), 0xFF, "CPU read is blocked");
    assert_eq!(bus.peek(0xFE00), 0x34);
    assert_eq!(bus.peek(0x8000), 0x12);
}

#[test]
fn test_unmapped_io_reads_ff_and_ignores_writes() {
    let mut bus = MemoryBus::empty();
    for addr in [
        0xFF03, 0xFF08, 0xFF27, 0xFF4C, 0xFF4D, 0xFF4F, 0xFF50, 0xFF70, 0xFF7F,
    ] {
        bus.write(addr, 0x00);
        assert_eq!(bus.read(addr), 0xFF, "{addr:#06X}");
    }
}

#[test]
fn test_cgb_wram_bank_select_has_no_effect_on_dmg() {
    let mut bus = MemoryBus::empty();
    bus.write(0xD000, 0x42);

    bus.write(0xFF70, 0x00); // would select WRAM bank 0 (as bank 1) on a CGB

    assert_eq!(bus.read(0xD000), 0x42);
}

#[test]
fn test_interrupt_flags_top_bits_read_as_one() {
    let mut bus = MemoryBus::empty();
    bus.write(0xFF0F, 0x00);
    assert_eq!(bus.read(0xFF0F), 0xE0);

    bus.request_interrupt(Interrupt::Timer);
    assert_eq!(bus.read(0xFF0F), 0xE4);
}

#[test]
fn test_interrupt_flags_never_report_nonexistent_interrupts() {
    let mut bus = MemoryBus::empty();
    bus.write(0xFF0F, 0xFF);

    assert_eq!(
        bus.get_interrupt_flags(),
        0x1F,
        "only the five real sources"
    );
}
