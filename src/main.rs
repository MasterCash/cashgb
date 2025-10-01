use cash_gb::{cpu::Cpu, read_cart};
use cash_gb::ppu::display::{Display, TerminalDisplay};
use clap::Parser;
use log::{info, LevelFilter};
use std::time::{Duration, Instant};
use std::thread;

#[derive(Parser)]
#[command(name = "cash-gb")]
#[command(about = "A Game Boy emulator written in Rust")]
struct Args {
    /// ROM file to load
    rom_file: String,

    /// Enable trace logging (shows all debug output)
    #[arg(long)]
    trace: bool,
}

fn main() {
    let args = Args::parse();

    // Initialize logger based on trace flag
    let log_level = if args.trace {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    // Load the cartridge
    let cart = match read_cart(&args.rom_file) {
        Ok(cart) => cart,
        Err(error) => {
            eprintln!("Error loading ROM: {}", error);
            std::process::exit(1);
        }
    };

    info!("Cart loaded successfully: {}", args.rom_file);

    // Create CPU and display
    let mut cpu = Cpu::new(cart);
    let mut display = TerminalDisplay::new(true); // Use color output

    println!("Starting Game Boy emulator...");
    println!("Press Ctrl+C to exit");

    // Game loop with frame rate limiting
    let target_fps = 60.0; // Game Boy runs at ~59.7 FPS
    let frame_duration = Duration::from_secs_f64(1.0 / target_fps);
    let mut last_frame_time = Instant::now();
    let mut frame_count: u32 = 0;

    let mut total_steps: usize = 0;
    loop {
        // Step CPU until a frame is ready
        let mut steps_this_frame: usize = 0;
        while !cpu.is_frame_ready() { // Max cycles per frame
            cpu.step();
            steps_this_frame = steps_this_frame.wrapping_add(1);
            total_steps = total_steps.wrapping_add(1);

            // Debug: Check if we're stuck after frame 19
            if frame_count >= 19 && steps_this_frame == 100000 {
                info!("Breaking infinite loop after 100k steps - PPU appears to be stuck after frame {}", frame_count);
                break;
            }
        }

        // Display the frame if ready
        if cpu.is_frame_ready() {
            let framebuffer = cpu.get_framebuffer();
            display.present_frame(framebuffer);
            frame_count = frame_count.wrapping_add(1);
            info!("Frame {} rendered after {} steps this frame, {} total steps", frame_count, steps_this_frame, total_steps);

            // Frame rate limiting
            let elapsed = last_frame_time.elapsed();
            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
            last_frame_time = Instant::now();

            // Exit after a reasonable number of frames for demo
            if frame_count >= 30000 { // About 5 seconds at 60 FPS
                println!("Demo complete - {} frames rendered", frame_count);
                break;
            }
        }
    }
}
