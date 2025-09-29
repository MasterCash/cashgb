use std::{fs::File, io::Read, path::Path};

use memory::cart::{Cart, CartError};

pub mod cpu;
pub mod memory;
pub mod ppu;

pub fn read_cart(path: &str) -> Result<Cart, CartError> {
    let mut data: Vec<u8> = vec![];

    let file_path = Path::new(path);

    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return Err(CartError::MissingCart(path.to_string())),
    };

    match file.read_to_end(&mut data) {
        Ok(_) => (),
        Err(_) => return Err(CartError::LoadError),
    };

    Cart::new(&data)
}
