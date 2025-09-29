/// Display output abstraction for the PPU
///
/// Provides a trait-based interface for different display backends
/// and color conversion utilities
/// Display trait for different output backends
pub trait Display {
    /// Present a completed frame to the display
    fn present_frame(&mut self, framebuffer: &[u8; 160 * 144 * 4]);

    /// Check if the display is ready for a new frame
    fn is_ready(&self) -> bool {
        true // Default implementation always ready
    }

    /// Get display name for debugging
    fn name(&self) -> &str;
}

/// Terminal-based display for testing and debugging
/// Outputs a simplified ASCII representation of the screen
pub struct TerminalDisplay {
    /// Whether to use color output
    use_color: bool,
    /// Frame counter for debugging
    frame_count: u64,
}

impl TerminalDisplay {
    pub fn new(use_color: bool) -> Self {
        Self {
            use_color,
            frame_count: 0,
        }
    }

    /// Convert RGBA pixel to ASCII character
    fn pixel_to_ascii(&self, r: u8, g: u8, b: u8) -> char {
        // Convert to grayscale
        let gray = ((r as u16 + g as u16 + b as u16) / 3) as u8;

        match gray {
            0..=63 => '█',    // Darkest
            64..=127 => '▓',  // Dark
            128..=191 => '▒', // Light
            192..=255 => '░', // Lightest
        }
    }

    /// Convert RGBA pixel to ANSI color code
    fn pixel_to_ansi(&self, r: u8, g: u8, b: u8) -> String {
        // Simple mapping to ANSI 8-bit colors
        let gray = ((r as u16 + g as u16 + b as u16) / 3) as u8;

        match gray {
            0..=63 => "\x1b[48;5;232m  \x1b[0m".to_string(),   // Dark gray background
            64..=127 => "\x1b[48;5;240m  \x1b[0m".to_string(),  // Medium gray background
            128..=191 => "\x1b[48;5;248m  \x1b[0m".to_string(), // Light gray background
            192..=255 => "\x1b[48;5;255m  \x1b[0m".to_string(), // White background
        }
    }
}

impl Display for TerminalDisplay {
    fn present_frame(&mut self, framebuffer: &[u8; 160 * 144 * 4]) {
        self.frame_count += 1;

        // Clear screen and move cursor to top
        print!("\x1b[2J\x1b[H");

        // Print frame header
        println!("Game Boy Display - Frame {}", self.frame_count);
        println!("┌{}┐", "─".repeat(if self.use_color { 320 } else { 160 }));

        // Render each line
        for y in 0..144 {
            print!("│");

            for x in 0..160 {
                let pixel_start = (y * 160 + x) * 4;
                let r = framebuffer[pixel_start];
                let g = framebuffer[pixel_start + 1];
                let b = framebuffer[pixel_start + 2];

                if self.use_color {
                    print!("{}", self.pixel_to_ansi(r, g, b));
                } else {
                    print!("{}", self.pixel_to_ascii(r, g, b));
                }
            }

            println!("│");
        }

        println!("└{}┘", "─".repeat(if self.use_color { 320 } else { 160 }));
        println!();
    }

    fn name(&self) -> &str {
        "Terminal Display"
    }
}

/// Null display that discards all frames (for testing)
pub struct NullDisplay;

impl Display for NullDisplay {
    fn present_frame(&mut self, _framebuffer: &[u8; 160 * 144 * 4]) {
        // Do nothing
    }

    fn name(&self) -> &str {
        "Null Display"
    }
}

/// Debug display that only logs frame information
pub struct DebugDisplay {
    frame_count: u64,
}

impl Default for DebugDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugDisplay {
    pub fn new() -> Self {
        Self { frame_count: 0 }
    }

    /// Get current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

impl Display for DebugDisplay {
    fn present_frame(&mut self, framebuffer: &[u8; 160 * 144 * 4]) {
        self.frame_count += 1;

        // Calculate some statistics about the frame
        let mut color_counts = [0u32; 4];
        let mut total_brightness = 0u64;

        for y in 0..144 {
            for x in 0..160 {
                let pixel_start = (y * 160 + x) * 4;
                let r = framebuffer[pixel_start] as u16;
                let g = framebuffer[pixel_start + 1] as u16;
                let b = framebuffer[pixel_start + 2] as u16;

                let brightness = (r + g + b) / 3;
                total_brightness += brightness as u64;

                // Classify into Game Boy color categories
                match brightness {
                    0..=63 => color_counts[3] += 1,    // Darkest
                    64..=127 => color_counts[2] += 1,  // Dark
                    128..=191 => color_counts[1] += 1, // Light
                    192..=255 => color_counts[0] += 1, // Lightest
                    _ => color_counts[0] += 1,         // Fallback for values > 255
                }
            }
        }

        let avg_brightness = total_brightness / (160 * 144);

        println!(
            "Frame {}: Avg brightness: {}, Colors: [{}]",
            self.frame_count,
            avg_brightness,
            color_counts
                .iter()
                .map(|&c| c.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    fn name(&self) -> &str {
        "Debug Display"
    }
}

/// Color conversion utilities
pub struct ColorConverter;

impl ColorConverter {
    /// Convert Game Boy color index to RGB values
    /// Uses the classic Game Boy green palette
    pub fn gb_color_to_rgb(color_index: u8) -> (u8, u8, u8) {
        match color_index & 0x03 {
            0 => (155, 188, 15),  // Lightest green
            1 => (139, 172, 15),  // Light green
            2 => (48, 98, 48),    // Dark green
            3 => (15, 56, 15),    // Darkest green
            _ => (0, 0, 0),       // Should never happen
        }
    }

    /// Convert Game Boy color index to grayscale
    pub fn gb_color_to_gray(color_index: u8) -> u8 {
        match color_index & 0x03 {
            0 => 255, // White
            1 => 170, // Light gray
            2 => 85,  // Dark gray
            3 => 0,   // Black
            _ => 0,
        }
    }

    /// Create RGBA pixel data from Game Boy color index
    pub fn gb_color_to_rgba(color_index: u8) -> [u8; 4] {
        let (r, g, b) = Self::gb_color_to_rgb(color_index);
        [r, g, b, 255] // Full alpha
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_conversion() {
        let (r, g, b) = ColorConverter::gb_color_to_rgb(0);
        assert_eq!((r, g, b), (155, 188, 15)); // Lightest green

        let (r, g, b) = ColorConverter::gb_color_to_rgb(3);
        assert_eq!((r, g, b), (15, 56, 15)); // Darkest green

        let gray = ColorConverter::gb_color_to_gray(0);
        assert_eq!(gray, 255); // White

        let gray = ColorConverter::gb_color_to_gray(3);
        assert_eq!(gray, 0); // Black

        let rgba = ColorConverter::gb_color_to_rgba(1);
        assert_eq!(rgba, [139, 172, 15, 255]);
    }

    #[test]
    fn test_null_display() {
        let mut display = NullDisplay;
        let framebuffer = [0u8; 160 * 144 * 4];

        // Should not panic
        display.present_frame(&framebuffer);
        assert_eq!(display.name(), "Null Display");
    }

    #[test]
    fn test_debug_display() {
        let mut display = DebugDisplay::new();
        assert_eq!(display.frame_count(), 0);

        let framebuffer = [0u8; 160 * 144 * 4];
        display.present_frame(&framebuffer);

        assert_eq!(display.frame_count(), 1);
    }

    #[test]
    fn test_terminal_display() {
        let mut display = TerminalDisplay::new(false);
        let framebuffer = [0u8; 160 * 144 * 4];

        // Should not panic
        display.present_frame(&framebuffer);
        assert_eq!(display.name(), "Terminal Display");

        // Test ASCII conversion
        assert_eq!(display.pixel_to_ascii(0, 0, 0), '█');
        assert_eq!(display.pixel_to_ascii(255, 255, 255), '░');
    }
}