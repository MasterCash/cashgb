use cash_gb::read_cart;

fn main() {
    let cart = match read_cart("/home/cash/dev/cash-gb/roms/Tetris.gb") {
        Ok(cart) => cart,
        Err(error) => panic!("error: {}", error),
    };
    println!("cart read:");
    println!("{}", cart);
    let cart = match read_cart("/home/cash/dev/cash-gb/roms/Pokemon-Crystal-USA-Europe-Rev-A.gbc") {
        Ok(cart) => cart,
        Err(error) => panic!("error: {}", error),
    };
    println!("cart read:");
    println!("{}", cart);
}
