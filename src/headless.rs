//! Run the emulator without a window, for test ROMs, CI, and tools.

use std::fmt;

use crate::{
    cpu::{Cpu, instructions::CpuStatus},
    memory::cart::Cart,
};

/// Game Boy frames per emulated second (4194304 Hz / 70224 dots per frame)
pub const FRAMES_PER_SECOND: u32 = 60;

/// How a test ROM that reports over the serial port finished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerialResult {
    /// The ROM printed "Passed"
    Passed,
    /// The ROM printed "Failed"
    Failed,
    /// The CPU hit an illegal opcode before the ROM reported a result
    CpuErrored,
    /// The frame limit was reached before the ROM reported a result
    TimedOut,
}

/// The outcome of running a test ROM, with everything it printed.
#[derive(Debug, Clone)]
pub struct SerialReport {
    pub result: SerialResult,
    /// Everything the ROM sent over the serial port
    pub output: String,
    /// Frames emulated before the result was known
    pub frames: u32,
}

impl fmt::Display for SerialReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = self.frames as f32 / FRAMES_PER_SECOND as f32;
        writeln!(
            f,
            "{:?} after {} frames ({seconds:.1}s emulated). Serial output:",
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

    /// Run a test ROM that reports over the serial port, such as Blargg's, until it prints
    /// "Passed" or "Failed", the CPU errors, or `max_frames` frames have run.
    pub fn run_serial_test(&mut self, max_frames: u32) -> SerialReport {
        let mut frames = 0;
        let result = loop {
            let output = self.cpu.serial_output();
            if contains(output, b"Passed") {
                break SerialResult::Passed;
            }
            if contains(output, b"Failed") {
                break SerialResult::Failed;
            }
            if self.cpu.status == CpuStatus::Errored {
                break SerialResult::CpuErrored;
            }
            if frames >= max_frames {
                break SerialResult::TimedOut;
            }
            self.run_frame();
            frames += 1;
        };
        SerialReport {
            result,
            output: self.serial_output(),
            frames,
        }
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
        let mut rom = vec![0u8; 0x8000];
        rom[0x100..0x104].copy_from_slice(&[0xC3, 0x50, 0x01, 0x00]); // JP 0150
        rom[0x104..0x104 + NINTENDO_LOGO.len()].copy_from_slice(&NINTENDO_LOGO);
        let mut checksum = 0u8;
        for byte in &rom[0x134..=0x14C] {
            checksum = checksum.wrapping_sub(*byte).wrapping_sub(1);
        }
        rom[0x14D] = checksum;

        let mut code = Vec::new();
        for &byte in text {
            // LD A,byte; LDH (01),A; LD A,81; LDH (02),A
            code.extend_from_slice(&[0x3E, byte, 0xE0, 0x01, 0x3E, 0x81, 0xE0, 0x02]);
        }
        code.extend_from_slice(&[0x18, 0xFE]); // JR -2
        rom[0x150..0x150 + code.len()].copy_from_slice(&code);
        Cart::new(&rom).unwrap()
    }

    #[test]
    fn serial_test_passes_when_rom_prints_passed() {
        let mut emulator = Headless::new(printing_rom(b"01-special\n\nPassed\n"));

        let report = emulator.run_serial_test(60);

        assert_eq!(report.result, SerialResult::Passed, "{report}");
        assert_eq!(report.output, "01-special\n\nPassed\n");
    }

    #[test]
    fn serial_test_fails_when_rom_prints_failed() {
        let mut emulator = Headless::new(printing_rom(b"Failed #3"));

        let report = emulator.run_serial_test(60);

        assert_eq!(report.result, SerialResult::Failed, "{report}");
    }

    #[test]
    fn serial_test_times_out_on_a_silent_rom() {
        let mut emulator = Headless::new(printing_rom(b""));

        let report = emulator.run_serial_test(5);

        assert_eq!(report.result, SerialResult::TimedOut);
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
