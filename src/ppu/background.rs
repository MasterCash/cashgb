/// Background and Window Tile Renderer
///
/// Handles rendering of background and window layers using:
/// - Tile data from VRAM (0x8000-0x97FF)
/// - Tile maps from VRAM (0x9800-0x9FFF)
/// - Scroll registers (SCX, SCY)
/// - Window registers (WX, WY)

use super::registers::PpuRegisters;

pub struct BackgroundRenderer {
    /// Internal window line counter
    window_internal_line: u8,
}

impl Default for BackgroundRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl BackgroundRenderer {
    pub fn new() -> Self {
        Self {
            window_internal_line: 0,
        }
    }

    pub fn reset(&mut self) {
        self.window_internal_line = 0;
    }

    /// Render background line to the provided buffer
    pub fn render_background_line(
        &self,
        vram: &[u8; 0x2000],
        registers: &PpuRegisters,
        line: u8,
        buffer: &mut [u8; 160],
    ) {
        if !registers.is_bg_window_enabled() {
            // Background disabled, fill with color 0
            buffer.fill(0);
            return;
        }

        let scroll_y = registers.get_scy();
        let scroll_x = registers.get_scx();
        let bg_tile_map_area = registers.bg_tile_map_area();
        let tile_data_area = registers.bg_window_tile_data_area();

        // Calculate which background line to render (with scrolling)
        let bg_line = scroll_y.wrapping_add(line);

        // Determine tile map base address
        let tile_map_base = if bg_tile_map_area { 0x1C00 } else { 0x1800 };

        // Which row of tiles (each tile is 8 pixels tall)
        let tile_row = (bg_line / 8) as u16;

        // Which pixel row within the tile (0-7)
        let pixel_row = bg_line % 8;

        for (screen_x, pixel) in buffer.iter_mut().enumerate().take(160) {
            // Calculate background X coordinate with scrolling
            let bg_x = scroll_x.wrapping_add(screen_x as u8);

            // Which column of tiles
            let tile_col = (bg_x / 8) as u16;

            // Which pixel column within the tile (0-7)
            let pixel_col = 7 - (bg_x % 8); // Game Boy tiles are stored left-to-right

            // Get tile index from tile map
            let tile_map_addr = tile_map_base + (tile_row * 32) + tile_col;
            let tile_index = vram[tile_map_addr as usize];

            // Get tile data
            let color_id = self.get_tile_pixel(vram, tile_index, pixel_col, pixel_row, tile_data_area);

            *pixel = color_id;
        }
    }

    /// Render window line to the provided buffer
    pub fn render_window_line(
        &mut self,
        vram: &[u8; 0x2000],
        registers: &PpuRegisters,
        line: u8,
        buffer: &mut [u8; 160],
    ) {
        if !registers.is_window_enabled() || !registers.is_bg_window_enabled() {
            return; // Window disabled
        }

        let window_y = registers.get_wy();
        let window_x = registers.get_wx().saturating_sub(7); // WX=7 means window starts at X=0

        // Check if window should be visible on this line
        if line < window_y {
            return; // Window hasn't started yet
        }

        let window_tile_map_area = registers.window_tile_map_area();
        let tile_data_area = registers.bg_window_tile_data_area();

        // Determine tile map base address for window
        let tile_map_base = if window_tile_map_area { 0x1C00 } else { 0x1800 };

        // Which row of tiles in the window
        let tile_row = (self.window_internal_line / 8) as u16;

        // Which pixel row within the tile (0-7)
        let pixel_row = self.window_internal_line % 8;

        // Render window pixels
        for screen_x in window_x..160 {
            let window_pixel_x = screen_x - window_x;
            let tile_col = (window_pixel_x / 8) as u16;
            let pixel_col = 7 - (window_pixel_x % 8);

            // Get tile index from window tile map
            let tile_map_addr = tile_map_base + (tile_row * 32) + tile_col;
            let tile_index = vram[tile_map_addr as usize];

            // Get tile data
            let color_id = self.get_tile_pixel(vram, tile_index, pixel_col, pixel_row, tile_data_area);

            buffer[screen_x as usize] = color_id;
        }

        // Increment window internal line counter
        self.window_internal_line += 1;
    }

    /// Get a single pixel from a tile
    fn get_tile_pixel(
        &self,
        vram: &[u8; 0x2000],
        tile_index: u8,
        pixel_x: u8,
        pixel_y: u8,
        tile_data_area: bool,
    ) -> u8 {
        // Calculate tile data address
        let tile_addr = if tile_data_area {
            // Unsigned mode: 0x8000-0x8FFF
            (tile_index as u16) * 16
        } else {
            // Signed mode: 0x8800-0x97FF
            let base = 0x1000; // Offset for 0x9000 in VRAM
            if tile_index < 128 {
                base + (tile_index as u16) * 16
            } else {
                base - ((256 - tile_index as u16) * 16)
            }
        };

        // Each tile row is 2 bytes
        let row_addr = tile_addr + (pixel_y as u16) * 2;

        // Get the two bytes for this row
        let byte1 = vram[row_addr as usize];     // LSB of color
        let byte2 = vram[(row_addr + 1) as usize]; // MSB of color

        // Extract the pixel (bit 7 is leftmost pixel)
        let bit_mask = 1 << (7 - pixel_x);
        let lsb = if byte1 & bit_mask != 0 { 1 } else { 0 };
        let msb = if byte2 & bit_mask != 0 { 2 } else { 0 };

        lsb | msb
    }

    /// Reset window internal line counter at frame start
    pub fn reset_window_line(&mut self) {
        self.window_internal_line = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_renderer_creation() {
        let renderer = BackgroundRenderer::new();
        assert_eq!(renderer.window_internal_line, 0);
    }

    #[test]
    fn test_get_tile_pixel() {
        let renderer = BackgroundRenderer::new();
        let mut vram = [0; 0x2000];

        // Set up a test pattern where each pixel_x has a specific expected color:
        // pixel_x=7 (bit 0): color 3 (both bits set)
        // pixel_x=6 (bit 1): color 2 (MSB only)
        // pixel_x=5 (bit 2): color 1 (LSB only)
        // pixel_x=4 (bit 3): color 0 (neither bit)
        vram[0x0000] = 0b00000101; // LSB: bits 0,2 set
        vram[0x0001] = 0b00000011; // MSB: bits 0,1 set

        // Test pixel extraction
        let pixel = renderer.get_tile_pixel(&vram, 0, 7, 0, true);
        assert_eq!(pixel, 3); // Both bits set = color 3 (bit 0 set in both)

        let pixel = renderer.get_tile_pixel(&vram, 0, 6, 0, true);
        assert_eq!(pixel, 2); // Only MSB set = color 2 (bit 1 set in MSB only)

        let pixel = renderer.get_tile_pixel(&vram, 0, 5, 0, true);
        assert_eq!(pixel, 1); // Only LSB set = color 1 (bit 2 set in LSB only)

        let pixel = renderer.get_tile_pixel(&vram, 0, 4, 0, true);
        assert_eq!(pixel, 0); // No bits set = color 0 (bit 3 clear in both)
    }
}