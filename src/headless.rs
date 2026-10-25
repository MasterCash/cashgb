//! Run the emulator without a window, for test ROMs, CI, and tools.

use std::fmt;

use crate::{
    cpu::{Cpu, instructions::CpuStatus},
    memory::cart::Cart,
    ppu::display::ColorConverter,
};

/// Game Boy frames per emulated second (4194304 Hz / 70224 dots per frame)
pub const FRAMES_PER_SECOND: u32 = 60;

/// Screen width in pixels
pub const SCREEN_WIDTH: usize = 160;
/// Screen height in pixels
pub const SCREEN_HEIGHT: usize = 144;

/// The bytes a Mooneye test ROM sends over the serial port when it passes
const MOONEYE_PASSED: [u8; 6] = [3, 5, 8, 13, 21, 34];
/// The bytes a Mooneye test ROM sends over the serial port when it fails
const MOONEYE_FAILED: [u8; 6] = [0x42; 6];

/// How a test ROM finished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestResult {
    /// The ROM reported that it passed
    Passed,
    /// The ROM reported that it failed
    Failed,
    /// The CPU hit an illegal opcode before the ROM reported a result
    CpuErrored,
    /// The frame limit was reached before the ROM reported a result
    TimedOut,
}

/// The outcome of running a test ROM, with everything it printed.
#[derive(Debug, Clone)]
pub struct TestReport {
    pub result: TestResult,
    /// Everything the ROM printed, over the serial port or on screen
    pub output: String,
    /// Frames emulated before the result was known
    pub frames: u32,
}

impl fmt::Display for TestReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = self.frames as f32 / FRAMES_PER_SECOND as f32;
        writeln!(
            f,
            "{:?} after {} frames ({seconds:.1}s emulated). Output:",
            self.result, self.frames
        )?;
        write!(f, "{}", self.output)
    }
}

/// An emulator with no window. Runs as fast as the host allows.
pub struct Headless {
    cpu: Cpu,
}

impl Headless {
    pub fn new(cart: Cart) -> Self {
        Self {
            cpu: Cpu::new(cart),
        }
    }

    /// Run one frame (see [`Cpu::run_frame`]).
    pub fn run_frame(&mut self) {
        self.cpu.run_frame();
        // Clear the flag so the next call runs a whole new frame
        if self.cpu.is_frame_ready() {
            self.cpu.get_framebuffer();
        }
    }

    pub fn run_frames(&mut self, frames: u32) {
        for _ in 0..frames {
            self.run_frame();
        }
    }

    /// Everything sent over the serial port so far, as text.
    pub fn serial_output(&self) -> String {
        String::from_utf8_lossy(self.cpu.serial_output()).into_owned()
    }

    /// The current screen as 160×144 RGBA pixels.
    pub fn framebuffer(&mut self) -> &[u8] {
        self.cpu.get_framebuffer()
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    /// The current screen as one shade per pixel, row by row: 0 is the lightest and 3 the
    /// darkest. Screenshot tests compare these, so they don't depend on the display palette.
    ///
    /// # Panics
    ///
    /// If a pixel isn't one of the four palette colours, which would be a PPU bug.
    pub fn screen_shades(&mut self) -> Vec<u8> {
        let colors: [[u8; 4]; 4] =
            std::array::from_fn(|shade| ColorConverter::gb_color_to_rgba(shade as u8));
        self.framebuffer()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| {
                colors
                    .iter()
                    .position(|color| color == pixel)
                    .unwrap_or_else(|| panic!("pixel {pixel:?} is not a palette colour"))
                    as u8
            })
            .collect()
    }

    /// The background tile map as text, one line per row of tiles, for test ROMs whose font
    /// tiles are numbered by their ASCII code, such as Blargg's. Other tiles become spaces,
    /// and trailing spaces are trimmed.
    pub fn background_text(&self) -> String {
        // LCDC bit 3 selects the tile map the background is drawn from
        let map: u16 = if self.cpu.read_memory(0xFF40) & 0x08 != 0 {
            0x9C00
        } else {
            0x9800
        };
        (0..32)
            .map(|row| {
                let line: String = (0..32)
                    .map(
                        |column| match self.cpu.read_memory(map + row * 32 + column) {
                            tile @ 0x20..=0x7E => tile as char,
                            _ => ' ',
                        },
                    )
                    .collect();
                line.trim_end().to_owned()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Run a test ROM that reports over the serial port, such as Blargg's, until it prints
    /// "Passed" or "Failed", the CPU errors, or `max_frames` frames have run.
    pub fn run_serial_test(&mut self, max_frames: u32) -> TestReport {
        let (result, frames) = self.run_until_result(max_frames, |emulator| {
            passed_or_failed(&emulator.serial_output())
        });
        TestReport {
            result,
            output: self.serial_output(),
            frames,
        }
    }

    /// Run a test ROM that prints its result only on screen, such as Blargg's `halt_bug`,
    /// until "Passed" or "Failed" appears in the background tile map (see
    /// [`background_text`](Self::background_text)), the CPU errors, or `max_frames` frames
    /// have run.
    pub fn run_screen_text_test(&mut self, max_frames: u32) -> TestReport {
        let (result, frames) = self.run_until_result(max_frames, |emulator| {
            passed_or_failed(&emulator.background_text())
        });
        TestReport {
            result,
            output: self.background_text(),
            frames,
        }
    }

    /// Run a Mooneye test ROM until it sends its pass or fail bytes over the serial port,
    /// the CPU errors, or `max_frames` frames have run. The report's output lists the bytes
    /// sent, in hex, since they aren't text.
    pub fn run_mooneye_test(&mut self, max_frames: u32) -> TestReport {
        let (result, frames) = self.run_until_result(max_frames, |emulator| {
            let output = emulator.cpu.serial_output();
            if contains(output, &MOONEYE_PASSED) {
                Some(TestResult::Passed)
            } else if contains(output, &MOONEYE_FAILED) {
                Some(TestResult::Failed)
            } else {
                None
            }
        });
        TestReport {
            result,
            output: format!("{:02X?}", self.cpu.serial_output()),
            frames,
        }
    }

    /// Run frames until `check` finds a result, the CPU errors, or `max_frames` frames have
    /// run. Returns the result and the frames run.
    fn run_until_result(
        &mut self,
        max_frames: u32,
        check: impl Fn(&Self) -> Option<TestResult>,
    ) -> (TestResult, u32) {
        let mut frames = 0;
        let result = loop {
            if let Some(result) = check(self) {
                break result;
            }
            if self.cpu.status == CpuStatus::Errored {
                break TestResult::CpuErrored;
            }
            if frames >= max_frames {
                break TestResult::TimedOut;
            }
            self.run_frame();
            frames += 1;
        };
        (result, frames)
    }
}

/// The result in text a test ROM printed, if it has printed "Passed" or "Failed" yet.
fn passed_or_failed(output: &str) -> Option<TestResult> {
    if output.contains("Passed") {
        Some(TestResult::Passed)
    } else if output.contains("Failed") {
        Some(TestResult::Failed)
    } else {
        None
    }
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::cart::cart_header::NINTENDO_LOGO;

    /// A ROM that prints `text` over the serial port and then loops forever.
    fn printing_rom(text: &[u8]) -> Cart {
        let mut code = Vec::new();
        for &byte in text {
            // LD A,byte; LDH (01),A; LD A,81; LDH (02),A
            code.extend_from_slice(&[0x3E, byte, 0xE0, 0x01, 0x3E, 0x81, 0xE0, 0x02]);
        }
        rom_running(&code)
    }

    /// A ROM that turns the LCD off, writes `text` to the start of the 0x9800 tile map, and
    /// then loops forever.
    fn screen_printing_rom(text: &[u8]) -> Cart {
        // XOR A; LDH (40),A; LD HL,9800
        let mut code = vec![0xAF, 0xE0, 0x40, 0x21, 0x00, 0x98];
        for &byte in text {
            // LD A,byte; LD (HL+),A
            code.extend_from_slice(&[0x3E, byte, 0x22]);
        }
        rom_running(&code)
    }

    /// A ROM that runs `code` from 0x150 and then loops forever.
    fn rom_running(code: &[u8]) -> Cart {
        let mut rom = vec![0u8; 0x8000];
        rom[0x100..0x104].copy_from_slice(&[0xC3, 0x50, 0x01, 0x00]); // JP 0150
        rom[0x104..0x104 + NINTENDO_LOGO.len()].copy_from_slice(&NINTENDO_LOGO);
        let mut checksum = 0u8;
        for byte in &rom[0x134..=0x14C] {
            checksum = checksum.wrapping_sub(*byte).wrapping_sub(1);
        }
        rom[0x14D] = checksum;

        rom[0x150..0x150 + code.len()].copy_from_slice(code);
        rom[0x150 + code.len()..][..2].copy_from_slice(&[0x18, 0xFE]); // JR -2
        Cart::new(&rom).unwrap()
    }

    #[test]
    fn screen_text_test_passes_when_rom_shows_passed() {
        let mut emulator = Headless::new(screen_printing_rom(b"Passed"));

        let report = emulator.run_screen_text_test(60);

        assert_eq!(report.result, TestResult::Passed, "{report}");
        assert!(report.output.starts_with("Passed\n"), "{report}");
    }

    #[test]
    fn screen_text_test_fails_when_rom_shows_failed() {
        let mut emulator = Headless::new(screen_printing_rom(b"Failed"));

        let report = emulator.run_screen_text_test(60);

        assert_eq!(report.result, TestResult::Failed, "{report}");
    }

    #[test]
    fn serial_test_passes_when_rom_prints_passed() {
        let mut emulator = Headless::new(printing_rom(b"01-special\n\nPassed\n"));

        let report = emulator.run_serial_test(60);

        assert_eq!(report.result, TestResult::Passed, "{report}");
        assert_eq!(report.output, "01-special\n\nPassed\n");
    }

    #[test]
    fn serial_test_fails_when_rom_prints_failed() {
        let mut emulator = Headless::new(printing_rom(b"Failed #3"));

        let report = emulator.run_serial_test(60);

        assert_eq!(report.result, TestResult::Failed, "{report}");
    }

    #[test]
    fn mooneye_test_passes_on_fibonacci_bytes() {
        let mut emulator = Headless::new(printing_rom(&MOONEYE_PASSED));

        let report = emulator.run_mooneye_test(60);

        assert_eq!(report.result, TestResult::Passed, "{report}");
        assert_eq!(report.output, "[03, 05, 08, 0D, 15, 22]");
    }

    #[test]
    fn mooneye_test_fails_on_0x42_bytes() {
        let mut emulator = Headless::new(printing_rom(&MOONEYE_FAILED));

        let report = emulator.run_mooneye_test(60);

        assert_eq!(report.result, TestResult::Failed, "{report}");
    }

    #[test]
    fn screen_shades_has_one_shade_per_pixel() {
        let mut emulator = Headless::new(printing_rom(b""));
        emulator.run_frames(2);

        let shades = emulator.screen_shades();

        assert_eq!(shades.len(), SCREEN_WIDTH * SCREEN_HEIGHT);
        assert!(shades.iter().all(|&shade| shade <= 3));
    }

    #[test]
    fn serial_test_times_out_on_a_silent_rom() {
        let mut emulator = Headless::new(printing_rom(b""));

        let report = emulator.run_serial_test(5);

        assert_eq!(report.result, TestResult::TimedOut);
        assert_eq!(report.frames, 5);
    }

    #[test]
    fn run_frame_advances_a_whole_frame_each_call() {
        let mut emulator = Headless::new(printing_rom(b""));

        emulator.run_frame();
        assert!(!emulator.cpu().is_frame_ready(), "flag consumed");
        let div_before = emulator.cpu().read_memory(0xFF04);
        emulator.run_frame();

        // DIV ticks every 64 M-cycles: a full frame (17556 M-cycles) moves it by 274 mod 256
        let div_after = emulator.cpu().read_memory(0xFF04);
        assert_eq!(div_after.wrapping_sub(div_before), (17556 / 64 % 256) as u8);
    }
}
