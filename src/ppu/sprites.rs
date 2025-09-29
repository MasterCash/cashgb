/// Sprite (Object) Rendering System
///
/// Handles rendering of sprites using:
/// - Object Attribute Memory (OAM) data
/// - Sprite tile data from VRAM
/// - Sprite palettes (OBP0, OBP1)
/// - Priority and transparency handling

use super::registers::PpuRegisters;

/// Sprite data structure (4 bytes per sprite in OAM)
#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    /// Y position (minus 16)
    pub y: u8,
    /// X position (minus 8)
    pub x: u8,
    /// Tile index
    pub tile_index: u8,
    /// Attributes flags
    pub attributes: u8,
}

impl Sprite {
    /// Create sprite from OAM data
    pub fn from_oam(oam_data: &[u8; 4]) -> Self {
        Self {
            y: oam_data[0],
            x: oam_data[1],
            tile_index: oam_data[2],
            attributes: oam_data[3],
        }
    }

    /// Get actual screen Y position
    pub fn screen_y(&self) -> i16 {
        self.y as i16 - 16
    }

    /// Get actual screen X position
    pub fn screen_x(&self) -> i16 {
        self.x as i16 - 8
    }

    /// Check if sprite has priority over background (bit 7 = 0)
    pub fn has_priority(&self) -> bool {
        self.attributes & 0x80 == 0
    }

    /// Check if sprite is flipped vertically (bit 6)
    pub fn is_y_flipped(&self) -> bool {
        self.attributes & 0x40 != 0
    }

    /// Check if sprite is flipped horizontally (bit 5)
    pub fn is_x_flipped(&self) -> bool {
        self.attributes & 0x20 != 0
    }

    /// Get sprite palette number (bit 4): 0 = OBP0, 1 = OBP1
    pub fn palette(&self) -> u8 {
        if self.attributes & 0x10 != 0 { 1 } else { 0 }
    }

    /// Check if sprite is visible on the given scanline
    pub fn is_on_line(&self, line: u8, sprite_height: u8) -> bool {
        let sprite_y = self.screen_y();
        let line_i16 = line as i16;
        line_i16 >= sprite_y && line_i16 < sprite_y + (sprite_height as i16)
    }
}

/// Parameters for sprite rendering
struct SpriteRenderParams<'a> {
    vram: &'a [u8; 0x2000],
    registers: &'a PpuRegisters,
    line: u8,
    sprite_height: u8,
    bg_buffer: &'a [u8; 160],
    sprite_buffer: &'a mut [u8; 160],
    priority_buffer: &'a mut [bool; 160],
}

pub struct SpriteRenderer {
    /// Sprites found during OAM scan for current line
    line_sprites: Vec<Sprite>,
}

impl Default for SpriteRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl SpriteRenderer {
    pub fn new() -> Self {
        Self {
            line_sprites: Vec::with_capacity(10), // Game Boy can show max 10 sprites per line
        }
    }

    /// Scan OAM for sprites on the given scanline
    pub fn scan_sprites_for_line(
        &mut self,
        oam: &[u8; 0xA0],
        line: u8,
        sprite_height: u8,
    ) {
        self.line_sprites.clear();

        // Scan all 40 sprites in OAM
        for sprite_index in 0..40 {
            let oam_offset = sprite_index * 4;
            let sprite_data = [
                oam[oam_offset],
                oam[oam_offset + 1],
                oam[oam_offset + 2],
                oam[oam_offset + 3],
            ];

            let sprite = Sprite::from_oam(&sprite_data);

            // Check if sprite is on this line
            if sprite.is_on_line(line, sprite_height) {
                self.line_sprites.push(sprite);

                // Game Boy hardware limit: max 10 sprites per line
                if self.line_sprites.len() >= 10 {
                    break;
                }
            }
        }

        // Sort sprites by X position (leftmost first)
        // In case of tie, lower OAM index has priority
        self.line_sprites.sort_by_key(|sprite| sprite.x);
    }

    /// Render sprites for the current line onto the background buffer
    pub fn render_sprites_line(
        &self,
        vram: &[u8; 0x2000],
        registers: &PpuRegisters,
        line: u8,
        bg_buffer: &[u8; 160],
        sprite_buffer: &mut [u8; 160],
        priority_buffer: &mut [bool; 160],
    ) {
        if !registers.is_sprites_enabled() {
            return;
        }

        let sprite_height = if registers.sprite_size() { 16 } else { 8 };

        // Clear sprite buffer
        sprite_buffer.fill(0);
        priority_buffer.fill(false);

        let mut params = SpriteRenderParams {
            vram,
            registers,
            line,
            sprite_height,
            bg_buffer,
            sprite_buffer,
            priority_buffer,
        };

        // Render each sprite (in reverse order for proper priority)
        for sprite in self.line_sprites.iter().rev() {
            self.render_sprite(sprite, &mut params);
        }
    }

    /// Render a single sprite
    fn render_sprite(&self, sprite: &Sprite, params: &mut SpriteRenderParams) {
        let sprite_y = sprite.screen_y();
        let sprite_x = sprite.screen_x();

        // Calculate which row of the sprite we're rendering
        let mut sprite_row = (params.line as i16 - sprite_y) as u8;

        // Handle vertical flipping
        if sprite.is_y_flipped() {
            sprite_row = (params.sprite_height - 1) - sprite_row;
        }

        // Get tile index (for 8x16 sprites, use tile_index & 0xFE for top tile)
        let tile_index = if params.sprite_height == 16 {
            if sprite_row < 8 {
                sprite.tile_index & 0xFE // Top tile
            } else {
                sprite.tile_index | 0x01 // Bottom tile
            }
        } else {
            sprite.tile_index
        };

        // Adjust row for 8x16 sprites
        if params.sprite_height == 16 && sprite_row >= 8 {
            sprite_row -= 8;
        }

        // Render each pixel of the sprite
        for pixel_x in 0..8 {
            let screen_x = sprite_x + pixel_x as i16;

            // Skip if pixel is off-screen
            if !(0..160).contains(&screen_x) {
                continue;
            }

            let screen_x_usize = screen_x as usize;

            // Handle horizontal flipping
            let sprite_pixel_x = if sprite.is_x_flipped() {
                7 - pixel_x
            } else {
                pixel_x
            };

            // Get pixel color from tile data
            let color_id = self.get_sprite_pixel(params.vram, tile_index, sprite_pixel_x, sprite_row);

            // Color 0 is transparent for sprites
            if color_id == 0 {
                continue;
            }

            // Check sprite priority
            let has_priority = sprite.has_priority();
            let bg_color = params.bg_buffer[screen_x_usize];

            // Sprite priority rules:
            // 1. If sprite has priority (bit 7 = 0), it appears over background
            // 2. If sprite doesn't have priority (bit 7 = 1), it only appears over background color 0
            // 3. Sprites always appear over each other based on X position and OAM index
            let should_render = if params.priority_buffer[screen_x_usize] {
                // Another sprite already rendered here, skip
                false
            } else if has_priority {
                // High priority sprite, always render
                true
            } else {
                // Low priority sprite, only render over background color 0
                bg_color == 0
            };

            if should_render {
                // Apply sprite palette
                let palette_color = params.registers.get_sprite_color(sprite.palette(), color_id);
                params.sprite_buffer[screen_x_usize] = palette_color;
                params.priority_buffer[screen_x_usize] = true;
            }
        }
    }

    /// Get a single pixel from sprite tile data
    fn get_sprite_pixel(&self, vram: &[u8; 0x2000], tile_index: u8, pixel_x: u8, pixel_y: u8) -> u8 {
        // Sprite tiles always use the 0x8000-0x8FFF area (unsigned mode)
        let tile_addr = (tile_index as u16) * 16;
        let row_addr = tile_addr + (pixel_y as u16) * 2;

        // Get the two bytes for this row
        let byte1 = vram[row_addr as usize];       // LSB of color
        let byte2 = vram[(row_addr + 1) as usize]; // MSB of color

        // Extract the pixel (bit 7 is leftmost pixel)
        let bit_mask = 1 << (7 - pixel_x);
        let lsb = if byte1 & bit_mask != 0 { 1 } else { 0 };
        let msb = if byte2 & bit_mask != 0 { 2 } else { 0 };

        lsb | msb
    }

    /// Get the number of sprites found for the current line
    pub fn sprite_count(&self) -> usize {
        self.line_sprites.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_from_oam() {
        let oam_data = [100, 50, 12, 0x20]; // Y=100, X=50, tile=12, attributes=0x20
        let sprite = Sprite::from_oam(&oam_data);

        assert_eq!(sprite.y, 100);
        assert_eq!(sprite.x, 50);
        assert_eq!(sprite.tile_index, 12);
        assert_eq!(sprite.attributes, 0x20);
    }

    #[test]
    fn test_sprite_screen_positions() {
        let sprite = Sprite {
            y: 16,
            x: 8,
            tile_index: 0,
            attributes: 0,
        };

        assert_eq!(sprite.screen_y(), 0);  // 16 - 16 = 0
        assert_eq!(sprite.screen_x(), 0);  // 8 - 8 = 0
    }

    #[test]
    fn test_sprite_attributes() {
        let sprite = Sprite {
            y: 0,
            x: 0,
            tile_index: 0,
            attributes: 0b11110000, // All flags set
        };

        assert!(!sprite.has_priority());    // Bit 7 = 1, no priority
        assert!(sprite.is_y_flipped());     // Bit 6 = 1
        assert!(sprite.is_x_flipped());     // Bit 5 = 1
        assert_eq!(sprite.palette(), 1);    // Bit 4 = 1, palette 1
    }

    #[test]
    fn test_sprite_line_visibility() {
        let sprite = Sprite {
            y: 26, // Screen Y = 10
            x: 8,
            tile_index: 0,
            attributes: 0,
        };

        // 8x8 sprite
        assert!(!sprite.is_on_line(9, 8));   // Above sprite
        assert!(sprite.is_on_line(10, 8));   // First line of sprite
        assert!(sprite.is_on_line(17, 8));   // Last line of sprite
        assert!(!sprite.is_on_line(18, 8));  // Below sprite

        // 8x16 sprite
        assert!(sprite.is_on_line(25, 16));  // Last line of 8x16 sprite
        assert!(!sprite.is_on_line(26, 16)); // Below 8x16 sprite
    }

    #[test]
    fn test_sprite_renderer_creation() {
        let renderer = SpriteRenderer::new();
        assert_eq!(renderer.sprite_count(), 0);
    }
}