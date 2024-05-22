use cash_gb::{cpu::Cpu, read_cart};

fn main() {
    //let cart = match read_cart("/home/cash/dev/cash-gb/roms/dmg_test_prog_ver1.gb") {
    //    Ok(cart) => cart,
    //    Err(error) => panic!("error: {}", error),
    //};
    //println!("cart read:");
    //println!("{}", cart);
    //let cart = match read_cart("/home/cash/dev/cash-gb/roms/Pokemon-Crystal-USA-Europe-Rev-A.gbc") {
    //    Ok(cart) => cart,
    //    Err(error) => panic!("error: {}", error),
    //};

    //println!("cart read:");
    //println!("{}", cart);

    let cart = match read_cart("/home/cash/dev/cash-gb/roms/Tetris.gb") {
        Ok(cart) => cart,
        Err(error) => panic!("error: {}", error),
    };

    println!("cart read:");
    println!("{}", cart);
    let mut cpu = Cpu::new(cart);
    let steps = 10000000;
    for _ in 0..=steps {
        cpu.step();
    }
}
