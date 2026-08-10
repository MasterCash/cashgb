use std::fmt::Debug;

use crate::memory::{
    cart::{CartError, MapperType},
    controller::{mbc1::MBC1, no_controller::NoMBC},
};

mod mbc1;
mod no_controller;

#[cfg(test)]
mod tests;

/// Memory Bank Controller trait for different cartridge types
pub trait MemoryBankController: Debug {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, value: u8);
    fn get_mapper_type(&self) -> MapperType;
}

/// Factory function to create appropriate MBC based on cartridge type
pub fn create_mbc(
    mapper_type: MapperType,
    rom: Vec<u8>,
) -> Result<Box<dyn MemoryBankController>, CartError> {
    match mapper_type {
        MapperType::None => Ok(Box::new(NoMBC::new(rom))),
        MapperType::MBC1 => Ok(Box::new(MBC1::new(rom)?)),
        _ => Err(CartError::UnsupportedMapper(mapper_type)),
    }
}
