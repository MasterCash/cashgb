use crate::memory::{
    cart::{
        CartError::{self, UnsupportedMapperRamSize},
        MapperType,
        cart_header::{addresses, get_ram_size, get_rom_size},
    },
    controller::MemoryBankController,
    memory_sizes,
};

/// MBC1 Memory Bank Controller
#[derive(Debug)]
pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: u8,
    ram_bank: u8,
    ram_enabled: bool,
    mode: MBC1Mode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MBC1Mode {
    // Large ROM > 1MB
    // static RAM
    // multi-game compilation cart
    // ----
    Rom, // 2MByte ROM/8KByte RAM mode
    Ram, // 512KByte ROM/32KByte RAM mode
}

pub mod mbc1_addresses {
    // read X0 rom banks in 1MB > banks otherwise 00 Bank
    pub const ROM_BANK_FIXED_START: u16 = 0x0000;
    pub const ROM_BANK_FIXED_END: u16 = 0x3FFF;
    pub const ROM_BANK_DYN_START: u16 = 0x4000;
    pub const ROM_BANK_DYN_END: u16 = 0x7FFF;
    pub const RAM_BANK_START: u16 = 0xA000;
    pub const RAM_BANK_END: u16 = 0xBFFF;
    pub const RAM_ENABLE_START: u16 = 0x0000;
    pub const RAM_ENABLE_END: u16 = 0x1FFF;
    pub const ROM_BANK_NUM_START: u16 = 0x2000;
    pub const ROM_BANK_NUM_END: u16 = 0x3FFF;
    pub const RAM_BANK_NUM_START: u16 = 0x4000;
    pub const RAM_BANK_NUM_END: u16 = 0x5FFF;
    pub const MODE_SELECT_START: u16 = 0x6000;
    pub const MODE_SELECT_END: u16 = 0x7FFF;
}

impl MBC1 {
    pub fn get_rom_bank(&self) -> u8 {
        if self.mode == MBC1Mode::Ram {
            self.rom_bank
        } else {
            self.rom_bank | ((self.ram_bank & 0x03) << 5)
        }
    }

    pub fn get_ram_bank(&self) -> u8 {
        if self.mode == MBC1Mode::Rom {
            0
        } else {
            self.ram_bank & 0x03
        }
    }

    pub fn new(rom: Vec<u8>) -> Result<Self, CartError> {
        let rom_code = rom[addresses::ROM_SIZE as usize];
        let ram_code = rom[addresses::RAM_SIZE as usize];
        let (rom_size, _) = get_rom_size(&rom[addresses::ROM_SIZE as usize])?;
        let (ram_size, _) = get_ram_size(&rom[addresses::RAM_SIZE as usize])?;

        if ram_size > memory_sizes::MEM_32_KILOBYTES {
            return Err(CartError::UnsupportedMapperRamSize(
                MapperType::MBC1,
                ram_code,
            ));
        }
        if rom_size > memory_sizes::MEM_2_MEGABYTES {
            return Err(CartError::UnsupportedMapperRomSize(
                MapperType::MBC1,
                rom_code,
            ));
        }
        if rom_size > memory_sizes::MEM_512_KILOBYTES && ram_size > memory_sizes::MEM_8_KILOBYTES {
            return Err(UnsupportedMapperRamSize(MapperType::MBC1, ram_code));
        }

        Ok(Self {
            rom,
            ram: vec![0; ram_size as usize],
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            mode: MBC1Mode::Rom,
        })
    }
}

impl MemoryBankController for MBC1 {
    fn read(&self, addr: u16) -> Option<u8> {
        use mbc1_addresses::*;
        // TODO: Address the rom bank type diff solved by as u8 currently
        match addr {
            // ROM Bank 0 (fixed)
            ROM_BANK_FIXED_START..=ROM_BANK_FIXED_END => Some(self.rom[addr as usize]),

            // ROM Bank 1-127 (switchable)
            ROM_BANK_DYN_START..=ROM_BANK_DYN_END => {
                let rom_addr = (self.get_rom_bank() as usize
                    * memory_sizes::MEM_16_KILOBYTES as usize)
                    + (addr - ROM_BANK_DYN_START) as usize;

                if rom_addr < self.rom.len() {
                    Some(self.rom[rom_addr])
                } else {
                    None
                }
            }

            // RAM Bank (if enabled)
            RAM_BANK_START..=RAM_BANK_END => {
                if !self.ram_enabled {
                    return Some(0xFF);
                }

                let ram_addr = (self.get_ram_bank() as usize
                    * memory_sizes::MEM_8_KILOBYTES as usize)
                    + (addr - RAM_BANK_START) as usize;
                if ram_addr < self.ram.len() {
                    Some(self.ram[ram_addr])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        use mbc1_addresses::*;
        match addr {
            // RAM Enable
            RAM_ENABLE_START..=RAM_ENABLE_END => {
                self.ram_enabled = (value & 0x0A) == 0x0A;
            }

            // ROM Bank Number (lower 5 bits)
            ROM_BANK_NUM_START..=ROM_BANK_NUM_END => {
                let value = value & 0x1F;
                self.rom_bank = if value == 0x00 { 0x01 } else { value }
            }

            // RAM Bank Number / Upper ROM Bank bits
            RAM_BANK_NUM_START..=RAM_BANK_NUM_END => {
                self.ram_bank = value & 0x03;
            }

            // Banking Mode Select
            MODE_SELECT_START..=MODE_SELECT_END => {
                self.mode = if (value & 0x01) == 0 {
                    MBC1Mode::Rom
                } else {
                    MBC1Mode::Ram
                };
            }

            // RAM Write
            RAM_BANK_START..=RAM_BANK_END => {
                if !self.ram_enabled {
                    return;
                }

                let effective_bank = self.get_ram_bank();
                let ram_addr = (effective_bank as usize * memory_sizes::MEM_8_KILOBYTES as usize)
                    + (addr - RAM_BANK_START) as usize;
                let ram_addr = ram_addr % self.ram.len();
                self.ram[ram_addr] = value;
            }

            _ => {}
        }
    }

    fn get_mapper_type(&self) -> MapperType {
        MapperType::MBC1
    }
}
