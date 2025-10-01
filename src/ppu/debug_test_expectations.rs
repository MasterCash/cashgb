/// Debug test to understand sprite test expectations
use crate::ppu::Ppu;

#[test]
fn debug_test_expectations() {
    let mut ppu = Ppu::new();

    // Enable LCD and sprites
    ppu.registers.write_lcdc(0x83); // LCD on, background on, sprites on
    ppu.registers.set_obp0(0xE4); // Standard Game Boy palette for sprites

    // Check what the background color should be
    let bg_color_0 = ppu.registers.get_bg_color(0);
    println!("Background color 0 maps to palette color: {}", bg_color_0);

    // Check what sprite colors map to
    for i in 0..4 {
        let sprite_color = ppu.registers.get_sprite_color(0, i);
        println!("Sprite palette 0, color {} maps to: {}", i, sprite_color);
    }

    // BGP = 0xFC means color 0 maps to palette color 0
    // This is correct Game Boy behavior
    assert_eq!(bg_color_0, 0, "With default BGP=0xFC, color 0 should map to palette color 0");
}