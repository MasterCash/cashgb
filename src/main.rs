use cash_gb::{cpu::Cpu, read_cart};
use std::env;

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

    let args: Vec<_> = env::args_os().collect();

    if args.len() == 1 {
        panic!("missing cart file");
    }

    if let Some(file) = env::args_os().nth(1) {
        let cart = match read_cart(&match file.into_string() {
            Ok(file) => file,
            Err(error) => panic!("error: {:?}", error),
        }) {
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
}
