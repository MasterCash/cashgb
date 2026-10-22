/// Regression tests from the PPU adversarial code review.
///
/// Each test asserts correct DMG hardware behaviour for one reported bug, so a failing
/// test here means that bug is still present. `line_buffer` holds final shades (0-3)
/// after compositing; the palettes below are chosen so each layer has a distinct shade.
use crate::ppu::Ppu;
use crate::ppu::display::ColorConverter;

/// Identity palette: colour id N maps to shade N.
const IDENTITY: u8 = 0xE4;

/// Scan OAM and render one scanline, as the PPU does between OAM Scan and HBlank.
fn render_line(ppu: &mut Ppu, line: u8) {
    ppu.scanline = line;
    let height = if ppu.registers.sprite_size() { 16 } else { 8 };
    ppu.sprite_renderer
        .scan_sprites_for_line(&ppu.oam, line, height);
    ppu.render_scanline();
}

fn set_sprite(ppu: &mut Ppu, index: usize, y: u8, x: u8, tile: u8, attrs: u8) {
    ppu.oam[index * 4..index * 4 + 4].copy_from_slice(&[y, x, tile, attrs]);
}

/// Fill a tile in the 0x8000 area with one colour id.
fn solid_tile(ppu: &mut Ppu, tile: usize, color: u8) {
    let low = if color & 1 != 0 { 0xFF } else { 0x00 };
    let high = if color & 2 != 0 { 0xFF } else { 0x00 };
    for row in 0..8 {
        ppu.vram[tile * 16 + row * 2] = low;
        ppu.vram[tile * 16 + row * 2 + 1] = high;
    }
}

/// Step the PPU by whole M-cycles until LY equals `line`.
fn step_to_line(ppu: &mut Ppu, line: u8) {
    for _ in 0..40_000 {
        if ppu.registers.get_ly() == line {
            return;
        }
        ppu.step(4);
    }
    panic!("LY never reached {line}");
}

fn rgb_at(ppu: &Ppu, x: usize, y: usize) -> (u8, u8, u8) {
    let i = (y * 160 + x) * 4;
    (
        ppu.framebuffer[i],
        ppu.framebuffer[i + 1],
        ppu.framebuffer[i + 2],
    )
}

// --- Background and window tile decoding ---------------------------------------------

/// Bit 7 of each tile row byte is the leftmost pixel.
#[test]
fn bg_tile_leftmost_pixel_is_bit7() {
    let mut ppu = Ppu::new();
    ppu.registers.write_lcdc(0x91);
    ppu.registers.set_bgp(IDENTITY);
    ppu.vram[0] = 0x80; // tile 0, row 0: only the leftmost pixel is colour 1
    render_line(&mut ppu, 0);
    assert_eq!(ppu.line_buffer[0], 1, "leftmost pixel");
    assert_eq!(ppu.line_buffer[7], 0, "rightmost pixel");
}

/// The window decodes tile rows with the same bit order as the background.
#[test]
fn window_tile_leftmost_pixel_is_bit7() {
    let mut ppu = Ppu::new();
    ppu.registers.write_lcdc(0xF1); // window on, window map 0x9C00, 0x8000 data, BG on
    ppu.registers.set_bgp(IDENTITY);
    ppu.registers.set_wy(0);
    ppu.registers.set_wx(7);
    ppu.vram[2 * 16] = 0x80; // tile 2, row 0: only the leftmost pixel is colour 1
    ppu.vram[0x1C00] = 2;
    render_line(&mut ppu, 0);
    assert_eq!(ppu.line_buffer[0], 1, "leftmost pixel");
    assert_eq!(ppu.line_buffer[7], 0, "rightmost pixel");
}

/// With LCDC bit 0 clear the background shows white (shade 0), whatever BGP says.
#[test]
fn bg_disabled_shows_white() {
    let mut ppu = Ppu::new();
    ppu.registers.write_lcdc(0x90); // LCD on, BG/window off
    ppu.registers.set_bgp(0xFF); // every colour id maps to shade 3
    render_line(&mut ppu, 0);
    assert_eq!(ppu.line_buffer[0], 0);
}

// --- Window line counter -------------------------------------------------------------

/// The window line counter must not overflow when the window runs for several frames.
#[test]
fn window_counter_survives_multiple_frames() {
    let mut ppu = Ppu::new();
    ppu.registers.write_lcdc(0xB1); // window on, 0x8000 data, BG on
    ppu.registers.set_wy(0);
    ppu.registers.set_wx(7);
    for _ in 0..(3 * 70224 / 4) {
        ppu.step(4);
    }
}

/// The window starts again from map row 0 at the top of every frame.
#[test]
fn window_row_restarts_each_frame() {
    let mut ppu = Ppu::new();
    ppu.registers.write_lcdc(0xF1);
    ppu.registers.set_bgp(IDENTITY);
    ppu.registers.set_wy(0);
    ppu.registers.set_wx(7);
    solid_tile(&mut ppu, 2, 2);
    ppu.vram[0x1C00] = 2; // window map row 0, col 0 -> colour 2

    for _ in 0..(70224 / 4) {
        ppu.step(4); // one full frame
    }
    step_to_line(&mut ppu, 1); // line 0 of frame 2 has been drawn

    assert_eq!(rgb_at(&ppu, 0, 0), ColorConverter::gb_color_to_rgb(2));
}

/// The counter only advances on lines where the window is actually drawn.
#[test]
fn window_counter_skips_lines_where_window_is_offscreen() {
    let mut ppu = Ppu::new();
    ppu.registers.write_lcdc(0xF1);
    ppu.registers.set_bgp(IDENTITY);
    ppu.registers.set_wy(0);
    solid_tile(&mut ppu, 2, 2);
    ppu.vram[0x1C00] = 2; // window map row 0 -> colour 2; row 1 stays tile 0 (colour 0)

    ppu.registers.set_wx(200); // window off the right edge for lines 0-9
    for line in 0..10 {
        render_line(&mut ppu, line);
    }
    ppu.registers.set_wx(7);
    render_line(&mut ppu, 10);

    assert_eq!(
        ppu.line_buffer[0], 2,
        "first drawn window line is window row 0"
    );
}

/// WX below 7 pushes the window partly off the left edge.
#[test]
fn window_wx_below_7_shifts_left() {
    let mut ppu = Ppu::new();
    ppu.registers.write_lcdc(0xF1);
    ppu.registers.set_bgp(IDENTITY);
    ppu.registers.set_wy(0);
    ppu.registers.set_wx(3); // window starts 4 pixels left of the screen
    ppu.vram[2 * 16] = 0x08; // tile 2, row 0: window pixel 4 is colour 3
    ppu.vram[2 * 16 + 1] = 0x08;
    ppu.vram[0x1C00] = 2;
    render_line(&mut ppu, 0);
    assert_eq!(ppu.line_buffer[0], 3, "screen x 0 shows window pixel 4");
    assert_eq!(ppu.line_buffer[1], 0);
}

// --- Sprites -------------------------------------------------------------------------

/// Where sprites overlap, the one with the lower X coordinate is drawn.
#[test]
fn overlapping_sprites_lower_x_wins() {
    let mut ppu = Ppu::new();
    ppu.registers.write_lcdc(0x93);
    solid_tile(&mut ppu, 1, 1);
    ppu.registers.set_obp0(0x04); // colour 1 -> shade 1
    ppu.registers.set_obp1(0x0C); // colour 1 -> shade 3
    set_sprite(&mut ppu, 0, 16, 20, 1, 0x10); // screen x 12-19, OBP1
    set_sprite(&mut ppu, 1, 16, 16, 1, 0x00); // screen x 8-15, OBP0 (lower X)
    render_line(&mut ppu, 0);
    assert_eq!(ppu.line_buffer[12], 1);
}

/// With equal X, the sprite earlier in OAM is drawn.
#[test]
fn overlapping_sprites_equal_x_lower_oam_index_wins() {
    let mut ppu = Ppu::new();
    ppu.registers.write_lcdc(0x93);
    solid_tile(&mut ppu, 1, 1);
    ppu.registers.set_obp0(0x04);
    ppu.registers.set_obp1(0x0C);
    set_sprite(&mut ppu, 0, 16, 16, 1, 0x00); // OBP0
    set_sprite(&mut ppu, 1, 16, 16, 1, 0x10); // OBP1, same position
    render_line(&mut ppu, 0);
    assert_eq!(ppu.line_buffer[8], 1);
}

/// Turning sprites off (LCDC bit 1) hides them immediately.
#[test]
fn disabling_sprites_hides_them() {
    let mut ppu = Ppu::new();
    ppu.registers.write_lcdc(0x93);
    ppu.registers.set_bgp(IDENTITY);
    ppu.registers.set_obp0(IDENTITY);
    solid_tile(&mut ppu, 1, 3);
    set_sprite(&mut ppu, 0, 16, 16, 1, 0x00);
    render_line(&mut ppu, 0);
    assert_eq!(ppu.line_buffer[8], 3, "sprite drawn while enabled");

    ppu.registers.write_lcdc(0x91);
    render_line(&mut ppu, 0);
    assert_eq!(ppu.line_buffer[8], 0, "sprite hidden once disabled");
}

/// Sprite transparency is colour id 0, not shade 0: a pixel the palette maps to white is opaque.
#[test]
fn sprite_mapped_to_shade_0_is_visible() {
    let mut ppu = Ppu::new();
    ppu.registers.write_lcdc(0x93);
    ppu.registers.set_bgp(IDENTITY);
    solid_tile(&mut ppu, 0, 3); // background colour 3 everywhere
    solid_tile(&mut ppu, 1, 1);
    ppu.registers.set_obp0(0xE0); // colour 1 -> shade 0
    set_sprite(&mut ppu, 0, 16, 16, 1, 0x00);
    render_line(&mut ppu, 0);
    assert_eq!(ppu.line_buffer[8], 0);
}

/// A sprite hidden behind the background still wins the pixel, so a lower-priority sprite
/// underneath doesn't show through.
#[test]
fn hidden_sprite_still_blocks_lower_priority_sprite() {
    let mut ppu = Ppu::new();
    ppu.registers.write_lcdc(0x93);
    ppu.registers.set_bgp(IDENTITY);
    solid_tile(&mut ppu, 0, 3); // background colour 3 everywhere
    solid_tile(&mut ppu, 1, 1);
    ppu.registers.set_obp0(0x04); // colour 1 -> shade 1
    ppu.registers.set_obp1(0x08); // colour 1 -> shade 2
    set_sprite(&mut ppu, 0, 16, 16, 1, 0x80); // behind BG, x 8-15
    set_sprite(&mut ppu, 1, 16, 17, 1, 0x10); // in front, x 9-16, lower priority
    render_line(&mut ppu, 0);
    assert_eq!(
        ppu.line_buffer[9], 3,
        "background, not the lower-priority sprite"
    );
    assert_eq!(
        ppu.line_buffer[16], 2,
        "lower-priority sprite where it doesn't overlap"
    );
}

// --- LCD off -------------------------------------------------------------------------

/// Turning the LCD off blanks the screen to white and hands that frame to the display.
#[test]
fn lcd_off_blanks_screen() {
    let mut ppu = Ppu::new();
    ppu.registers.set_bgp(0xFF);
    render_line(&mut ppu, 0); // line 0 is now shade 3
    ppu.write_register(0xFF40, 0x00);

    assert_eq!(rgb_at(&ppu, 0, 0), ColorConverter::gb_color_to_rgb(0));
    assert_eq!(rgb_at(&ppu, 159, 143), ColorConverter::gb_color_to_rgb(0));
    assert!(ppu.is_frame_ready());
}
