use std::{fs::File, io::Read, path::Path};

use memory::cart::{Cart, CartError};

pub mod cpu;
pub mod headless;
pub mod memory;
pub mod ppu;
pub mod ui;

/// Load and validate a ROM file.
pub fn read_cart(path: impl AsRef<Path>) -> Result<Cart, CartError> {
    let mut data: Vec<u8> = vec![];

    let file_path = path.as_ref();

    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return Err(CartError::MissingCart(file_path.display().to_string())),
    };

    match file.read_to_end(&mut data) {
        Ok(_) => (),
        Err(_) => return Err(CartError::LoadError),
    };

    Cart::new(&data)
}
