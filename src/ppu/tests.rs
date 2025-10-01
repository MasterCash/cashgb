#[cfg(test)]
mod tests {
    use crate::ppu::{Ppu, LcdMode};
    use crate::ppu::display::{DebugDisplay, Display};
    use crate::cpu::instructions::Interrupt;

    /// Helper function to create a PPU for testing
    fn create_test_ppu() -> Ppu {
        Ppu::new()
    }

    #[test]
    fn test_ppu_initialization() {
        let ppu = create_test_ppu();

        assert_eq!(ppu.mode, LcdMode::OamScan);
        assert_eq!(ppu.dots, 0);
        assert_eq!(ppu.scanline, 0);
        assert!(!ppu.frame_ready);
        assert_eq!(ppu.get_current_line(), 0);
        assert_eq!(ppu.get_current_mode(), LcdMode::OamScan);
    }

    #[test]
    fn test_ppu_reset() {
        let mut ppu = create_test_ppu();

        // Modify some state
        ppu.dots = 100;
        ppu.scanline = 50;
        ppu.mode = LcdMode::VBlank;
        ppu.frame_ready = true;

        // Reset and verify
        ppu.reset();

        assert_eq!(ppu.mode, LcdMode::OamScan);
        assert_eq!(ppu.dots, 0);
        assert_eq!(ppu.scanline, 0);
        assert!(!ppu.frame_ready);
    }

    #[test]
    fn test_lcd_disabled() {
        let mut ppu = create_test_ppu();

        // Disable LCD
        ppu.registers.write_lcdc(0x00);

        // Step should return None when LCD is disabled
        let interrupt = ppu.step(10);
        assert!(interrupt.is_none());
    }

    #[test]
    fn test_oam_scan_mode() {
        let mut ppu = create_test_ppu();
        assert_eq!(ppu.mode, LcdMode::OamScan);

        // Step through OAM scan (should take 80 dots)
        for _ in 0..7 {
            let interrupt = ppu.step(10);
            assert!(interrupt.is_none());
            assert_eq!(ppu.mode, LcdMode::OamScan);
        }

        // Should transition to Drawing mode after 80 dots
        let interrupt = ppu.step(10); // Total: 80 dots
        assert!(interrupt.is_none());
        assert_eq!(ppu.mode, LcdMode::Drawing);
        assert_eq!(ppu.dots, 0); // Dots should reset
    }

    #[test]
    fn test_drawing_mode() {
        let mut ppu = create_test_ppu();

        // Move to Drawing mode
        ppu.mode = LcdMode::Drawing;
        ppu.registers.set_mode(LcdMode::Drawing);

        // Step through Drawing mode (minimum 172 dots)
        for _ in 0..17 {
            let interrupt = ppu.step(10);
            assert!(interrupt.is_none());
            assert_eq!(ppu.mode, LcdMode::Drawing);
        }

        // Should transition to HBlank after 172+ dots
        let interrupt = ppu.step(10); // Total: 180 dots
        assert!(interrupt.is_none());
        assert_eq!(ppu.mode, LcdMode::HBlank);
    }

    #[test]
    fn test_hblank_mode() {
        let mut ppu = create_test_ppu();

        // Set up HBlank mode
        ppu.mode = LcdMode::HBlank;
        ppu.registers.set_mode(LcdMode::HBlank);
        ppu.scanline = 10; // Mid-frame

        // Step through HBlank
        for _ in 0..20 {
            let interrupt = ppu.step(10);
            assert!(interrupt.is_none());
            assert_eq!(ppu.mode, LcdMode::HBlank);
        }

        // Should transition to next OAM scan
        let interrupt = ppu.step(10); // Total: 210 dots (> 204)
        assert!(interrupt.is_none());
        assert_eq!(ppu.mode, LcdMode::OamScan);
        assert_eq!(ppu.scanline, 11); // Should increment scanline
    }

    #[test]
    fn test_vblank_transition() {
        let mut ppu = create_test_ppu();

        // Set up HBlank at line 143 (last visible line)
        ppu.mode = LcdMode::HBlank;
        ppu.registers.set_mode(LcdMode::HBlank);
        ppu.scanline = 143;
        ppu.registers.set_ly(143);

        // Complete HBlank - should transition to VBlank
        ppu.dots = 204;
        let interrupt = ppu.step(1);

        assert_eq!(interrupt, Some(Interrupt::VBlank));
        assert_eq!(ppu.mode, LcdMode::VBlank);
        assert_eq!(ppu.scanline, 144);
        assert!(ppu.frame_ready);
    }

    #[test]
    fn test_vblank_mode() {
        let mut ppu = create_test_ppu();

        // Set up VBlank mode
        ppu.mode = LcdMode::VBlank;
        ppu.registers.set_mode(LcdMode::VBlank);
        ppu.scanline = 150;

        // Step through VBlank scanline
        for _ in 0..45 {
            let interrupt = ppu.step(10);
            assert!(interrupt.is_none());
            assert_eq!(ppu.mode, LcdMode::VBlank);
        }

        // Should move to next VBlank line
        let interrupt = ppu.step(10); // Total: 460 dots (> 456)
        assert!(interrupt.is_none());
        assert_eq!(ppu.mode, LcdMode::VBlank);
        assert_eq!(ppu.scanline, 151);
    }

    #[test]
    fn test_frame_completion() {
        let mut ppu = create_test_ppu();

        // Set up end of VBlank
        ppu.mode = LcdMode::VBlank;
        ppu.registers.set_mode(LcdMode::VBlank);
        ppu.scanline = 153;
        ppu.dots = 456;

        // Should start new frame
        let interrupt = ppu.step(1);
        assert!(interrupt.is_none());
        assert_eq!(ppu.mode, LcdMode::OamScan);
        assert_eq!(ppu.scanline, 0);
        assert_eq!(ppu.dots, 0);
    }

    #[test]
    fn test_vram_access_restrictions() {
        let mut ppu = create_test_ppu();

        // Test VRAM access during different modes
        ppu.write_vram(0x8000, 0x42);

        // HBlank: should allow access
        ppu.mode = LcdMode::HBlank;
        assert_eq!(ppu.read_vram(0x8000), 0x42);

        // Drawing: should block access
        ppu.mode = LcdMode::Drawing;
        assert_eq!(ppu.read_vram(0x8000), 0xFF); // Returns open bus value

        ppu.write_vram(0x8000, 0x99); // Should be ignored
        ppu.mode = LcdMode::HBlank;
        assert_eq!(ppu.read_vram(0x8000), 0x42); // Value unchanged
    }

    #[test]
    fn test_oam_access_restrictions() {
        let mut ppu = create_test_ppu();

        // Start in HBlank mode to allow initial write
        ppu.mode = LcdMode::HBlank;
        ppu.write_oam(0xFE00, 0x42);

        // HBlank: should allow access
        assert_eq!(ppu.read_oam(0xFE00), 0x42);

        // OAM Scan: should block access
        ppu.mode = LcdMode::OamScan;
        assert_eq!(ppu.read_oam(0xFE00), 0xFF); // Returns open bus value

        // Drawing: should block access
        ppu.mode = LcdMode::Drawing;
        assert_eq!(ppu.read_oam(0xFE00), 0xFF); // Returns open bus value
    }

    #[test]
    fn test_frame_ready_flag() {
        let mut ppu = create_test_ppu();

        assert!(!ppu.is_frame_ready());

        // Manually trigger frame completion
        ppu.frame_ready = true;
        assert!(ppu.is_frame_ready());

        // Getting framebuffer should clear the flag
        let _framebuffer = ppu.get_framebuffer();
        assert!(!ppu.is_frame_ready());
    }

    #[test]
    fn test_stat_interrupts() {
        let mut ppu = create_test_ppu();

        // Enable HBlank interrupt
        ppu.registers.write_stat(0x08); // Bit 3 = HBlank interrupt enable

        // Transition to HBlank should generate interrupt
        ppu.mode = LcdMode::Drawing;
        ppu.dots = 172;

        let interrupt = ppu.step(1);
        assert_eq!(interrupt, Some(Interrupt::LCD));
        assert_eq!(ppu.mode, LcdMode::HBlank);
    }

    #[test]
    fn test_lyc_interrupt() {
        let mut ppu = create_test_ppu();

        // Set up LYC interrupt
        ppu.registers.write_stat(0x40); // Enable LYC interrupt
        ppu.registers.set_lyc(50);       // Trigger on line 50

        // Set up HBlank at line 49
        ppu.mode = LcdMode::HBlank;
        ppu.scanline = 49;
        ppu.dots = 204;

        // Should trigger LYC interrupt when moving to line 50
        let interrupt = ppu.step(1);
        assert_eq!(interrupt, Some(Interrupt::LCD));
        assert_eq!(ppu.scanline, 50);
        assert!(ppu.registers.get_lyc_flag());
    }

    #[test]
    fn test_scanline_rendering() {
        let mut ppu = create_test_ppu();

        // Enable background rendering
        ppu.registers.write_lcdc(0x81); // LCD on, background on

        // Test that render_scanline doesn't panic
        ppu.scanline = 50;
        ppu.render_scanline();

        // With empty VRAM, background should render as color 0 (after palette translation)
        // The background palette (BGP) default is 0xFC, so color 0 maps to palette color 0
        for x in 0..160 {
            let expected_bg_color = ppu.registers.get_bg_color(0); // Background tile color 0
            assert_eq!(ppu.line_buffer[x], expected_bg_color);
        }
    }

    #[test]
    fn test_display_integration() {
        let mut ppu = create_test_ppu();
        let mut display = DebugDisplay::new();

        // Simulate frame completion
        ppu.frame_ready = true;

        if ppu.is_frame_ready() {
            let framebuffer = ppu.get_framebuffer();
            display.present_frame(framebuffer);
        }

        assert_eq!(display.frame_count(), 1);
        assert!(!ppu.is_frame_ready()); // Should be cleared after getting framebuffer
    }

    #[test]
    fn test_full_frame_cycle() {
        let mut ppu = create_test_ppu();
        let mut frame_count = 0;

        // Simulate multiple frames
        for _frame in 0..2 {
            for _line in 0..154 {
                // Simulate one complete scanline
                while ppu.get_current_line() == _line {
                    let _interrupt = ppu.step(10);

                    if ppu.is_frame_ready() {
                        frame_count += 1;
                        let _framebuffer = ppu.get_framebuffer();
                        break;
                    }
                }
            }
        }

        assert_eq!(frame_count, 2);
    }
}