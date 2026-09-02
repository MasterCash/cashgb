use crate::memory::{
    cart::{
        MapperType,
        cart_header::{addresses, ram_codes},
    },
    controller::InternalMemoryBankController,
    controller::MemoryBankController,
    memory_sizes,
};

/// No Memory Bank Controller (32KB ROM only)
#[derive(Debug)]
pub struct NoMBC {
    rom: Vec<u8>,
    ram: Option<Vec<u8>>,
}

impl NoMBC {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            ram: if rom[addresses::ROM_SIZE as usize] == ram_codes::CODE_8_KILOBYTES {
                Some(Vec::with_capacity(memory_sizes::MEM_8_KILOBYTES as usize))
            } else {
                None
            },
            rom,
        }
    }
}

impl InternalMemoryBankController for NoMBC {
    fn rom(&self) -> &Vec<u8> {
        &self.rom
    }

    fn ram(&self) -> Option<&Vec<u8>> {
        if let Some(ram) = &self.ram {
            Some(ram)
        } else {
            None
        }
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
            0xA000..=0xBFFF => {
                if let Some(ram) = &self.ram {
                    let addr = (addr - 0xA000) as usize;
                    if addr < ram.len() {
                        Some(ram[addr])
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        // ROM is read-only for NoMBC
        if let 0xA000..=0xBFFF = addr
            && let Some(ram) = &mut self.ram
        {
            ram[addr as usize - 0xA000] = value;
        }
    }

    fn get_mapper_type(&self) -> MapperType {
        MapperType::None
    }
}
