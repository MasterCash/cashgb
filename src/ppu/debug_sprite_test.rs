/// Debug test to understand sprite rendering issues
use crate::ppu::Ppu;

#[test]
fn debug_sprite_basic() {
    let mut ppu = Ppu::new();

    // Enable LCD and sprites
    ppu.registers.write_lcdc(0x83); // LCD on, background on, sprites on
    ppu.registers.set_obp0(0xE4); // Standard palette

    // Create simple sprite tile (all pixels color 1)
    for i in 0..16 {
        ppu.vram[i] = if i % 2 == 0 { 0xFF } else { 0x00 }; // All color 1
    }

    // Create sprite at Y=16, X=16, tile=0
    ppu.oam[0] = 16; // Y
    ppu.oam[1] = 16; // X
    ppu.oam[2] = 0;  // Tile
    ppu.oam[3] = 0;  // Attributes

    println!("Before OAM scan - sprite count: {}", ppu.sprite_renderer.sprite_count());

    // Manual OAM scan for line 0
    ppu.scanline = 0;
    let sprite_height = if ppu.registers.sprite_size() { 16 } else { 8 };
    ppu.sprite_renderer.scan_sprites_for_line(&ppu.oam, ppu.scanline, sprite_height);

    println!("After OAM scan - sprite count: {}", ppu.sprite_renderer.sprite_count());

    // Check if sprite should be on this line
    // Sprite Y=16 means screen Y=0, so it should be visible on line 0
    assert_eq!(ppu.sprite_renderer.sprite_count(), 1, "Sprite should be found on line 0");

    // Now render
    ppu.render_scanline();

    // Check sprite buffer
    println!("Sprite buffer at X=8: {}", ppu.sprite_buffer[8]);
    println!("Sprite priority buffer at X=8: {}", ppu.sprite_priority_buffer[8]);
    println!("Line buffer at X=8: {}", ppu.line_buffer[8]);

    // Debug: print first 16 pixels of each buffer
    print!("BG buffer: ");
    for x in 0..16 {
        print!("{} ", ppu.line_buffer[x]);
    }
    println!();

    print!("Sprite buffer: ");
    for x in 0..16 {
        print!("{} ", ppu.sprite_buffer[x]);
    }
    println!();

    print!("Priority buffer: ");
    for x in 0..16 {
        print!("{} ", if ppu.sprite_priority_buffer[x] { 1 } else { 0 });
    }
    println!();
}