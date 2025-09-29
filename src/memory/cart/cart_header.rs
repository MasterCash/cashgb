use super::cart_types::CartError;

/// Nintendo logo data for header validation
pub const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

/// Licensee codes mapping
const LICENSEES: &[(&str, &str)] = &[
    ("00", "None"),
    ("01", "Nintendo R&D1"),
    ("08", "Capcom"),
    ("13", "Electronic Arts"),
    ("18", "Hudson Soft"),
    ("19", "B-AI"),
    ("20", "KSS"),
    ("22", "POW"),
    ("24", "PCM Complete"),
    ("25", "San-X"),
    ("28", "Kemco Japan"),
    ("29", "SETA"),
    ("30", "Viacom"),
    ("31", "Nintendo"),
    ("32", "Bandai"),
    ("33", "Ocean/Acclaim"),
    ("34", "Konami"),
    ("35", "HectorSoft"),
    ("37", "Taito"),
    ("38", "Hudson"),
    ("39", "Banpresto"),
    ("41", "Ubi Soft"),
    ("42", "Atlus"),
    ("44", "Malibu"),
    ("46", "Angel"),
    ("47", "Bullet-Proof"),
    ("49", "Irem"),
    ("50", "Absolute"),
    ("51", "Acclaim"),
    ("52", "Activision"),
    ("53", "American Sammy"),
    ("54", "Konami"),
    ("55", "Hi tech entertainment"),
    ("56", "LJN"),
    ("57", "Matchbox"),
    ("58", "Mattel"),
    ("59", "Milton Bradley"),
    ("60", "Titus"),
    ("61", "Virgin"),
    ("64", "LucasArts"),
    ("67", "Ocean"),
    ("69", "Electronic Arts"),
    ("70", "Infogrames"),
    ("71", "Interplay"),
    ("72", "Broderbund"),
    ("73", "Sculptured"),
    ("75", "SCI"),
    ("78", "THQ"),
    ("79", "Accolade"),
    ("80", "Misawa"),
    ("83", "LOZC"),
    ("86", "Tokuma Shoten Intermedia"),
    ("87", "Tsukuda Original"),
    ("91", "Chunsoft"),
    ("92", "Video system"),
    ("93", "Ocean/Acclaim"),
    ("95", "Varie"),
    ("96", "Yonezawa/S'PAL"),
    ("97", "Kaneko"),
    ("99", "Pack in soft"),
    ("9H", "Bottom Up"),
    ("A4", "Konami (Yu-Gi-Oh!)"),
];

/// Parse ROM size from header byte
pub fn get_rom_size(code: &u8) -> Result<(u32, u8), CartError> {
    match code {
        0x00..=0x08 => Ok((0x8000 * (1 << code), 2u8 << code)),
        _ => Err(CartError::InvalidRomSize(*code)),
    }
}

/// Parse RAM size from header byte
pub fn get_ram_size(code: &u8) -> Result<(u32, u8), CartError> {
    match code {
        0x00 => Ok((0, 0)),
        0x02 => Ok((0x2000, 1)),
        0x03 => Ok((0x8000, 4)),
        0x04 => Ok((0x20000, 16)),
        0x05 => Ok((0x10000, 8)),
        _ => Err(CartError::InvalidRamSize(*code)),
    }
}

/// Get licensee name from header bytes
pub fn get_licensee(old_licensee: &u8, licensee_hi: &u8, licensee_lo: &u8) -> String {
    if *old_licensee == 0x33 {
        let licensee_code = format!("{:X}{:X}", licensee_hi, licensee_lo);
        for (code, name) in LICENSEES {
            if licensee_code == *code {
                return name.to_string();
            }
        }
        format!("Unknown ({})", licensee_code)
    } else {
        match old_licensee {
            0x00 => "None".to_string(),
            0x01 => "Nintendo".to_string(),
            0x08 => "Capcom".to_string(),
            0x09 => "Hot-B".to_string(),
            0x0A => "Jaleco".to_string(),
            0x0B => "Coconuts Japan".to_string(),
            0x0C => "Elite Systems".to_string(),
            0x13 => "Electronic Arts".to_string(),
            0x18 => "Hudson Soft".to_string(),
            0x19 => "ITC Entertainment".to_string(),
            0x1A => "Yanoman".to_string(),
            0x1D => "Japan Clary".to_string(),
            0x1F => "Virgin Interactive".to_string(),
            0x24 => "PCM Complete".to_string(),
            0x25 => "San-X".to_string(),
            0x28 => "Kotobuki Systems".to_string(),
            0x29 => "Seta".to_string(),
            0x30 => "Infogrames".to_string(),
            0x31 => "Nintendo".to_string(),
            0x32 => "Bandai".to_string(),
            0x34 => "Konami".to_string(),
            0x35 => "HectorSoft".to_string(),
            0x38 => "Capcom".to_string(),
            0x39 => "Banpresto".to_string(),
            0x3C => "Entertainment i".to_string(),
            0x3E => "Gremlin".to_string(),
            0x41 => "Ubisoft".to_string(),
            0x42 => "Atlus".to_string(),
            0x44 => "Malibu".to_string(),
            0x46 => "Angel".to_string(),
            0x47 => "Spectrum Holobyte".to_string(),
            0x49 => "Irem".to_string(),
            0x4A => "Virgin Interactive".to_string(),
            0x4D => "Malibu".to_string(),
            0x4F => "U.S. Gold".to_string(),
            0x50 => "Absolute".to_string(),
            0x51 => "Acclaim".to_string(),
            0x52 => "Activision".to_string(),
            0x53 => "American Sammy".to_string(),
            0x54 => "GameTek".to_string(),
            0x55 => "Park Place".to_string(),
            0x56 => "LJN".to_string(),
            0x57 => "Matchbox".to_string(),
            0x59 => "Milton Bradley".to_string(),
            0x5A => "Mindscape".to_string(),
            0x5B => "Romstar".to_string(),
            0x5C => "Naxat Soft".to_string(),
            0x5D => "Tradewest".to_string(),
            0x60 => "Titus".to_string(),
            0x61 => "Virgin Interactive".to_string(),
            0x67 => "Ocean".to_string(),
            0x69 => "Electronic Arts".to_string(),
            0x6E => "Elite Systems".to_string(),
            0x6F => "Electro Brain".to_string(),
            0x70 => "Infogrames".to_string(),
            0x71 => "Interplay".to_string(),
            0x72 => "Broderbund".to_string(),
            0x73 => "Sculptered Soft".to_string(),
            0x75 => "The Sales Curve".to_string(),
            0x78 => "t.hq".to_string(),
            0x79 => "Accolade".to_string(),
            0x7A => "Triffix Entertainment".to_string(),
            0x7C => "Microprose".to_string(),
            0x7F => "Kemco".to_string(),
            0x80 => "Misawa Entertainment".to_string(),
            0x83 => "Lozc".to_string(),
            0x86 => "Tokuma Shoten I*".to_string(),
            0x8B => "Bullet-Proof Software".to_string(),
            0x8C => "Vic Tokai".to_string(),
            0x8E => "Ape".to_string(),
            0x8F => "I'Max".to_string(),
            0x91 => "Chun Soft".to_string(),
            0x92 => "Video System".to_string(),
            0x93 => "Tsubaraya Productions Co.".to_string(),
            0x95 => "Varie Corporation".to_string(),
            0x96 => "Yonezawa/S'Pal".to_string(),
            0x97 => "Kaneko".to_string(),
            0x99 => "Arc".to_string(),
            0x9A => "Nihon Bussan".to_string(),
            0x9B => "Tecmo".to_string(),
            0x9C => "Imagineer".to_string(),
            0x9D => "Banpresto".to_string(),
            0x9F => "Nova".to_string(),
            0xA1 => "Hori Electric".to_string(),
            0xA2 => "Bandai".to_string(),
            0xA4 => "Konami".to_string(),
            0xA6 => "Kawada".to_string(),
            0xA7 => "Takara".to_string(),
            0xA9 => "Technos Japan".to_string(),
            0xAA => "Broderbund".to_string(),
            0xAC => "Toei Animation".to_string(),
            0xAD => "Toho".to_string(),
            0xAF => "Namco".to_string(),
            0xB0 => "acclaim".to_string(),
            0xB1 => "ASCII or Nexsoft".to_string(),
            0xB2 => "Bandai".to_string(),
            0xB4 => "Square Enix".to_string(),
            0xB6 => "HAL Laboratory".to_string(),
            0xB7 => "SNK".to_string(),
            0xB9 => "Pony Canyon".to_string(),
            0xBA => "Culture Brain".to_string(),
            0xBB => "Sunsoft".to_string(),
            0xBD => "Sony Imagesoft".to_string(),
            0xBF => "Sammy".to_string(),
            0xC0 => "Taito".to_string(),
            0xC2 => "Kemco".to_string(),
            0xC3 => "Squaresoft".to_string(),
            0xC4 => "Tokuma Shoten Intermedia".to_string(),
            0xC5 => "Data East".to_string(),
            0xC6 => "Tonkinhouse".to_string(),
            0xC8 => "Koei".to_string(),
            0xC9 => "UFL".to_string(),
            0xCA => "Ultra".to_string(),
            0xCB => "Vap".to_string(),
            0xCC => "Use Corporation".to_string(),
            0xCD => "Meldac".to_string(),
            0xCE => ".Pony Canyon or".to_string(),
            0xCF => "Angel".to_string(),
            0xD0 => "Taito".to_string(),
            0xD1 => "Sofel".to_string(),
            0xD2 => "Quest".to_string(),
            0xD3 => "Sigma Enterprises".to_string(),
            0xD4 => "ASK Kodansha Co.".to_string(),
            0xD6 => "Naxat Soft".to_string(),
            0xD7 => "Copya System".to_string(),
            0xD9 => "Banpresto".to_string(),
            0xDA => "Tomy".to_string(),
            0xDB => "LJN".to_string(),
            0xDD => "NCS".to_string(),
            0xDE => "Human".to_string(),
            0xDF => "Altron".to_string(),
            0xE0 => "Jaleco".to_string(),
            0xE1 => "Towa Chiki".to_string(),
            0xE2 => "Yutaka".to_string(),
            0xE3 => "Varie".to_string(),
            0xE5 => "Epcoh".to_string(),
            0xE7 => "Athena".to_string(),
            0xE8 => "Asmik ACE Entertainment".to_string(),
            0xE9 => "Natsume".to_string(),
            0xEA => "King Records".to_string(),
            0xEB => "Atlus".to_string(),
            0xEC => "Epic/Sony Records".to_string(),
            0xEE => "IGS".to_string(),
            0xF0 => "A Wave".to_string(),
            0xF3 => "Extreme Entertainment".to_string(),
            0xFF => "LJN".to_string(),
            _ => format!("Unknown ({:02X})", old_licensee),
        }
    }
}

/// Validate Nintendo logo in header
pub fn validate_nintendo_logo(rom: &[u8]) -> Result<(), CartError> {
    for (i, byte) in rom[0x0104..=0x0133].iter().enumerate() {
        if *byte != NINTENDO_LOGO[i] {
            return Err(CartError::InvalidLogo);
        }
    }
    Ok(())
}

/// Validate header checksum
pub fn validate_header_checksum(rom: &[u8]) -> Result<(), CartError> {
    let mut header_checksum = 0u8;
    for byte in &rom[0x0134..=0x014C] {
        (header_checksum, _) = header_checksum.overflowing_sub(*byte);
        (header_checksum, _) = header_checksum.overflowing_sub(1);
    }
    let expected_header_checksum = rom[0x014d];
    if header_checksum != expected_header_checksum {
        return Err(CartError::HeaderCheckSumFailure {
            computed_checksum: header_checksum,
            expected_checksum: expected_header_checksum,
        });
    }
    Ok(())
}