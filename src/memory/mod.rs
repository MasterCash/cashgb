pub mod cart;
pub mod mbc;

use crate::cpu::instructions::{CpuStatus, Interrupt};
use crate::ppu::Ppu;
use cart::Cart;
use log::{debug, trace, warn};

pub struct MemoryBus {
    cart: Cart,
    ppu: Ppu,
    v_ram_bank: u8,
    w_ram: [[u8; 0x2000]; 8],
    w_ram_bank: u8,
    io_registers: [u8; 0x80],
    h_ram: [u8; 0x80],
    ie: u8,
}

impl MemoryBus {
    pub fn new(cart: Cart) -> Self {
        Self {
            cart,
            ppu: Ppu::new(),
            v_ram_bank: 0,
            w_ram: [[0; 0x2000]; 8],
            w_ram_bank: 1,
            io_registers: [0; 0x80],
            h_ram: [0; 0x80],
            ie: 0,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // ROM banks and cartridge RAM
            0x0000..=0x7fff | 0xA000..=0xbfff => self
                .cart
                .read(&addr)
                .unwrap_or_else(|| panic!("failed to read from cart: {:#x}", addr)),

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

            // Unusable memory area
            0xfea0..=0xfeff => {
                warn!("accessing unusable memory: {:#x}", addr);
                0xff
            }

            // I/O registers
            0xff00..=0xff7f => {
                match addr {
                    // PPU registers (0xFF40-0xFF4B)
                    0xFF40..=0xFF4B => self.ppu.registers.read_register(addr),
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
                // MBC banking writes would go here
                debug!(
                    "attempting to write {:#x} to ROM address {:#x}",
                    value, addr
                );
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
            0xe000..=0xfdff => {
                debug!("attempting to write to echo RAM at {:#x}", addr);
            }

            // Object Attribute Memory (OAM) - route through PPU for timing restrictions
            0xfe00..=0xfe9f => {
                self.ppu.write_oam(addr, value);
                trace!("writing {:#x} to OAM at {:#x}", value, addr);
            }

            // Unusable memory area
            0xfea0..=0xfeff => {
                panic!("attempting to write to unusable address {:#x}", addr);
            }

            // I/O registers
            0xff00..=0xff7f => {
                match addr {
                    // PPU registers (0xFF40-0xFF4B)
                    0xFF40..=0xFF4B => {
                        self.ppu.registers.write_register(addr, value);
                    }
                    // Special I/O register writes
                    0xff4f => self.v_ram_bank = value & 1,     // VRAM bank select
                    0xff50 => return Some(CpuStatus::Stopped), // Boot ROM disable
                    0xff70 => self.w_ram_bank = value & 0x07,  // WRAM bank select
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

    // Interrupt handling
    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        let flags = self.read(0xff0f);
        self.write(0xff0f, flags | interrupt as u8);
    }

    // Getters for CPU state that depends on memory
    pub fn get_interrupt_flags(&self) -> u8 {
        self.read(0xff0f)
    }

    pub fn get_interrupt_enable(&self) -> u8 {
        self.ie
    }

    // PPU integration methods
    pub fn step_ppu(&mut self, cycles: u8) -> Option<Interrupt> {
        self.ppu.step(cycles)
    }

    pub fn is_frame_ready(&self) -> bool {
        self.ppu.is_frame_ready()
    }

    pub fn get_framebuffer(&mut self) -> &[u8; 160 * 144 * 4] {
        self.ppu.get_framebuffer()
    }

    pub fn get_ppu(&self) -> &crate::ppu::Ppu {
        &self.ppu
    }

    pub fn get_ppu_mut(&mut self) -> &mut crate::ppu::Ppu {
        &mut self.ppu
    }
}
