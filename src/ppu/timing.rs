/// LCD Mode enumeration
/// Represents the four different states the PPU cycles through
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LcdMode {
    /// Mode 0: Horizontal Blank
    /// Duration: 87-204 dots (variable)
    /// CPU can access VRAM, OAM, and CGB palettes
    HBlank = 0,

    /// Mode 1: Vertical Blank
    /// Duration: 4560 dots (10 scanlines × 456 dots)
    /// CPU can access all video memory
    VBlank = 1,

    /// Mode 2: OAM Scan
    /// Duration: 80 dots
    /// CPU can access VRAM and CGB palettes, but not OAM
    OamScan = 2,

    /// Mode 3: Drawing Pixels
    /// Duration: 172-289 dots (variable)
    /// CPU cannot access VRAM or OAM
    Drawing = 3,
}

impl LcdMode {
    /// Get the typical duration of this mode in dots
    pub fn typical_duration(&self) -> u16 {
        match self {
            LcdMode::HBlank => 204,
            LcdMode::VBlank => 456, // Per scanline in VBlank
            LcdMode::OamScan => 80,
            LcdMode::Drawing => 172,
        }
    }

    /// Check if CPU can access VRAM during this mode
    pub fn can_access_vram(&self) -> bool {
        !matches!(self, LcdMode::Drawing)
    }

    /// Check if CPU can access OAM during this mode
    pub fn can_access_oam(&self) -> bool {
        matches!(self, LcdMode::HBlank | LcdMode::VBlank)
    }

    /// Get the next mode in the sequence
    pub fn next_mode(&self, scanline: u8) -> LcdMode {
        match self {
            LcdMode::OamScan => LcdMode::Drawing,
            LcdMode::Drawing => LcdMode::HBlank,
            LcdMode::HBlank => {
                if scanline >= 144 {
                    LcdMode::VBlank
                } else {
                    LcdMode::OamScan
                }
            }
            LcdMode::VBlank => {
                if scanline >= 154 {
                    LcdMode::OamScan
                } else {
                    LcdMode::VBlank
                }
            }
        }
    }
}

/// PPU Timing Constants
pub struct PpuTiming;

impl PpuTiming {
    /// Total dots per scanline
    pub const DOTS_PER_SCANLINE: u16 = 456;

    /// Total scanlines per frame
    pub const SCANLINES_PER_FRAME: u8 = 154;

    /// Visible scanlines (0-143)
    pub const VISIBLE_SCANLINES: u8 = 144;

    /// VBlank scanlines (144-153)
    pub const VBLANK_SCANLINES: u8 = 10;

    /// Total dots per frame
    pub const DOTS_PER_FRAME: u32 = Self::DOTS_PER_SCANLINE as u32 * Self::SCANLINES_PER_FRAME as u32;

    /// OAM Scan duration
    pub const OAM_SCAN_DOTS: u16 = 80;

    /// Minimum Drawing duration
    pub const DRAWING_MIN_DOTS: u16 = 172;

    /// Maximum Drawing duration
    pub const DRAWING_MAX_DOTS: u16 = 289;

    /// Minimum HBlank duration
    pub const HBLANK_MIN_DOTS: u16 = 87;

    /// Maximum HBlank duration
    pub const HBLANK_MAX_DOTS: u16 = 204;

    /// VBlank total duration (for all 10 scanlines)
    pub const VBLANK_TOTAL_DOTS: u32 = 4560;

    /// Screen width in pixels
    pub const SCREEN_WIDTH: usize = 160;

    /// Screen height in pixels
    pub const SCREEN_HEIGHT: usize = 144;

    /// Calculate the duration of Drawing mode based on rendering complexity
    /// This is a simplified version - real hardware has complex timing based on:
    /// - Background scrolling (SCX % 8 adds dots)
    /// - Window rendering (adds 6 dots setup penalty)
    /// - Sprite rendering (adds 6-11 dots per sprite)
    pub fn calculate_drawing_duration(
        _scroll_x: u8,
        _window_enabled: bool,
        _sprite_count: u8,
    ) -> u16 {
        // For now, return the minimum duration
        // TODO: Implement actual timing penalties
        Self::DRAWING_MIN_DOTS
    }

    /// Calculate HBlank duration based on Drawing duration
    pub fn calculate_hblank_duration(drawing_duration: u16) -> u16 {
        Self::DOTS_PER_SCANLINE - Self::OAM_SCAN_DOTS - drawing_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcd_mode_durations() {
        assert_eq!(LcdMode::OamScan.typical_duration(), 80);
        assert_eq!(LcdMode::Drawing.typical_duration(), 172);
        assert_eq!(LcdMode::HBlank.typical_duration(), 204);
        assert_eq!(LcdMode::VBlank.typical_duration(), 456);
    }

    #[test]
    fn test_memory_access_permissions() {
        assert!(LcdMode::HBlank.can_access_vram());
        assert!(LcdMode::VBlank.can_access_vram());
        assert!(LcdMode::OamScan.can_access_vram());
        assert!(!LcdMode::Drawing.can_access_vram());

        assert!(LcdMode::HBlank.can_access_oam());
        assert!(LcdMode::VBlank.can_access_oam());
        assert!(!LcdMode::OamScan.can_access_oam());
        assert!(!LcdMode::Drawing.can_access_oam());
    }

    #[test]
    fn test_mode_transitions() {
        // Normal scanline progression
        assert_eq!(LcdMode::OamScan.next_mode(0), LcdMode::Drawing);
        assert_eq!(LcdMode::Drawing.next_mode(0), LcdMode::HBlank);
        assert_eq!(LcdMode::HBlank.next_mode(0), LcdMode::OamScan);

        // Transition to VBlank
        assert_eq!(LcdMode::HBlank.next_mode(144), LcdMode::VBlank);
        assert_eq!(LcdMode::VBlank.next_mode(150), LcdMode::VBlank);

        // Transition back to new frame
        assert_eq!(LcdMode::VBlank.next_mode(154), LcdMode::OamScan);
    }

    #[test]
    fn test_timing_constants() {
        assert_eq!(PpuTiming::DOTS_PER_SCANLINE, 456);
        assert_eq!(PpuTiming::SCANLINES_PER_FRAME, 154);
        assert_eq!(PpuTiming::VISIBLE_SCANLINES, 144);
        assert_eq!(PpuTiming::SCREEN_WIDTH, 160);
        assert_eq!(PpuTiming::SCREEN_HEIGHT, 144);
    }

    #[test]
    fn test_drawing_and_hblank_duration_calculation() {
        let drawing_duration = PpuTiming::calculate_drawing_duration(0, false, 0);
        let hblank_duration = PpuTiming::calculate_hblank_duration(drawing_duration);

        // Total should equal one scanline
        assert_eq!(
            PpuTiming::OAM_SCAN_DOTS + drawing_duration + hblank_duration,
            PpuTiming::DOTS_PER_SCANLINE
        );
    }
}