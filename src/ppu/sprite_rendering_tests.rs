/// Comprehensive tests for sprite rendering functionality
use crate::ppu::Ppu;
use crate::ppu::display::ColorConverter;

fn create_test_ppu_with_sprites() -> Ppu {
    let mut ppu = Ppu::new();

    // Enable LCD and sprites
    ppu.registers.write_lcdc(0x83); // LCD on, background on, sprites on

    // Set up sprite palettes
    ppu.registers.set_obp0(0xE4); // Standard Game Boy palette for sprites
    ppu.registers.set_obp1(0xD2); // Different palette for testing

    ppu
}

fn create_test_sprite_tile_data(ppu: &mut Ppu) {
    // Create a simple test sprite tile (8x8 pixels)
    // Pattern: all pixels are color 1 (LSB set, MSB clear)
    let tile_data = [
        0b11111111, 0b00000000, // Row 0: all color 1
        0b11111111, 0b00000000, // Row 1: all color 1
        0b11111111, 0b00000000, // Row 2: all color 1
        0b11111111, 0b00000000, // Row 3: all color 1
        0b11111111, 0b00000000, // Row 4: all color 1
        0b11111111, 0b00000000, // Row 5: all color 1
        0b11111111, 0b00000000, // Row 6: all color 1
        0b11111111, 0b00000000, // Row 7: all color 1
    ];

    // Write tile data to VRAM at tile 0
    for (i, &byte) in tile_data.iter().enumerate() {
        ppu.vram[i] = byte;
    }
}

fn create_test_sprite_oam_data(ppu: &mut Ppu, sprite_index: usize, y: u8, x: u8, tile: u8, attrs: u8) {
    let oam_offset = sprite_index * 4;
    ppu.oam[oam_offset] = y;
    ppu.oam[oam_offset + 1] = x;
    ppu.oam[oam_offset + 2] = tile;
    ppu.oam[oam_offset + 3] = attrs;
}

/// Helper function to render a scanline with proper OAM scan
fn render_line_with_oam_scan(ppu: &mut Ppu, line: u8) {
    ppu.scanline = line;

    // Perform OAM scan (normally done during OAM scan mode)
    let sprite_height = if ppu.registers.sprite_size() { 16 } else { 8 };
    ppu.sprite_renderer.scan_sprites_for_line(&ppu.oam, ppu.scanline, sprite_height);

    // Render the line
    ppu.render_scanline();
}

#[test]
fn test_sprite_rendering_basic() {
    let mut ppu = create_test_ppu_with_sprites();
    create_test_sprite_tile_data(&mut ppu);

    // Create a sprite at position (16, 16) using tile 0
    create_test_sprite_oam_data(&mut ppu, 0, 16, 16, 0, 0x00); // No flipping, palette 0, priority

    // Test rendering at line 0 (sprite Y=16 means screen Y=0)
    render_line_with_oam_scan(&mut ppu, 0);

    // Sprite should be visible from screen X=8 (sprite X=16 means screen X=8)
    // Check that sprite pixels were rendered
    let expected_bg = ppu.registers.get_bg_color(0);
    let expected_sprite = ppu.registers.get_sprite_color(0, 1); // Color 1 from our test pattern

    for x in 8..16 {
        let color = ppu.line_buffer[x];
        // Should be sprite color, not background color
        assert_eq!(color, expected_sprite, "Sprite pixel should be rendered at X={}", x);
    }

    // Pixels outside sprite area should be background
    for x in 0..8 {
        let color = ppu.line_buffer[x];
        assert_eq!(color, expected_bg, "Background pixel should be rendered at X={}", x);
    }
}

#[test]
fn test_sprite_transparency() {
    let mut ppu = create_test_ppu_with_sprites();

    // Create sprite tile with transparent pixels (color 0)
    let tile_data = [
        0b00000000, 0b00000000, // Row 0: all transparent
        0b11111111, 0b00000000, // Row 1: all color 1 (opaque)
        0b00000000, 0b11111111, // Row 2: all color 2 (opaque)
        0b11111111, 0b11111111, // Row 3: all color 3 (opaque)
        0b00000000, 0b00000000, // Row 4: all transparent
        0b00000000, 0b00000000, // Row 5: all transparent
        0b00000000, 0b00000000, // Row 6: all transparent
        0b00000000, 0b00000000, // Row 7: all transparent
    ];

    for (i, &byte) in tile_data.iter().enumerate() {
        ppu.vram[i] = byte;
    }

    create_test_sprite_oam_data(&mut ppu, 0, 16, 16, 0, 0x00);

    // Test line 0 (first row - transparent)
    ppu.scanline = 0;
    ppu.render_scanline();

    // All pixels should be background (sprite transparent)
    for x in 8..16 {
        let color = ppu.line_buffer[x];
        let expected_bg = ppu.registers.get_bg_color(0);
        assert_eq!(color, expected_bg, "Transparent sprite pixel should show background at X={}", x);
    }

    // Test line 1 (second row - opaque color 1)
    render_line_with_oam_scan(&mut ppu, 1);

    // Sprite pixels should be rendered
    for x in 8..16 {
        let color = ppu.line_buffer[x];
        let expected_sprite = ppu.registers.get_sprite_color(0, 1);
        assert_eq!(color, expected_sprite, "Opaque sprite pixel should be rendered at X={}", x);
    }
}

#[test]
fn test_sprite_priority() {
    let mut ppu = create_test_ppu_with_sprites();
    create_test_sprite_tile_data(&mut ppu);

    // Create a high priority sprite (bit 7 = 0)
    create_test_sprite_oam_data(&mut ppu, 0, 16, 16, 0, 0x00);

    // Create a low priority sprite (bit 7 = 1)
    create_test_sprite_oam_data(&mut ppu, 1, 16, 20, 0, 0x80);

    render_line_with_oam_scan(&mut ppu, 0);

    // High priority sprite should be visible at X=8-15
    for x in 8..16 {
        let color = ppu.line_buffer[x];
        assert_ne!(color, 0, "High priority sprite should be rendered at X={}", x);
    }

    // Low priority sprite area depends on background - if BG is color 0, sprite shows
    // If BG is non-zero, sprite is hidden by priority rules
}

#[test]
fn test_sprite_palette_selection() {
    let mut ppu = create_test_ppu_with_sprites();
    create_test_sprite_tile_data(&mut ppu);

    // Create sprite using palette 0
    create_test_sprite_oam_data(&mut ppu, 0, 16, 16, 0, 0x00);

    // Create sprite using palette 1
    create_test_sprite_oam_data(&mut ppu, 1, 16, 32, 0, 0x10);

    render_line_with_oam_scan(&mut ppu, 0);

    // First sprite should use OBP0 palette
    let color0 = ppu.line_buffer[8];
    let expected0 = ppu.registers.get_sprite_color(0, 1); // Palette 0, color 1
    assert_eq!(color0, expected0, "First sprite should use palette 0");

    // Second sprite should use OBP1 palette
    let color1 = ppu.line_buffer[24]; // X=32 -> screen X=24
    let expected1 = ppu.registers.get_sprite_color(1, 1); // Palette 1, color 1
    assert_eq!(color1, expected1, "Second sprite should use palette 1");
}

#[test]
fn test_sprite_horizontal_flipping() {
    let mut ppu = create_test_ppu_with_sprites();

    // Create asymmetric sprite pattern
    let tile_data = [
        0b11110000, 0b00000000, // Row 0: left half color 1, right half color 0
        0b11110000, 0b00000000,
        0b11110000, 0b00000000,
        0b11110000, 0b00000000,
        0b11110000, 0b00000000,
        0b11110000, 0b00000000,
        0b11110000, 0b00000000,
        0b11110000, 0b00000000,
    ];

    for (i, &byte) in tile_data.iter().enumerate() {
        ppu.vram[i] = byte;
    }

    // Create normal sprite
    create_test_sprite_oam_data(&mut ppu, 0, 16, 16, 0, 0x00);

    // Create horizontally flipped sprite
    create_test_sprite_oam_data(&mut ppu, 1, 16, 32, 0, 0x20);

    render_line_with_oam_scan(&mut ppu, 0);

    // Normal sprite: left side should be opaque, right side background
    let left_normal = ppu.line_buffer[8];  // First pixel of normal sprite
    let right_normal = ppu.line_buffer[15]; // Last pixel of normal sprite
    let bg_color = ppu.registers.get_bg_color(0);

    assert_ne!(left_normal, bg_color, "Normal sprite left should be opaque");
    assert_eq!(right_normal, bg_color, "Normal sprite right should be transparent");

    // Flipped sprite: right side should be opaque, left side background
    let left_flipped = ppu.line_buffer[24];  // First pixel of flipped sprite
    let right_flipped = ppu.line_buffer[31]; // Last pixel of flipped sprite

    assert_eq!(left_flipped, bg_color, "Flipped sprite left should be transparent");
    assert_ne!(right_flipped, bg_color, "Flipped sprite right should be opaque");
}

#[test]
fn test_sprite_vertical_flipping() {
    let mut ppu = create_test_ppu_with_sprites();

    // Create asymmetric sprite pattern (different top and bottom)
    let tile_data = [
        0b11111111, 0b00000000, // Row 0: top - all color 1
        0b11111111, 0b00000000,
        0b11111111, 0b00000000,
        0b11111111, 0b00000000,
        0b00000000, 0b11111111, // Row 4: bottom - all color 2
        0b00000000, 0b11111111,
        0b00000000, 0b11111111,
        0b00000000, 0b11111111,
    ];

    for (i, &byte) in tile_data.iter().enumerate() {
        ppu.vram[i] = byte;
    }

    // Create normal sprite
    create_test_sprite_oam_data(&mut ppu, 0, 16, 16, 0, 0x00);

    // Create vertically flipped sprite
    create_test_sprite_oam_data(&mut ppu, 1, 16, 32, 0, 0x40);

    // Test top row (line 0)
    render_line_with_oam_scan(&mut ppu, 0);

    let normal_top = ppu.line_buffer[8];
    let flipped_top = ppu.line_buffer[24];

    let color1 = ppu.registers.get_sprite_color(0, 1);
    let color2 = ppu.registers.get_sprite_color(0, 2);

    assert_eq!(normal_top, color1, "Normal sprite top should be color 1");
    assert_eq!(flipped_top, color2, "Flipped sprite top should be color 2 (flipped from bottom)");
}

#[test]
fn test_sprite_8x16_mode() {
    let mut ppu = create_test_ppu_with_sprites();

    // Enable 8x16 sprite mode
    ppu.registers.write_lcdc(0x87); // LCD on, background on, sprites on, 8x16 sprites

    // Create tile data for 8x16 sprite (uses tiles 0 and 1)
    // Top tile (tile 0)
    let top_tile = [
        0b11111111, 0b00000000, // All color 1
        0b11111111, 0b00000000,
        0b11111111, 0b00000000,
        0b11111111, 0b00000000,
        0b11111111, 0b00000000,
        0b11111111, 0b00000000,
        0b11111111, 0b00000000,
        0b11111111, 0b00000000,
    ];

    // Bottom tile (tile 1)
    let bottom_tile = [
        0b00000000, 0b11111111, // All color 2
        0b00000000, 0b11111111,
        0b00000000, 0b11111111,
        0b00000000, 0b11111111,
        0b00000000, 0b11111111,
        0b00000000, 0b11111111,
        0b00000000, 0b11111111,
        0b00000000, 0b11111111,
    ];

    // Write tile data
    for (i, &byte) in top_tile.iter().enumerate() {
        ppu.vram[i] = byte;
    }
    for (i, &byte) in bottom_tile.iter().enumerate() {
        ppu.vram[16 + i] = byte; // Tile 1 starts at offset 16
    }

    // Create 8x16 sprite using tile 0 (will automatically use tiles 0 and 1)
    create_test_sprite_oam_data(&mut ppu, 0, 16, 16, 0, 0x00);

    // Test top half (line 0)
    render_line_with_oam_scan(&mut ppu, 0);

    let top_color = ppu.line_buffer[8];
    let expected_top = ppu.registers.get_sprite_color(0, 1);
    assert_eq!(top_color, expected_top, "8x16 sprite top should use top tile");

    // Test bottom half (line 8)
    render_line_with_oam_scan(&mut ppu, 8);

    let bottom_color = ppu.line_buffer[8];
    let expected_bottom = ppu.registers.get_sprite_color(0, 2);
    assert_eq!(bottom_color, expected_bottom, "8x16 sprite bottom should use bottom tile");
}

#[test]
fn test_sprite_oam_scan_integration() {
    let mut ppu = create_test_ppu_with_sprites();
    create_test_sprite_tile_data(&mut ppu);

    // Create multiple sprites on the same line
    for i in 0..5 {
        create_test_sprite_oam_data(&mut ppu, i, 16, 16 + (i as u8 * 16), 0, 0x00);
    }

    // Manually trigger OAM scan
    ppu.scanline = 0;
    let sprite_height = if ppu.registers.sprite_size() { 16 } else { 8 };
    ppu.sprite_renderer.scan_sprites_for_line(&ppu.oam, ppu.scanline, sprite_height);

    // All 5 sprites should be found
    assert_eq!(ppu.sprite_renderer.sprite_count(), 5);

    // Test rendering with all sprites
    ppu.render_scanline();

    // Each sprite should contribute pixels
    for i in 0..5 {
        let sprite_x = 8 + (i * 16); // Screen X positions
        if sprite_x < 160 {
            let color = ppu.line_buffer[sprite_x];
            assert_ne!(color, 0, "Sprite {} should be rendered at X={}", i, sprite_x);
        }
    }
}

#[test]
fn test_sprite_background_interaction() {
    let mut ppu = create_test_ppu_with_sprites();

    // Set up background tile data (non-zero pattern)
    for i in 0..16 {
        ppu.vram[i] = if i % 2 == 0 { 0xFF } else { 0x00 }; // Alternating pattern
    }

    // Set up background tilemap to use tile 0
    ppu.vram[0x1800] = 0; // First tile in tilemap uses tile 0

    create_test_sprite_tile_data(&mut ppu);

    // Create high priority sprite
    create_test_sprite_oam_data(&mut ppu, 0, 16, 16, 0, 0x00); // Priority = 0 (high)

    // Create low priority sprite
    create_test_sprite_oam_data(&mut ppu, 1, 16, 32, 0, 0x80); // Priority = 1 (low)

    render_line_with_oam_scan(&mut ppu, 0);

    // High priority sprite should always show over background
    let high_priority_color = ppu.line_buffer[8];
    assert_ne!(high_priority_color, 0, "High priority sprite should be visible");

    // Low priority sprite visibility depends on background color
    // This test verifies the priority system is working
    let _low_priority_color = ppu.line_buffer[24];
    // Actual behavior depends on specific background color at that position
}

#[test]
fn test_sprite_render_pipeline_integration() {
    let mut ppu = create_test_ppu_with_sprites();
    create_test_sprite_tile_data(&mut ppu);
    create_test_sprite_oam_data(&mut ppu, 0, 16, 16, 0, 0x00);

    // Test complete render pipeline
    ppu.scanline = 0;

    // 1. OAM Scan should find sprites
    let sprite_height = if ppu.registers.sprite_size() { 16 } else { 8 };
    ppu.sprite_renderer.scan_sprites_for_line(&ppu.oam, ppu.scanline, sprite_height);
    assert_eq!(ppu.sprite_renderer.sprite_count(), 1);

    // 2. Rendering should work without panic
    ppu.render_scanline();

    // 3. Framebuffer should be updated
    let pixel_start = 8 * 4; // X=8, first sprite pixel
    let r = ppu.framebuffer[pixel_start];
    let g = ppu.framebuffer[pixel_start + 1];
    let b = ppu.framebuffer[pixel_start + 2];
    let a = ppu.framebuffer[pixel_start + 3];

    // Should be valid Game Boy color (not black unless that's the actual color)
    assert_eq!(a, 255, "Alpha should be fully opaque");

    // Color should match expected Game Boy palette
    let color_index = ppu.line_buffer[8];
    let (expected_r, expected_g, expected_b) = ColorConverter::gb_color_to_rgb(color_index);
    assert_eq!((r, g, b), (expected_r, expected_g, expected_b),
              "Framebuffer should match color conversion");
}