use super::timing::LcdMode;

/// PPU Registers
/// Manages all LCD control and status registers
pub struct PpuRegisters {
    /// LCDC (0xFF40) - LCD Control Register
    lcdc: u8,

    /// STAT (0xFF41) - LCD Status Register
    stat: u8,

    /// SCY (0xFF42) - Background Scroll Y
    scy: u8,

    /// SCX (0xFF43) - Background Scroll X
    scx: u8,

    /// LY (0xFF44) - Current Scanline (read-only)
    ly: u8,

    /// LYC (0xFF45) - Scanline Compare
    lyc: u8,

    /// BGP (0xFF47) - Background Palette Data
    bgp: u8,

    /// OBP0 (0xFF48) - Object Palette 0 Data
    obp0: u8,

    /// OBP1 (0xFF49) - Object Palette 1 Data
    obp1: u8,

    /// WY (0xFF4A) - Window Y Position
    wy: u8,

    /// WX (0xFF4B) - Window X Position
    wx: u8,
}

impl Default for PpuRegisters {
    fn default() -> Self {
        Self::new()
    }
}

impl PpuRegisters {
    /// Create new PPU registers with Game Boy boot values
    pub fn new() -> Self {
        Self {
            lcdc: 0x91,   // LCD enabled, background on, sprites on
            stat: 0x00,   // Mode 0, no interrupts
            scy: 0x00,
            scx: 0x00,
            ly: 0x00,
            lyc: 0x00,
            bgp: 0xFC,    // Standard Game Boy palette
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0x00,
            wx: 0x00,
        }
    }

    /// Reset registers to boot state
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    // LCDC Register (0xFF40) methods

    /// Check if LCD is enabled (LCDC.7)
    pub fn is_lcd_enabled(&self) -> bool {
        self.lcdc & 0x80 != 0
    }

    /// Check window tile map area (LCDC.6)
    /// false = 0x9800-0x9BFF, true = 0x9C00-0x9FFF
    pub fn window_tile_map_area(&self) -> bool {
        self.lcdc & 0x40 != 0
    }

    /// Check if window is enabled (LCDC.5)
    pub fn is_window_enabled(&self) -> bool {
        self.lcdc & 0x20 != 0
    }

    /// Check background & window tile data area (LCDC.4)
    /// false = 0x8800-0x97FF (signed), true = 0x8000-0x8FFF (unsigned)
    pub fn bg_window_tile_data_area(&self) -> bool {
        self.lcdc & 0x10 != 0
    }

    /// Check background tile map area (LCDC.3)
    /// false = 0x9800-0x9BFF, true = 0x9C00-0x9FFF
    pub fn bg_tile_map_area(&self) -> bool {
        self.lcdc & 0x08 != 0
    }

    /// Check sprite size (LCDC.2)
    /// false = 8x8, true = 8x16
    pub fn sprite_size(&self) -> bool {
        self.lcdc & 0x04 != 0
    }

    /// Check if sprites are enabled (LCDC.1)
    pub fn is_sprites_enabled(&self) -> bool {
        self.lcdc & 0x02 != 0
    }

    /// Check if background/window is enabled (LCDC.0)
    pub fn is_bg_window_enabled(&self) -> bool {
        self.lcdc & 0x01 != 0
    }

    /// Read LCDC register
    pub fn read_lcdc(&self) -> u8 {
        self.lcdc
    }

    /// Write LCDC register
    pub fn write_lcdc(&mut self, value: u8) {
        self.lcdc = value;
    }

    // STAT Register (0xFF41) methods

    /// Check if LYC=LY interrupt is enabled (STAT.6)
    pub fn is_lyc_interrupt_enabled(&self) -> bool {
        self.stat & 0x40 != 0
    }

    /// Check if OAM interrupt is enabled (STAT.5)
    pub fn is_oam_interrupt_enabled(&self) -> bool {
        self.stat & 0x20 != 0
    }

    /// Check if VBlank interrupt is enabled (STAT.4)
    pub fn is_vblank_interrupt_enabled(&self) -> bool {
        self.stat & 0x10 != 0
    }

    /// Check if HBlank interrupt is enabled (STAT.3)
    pub fn is_hblank_interrupt_enabled(&self) -> bool {
        self.stat & 0x08 != 0
    }

    /// Get LYC=LY flag (STAT.2)
    pub fn get_lyc_flag(&self) -> bool {
        self.stat & 0x04 != 0
    }

    /// Set LYC=LY flag (STAT.2)
    pub fn set_lyc_flag(&mut self, value: bool) {
        if value {
            self.stat |= 0x04;
        } else {
            self.stat &= !0x04;
        }
    }

    /// Get current LCD mode (STAT.1-0)
    pub fn get_mode(&self) -> LcdMode {
        match self.stat & 0x03 {
            0 => LcdMode::HBlank,
            1 => LcdMode::VBlank,
            2 => LcdMode::OamScan,
            3 => LcdMode::Drawing,
            _ => unreachable!(),
        }
    }

    /// Set LCD mode (STAT.1-0)
    pub fn set_mode(&mut self, mode: LcdMode) {
        self.stat = (self.stat & 0xFC) | (mode as u8);
    }

    /// Read STAT register
    pub fn read_stat(&self) -> u8 {
        self.stat | 0x80 // Bit 7 is always 1
    }

    /// Write STAT register (only bits 6-3 are writable)
    pub fn write_stat(&mut self, value: u8) {
        self.stat = (self.stat & 0x07) | (value & 0x78);
    }

    // Scroll registers

    /// Get background scroll Y
    pub fn get_scy(&self) -> u8 {
        self.scy
    }

    /// Set background scroll Y
    pub fn set_scy(&mut self, value: u8) {
        self.scy = value;
    }

    /// Get background scroll X
    pub fn get_scx(&self) -> u8 {
        self.scx
    }

    /// Set background scroll X
    pub fn set_scx(&mut self, value: u8) {
        self.scx = value;
    }

    // Scanline registers

    /// Get current scanline (LY)
    pub fn get_ly(&self) -> u8 {
        self.ly
    }

    /// Set current scanline (internal use only)
    pub(super) fn set_ly(&mut self, value: u8) {
        self.ly = value;
    }

    /// Get scanline compare value (LYC)
    pub fn get_lyc(&self) -> u8 {
        self.lyc
    }

    /// Set scanline compare value (LYC)
    pub fn set_lyc(&mut self, value: u8) {
        self.lyc = value;
    }

    // Palette registers

    /// Get background palette
    pub fn get_bgp(&self) -> u8 {
        self.bgp
    }

    /// Set background palette
    pub fn set_bgp(&mut self, value: u8) {
        self.bgp = value;
    }

    /// Get object palette 0
    pub fn get_obp0(&self) -> u8 {
        self.obp0
    }

    /// Set object palette 0
    pub fn set_obp0(&mut self, value: u8) {
        self.obp0 = value;
    }

    /// Get object palette 1
    pub fn get_obp1(&self) -> u8 {
        self.obp1
    }

    /// Set object palette 1
    pub fn set_obp1(&mut self, value: u8) {
        self.obp1 = value;
    }

    /// Convert palette data to color indices
    pub fn get_bg_color(&self, color_id: u8) -> u8 {
        (self.bgp >> (color_id * 2)) & 0x03
    }

    /// Convert sprite palette data to color indices
    pub fn get_sprite_color(&self, palette: u8, color_id: u8) -> u8 {
        let palette_data = if palette == 0 { self.obp0 } else { self.obp1 };
        (palette_data >> (color_id * 2)) & 0x03
    }

    // Window position registers

    /// Get window Y position
    pub fn get_wy(&self) -> u8 {
        self.wy
    }

    /// Set window Y position
    pub fn set_wy(&mut self, value: u8) {
        self.wy = value;
    }

    /// Get window X position
    pub fn get_wx(&self) -> u8 {
        self.wx
    }

    /// Set window X position
    pub fn set_wx(&mut self, value: u8) {
        self.wx = value;
    }

    /// Read from any PPU register by address
    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.read_lcdc(),
            0xFF41 => self.read_stat(),
            0xFF42 => self.get_scy(),
            0xFF43 => self.get_scx(),
            0xFF44 => self.get_ly(),
            0xFF45 => self.get_lyc(),
            0xFF47 => self.get_bgp(),
            0xFF48 => self.get_obp0(),
            0xFF49 => self.get_obp1(),
            0xFF4A => self.get_wy(),
            0xFF4B => self.get_wx(),
            _ => 0xFF, // Invalid register
        }
    }

    /// Write to any PPU register by address
    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF40 => self.write_lcdc(value),
            0xFF41 => self.write_stat(value),
            0xFF42 => self.set_scy(value),
            0xFF43 => self.set_scx(value),
            0xFF44 => {}, // LY is read-only
            0xFF45 => self.set_lyc(value),
            0xFF47 => self.set_bgp(value),
            0xFF48 => self.set_obp0(value),
            0xFF49 => self.set_obp1(value),
            0xFF4A => self.set_wy(value),
            0xFF4B => self.set_wx(value),
            _ => {}, // Invalid register, ignore
        }
    }
}