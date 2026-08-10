use crate::memory::{
    cart::{
        MapperType,
        cart_header::{addresses, ram_codes},
    },
    controller::MemoryBankController,
    memory_sizes,
};

/// No Memory Bank Controller (32KB ROM only)
#[derive(Debug)]
pub struct NoMBC {
    rom: Vec<u8>,
    ram: Option<[u8; memory_sizes::MEM_8_KILOBYTES as usize]>,
}

impl NoMBC {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            ram: if rom[addresses::ROM_SIZE as usize] == ram_codes::CODE_8_KILOBYTES {
                Some([0; memory_sizes::MEM_8_KILOBYTES as usize])
            } else {
                None
            },
            rom,
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
            0xA000..=0xBFFF => self.ram.map(|ram| ram[addr as usize - 0xA000]),
            _ => None,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        // ROM is read-only for NoMBC
        if let 0xA000..=0xBFFF = addr
            && let Some(mut ram) = self.ram
        {
            ram[addr as usize - 0xA000] = value;
            self.ram = Some(ram);
        }
    }

    fn get_mapper_type(&self) -> MapperType {
        MapperType::None
    }
}
