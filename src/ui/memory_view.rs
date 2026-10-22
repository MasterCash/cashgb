//! Hex viewer for the ROM file and the CPU address space.

use std::ops::RangeInclusive;

use egui::{
    Color32, FontId, TextFormat,
    text::{LayoutJob, TextWrapping},
};

use crate::cpu::Cpu;

/// What the memory viewer shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegion {
    /// The raw ROM file, by file offset (every bank, not just the mapped ones)
    RomFile,
    /// The whole 64 KiB CPU address space
    AddressSpace,
    RomBank0,
    RomBankN,
    Vram,
    CartRam,
    Wram,
    Oam,
    Io,
    Hram,
}

impl MemoryRegion {
    pub const ALL: [MemoryRegion; 10] = [
        MemoryRegion::RomFile,
        MemoryRegion::AddressSpace,
        MemoryRegion::RomBank0,
        MemoryRegion::RomBankN,
        MemoryRegion::Vram,
        MemoryRegion::CartRam,
        MemoryRegion::Wram,
        MemoryRegion::Oam,
        MemoryRegion::Io,
        MemoryRegion::Hram,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MemoryRegion::RomFile => "ROM file",
            MemoryRegion::AddressSpace => "Address space (0000-FFFF)",
            MemoryRegion::RomBank0 => "ROM bank 0 (0000-3FFF)",
            MemoryRegion::RomBankN => "ROM bank N (4000-7FFF)",
            MemoryRegion::Vram => "VRAM (8000-9FFF)",
            MemoryRegion::CartRam => "Cart RAM (A000-BFFF)",
            MemoryRegion::Wram => "WRAM (C000-DFFF)",
            MemoryRegion::Oam => "OAM (FE00-FE9F)",
            MemoryRegion::Io => "I/O (FF00-FF7F)",
            MemoryRegion::Hram => "HRAM + IE (FF80-FFFF)",
        }
    }

    /// CPU addresses covered, or `None` for the ROM file view
    pub fn address_range(self) -> Option<RangeInclusive<u16>> {
        match self {
            MemoryRegion::RomFile => None,
            MemoryRegion::AddressSpace => Some(0x0000..=0xFFFF),
            MemoryRegion::RomBank0 => Some(0x0000..=0x3FFF),
            MemoryRegion::RomBankN => Some(0x4000..=0x7FFF),
            MemoryRegion::Vram => Some(0x8000..=0x9FFF),
            MemoryRegion::CartRam => Some(0xA000..=0xBFFF),
            MemoryRegion::Wram => Some(0xC000..=0xDFFF),
            MemoryRegion::Oam => Some(0xFE00..=0xFE9F),
            MemoryRegion::Io => Some(0xFF00..=0xFF7F),
            MemoryRegion::Hram => Some(0xFF80..=0xFFFF),
        }
    }
}

/// The bytes the viewer is currently showing, addressed from `base`.
struct ByteSource<'a> {
    base: usize,
    len: usize,
    /// Digits needed to print the highest address
    address_digits: usize,
    /// Position in this view of the byte at PC, if visible
    pc: Option<usize>,
    /// Position in this view of the byte at SP, if visible
    sp: Option<usize>,
    read: Box<dyn Fn(usize) -> u8 + 'a>,
}

impl<'a> ByteSource<'a> {
    fn new(region: MemoryRegion, cpu: &'a Cpu) -> Self {
        let pc = cpu.get_pc();
        let sp = cpu.get_sp();
        match region.address_range() {
            None => {
                let mbc = &cpu.memory.cart.mbc;
                let rom = mbc.rom();
                Self {
                    base: 0,
                    len: rom.len(),
                    address_digits: hex_digits(rom.len().saturating_sub(1)).max(4),
                    // PC maps into the file through the current bank; SP never points at ROM
                    pc: mbc.rom_offset(pc),
                    sp: None,
                    read: Box::new(move |offset| rom[offset]),
                }
            }
            Some(range) => {
                let start = *range.start() as usize;
                let end = *range.end() as usize;
                let position = |addr: u16| {
                    (start..=end)
                        .contains(&(addr as usize))
                        .then(|| addr as usize - start)
                };
                Self {
                    base: start,
                    len: end - start + 1,
                    address_digits: 4,
                    pc: position(pc),
                    sp: position(sp),
                    read: Box::new(move |offset| cpu.read_memory((start + offset) as u16)),
                }
            }
        }
    }
}

fn hex_digits(value: usize) -> usize {
    (usize::BITS - value.leading_zeros()).div_ceil(4).max(1) as usize
}

pub struct MemoryView {
    region: MemoryRegion,
    /// Text in the "go to address" box
    goto_text: String,
    /// Row to scroll to on the next frame, after a "go to"
    scroll_to_row: Option<usize>,
}

impl Default for MemoryView {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryView {
    pub fn new() -> Self {
        Self::with_region(MemoryRegion::RomFile)
    }

    /// A viewer starting on the given region
    pub fn with_region(region: MemoryRegion) -> Self {
        Self {
            region,
            goto_text: String::new(),
            scroll_to_row: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, cpu: &Cpu) {
        let source = ByteSource::new(self.region, cpu);
        let font = FontId::monospace(egui::TextStyle::Monospace.resolve(ui.style()).size);
        let char_width = ui.fonts_mut(|f| f.glyph_width(&font, '0'));
        let bytes_per_row = bytes_per_row(ui.available_width(), char_width, source.address_digits);

        self.controls(ui, &source, bytes_per_row);
        ui.separator();

        let row_height = ui.fonts_mut(|f| f.row_height(&font));
        let total_rows = source.len.div_ceil(bytes_per_row);
        let mut scroll_area = egui::ScrollArea::both().auto_shrink([false, false]);
        if let Some(row) = self.scroll_to_row.take() {
            let spacing = ui.spacing().item_spacing.y;
            scroll_area = scroll_area.vertical_scroll_offset(row as f32 * (row_height + spacing));
        }
        scroll_area.show_rows(ui, row_height, total_rows, |ui, rows| {
            for row in rows {
                let job = row_layout(&source, row, bytes_per_row, &font, ui.visuals());
                ui.label(job);
            }
        });
    }

    fn controls(&mut self, ui: &mut egui::Ui, source: &ByteSource, bytes_per_row: usize) {
        ui.horizontal_wrapped(|ui| {
            egui::ComboBox::from_id_salt("memory_region")
                .selected_text(self.region.label())
                .show_ui(ui, |ui| {
                    for region in MemoryRegion::ALL {
                        if ui
                            .selectable_value(&mut self.region, region, region.label())
                            .changed()
                        {
                            self.scroll_to_row = Some(0);
                        }
                    }
                });

            let goto = ui.add(
                egui::TextEdit::singleline(&mut self.goto_text)
                    .hint_text("go to (hex)")
                    .desired_width(80.0)
                    .font(egui::TextStyle::Monospace),
            );
            let submitted = goto.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if (submitted || ui.button("Go").clicked())
                && let Some(position) = parse_address(&self.goto_text)
                    .and_then(|addr| addr.checked_sub(source.base))
                    .filter(|position| *position < source.len)
            {
                self.scroll_to_row = Some(position / bytes_per_row);
            }
            if let Some(pc) = source.pc
                && ui.button("PC").clicked()
            {
                self.scroll_to_row = Some(pc / bytes_per_row);
            }
            if let Some(sp) = source.sp
                && ui.button("SP").clicked()
            {
                self.scroll_to_row = Some(sp / bytes_per_row);
            }
        });
    }
}

/// Parse a hex address, with or without a `0x` or `$` prefix.
fn parse_address(text: &str) -> Option<usize> {
    let text = text.trim();
    let digits = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .or_else(|| text.strip_prefix('$'))
        .unwrap_or(text);
    usize::from_str_radix(digits, 16).ok()
}

/// Largest power-of-two row width (4 to 32 bytes) that fits. Each byte takes three characters
/// in the hex column plus one in the ASCII column, after the address and three separator spaces.
fn bytes_per_row(available_width: f32, char_width: f32, address_digits: usize) -> usize {
    const SEPARATORS: usize = 3; // ": " after the address, " " before the ASCII column
    let available_chars = (available_width / char_width.max(1.0)) as usize;
    [32, 16, 8]
        .into_iter()
        .find(|bytes| address_digits + SEPARATORS + bytes * 4 <= available_chars)
        .unwrap_or(4)
}

fn row_layout(
    source: &ByteSource,
    row: usize,
    bytes_per_row: usize,
    font: &FontId,
    visuals: &egui::Visuals,
) -> LayoutJob {
    let normal = TextFormat::simple(font.clone(), visuals.text_color());
    let dim = TextFormat::simple(font.clone(), visuals.weak_text_color());
    let highlight = |color: Color32| TextFormat {
        background: color,
        color: Color32::BLACK,
        ..normal.clone()
    };

    let mut job = LayoutJob {
        wrap: TextWrapping::no_max_width(),
        ..Default::default()
    };
    let first = row * bytes_per_row;
    let last = (first + bytes_per_row).min(source.len);

    job.append(
        &format!(
            "{:0width$X}: ",
            source.base + first,
            width = source.address_digits
        ),
        0.0,
        dim.clone(),
    );

    let mut ascii = String::with_capacity(bytes_per_row);
    for position in first..last {
        let byte = (source.read)(position);
        let format = if Some(position) == source.pc {
            highlight(Color32::LIGHT_GREEN)
        } else if Some(position) == source.sp {
            highlight(Color32::LIGHT_YELLOW)
        } else {
            normal.clone()
        };
        job.append(&format!("{byte:02X}"), 0.0, format);
        job.append(" ", 0.0, normal.clone());
        ascii.push(if byte.is_ascii_graphic() || byte == b' ' {
            byte as char
        } else {
            '.'
        });
    }
    // Pad a short final row so the ASCII column lines up
    for _ in last..first + bytes_per_row {
        job.append("   ", 0.0, normal.clone());
    }
    job.append(&format!(" {ascii}"), 0.0, dim);
    job
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_address_accepts_common_prefixes() {
        assert_eq!(parse_address("c000"), Some(0xC000));
        assert_eq!(parse_address("0xFF40"), Some(0xFF40));
        assert_eq!(parse_address("$8000"), Some(0x8000));
        assert_eq!(parse_address(" 1F "), Some(0x1F));
        assert_eq!(parse_address("xyz"), None);
        assert_eq!(parse_address(""), None);
    }

    #[test]
    fn bytes_per_row_fits_width() {
        // 4 address digits: 32 bytes needs 4 + 2 + 128 + 1 = 135 characters
        assert_eq!(bytes_per_row(135.0, 1.0, 4), 32);
        assert_eq!(bytes_per_row(134.0, 1.0, 4), 16);
        assert_eq!(bytes_per_row(71.0, 1.0, 4), 16);
        assert_eq!(bytes_per_row(70.0, 1.0, 4), 8);
        assert_eq!(bytes_per_row(10.0, 1.0, 4), 4);
    }

    #[test]
    fn hex_digits_counts_address_width() {
        assert_eq!(hex_digits(0xFFFF), 4);
        assert_eq!(hex_digits(0x1FFFFF), 6);
        assert_eq!(hex_digits(0), 1);
    }

    #[test]
    fn region_ranges_cover_their_labels() {
        assert_eq!(MemoryRegion::Wram.address_range(), Some(0xC000..=0xDFFF));
        assert_eq!(MemoryRegion::Hram.address_range(), Some(0xFF80..=0xFFFF));
        assert_eq!(MemoryRegion::RomFile.address_range(), None);
    }
}
