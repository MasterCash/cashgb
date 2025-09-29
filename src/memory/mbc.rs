use super::cart::{MapperType, CartError};

/// Memory Bank Controller trait for different cartridge types
pub trait MemoryBankController {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, value: u8);
    fn get_mapper_type(&self) -> MapperType;
}

/// No Memory Bank Controller (32KB ROM only)
pub struct NoMBC {
    rom: Vec<u8>,
}

impl NoMBC {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { rom }
    }
}

impl MemoryBankController for NoMBC {
    fn read(&self, addr: u16) -> Option<u8> {
        match addr {
            0x0000..=0x7fff => {
                if (addr as usize) < self.rom.len() {
                    Some(self.rom[addr as usize])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn write(&mut self, _addr: u16, _value: u8) {
        // ROM is read-only for NoMBC
    }

    fn get_mapper_type(&self) -> MapperType {
        MapperType::None
    }
}

/// MBC1 Memory Bank Controller
pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: u8,
    ram_bank: u8,
    ram_enabled: bool,
    mode: MBC1Mode,
    rom_banks: u8,
    ram_banks: u8,
}

#[derive(Debug, Clone, Copy)]
enum MBC1Mode {
    Rom,  // 16Mbit ROM/8KByte RAM mode
    Ram,  // 4Mbit ROM/32KByte RAM mode
}

impl MBC1 {
    pub fn new(rom: Vec<u8>, ram_size: u32, rom_banks: u8, ram_banks: u8) -> Self {
        Self {
            rom,
            ram: vec![0; ram_size as usize],
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            mode: MBC1Mode::Rom,
            rom_banks,
            ram_banks,
        }
    }
}

impl MemoryBankController for MBC1 {
    fn read(&self, addr: u16) -> Option<u8> {
        match addr {
            // ROM Bank 0 (fixed)
            0x0000..=0x3fff => {
                let effective_bank = match self.mode {
                    MBC1Mode::Rom => 0,
                    MBC1Mode::Ram => (self.ram_bank << 5) & (self.rom_banks - 1),
                };
                let rom_addr = (effective_bank as usize * 0x4000) + addr as usize;
                if rom_addr < self.rom.len() {
                    Some(self.rom[rom_addr])
                } else {
                    None
                }
            }

            // ROM Bank 1-127 (switchable)
            0x4000..=0x7fff => {
                let effective_bank = match self.mode {
                    MBC1Mode::Rom => self.rom_bank | (self.ram_bank << 5),
                    MBC1Mode::Ram => self.rom_bank,
                } & (self.rom_banks - 1);

                let rom_addr = (effective_bank as usize * 0x4000) + (addr - 0x4000) as usize;
                if rom_addr < self.rom.len() {
                    Some(self.rom[rom_addr])
                } else {
                    None
                }
            }

            // RAM Bank (if enabled)
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return Some(0xFF);
                }

                let effective_bank = match self.mode {
                    MBC1Mode::Rom => 0,
                    MBC1Mode::Ram => self.ram_bank & (self.ram_banks - 1),
                };

                let ram_addr = (effective_bank as usize * 0x2000) + (addr - 0xA000) as usize;
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
        match addr {
            // RAM Enable
            0x0000..=0x1fff => {
                self.ram_enabled = (value & 0x0A) == 0x0A;
            }

            // ROM Bank Number (lower 5 bits)
            0x2000..=0x3fff => {
                let mut bank = value & 0x1F;
                if bank == 0 {
                    bank = 1;
                }
                self.rom_bank = bank;
            }

            // RAM Bank Number / Upper ROM Bank bits
            0x4000..=0x5fff => {
                self.ram_bank = value & 0x03;
            }

            // Banking Mode Select
            0x6000..=0x7fff => {
                self.mode = if (value & 0x01) == 0 {
                    MBC1Mode::Rom
                } else {
                    MBC1Mode::Ram
                };
            }

            // RAM Write
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return;
                }

                let effective_bank = match self.mode {
                    MBC1Mode::Rom => 0,
                    MBC1Mode::Ram => self.ram_bank & (self.ram_banks - 1),
                };

                let ram_addr = (effective_bank as usize * 0x2000) + (addr - 0xA000) as usize;
                if ram_addr < self.ram.len() {
                    self.ram[ram_addr] = value;
                }
            }

            _ => {}
        }
    }

    fn get_mapper_type(&self) -> MapperType {
        MapperType::MBC1
    }
}

/// Factory function to create appropriate MBC based on cartridge type
pub fn create_mbc(
    mapper_type: MapperType,
    rom: Vec<u8>,
    ram_size: u32,
    rom_banks: u8,
    ram_banks: u8,
) -> Result<Box<dyn MemoryBankController>, CartError> {
    match mapper_type {
        MapperType::None => Ok(Box::new(NoMBC::new(rom))),
        MapperType::MBC1 => Ok(Box::new(MBC1::new(rom, ram_size, rom_banks, ram_banks))),
        _ => Err(CartError::UnsupportedMapper(mapper_type)),
    }
}