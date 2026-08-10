/// Simple demo that creates a minimal "ROM" and shows terminal output
use cash_gb::cpu::Cpu;
use cash_gb::memory::cart::{
    Cart,
    cart_header::{NINTENDO_LOGO, addresses},
};
use cash_gb::ppu::display::{Display, TerminalDisplay};
use log::LevelFilter;

fn main() {
    // Initialize logger - set to Info level to reduce debug spam
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .init();
    println!("Creating a simple test to show terminal display...");

    // Create a minimal ROM data that will pass validation
    let mut rom_data = vec![0u8; 0x8000]; // 32KB minimum ROM

    // Set required Game Boy header fields
    rom_data[addresses::NINTENDO_LOGO_START as usize..0x134].copy_from_slice(&NINTENDO_LOGO); // Nintendo logo

    rom_data[0x134..0x144].copy_from_slice(b"TEST\0\0\0\0\0\0\0\0\0\0\0\0"); // Title
    rom_data[0x147] = 0x00; // Cart type: ROM only
    rom_data[0x148] = 0x00; // ROM size: 32KB
    rom_data[0x149] = 0x00; // RAM size: None

    // Calculate header checksum
    let mut checksum: u8 = 0;
    for datum in rom_data.iter().take(0x14D).skip(0x134) {
        checksum = checksum.wrapping_sub(*datum).wrapping_sub(1);
    }
    rom_data[0x14D] = checksum;

    // Create cart from ROM data
    let cart = match Cart::new(&rom_data) {
        Ok(cart) => cart,
        Err(e) => {
            eprintln!("Failed to create cart: {}", e);
            return;
        }
    };

    // Create CPU and display
    let mut cpu = Cpu::new(cart);
    let mut display = TerminalDisplay::new(true); // Use color

    println!("Starting emulation to generate frames...");

    let mut frame_count = 0;
    let mut steps = 0;
    const MAX_STEPS: usize = 500000; // Limit steps to avoid infinite loop

    while frame_count < 3 && steps < MAX_STEPS {
        cpu.step();
        steps += 1;

        if cpu.is_frame_ready() {
            let framebuffer = cpu.get_framebuffer();
            display.present_frame(framebuffer);

            frame_count += 1;
            println!("Frame {} completed after {} CPU steps", frame_count, steps);

            // Add a small delay to see each frame
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    }

    if frame_count == 0 {
        println!("No frames were generated in {} steps", steps);
        println!(
            "This might be because the LCD is disabled or the boot ROM needs to be implemented"
        );
    } else {
        println!("Demo complete! Rendered {} frames", frame_count);
    }
}
