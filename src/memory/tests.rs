use crate::memory::MemoryBus;

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
