pub mod cart;
pub mod controller;
pub mod joypad;
pub mod serial;

#[cfg(test)]
mod tests;

use crate::cpu::instructions::{CpuStatus, Interrupt};
use crate::ppu::Ppu;
use cart::Cart;
use joypad::{Button, Joypad};
use log::trace;
use serial::Serial;

pub mod memory_sizes {
    pub const MEM_4_KILOBYTES: u32 = 0x1000;
    pub const MEM_8_KILOBYTES: u32 = 0x2000;
    pub const MEM_16_KILOBYTES: u32 = MEM_8_KILOBYTES * 2;
    pub const MEM_32_KILOBYTES: u32 = MEM_16_KILOBYTES * 2;
    pub const MEM_64_KILOBYTES: u32 = MEM_32_KILOBYTES * 2;
    pub const MEM_128_KILOBYTES: u32 = MEM_64_KILOBYTES * 2;
    pub const MEM_256_KILOBYTES: u32 = MEM_128_KILOBYTES * 2;
    pub const MEM_512_KILOBYTES: u32 = MEM_256_KILOBYTES * 2;
    pub const MEM_1_MEGABYTE: u32 = MEM_512_KILOBYTES * 2;
    pub const MEM_2_MEGABYTES: u32 = MEM_1_MEGABYTE * 2;
    pub const MEM_BUFF_SIZE: usize = 160 * 144 * 4;
}

pub struct MemoryBus {
    pub(crate) cart: Cart,
    pub(crate) ppu: Ppu,
    pub(crate) w_ram: Vec<[u8; memory_sizes::MEM_4_KILOBYTES as usize]>,
    pub(crate) w_ram_bank: u8,
    pub(crate) io_registers: [u8; 0x80],
    pub(crate) h_ram: [u8; 0x80],
    pub(crate) ie: u8,
    pub(crate) joypad: Joypad,
    pub(crate) serial: Serial,
}

impl MemoryBus {
    pub fn new(cart: Cart) -> Self {
        Self {
            cart,
            ppu: Ppu::new(),
            w_ram: vec![[0; memory_sizes::MEM_4_KILOBYTES as usize]; 8],
            w_ram_bank: 1,
            io_registers: [0; 0x80],
            h_ram: [0; 0x80],
            ie: 0,
            joypad: Joypad::new(),
            serial: Serial::new(),
        }
    }
    pub fn empty() -> Self {
        Self {
            cart: Cart::empty(),
            ppu: Ppu::new(),
            w_ram: vec![[0; memory_sizes::MEM_4_KILOBYTES as usize]; 8],
            w_ram_bank: 1,
            io_registers: [0; 0x80],
            h_ram: [0; 0x80],
            ie: 0,
            joypad: Joypad::new(),
            serial: Serial::new(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // ROM banks and cartridge RAM
            0x0000..=0x7fff | 0xA000..=0xbfff => self.cart.read(&addr).unwrap_or(0xFF),

            // Video RAM - route through PPU for timing restrictions
            0x8000..=0x9fff => self.ppu.read_vram(addr),

            // Work RAM bank 0
            0xC000..=0xcfff => self.w_ram[0][(addr - 0xC000) as usize],

            // Work RAM bank 1-7 (switchable)
            0xd000..=0xdfff => self.w_ram[self.w_ram_bank as usize][(addr - 0xd000) as usize],

            // Echo RAM (mirrors work RAM)
            0xe000..=0xefff => self.w_ram[0][(addr - 0xe000) as usize],
            0xf000..=0xfdff => self.w_ram[self.w_ram_bank as usize][(addr - 0xf000) as usize],

            // Object Attribute Memory (OAM) - route through PPU for timing restrictions
            0xfe00..=0xfe9f => self.ppu.read_oam(addr),

            // Unusable memory area: reads 0xFF. Games touch it routinely (for example when a
            // loop clearing OAM runs past its end), so this is only traced, not warned about.
            0xfea0..=0xfeff => {
                trace!("read from unusable address {:#x}", addr);
                0xff
            }

            // I/O registers
            0xff00..=0xff7f => {
                match addr {
                    // OAM DMA source register reads back the last value written
                    0xff46 => self.io_registers[0x46],
                    // PPU registers (0xFF40-0xFF4B)
                    0xFF40..=0xFF4B => self.ppu.registers.read_register(addr),
                    // Joypad input
                    0xff00 => self.joypad.read(),
                    // Serial port
                    0xff01 | 0xff02 => self.serial.read(addr),
                    // Interrupt flags: the unused top 3 bits read as 1
                    0xff0f => 0xE0 | self.get_interrupt_flags(),
                    // No register on the DMG, including the CGB-only ones (KEY1, VBK, SVBK...)
                    _ if !is_dmg_io_register(addr) => 0xff,
                    // Other I/O registers
                    _ => self.io_registers[(addr - 0xff00) as usize],
                }
            }

            // High RAM
            0xff80..=0xfffe => self.h_ram[(addr - 0xff80) as usize],

            // Interrupt Enable register
            0xffff => self.ie,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) -> Option<CpuStatus> {
        match addr {
            // ROM area - should not be writable, but cartridge may handle banking
            0x0000..=0x7fff => {
                self.cart.write(&addr, value);
            }

            // Video RAM - route through PPU for timing restrictions
            0x8000..=0x9fff => {
                self.ppu.write_vram(addr, value);
            }

            // Cartridge RAM
            0xa000..=0xbfff => {
                self.cart.write(&addr, value);
            }

            // Work RAM bank 0
            0xc000..=0xcfff => {
                self.w_ram[0][(addr - 0xc000) as usize] = value;
            }

            // Work RAM bank 1-7 (switchable)
            0xd000..=0xdfff => {
                self.w_ram[self.w_ram_bank as usize][(addr - 0xd000) as usize] = value;
            }

            // Echo RAM - should mirror work RAM writes
            0xe000..=0xefff => self.w_ram[0][(addr - 0xe000) as usize] = value,
            0xf000..=0xfdff => {
                self.w_ram[self.w_ram_bank as usize][(addr - 0xf000) as usize] = value
            }

            // Object Attribute Memory (OAM) - route through PPU for timing restrictions
            0xfe00..=0xfe9f => {
                self.ppu.write_oam(addr, value);
                trace!("writing {:#x} to OAM at {:#x}", value, addr);
            }

            // Unusable memory area: writes are ignored (see `read`)
            0xfea0..=0xfeff => {
                trace!(
                    "ignored write of {:#x} to unusable address {:#x}",
                    value, addr
                );
            }

            // I/O registers
            0xff00..=0xff7f => {
                match addr {
                    // OAM DMA (inside the PPU register range, so it must match first)
                    0xff46 => self.oam_dma(value),
                    // PPU registers (0xFF40-0xFF4B)
                    0xFF40..=0xFF4B => {
                        self.ppu.write_register(addr, value);
                    }
                    // Special I/O register writes
                    0xff00 => self.joypad.write(value), // Joypad group select
                    0xff01 | 0xff02 => self.serial.write(addr, value), // Serial port
                    0xff0f => self.io_registers[0x0f] = value & 0x1F, // Interrupt flags
                    // No register on the DMG: writes are ignored. This includes the CGB
                    // VRAM/WRAM bank selects (0xFF4F, 0xFF70) and the boot ROM flag (0xFF50).
                    _ if !is_dmg_io_register(addr) => (),
                    // Other I/O registers
                    _ => {
                        self.io_registers[(addr - 0xff00) as usize] = value;
                    }
                }
            }

            // High RAM
            0xff80..=0xfffe => {
                self.h_ram[(addr - 0xff80) as usize] = value;
            }

            // Interrupt Enable register
            0xffff => {
                self.ie = value;
            }
        }

        trace!("writing {:#x} to {:#x}", value, addr);
        None
    }

    /// OAM DMA: copy 160 bytes from `source_high << 8` into OAM.
    ///
    /// The copy happens instantly. Hardware takes 160 M-cycles and restricts the CPU to HRAM
    /// meanwhile, but games run their DMA routine from HRAM and wait it out, so they can't tell.
    fn oam_dma(&mut self, source_high: u8) {
        self.io_registers[0x46] = source_high;
        let source = (source_high as u16) << 8;
        for offset in 0..0xA0u16 {
            let byte = self.read(source + offset);
            self.ppu.dma_write_oam(offset as usize, byte);
        }
    }

    // Interrupt handling
    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.request_interrupts(interrupt as u8);
    }

    /// Request several interrupts at once, given as IF bits
    pub fn request_interrupts(&mut self, bits: u8) {
        if bits == 0 {
            return;
        }
        self.io_registers[0x0f] |= bits & 0x1F;
    }

    // Getters for CPU state that depends on memory
    /// Press or release a joypad button, requesting the Joypad interrupt on a new press.
    /// Returns true if the press requested the interrupt.
    pub fn set_button(&mut self, button: Button, pressed: bool) -> bool {
        let interrupt = self.joypad.set_pressed(button, pressed);
        if interrupt {
            self.request_interrupt(Interrupt::Joypad);
        }
        interrupt
    }

    /// Advance the serial port one M-cycle, requesting the Serial interrupt when a
    /// transfer finishes.
    pub fn step_serial(&mut self) {
        if self.serial.step() {
            self.request_interrupt(Interrupt::Serial);
        }
    }

    /// Every byte sent over the serial port so far.
    pub fn serial_output(&self) -> &[u8] {
        self.serial.output()
    }

    /// Read a byte for debugging: like `read`, but ignores the PPU's VRAM/OAM access
    /// restrictions and doesn't log, so a debugger always sees the real contents.
    pub fn peek(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9fff => self.ppu.peek_vram(addr),
            0xfe00..=0xfe9f => self.ppu.peek_oam(addr),
            0xfea0..=0xfeff => 0xff,
            _ => self.read(addr),
        }
    }

    /// Requested interrupts (IF bits 0-4)
    pub fn get_interrupt_flags(&self) -> u8 {
        self.io_registers[0x0f] & 0x1F
    }

    pub fn get_interrupt_enable(&self) -> u8 {
        self.ie
    }

    // PPU integration methods
    /// Step the PPU, returning the requested interrupts as IF bits
    pub fn step_ppu(&mut self, cycles: u8) -> u8 {
        self.ppu.step(cycles)
    }

    pub fn is_frame_ready(&self) -> bool {
        self.ppu.is_frame_ready()
    }

    pub fn get_framebuffer(&mut self) -> &Vec<u8> {
        self.ppu.get_framebuffer()
    }

    pub fn get_ppu(&self) -> &crate::ppu::Ppu {
        &self.ppu
    }

    pub fn get_ppu_mut(&mut self) -> &mut crate::ppu::Ppu {
        &mut self.ppu
    }
}

/// Whether a DMG has a register at this I/O address (0xFF00-0xFF7F). The rest read as 0xFF.
fn is_dmg_io_register(addr: u16) -> bool {
    matches!(
        addr,
        0xff00..=0xff02     // joypad, serial
            | 0xff04..=0xff07 // timer
            | 0xff0f          // interrupt flags
            | 0xff10..=0xff26 // sound
            | 0xff30..=0xff3f // wave RAM
            | 0xff40..=0xff4b // LCD
    )
}
