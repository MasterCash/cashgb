use std::fmt::Debug;

use crate::memory::{
    cart::{CartError, MapperType},
    controller::{mbc1::MBC1, no_controller::NoMBC},
};

mod mbc1;
mod no_controller;

#[cfg(test)]
mod tests;

pub(crate) trait InternalMemoryBankController: Debug {
    fn rom(&self) -> &Vec<u8>;
    // Not used yet; kept for reading cartridge RAM when saving battery-backed carts.
    #[allow(dead_code)]
    fn ram(&self) -> Option<&Vec<u8>>;
}
/// Memory Bank Controller trait for different cartridge types
pub(crate) trait MemoryBankController: InternalMemoryBankController {
    fn read(&self, addr: u16) -> Option<u8>;
    /// Offset into the ROM file that a CPU address in 0x0000-0x7FFF currently maps to,
    /// given the selected bank. `None` for addresses outside ROM or past the end of the file.
    fn rom_offset(&self, addr: u16) -> Option<usize>;
    fn write(&mut self, addr: u16, value: u8);
    fn get_mapper_type(&self) -> MapperType;
}

/// Factory function to create appropriate MBC based on cartridge type
pub(crate) fn create_mbc(
    mapper_type: MapperType,
    rom: Vec<u8>,
) -> Result<Box<dyn MemoryBankController>, CartError> {
    match mapper_type {
        MapperType::None => Ok(Box::new(NoMBC::new(rom))),
        MapperType::MBC1 => Ok(Box::new(MBC1::new(rom)?)),
        _ => Err(CartError::UnsupportedMapper(mapper_type)),
    }
}
