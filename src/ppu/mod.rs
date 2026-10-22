pub mod background;
pub mod display;
pub mod registers;
pub mod sprites;
pub mod timing;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod sprite_rendering_tests;

#[cfg(test)]
mod review_regressions;

#[cfg(test)]
mod debug_sprite_test;

#[cfg(test)]
mod debug_test_expectations;

use crate::{cpu::instructions::Interrupt, memory::memory_sizes};
use background::BackgroundRenderer;
use display::ColorConverter;
use registers::PpuRegisters;
use sprites::SpriteRenderer;
pub use timing::LcdMode;

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
    vram: Vec<u8>,

    /// Object Attribute Memory (0xFE00-0xFE9F)
    /// Contains sprite data (40 sprites × 4 bytes each)
    oam: [u8; 0xA0],

    /// Frame buffer for output (160×144 pixels, RGBA format)
    /// Each pixel is 4 bytes: [R, G, B, A]
    framebuffer: Vec<u8>,

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

    /// Combined STAT interrupt line (all enabled STAT sources ORed), for rising-edge detection
    stat_line: bool,
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
            vram: vec![0; 0x2000],
            oam: [0; 0xA0],
            framebuffer: vec![0; memory_sizes::MEM_BUFF_SIZE],
            frame_ready: false,
            line_buffer: [0; 160],
            bg_renderer: BackgroundRenderer::new(),
            sprite_renderer: SpriteRenderer::new(),
            sprite_buffer: [0; 160],
            sprite_priority_buffer: [false; 160],
            stat_line: false,
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
        self.stat_line = false;
    }

    /// Step the PPU by the given number of CPU cycles
    /// Returns the interrupts requested during this step as IF bits (VBlank and/or LCD STAT)
    pub fn step(&mut self, cycles: u8) -> u8 {
        // If LCD is disabled, don't process anything
        if !self.registers.is_lcd_enabled() {
            return 0;
        }

        self.dots += cycles as u16;

        let mut interrupts = match self.mode {
            LcdMode::OamScan => self.handle_oam_scan(),
            LcdMode::Drawing => self.handle_drawing(),
            LcdMode::HBlank => self.handle_hblank(),
            LcdMode::VBlank => self.handle_vblank(),
        };

        if self.update_stat() {
            interrupts |= Interrupt::LCD as u8;
        }
        interrupts
    }

    /// Refresh the LYC=LY flag and the STAT interrupt line.
    /// All STAT sources share one line, so the LCD interrupt fires only on its rising edge.
    fn update_stat(&mut self) -> bool {
        self.update_lyc_flag();

        let registers = &self.registers;
        let line = (registers.get_lyc_flag() && registers.is_lyc_interrupt_enabled())
            || match self.mode {
                LcdMode::HBlank => registers.is_hblank_interrupt_enabled(),
                LcdMode::VBlank => registers.is_vblank_interrupt_enabled(),
                LcdMode::OamScan => registers.is_oam_interrupt_enabled(),
                LcdMode::Drawing => false,
            };

        let rising = line && !self.stat_line;
        self.stat_line = line;
        rising
    }

    /// STAT bit 2 always reflects LY == LYC, whether or not the LYC interrupt is enabled.
    fn update_lyc_flag(&mut self) {
        let equal = self.registers.get_ly() == self.registers.get_lyc();
        self.registers.set_lyc_flag(equal);
    }

    /// Handle OAM Scan mode (Mode 2)
    /// Scans OAM for sprites on current scanline
    fn handle_oam_scan(&mut self) -> u8 {
        if self.dots >= 80 {
            // Perform OAM scan for sprites on this line
            let sprite_height = if self.registers.sprite_size() { 16 } else { 8 };
            self.sprite_renderer
                .scan_sprites_for_line(&self.oam, self.scanline, sprite_height);

            self.dots = 0;
            self.mode = LcdMode::Drawing;
            self.registers.set_mode(LcdMode::Drawing);
        }
        0
    }

    /// Handle Drawing mode (Mode 3)
    /// Renders pixels for current scanline
    fn handle_drawing(&mut self) -> u8 {
        if self.dots >= 172 {
            // Render the current scanline
            self.render_scanline();

            self.dots = 0;
            self.mode = LcdMode::HBlank;
            self.registers.set_mode(LcdMode::HBlank);
        }
        0
    }

    /// Handle Horizontal Blank mode (Mode 0)
    /// Waits until end of scanline
    fn handle_hblank(&mut self) -> u8 {
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
                return Interrupt::VBlank as u8;
            }

            // Next scanline
            self.mode = LcdMode::OamScan;
            self.registers.set_mode(LcdMode::OamScan);
        }
        0
    }

    /// Handle Vertical Blank mode (Mode 1)
    /// Waits for next frame
    fn handle_vblank(&mut self) -> u8 {
        if self.dots >= 456 {
            self.dots = 0;
            self.scanline = self.scanline.wrapping_add(1);
            self.registers.set_ly(self.scanline);

            if self.scanline >= 154 {
                // Start new frame
                self.scanline = 0;
                self.registers.set_ly(0);
                self.bg_renderer.reset_window_line();
                self.mode = LcdMode::OamScan;
                self.registers.set_mode(LcdMode::OamScan);
            }
        }
        0
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
        let bg_enabled = self.registers.is_bg_window_enabled();
        for x in 0..160 {
            // Apply the background palette; a disabled background is always white (shade 0)
            self.line_buffer[x] = if bg_enabled {
                self.registers.get_bg_color(self.line_buffer[x])
            } else {
                0
            };

            // sprite_priority_buffer marks the pixels where a sprite is visible. The sprite
            // buffer holds palette shades, so shade 0 (white) is still an opaque sprite pixel.
            if self.sprite_priority_buffer[x] {
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

            self.framebuffer[pixel_start] = r; // Red
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
    pub fn get_framebuffer(&mut self) -> &Vec<u8> {
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

    /// Read VRAM for debugging, ignoring the Drawing-mode access restriction.
    pub fn peek_vram(&self, addr: u16) -> u8 {
        self.vram
            .get((addr as usize).wrapping_sub(0x8000))
            .copied()
            .unwrap_or(0xFF)
    }

    /// Read OAM for debugging, ignoring the OAM Scan/Drawing access restriction.
    pub fn peek_oam(&self, addr: u16) -> u8 {
        self.oam
            .get((addr as usize).wrapping_sub(0xFE00))
            .copied()
            .unwrap_or(0xFF)
    }

    /// Write one byte of an OAM DMA transfer. DMA has its own path into OAM,
    /// so unlike `write_oam` it isn't blocked during OAM Scan and Drawing.
    pub fn dma_write_oam(&mut self, index: usize, value: u8) {
        if index < self.oam.len() {
            self.oam[index] = value;
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

    /// Write a PPU register (0xFF40-0xFF4B), applying LCD enable/disable side effects.
    pub fn write_register(&mut self, addr: u16, value: u8) {
        let was_enabled = self.registers.is_lcd_enabled();
        self.registers.write_register(addr, value);
        let is_enabled = self.registers.is_lcd_enabled();

        if was_enabled && !is_enabled {
            self.disable_lcd();
        } else if !was_enabled && is_enabled {
            self.enable_lcd();
        }

        // A new LYC (or LY reset) is reflected in STAT immediately. The STAT line itself is
        // only re-evaluated on the next step, so a resulting interrupt still fires there.
        self.update_lyc_flag();
    }

    /// LCD off: LY resets to 0, mode reads 0 (HBlank) and VRAM/OAM are freely accessible.
    fn disable_lcd(&mut self) {
        self.dots = 0;
        self.scanline = 0;
        self.registers.set_ly(0);
        self.mode = LcdMode::HBlank;
        self.registers.set_mode(LcdMode::HBlank);
        self.stat_line = false;

        // The screen goes blank (white) while the LCD is off; hand that frame to the display
        let white = ColorConverter::gb_color_to_rgba(0);
        for pixel in self.framebuffer.as_chunks_mut::<4>().0 {
            *pixel = white;
        }
        self.frame_ready = true;
    }

    /// LCD on: start a fresh frame at line 0.
    fn enable_lcd(&mut self) {
        self.dots = 0;
        self.scanline = 0;
        self.registers.set_ly(0);
        self.bg_renderer.reset_window_line();
        self.mode = LcdMode::OamScan;
        self.registers.set_mode(LcdMode::OamScan);
    }
}
