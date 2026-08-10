use std::fmt::{Debug, Display};

use super::controller::MemoryBankController;
use cart_header::{
    get_licensee, get_ram_size, get_rom_size, validate_header_checksum, validate_nintendo_logo,
};
pub use cart_types::{BootFailure, CartError, CartType, MapperType};

pub mod cart_header;
mod cart_types;

#[cfg(test)]
pub mod tests;

pub struct Cart {
    mbc: Box<dyn MemoryBankController>,
    title: String,
    cgb: bool,
    cart_type: CartType,
    licensee: String,
    sgb: bool,
    rom_size: u32,
    rom_banks: u16,
    ram_size: u32,
    ram_banks: u8,
    destination: bool,
    version: u8,
}

impl Display for Cart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\t title: {}", self.title)?;
        writeln!(f, "\t cgb: {}", self.cgb)?;
        writeln!(f, "\t cart type: {}", self.cart_type)?;
        writeln!(f, "\t licensee: {}", self.licensee)?;
        writeln!(f, "\t sgb: {}", self.sgb)?;
        writeln!(f, "\t rom size: {}", self.rom_size)?;
        writeln!(f, "\t rom banks: {}", self.rom_banks)?;
        writeln!(f, "\t ram size: {}", self.ram_size)?;
        writeln!(f, "\t ram banks: {}", self.ram_banks)?;
        writeln!(f, "\t destination: {}", self.destination)?;
        writeln!(f, "\t version: {}", self.version)?;
        Ok(())
    }
}

impl Debug for Cart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cart")
            .field("mbc", &self.mbc.as_ref().get_mapper_type())
            .field("title", &self.title)
            .field("cgb", &self.cgb)
            .field("cart_type", &self.cart_type)
            .field("licensee", &self.licensee)
            .field("sgb", &self.sgb)
            .field("rom_size", &self.rom_size)
            .field("rom_banks", &self.rom_banks)
            .field("ram_size", &self.ram_size)
            .field("ram_banks", &self.ram_banks)
            .field("destination", &self.destination)
            .field("version", &self.version)
            .finish()
    }
}

impl Cart {
    pub fn new(data: &[u8]) -> Result<Self, CartError> {
        let mut title: Vec<char> = vec![];
        let (rom_size, rom_banks) = get_rom_size(&data[0x0148])?;
        let mut rom = vec![0; rom_size as usize];

        for (i, byte) in data.iter().enumerate() {
            rom[i] = *byte;
        }

        for byte in &rom[0x0134..=(0x0134 + 15)] {
            if *byte == 0x00 {
                break;
            }
            title.insert(title.len(), char::from(*byte));
        }

        let (ram_size, ram_banks) = get_ram_size(&rom[0x0149])?;

        // Validate header checksum
        validate_header_checksum(&rom)?;

        // Validate Nintendo logo
        validate_nintendo_logo(&rom)?;

        let cart_type = CartType::new(&rom[0x0147])?;
        let cgb = (rom[0x0143] & 0x80) > 0;
        let sgb = rom[0x0146] == 0x03;
        let destination = rom[0x014A] == 0x01;
        let version = rom[0x014c];
        let licensee = get_licensee(&rom[0x014B], &rom[0x0144], &rom[0x0145]);

        let mbc = super::controller::create_mbc(cart_type.mapper, rom)?;

        Ok(Self {
            cgb,
            title: title.iter().collect(),
            cart_type,
            licensee,
            sgb,
            rom_size,
            rom_banks,
            ram_size,
            ram_banks,
            destination,
            version,
            mbc,
        })
    }

    pub fn read(&self, addr: &u16) -> Option<u8> {
        self.mbc.read(*addr)
    }

    pub fn write(&mut self, addr: &u16, value: u8) {
        self.mbc.write(*addr, value);
    }
}
