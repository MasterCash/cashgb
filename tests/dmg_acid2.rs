//! dmg-acid2: one screenshot that checks most of the PPU's rendering rules. See
//! `tests/roms/dmg-acid2/README.md`.

use std::{fs::File, io::BufWriter, path::Path};

use cash_gb::{
    headless::{Headless, SCREEN_HEIGHT, SCREEN_WIDTH},
    read_cart,
};

/// The test draws its face within a couple of frames; this leaves headroom
const FRAMES_TO_RUN: u32 = 60;

/// Load a 2-bit greyscale PNG as one shade per pixel, 0 (white) to 3 (black).
fn load_shades(path: &Path) -> Vec<u8> {
    let mut decoder = png::Decoder::new(File::open(path).unwrap());
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().unwrap();
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).unwrap();
    assert_eq!(info.color_type, png::ColorType::Grayscale);
    assert_eq!(
        (info.width as usize, info.height as usize),
        (SCREEN_WIDTH, SCREEN_HEIGHT)
    );
    buffer[..info.buffer_size()]
        .iter()
        .map(|&grey| 3 - grey / 85)
        .collect()
}

/// Save shades as a greyscale PNG, so a failing screenshot can be compared by eye.
fn save_shades(path: &Path, shades: &[u8]) {
    let file = BufWriter::new(File::create(path).unwrap());
    let mut encoder = png::Encoder::new(file, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
    encoder.set_color(png::ColorType::Grayscale);
    let pixels: Vec<u8> = shades.iter().map(|&shade| 255 - shade * 85).collect();
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&pixels)
        .unwrap();
}

#[test]
fn dmg_acid2_matches_reference() {
    let roms = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/roms/dmg-acid2");
    let cart = read_cart(roms.join("dmg-acid2.gb")).unwrap();
    let mut emulator = Headless::new(cart);
    emulator.run_frames(FRAMES_TO_RUN);

    let actual = emulator.screen_shades();
    let expected = load_shades(&roms.join("reference-dmg.png"));

    let wrong = actual.iter().zip(&expected).filter(|(a, e)| a != e).count();
    if wrong > 0 {
        let saved = Path::new(env!("CARGO_TARGET_TMPDIR")).join("dmg-acid2-actual.png");
        save_shades(&saved, &actual);
        panic!(
            "{wrong} of {} pixels differ from reference-dmg.png. Screenshot saved to {}",
            actual.len(),
            saved.display()
        );
    }
}
