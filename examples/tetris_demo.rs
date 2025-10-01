/// Demo to run Tetris ROM with terminal display
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
        eprintln!("Usage: {} <tetris_rom_path>", args[0]);
        std::process::exit(1);
    }

    // Load the Tetris ROM
    let cart = match read_cart(&args[1]) {
        Ok(cart) => cart,
        Err(error) => {
            eprintln!("Error loading ROM: {}", error);
            std::process::exit(1);
        }
    };

    println!("ROM loaded successfully!");

    // Create CPU and display
    let mut cpu = Cpu::new(cart);
    let mut display = TerminalDisplay::new(true); // Use color

    println!("Starting Game Boy emulator...");
    println!("This will run for a limited time to demonstrate the display.");
    println!();

    let mut frame_count = 0;
    let mut step_count = 0;
    const MAX_STEPS: usize = 100000; // Limit to prevent infinite loops
    const MAX_FRAMES: u32 = 5; // Show first few frames

    while frame_count < MAX_FRAMES && step_count < MAX_STEPS {
        // Step CPU (this automatically steps PPU)
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cpu.step();
        })) {
            Ok(_) => {
                step_count += 1;

                // Check if a frame is ready
                if cpu.is_frame_ready() {
                    let framebuffer = cpu.get_framebuffer();
                    display.present_frame(framebuffer);

                    frame_count += 1;
                    println!("Frame {} completed after {} CPU steps", frame_count, step_count);

                    // Add a delay between frames for visibility
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            }
            Err(e) => {
                eprintln!("CPU execution error after {} steps: {:?}", step_count, e);
                break;
            }
        }
    }

    if frame_count == 0 {
        println!("No frames were generated in {} steps", step_count);
        println!("This is normal for commercial ROMs without a boot ROM implementation");
        println!("The ROM expects boot ROM initialization which we don't have yet");
    } else {
        println!("Demo complete! Rendered {} frames", frame_count);
    }

    println!();
    println!("Note: Full Game Boy compatibility requires:");
    println!("- Boot ROM implementation");
    println!("- Input handling (joypad)");
    println!("- More accurate timing");
    println!("- Memory banking support");
}