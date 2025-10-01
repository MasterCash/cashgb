/// Example showing how to use TerminalDisplay to render Game Boy output
use cash_gb::{cpu::Cpu, read_cart};
use cash_gb::ppu::display::{Display, TerminalDisplay};
use log::LevelFilter;
use std::env;

fn main() {
    // Initialize logger - set to Info level to reduce debug spam
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .init();
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <rom_file>", args[0]);
        std::process::exit(1);
    }

    // Load the cartridge
    let cart = match read_cart(&args[1]) {
        Ok(cart) => cart,
        Err(error) => {
            eprintln!("Error loading cart: {}", error);
            std::process::exit(1);
        }
    };

    println!("Cartridge loaded:");

    // Create CPU and display
    let mut cpu = Cpu::new(cart);
    let mut display = TerminalDisplay::new(true); // Use color ANSI output

    println!("Starting emulation... Press Ctrl+C to exit");

    let mut frame_count = 0;
    const MAX_FRAMES: u32 = 10; // Show first 10 frames for demo

    loop {
        // Step CPU
        cpu.step();

        // Check if a frame is ready
        if cpu.is_frame_ready() {
            let framebuffer = cpu.get_framebuffer();
            display.present_frame(framebuffer);

            frame_count += 1;
            println!("Frame {} displayed", frame_count);

            // Exit after showing a few frames
            if frame_count >= MAX_FRAMES {
                println!("Demo complete!");
                break;
            }

            // Small delay to see the frames
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
}