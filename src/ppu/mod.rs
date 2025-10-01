pub mod registers;
pub mod timing;
pub mod background;
pub mod sprites;
pub mod display;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod sprite_rendering_tests;

#[cfg(test)]
mod debug_sprite_test;

#[cfg(test)]
mod debug_test_expectations;

use registers::PpuRegisters;
pub use timing::LcdMode;
use crate::cpu::instructions::Interrupt;
use background::BackgroundRenderer;
use sprites::SpriteRenderer;
use display::ColorConverter;

/// Game Boy Picture Processing Unit (PPU)
///
/// Handles all graphics rendering including:
/// - LCD timing and mode management
/// - Background and window tile rendering
/// - Sprite (OAM) rendering
/// - Frame buffer management
pub struct Ppu {
    /// Current LCD mode and timing state
    mode: LcdMode,
    /// Dot counter within current mode
    dots: u16,
    /// Current scanline being processed (0-153)
    scanline: u8,

    /// PPU control and status registers
    pub registers: PpuRegisters,

    /// Video RAM (0x8000-0x9FFF)
    /// Contains tile data and tile maps
    vram: [u8; 0x2000],

    /// Object Attribute Memory (0xFE00-0xFE9F)
    /// Contains sprite data (40 sprites × 4 bytes each)
    oam: [u8; 0xA0],

    /// Frame buffer for output (160×144 pixels, RGBA format)
    /// Each pixel is 4 bytes: [R, G, B, A]
    framebuffer: [u8; 160 * 144 * 4],

    /// Flag indicating if a new frame is ready for display
    frame_ready: bool,

    /// Internal line buffer for current scanline rendering
    line_buffer: [u8; 160],

    /// Background and window renderer
    bg_renderer: BackgroundRenderer,

    /// Sprite renderer
    sprite_renderer: SpriteRenderer,

    /// Sprite rendering buffers
    sprite_buffer: [u8; 160],
    sprite_priority_buffer: [bool; 160],
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

impl Ppu {
    /// Create a new PPU instance with default Game Boy initialization
    pub fn new() -> Self {
        Self {
            mode: LcdMode::OamScan,
            dots: 0,
            scanline: 0,
            registers: PpuRegisters::new(),
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            framebuffer: [0; 160 * 144 * 4],
            frame_ready: false,
            line_buffer: [0; 160],
            bg_renderer: BackgroundRenderer::new(),
            sprite_renderer: SpriteRenderer::new(),
            sprite_buffer: [0; 160],
            sprite_priority_buffer: [false; 160],
        }
    }

    /// Reset PPU to initial state
    pub fn reset(&mut self) {
        self.mode = LcdMode::OamScan;
        self.dots = 0;
        self.scanline = 0;
        self.registers.reset();
        self.vram.fill(0);
        self.oam.fill(0);
        self.framebuffer.fill(0);
        self.frame_ready = false;
        self.line_buffer.fill(0);
        self.bg_renderer.reset();
        self.sprite_buffer.fill(0);
        self.sprite_priority_buffer.fill(false);
    }

    /// Step the PPU by the given number of CPU cycles
    /// Returns an interrupt if one should be triggered
    pub fn step(&mut self, cycles: u8) -> Option<Interrupt> {
        // If LCD is disabled, don't process anything
        if !self.registers.is_lcd_enabled() {
            return None;
        }

        self.dots += cycles as u16;

        match self.mode {
            LcdMode::OamScan => self.handle_oam_scan(),
            LcdMode::Drawing => self.handle_drawing(),
            LcdMode::HBlank => self.handle_hblank(),
            LcdMode::VBlank => self.handle_vblank(),
        }
    }

    /// Handle OAM Scan mode (Mode 2)
    /// Scans OAM for sprites on current scanline
    fn handle_oam_scan(&mut self) -> Option<Interrupt> {
        if self.dots >= 80 {
            // Perform OAM scan for sprites on this line
            let sprite_height = if self.registers.sprite_size() { 16 } else { 8 };
            self.sprite_renderer.scan_sprites_for_line(&self.oam, self.scanline, sprite_height);

            self.dots = 0;
            self.mode = LcdMode::Drawing;
            self.registers.set_mode(LcdMode::Drawing);
        }
        None
    }

    /// Handle Drawing mode (Mode 3)
    /// Renders pixels for current scanline
    fn handle_drawing(&mut self) -> Option<Interrupt> {
        if self.dots >= 172 {
            // Render the current scanline
            self.render_scanline();

            self.dots = 0;
            self.mode = LcdMode::HBlank;
            self.registers.set_mode(LcdMode::HBlank);

            // Check for STAT interrupt on Mode 0 (HBlank)
            if self.registers.is_hblank_interrupt_enabled() {
                return Some(Interrupt::LCD);
            }
        }
        None
    }

    /// Handle Horizontal Blank mode (Mode 0)
    /// Waits until end of scanline
    fn handle_hblank(&mut self) -> Option<Interrupt> {
        if self.dots >= 204 {
            self.dots = 0;
            self.scanline = self.scanline.wrapping_add(1);
            self.registers.set_ly(self.scanline);

            if self.scanline >= 144 {
                // Enter VBlank
                self.mode = LcdMode::VBlank;
                self.registers.set_mode(LcdMode::VBlank);
                self.frame_ready = true;

                // VBlank interrupt
                return Some(Interrupt::VBlank);
            } else {
                // Next scanline
                self.mode = LcdMode::OamScan;
                self.registers.set_mode(LcdMode::OamScan);

                // Check for STAT interrupt on Mode 2 (OAM)
                if self.registers.is_oam_interrupt_enabled() {
                    return Some(Interrupt::LCD);
                }
            }

            // Check LYC=LY interrupt
            if self.registers.is_lyc_interrupt_enabled() && self.scanline == self.registers.get_lyc() {
                self.registers.set_lyc_flag(true);
                return Some(Interrupt::LCD);
            } else {
                self.registers.set_lyc_flag(false);
            }
        }
        None
    }

    /// Handle Vertical Blank mode (Mode 1)
    /// Waits for next frame
    fn handle_vblank(&mut self) -> Option<Interrupt> {
        if self.dots >= 456 {
            self.dots = 0;
            self.scanline = self.scanline.wrapping_add(1);
            self.registers.set_ly(self.scanline);

            if self.scanline >= 154 {
                // Start new frame
                self.scanline = 0;
                self.registers.set_ly(0);
                self.mode = LcdMode::OamScan;
                self.registers.set_mode(LcdMode::OamScan);

                // Check for STAT interrupt on Mode 2 (OAM)
                if self.registers.is_oam_interrupt_enabled() {
                    return Some(Interrupt::LCD);
                }
            }
        }
        None
    }

    /// Render the current scanline to the line buffer
    fn render_scanline(&mut self) {
        // Only render if we're in the visible area
        if self.scanline < 144 {
            // Clear line buffer
            self.line_buffer.fill(0);

            // 1. Render background layer
            self.bg_renderer.render_background_line(
                &self.vram,
                &self.registers,
                self.scanline,
                &mut self.line_buffer,
            );

            // 2. Render window layer (on top of background)
            self.bg_renderer.render_window_line(
                &self.vram,
                &self.registers,
                self.scanline,
                &mut self.line_buffer,
            );

            // 3. Render sprites (on top of background/window with priority)
            self.sprite_renderer.render_sprites_line(
                &self.vram,
                &self.registers,
                self.scanline,
                &self.line_buffer, // background buffer
                &mut self.sprite_buffer,
                &mut self.sprite_priority_buffer,
            );

            // 4. Composite final line with sprite priority
            self.composite_line();

            // 5. Copy final line to framebuffer
            self.copy_line_to_framebuffer();
        }
    }

    /// Composite background and sprite layers with proper priority
    fn composite_line(&mut self) {
        for x in 0..160 {
            // Apply background palette to background pixels first
            self.line_buffer[x] = self.registers.get_bg_color(self.line_buffer[x]);

            // If a sprite pixel is present at this position and has priority
            if self.sprite_priority_buffer[x] && self.sprite_buffer[x] != 0 {
                // Sprite pixel overrides background pixel
                // Note: sprite_buffer already contains palette-adjusted color from sprite renderer
                self.line_buffer[x] = self.sprite_buffer[x];
            }
        }
    }

    /// Copy the current line buffer to the framebuffer
    fn copy_line_to_framebuffer(&mut self) {
        let line_start = (self.scanline as usize) * 160 * 4;

        for x in 0..160 {
            let pixel_start = line_start + (x * 4);
            let color_index = self.line_buffer[x];

            // Convert Game Boy color index to RGBA using color converter
            let (r, g, b) = ColorConverter::gb_color_to_rgb(color_index);

            self.framebuffer[pixel_start] = r;     // Red
            self.framebuffer[pixel_start + 1] = g; // Green
            self.framebuffer[pixel_start + 2] = b; // Blue
            self.framebuffer[pixel_start + 3] = 255; // Alpha
        }
    }

    /// Check if a new frame is ready for display
    pub fn is_frame_ready(&self) -> bool {
        self.frame_ready
    }

    /// Get the current framebuffer
    /// Should only be called after checking is_frame_ready()
    pub fn get_framebuffer(&mut self) -> &[u8; 160 * 144 * 4] {
        self.frame_ready = false;
        &self.framebuffer
    }

    /// Read from VRAM (0x8000-0x9FFF)
    pub fn read_vram(&self, addr: u16) -> u8 {
        // Memory access restrictions during Drawing mode
        if matches!(self.mode, LcdMode::Drawing) {
            return 0xFF; // Return open bus value
        }

        let offset = (addr - 0x8000) as usize;
        if offset < self.vram.len() {
            self.vram[offset]
        } else {
            0xFF
        }
    }

    /// Write to VRAM (0x8000-0x9FFF)
    pub fn write_vram(&mut self, addr: u16, value: u8) {
        // Memory access restrictions during Drawing mode
        if matches!(self.mode, LcdMode::Drawing) {
            return; // Ignore write
        }

        let offset = (addr - 0x8000) as usize;
        if offset < self.vram.len() {
            self.vram[offset] = value;
        }
    }

    /// Read from OAM (0xFE00-0xFE9F)
    pub fn read_oam(&self, addr: u16) -> u8 {
        // Memory access restrictions during OAM Scan and Drawing modes
        if matches!(self.mode, LcdMode::OamScan | LcdMode::Drawing) {
            return 0xFF; // Return open bus value
        }

        let offset = (addr - 0xFE00) as usize;
        if offset < self.oam.len() {
            self.oam[offset]
        } else {
            0xFF
        }
    }

    /// Write to OAM (0xFE00-0xFE9F)
    pub fn write_oam(&mut self, addr: u16, value: u8) {
        // Memory access restrictions during OAM Scan and Drawing modes
        if matches!(self.mode, LcdMode::OamScan | LcdMode::Drawing) {
            return; // Ignore write
        }

        let offset = (addr - 0xFE00) as usize;
        if offset < self.oam.len() {
            self.oam[offset] = value;
        }
    }

    /// Get current scanline for debugging
    pub fn get_current_line(&self) -> u8 {
        self.scanline
    }

    /// Get current mode for debugging
    pub fn get_current_mode(&self) -> LcdMode {
        self.mode
    }
}