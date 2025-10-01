/// Display output abstraction for the PPU
///
/// Provides a trait-based interface for different display backends
/// and color conversion utilities

use log::debug;
use crate::cpu::{Cpu, instructions::Instruction};
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

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

        debug!(
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

/// GUI Display using egui for multiple debugging windows
pub struct GuiDisplay {
    framebuffer: [u8; 160 * 144 * 4],
    cpu_state: Arc<Mutex<CpuDebugState>>,
    frame_count: u64,
}

#[derive(Clone)]
pub struct CpuDebugState {
    pub pc: u16,
    pub sp: u16,
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub flags: (bool, bool, bool, bool), // Z, N, H, C
    pub current_instruction: String,
    pub instruction_history: VecDeque<String>,
    pub memory: [u8; 0x10000], // Full memory space
}

impl Default for CpuDebugState {
    fn default() -> Self {
        Self {
            pc: 0,
            sp: 0,
            af: 0,
            bc: 0,
            de: 0,
            hl: 0,
            flags: (false, false, false, false),
            current_instruction: "NOP".to_string(),
            instruction_history: VecDeque::with_capacity(100),
            memory: [0; 0x10000],
        }
    }
}

impl GuiDisplay {
    pub fn new() -> Self {
        Self {
            framebuffer: [0; 160 * 144 * 4],
            cpu_state: Arc::new(Mutex::new(CpuDebugState::default())),
            frame_count: 0,
        }
    }

    pub fn update_cpu_state(&mut self, cpu: &Cpu) {
        if let Ok(mut state) = self.cpu_state.lock() {
            state.pc = cpu.get_pc();
            state.sp = cpu.get_sp();
            state.af = cpu.get_af();
            state.bc = cpu.get_bc();
            state.de = cpu.get_de();
            state.hl = cpu.get_hl();
            state.flags = cpu.get_flags();

            let instruction_str = format!("{:?}", cpu.get_current_instruction());
            state.current_instruction = instruction_str.clone();

            // Add to history and keep only last 100 instructions
            state.instruction_history.push_back(format!("{:04X}: {}", state.pc, instruction_str));
            if state.instruction_history.len() > 100 {
                state.instruction_history.pop_front();
            }

            // Copy relevant memory regions for debugging
            for addr in 0..0x10000 {
                state.memory[addr] = cpu.read_memory(addr as u16);
            }
        }
    }

    pub fn get_cpu_state(&self) -> Arc<Mutex<CpuDebugState>> {
        Arc::clone(&self.cpu_state)
    }
}

impl Display for GuiDisplay {
    fn present_frame(&mut self, framebuffer: &[u8; 160 * 144 * 4]) {
        self.framebuffer.copy_from_slice(framebuffer);
        self.frame_count += 1;
    }

    fn name(&self) -> &str {
        "GUI Display"
    }
}

/// Main GUI application structure
pub struct EmulatorApp {
    cpu_state: Arc<Mutex<CpuDebugState>>,
    framebuffer: [u8; 160 * 144 * 4],
    texture: Option<egui::TextureHandle>,
    show_memory_viewer: bool,
    show_cpu_debugger: bool,
    memory_address: u16,
    memory_view_start: u16,
}

impl EmulatorApp {
    pub fn new(cpu_state: Arc<Mutex<CpuDebugState>>) -> Self {
        Self {
            cpu_state,
            framebuffer: [0; 160 * 144 * 4],
            texture: None,
            show_memory_viewer: true,
            show_cpu_debugger: true,
            memory_address: 0,
            memory_view_start: 0,
        }
    }

    pub fn update_framebuffer(&mut self, framebuffer: &[u8; 160 * 144 * 4]) {
        self.framebuffer.copy_from_slice(framebuffer);
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request a repaint to keep the UI responsive
        ctx.request_repaint();

        // Main game window
        egui::Window::new("Game Boy Display")
            .default_size([480.0, 432.0])
            .resizable(false)
            .show(ctx, |ui| {
                // Convert framebuffer to egui texture
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [160, 144],
                    &self.framebuffer,
                );

                // Create or update texture
                let texture = self.texture.get_or_insert_with(|| {
                    ctx.load_texture("game_display", color_image.clone(), egui::TextureOptions::NEAREST)
                });

                // Update texture data
                texture.set(color_image, egui::TextureOptions::NEAREST);

                // Display at 3x scale
                ui.image((texture.id(), egui::vec2(480.0, 432.0)));
            });

        // CPU Debug window
        if self.show_cpu_debugger {
            egui::Window::new("CPU Debugger")
                .default_size([300.0, 400.0])
                .open(&mut self.show_cpu_debugger)
                .show(ctx, |ui| {
                    if let Ok(state) = self.cpu_state.lock() {
                        ui.heading("Registers");
                        ui.horizontal(|ui| {
                            ui.label(format!("PC: {:04X}", state.pc));
                            ui.label(format!("SP: {:04X}", state.sp));
                        });
                        ui.horizontal(|ui| {
                            ui.label(format!("AF: {:04X}", state.af));
                            ui.label(format!("BC: {:04X}", state.bc));
                        });
                        ui.horizontal(|ui| {
                            ui.label(format!("DE: {:04X}", state.de));
                            ui.label(format!("HL: {:04X}", state.hl));
                        });

                        ui.separator();
                        ui.heading("Flags");
                        ui.horizontal(|ui| {
                            ui.label(format!("Z: {}", if state.flags.0 { "1" } else { "0" }));
                            ui.label(format!("N: {}", if state.flags.1 { "1" } else { "0" }));
                            ui.label(format!("H: {}", if state.flags.2 { "1" } else { "0" }));
                            ui.label(format!("C: {}", if state.flags.3 { "1" } else { "0" }));
                        });

                        ui.separator();
                        ui.heading("Current Instruction");
                        ui.label(&state.current_instruction);

                        ui.separator();
                        ui.heading("Instruction History");
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for instruction in state.instruction_history.iter().rev().take(20) {
                                    ui.label(instruction);
                                }
                            });
                    }
                });
        }

        // Memory Viewer window
        if self.show_memory_viewer {
            egui::Window::new("Memory Viewer")
                .default_size([400.0, 500.0])
                .open(&mut self.show_memory_viewer)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Address:");
                        ui.add(egui::DragValue::new(&mut self.memory_address)
                            .range(0..=0xFFFF)
                            .hexadecimal(4, false, true));
                        if ui.button("Go").clicked() {
                            self.memory_view_start = self.memory_address & 0xFFF0; // Align to 16 bytes
                        }
                    });

                    ui.separator();

                    if let Ok(state) = self.cpu_state.lock() {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            egui::Grid::new("memory_grid")
                                .spacing([5.0, 2.0])
                                .show(ui, |ui| {
                                    // Header
                                    ui.label("Address");
                                    for i in 0..16 {
                                        ui.label(format!("{:X}", i));
                                    }
                                    ui.label("ASCII");
                                    ui.end_row();

                                    // Memory rows
                                    for row in 0..32 {
                                        let base_addr = self.memory_view_start.wrapping_add(row * 16);
                                        ui.label(format!("{:04X}", base_addr));

                                        let mut ascii = String::new();
                                        for col in 0..16 {
                                            let addr = base_addr.wrapping_add(col);
                                            let byte = state.memory[addr as usize];

                                            // Highlight PC address
                                            if addr == state.pc {
                                                ui.colored_label(egui::Color32::RED, format!("{:02X}", byte));
                                            } else {
                                                ui.label(format!("{:02X}", byte));
                                            }

                                            // Build ASCII representation
                                            if byte >= 32 && byte <= 126 {
                                                ascii.push(byte as char);
                                            } else {
                                                ascii.push('.');
                                            }
                                        }
                                        ui.label(ascii);
                                        ui.end_row();
                                    }
                                });
                        });
                    }
                });
        }

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Windows", |ui| {
                    ui.checkbox(&mut self.show_cpu_debugger, "CPU Debugger");
                    ui.checkbox(&mut self.show_memory_viewer, "Memory Viewer");
                });
            });
        });
    }
}