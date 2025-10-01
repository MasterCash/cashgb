/// Direct PPU demonstration without running CPU instructions
/// This shows how TerminalDisplay works by directly manipulating the PPU
use cash_gb::ppu::Ppu;
use cash_gb::ppu::display::{Display, TerminalDisplay};
use log::LevelFilter;

fn main() {
    // Initialize logger - set to Info level to reduce debug spam
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .init();
    println!("PPU Direct Terminal Display Demo");
    println!("================================\n");

    // Create PPU and display
    let mut ppu = Ppu::new();
    let mut display = TerminalDisplay::new(true); // Use color

    // Enable LCD and background
    ppu.registers.write_lcdc(0x91); // LCD on, background on, sprites on
    ppu.registers.set_bgp(0xE4);    // Background palette: 11100100 (dark to light)

    // Create some simple background tile data
    // Tile 0: Checkerboard pattern
    let tile_data = [
        0xAA, 0x55, // Row 0: alternating pattern
        0x55, 0xAA, // Row 1: inverted pattern
        0xAA, 0x55, // Row 2: alternating pattern
        0x55, 0xAA, // Row 3: inverted pattern
        0xAA, 0x55, // Row 4: alternating pattern
        0x55, 0xAA, // Row 5: inverted pattern
        0xAA, 0x55, // Row 6: alternating pattern
        0x55, 0xAA, // Row 7: inverted pattern
    ];

    // Write tile data to VRAM
    for (i, &byte) in tile_data.iter().enumerate() {
        ppu.write_vram(0x8000 + i as u16, byte);
    }

    // Set up background tilemap to use our tile
    for i in 0..32*32 {
        ppu.write_vram(0x9800 + i, 0); // All tiles use tile 0
    }

    // Create some sprite data for demonstration
    // Sprite 0: Y=50, X=50, Tile=0, Attributes=0
    ppu.write_oam(0xFE00, 50);  // Y position
    ppu.write_oam(0xFE01, 50);  // X position
    ppu.write_oam(0xFE02, 0);   // Tile number
    ppu.write_oam(0xFE03, 0);   // Attributes

    println!("Stepping PPU through a complete frame...");

    let mut frame_count = 0;
    let mut cycles = 0;
    const MAX_CYCLES: u32 = 200000; // Prevent infinite loop

    // Step the PPU until we get a frame
    while !ppu.is_frame_ready() && cycles < MAX_CYCLES {
        ppu.step(4); // Step by 4 cycles (typical CPU step size)
        cycles += 4;
    }

    if ppu.is_frame_ready() {
        println!("Frame ready after {} cycles!", cycles);
        let framebuffer = ppu.get_framebuffer();
        display.present_frame(framebuffer);
        frame_count += 1;
    } else {
        println!("Failed to generate frame within {} cycles", cycles);

        // Show PPU state for debugging
        println!("PPU State:");
        println!("  Current line: {}", ppu.get_current_line());
        println!("  Current mode: {:?}", ppu.get_current_mode());
        println!("  LCD enabled: {}", ppu.registers.is_lcd_enabled());
    }

    println!("\nDemo complete! Generated {} frame(s)", frame_count);
}