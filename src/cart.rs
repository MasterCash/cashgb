use std::fmt::Display;

#[derive(Debug)]
pub enum MapperType {
    None,
    MBC1,
    MBC2,
    MMM01,
    MBC3,
    MBC5,
    MBC6,
    MBC7,
    PocketCamera,
    BandaiTama5,
    HuC3,
    HuC1,
}

pub enum BootFailure {
    None,
    HeaderCheckSum,
    GlobalCheckSum,
    Logo,
}

#[derive(Debug)]
struct CartType {
    mapper: MapperType,
    ram: bool,
    battery: bool,
    timer: bool,
    rumble: bool,
    sensor: bool,
}

impl CartType {
    fn new(code: &u8) -> Result<Self, CartError> {
        let mut cart_type = Self {
            mapper: MapperType::None,
            ram: false,
            battery: false,
            timer: false,
            rumble: false,
            sensor: false,
        };
        match code {
            0x00 => (),
            0x03 => {
                cart_type.battery = true;
                cart_type.ram = true;
                cart_type.mapper = MapperType::MBC1;
                return Err(CartError::UnsupportedMapper);
            }
            0x02 => {
                cart_type.ram = true;
                cart_type.mapper = MapperType::MBC1;
                return Err(CartError::UnsupportedMapper);
            }
            0x01 => {
                cart_type.mapper = MapperType::MBC1;
                return Err(CartError::UnsupportedMapper);
            }
            0x06 => {
                cart_type.battery = true;
                cart_type.mapper = MapperType::MBC2;
                return Err(CartError::UnsupportedMapper);
            }
            0x05 => {
                cart_type.mapper = MapperType::MBC2;
                return Err(CartError::UnsupportedMapper);
            }
            0x09 => {
                cart_type.battery = true;
                cart_type.ram = true;
                return Err(CartError::UnsupportedMapper);
            }
            0x08 => {
                cart_type.ram = true;
                return Err(CartError::UnsupportedMapper);
            }
            0x0D => {
                cart_type.battery = true;
                cart_type.ram = true;
                cart_type.mapper = MapperType::MMM01;
                return Err(CartError::UnsupportedMapper);
            }
            0x0C => {
                cart_type.ram = true;
                cart_type.mapper = MapperType::MMM01;
                return Err(CartError::UnsupportedMapper);
            }
            0x0B => {
                cart_type.mapper = MapperType::MMM01;
                return Err(CartError::UnsupportedMapper);
            }
            0x10 => {
                cart_type.ram = true;
                cart_type.timer = true;
                cart_type.battery = true;
                cart_type.mapper = MapperType::MBC3;
                return Err(CartError::UnsupportedMapper);
            }
            0x0F => {
                cart_type.timer = true;
                cart_type.battery = true;
                cart_type.mapper = MapperType::MBC3;
                return Err(CartError::UnsupportedMapper);
            }
            0x13 => {
                cart_type.battery = true;
                cart_type.ram = true;
                cart_type.mapper = MapperType::MBC3;
                return Err(CartError::UnsupportedMapper);
            }
            0x12 => {
                cart_type.ram = true;
                cart_type.mapper = MapperType::MBC3;
                return Err(CartError::UnsupportedMapper);
            }
            0x11 => {
                cart_type.mapper = MapperType::MBC3;
                return Err(CartError::UnsupportedMapper);
            }
            0x1b => {
                cart_type.battery = true;
                cart_type.ram = true;
                cart_type.mapper = MapperType::MBC5;
                return Err(CartError::UnsupportedMapper);
            }
            0x1a => {
                cart_type.ram = true;
                cart_type.mapper = MapperType::MBC5;
                return Err(CartError::UnsupportedMapper);
            }
            0x19 => {
                cart_type.mapper = MapperType::MBC5;
                return Err(CartError::UnsupportedMapper);
            }
            0x1e => {
                cart_type.battery = true;
                cart_type.rumble = true;
                cart_type.ram = true;
                cart_type.mapper = MapperType::MBC5;
                return Err(CartError::UnsupportedMapper);
            }
            0x1d => {
                cart_type.ram = true;
                cart_type.rumble = true;
                cart_type.mapper = MapperType::MBC5;
                return Err(CartError::UnsupportedMapper);
            }
            0x1c => {
                cart_type.rumble = true;
                cart_type.mapper = MapperType::MBC5;
                return Err(CartError::UnsupportedMapper);
            }
            0x20 => {
                cart_type.mapper = MapperType::MBC6;
                return Err(CartError::UnsupportedMapper);
            }
            0x22 => {
                cart_type.battery = true;
                cart_type.rumble = true;
                cart_type.ram = true;
                cart_type.sensor = true;
                cart_type.mapper = MapperType::MBC7;
                return Err(CartError::UnsupportedMapper);
            }
            0xfc => {
                cart_type.mapper = MapperType::PocketCamera;
                return Err(CartError::UnsupportedMapper);
            }
            0xfd => {
                cart_type.mapper = MapperType::BandaiTama5;
                return Err(CartError::UnsupportedMapper);
            }
            0xfe => {
                cart_type.mapper = MapperType::HuC3;
                return Err(CartError::UnsupportedMapper);
            }
            0xff => {
                cart_type.mapper = MapperType::HuC1;
                cart_type.ram = true;
                cart_type.battery = true;
                return Err(CartError::UnsupportedMapper);
            }
            _ => return Err(CartError::InvalidCartType(code.clone())),
        };
        Ok(cart_type)
    }
}

impl Display for CartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.mapper)?;
        if self.battery {
            write!(f, " + battery")?;
        }
        if self.ram {
            write!(f, " + ram")?;
        }
        if self.rumble {
            write!(f, " + rumble")?;
        }
        if self.sensor {
            write!(f, " + sensor")?;
        }
        if self.timer {
            write!(f, " + timer")?;
        }

        Ok(())
    }
}

static NINTENDO_LOGO: [u8; 0x30] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

pub struct Cart {
    rom: Vec<u8>,
    ram: Vec<u8>,
    title: String,
    cgb: bool,
    cart_type: CartType,
    licensee: String,
    sgb: bool,
    rom_size: u32,
    rom_banks: u8,
    current_rom_bank: u8,
    ram_size: u32,
    ram_banks: u8,
    current_ram_bank: u8,
    destination: bool,
    version: u8,
}

impl Display for Cart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\t title: {}", self.title)?;
        writeln!(f, "\t cgb: {}", self.cgb)?;
        writeln!(f, "\t cart type: {}", self.cart_type)?;
        writeln!(f, "\t licensee: {}", self.licensee)?;
        writeln!(f, "\t sgb: {}", self.sgb)?;
        writeln!(f, "\t rom size: {}", self.rom_size)?;
        writeln!(f, "\t rom banks: {}", self.rom_banks)?;
        writeln!(f, "\t current rom bank: {}", self.current_rom_bank)?;
        writeln!(f, "\t ram size: {}", self.ram_size)?;
        writeln!(f, "\t ram banks: {}", self.ram_banks)?;
        writeln!(f, "\t current ram bank: {}", self.current_ram_bank)?;
        writeln!(f, "\t destination: {}", self.destination)?;
        writeln!(f, "\t version: {}", self.version)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum CartError {
    MissingCart(String),
    InvalidRomSize(u8),
    InvalidRamSize(u8),
    InvalidCartType(u8),
    UnsupportedMapper,
    InvalidLogo,
    HeaderCheckSumFailure {
        computed_checksum: u8,
        expected_checksum: u8,
    },
    GlobalCheckSumFailure {
        computed_checksum: u8,
        expected_checksum: u8,
    },
    ReadError,
    LoadError,
}
impl Display for CartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)?;
        Ok(())
    }
}
impl std::error::Error for CartError {}

impl Cart {
    pub fn new(data: &Vec<u8>) -> Result<Self, CartError> {
        let mut title: Vec<char> = vec![];
        let (rom_size, rom_banks) = Cart::get_rom_size(&data[0x0148])?;
        let mut rom = vec![0; rom_size as usize];

        for (i, byte) in data.iter().enumerate() {
            rom[i] = byte.clone();
        }

        for byte in &rom[0x0134..=(0x0134 + 15)] {
            if *byte == 0x00 {
                break;
            }
            title.insert(title.len(), char::from(byte.clone()));
        }

        let (ram_size, ram_banks) = Cart::get_ram_size(&rom[0x0149])?;

        let mut header_checksum = 0u8;
        for byte in &rom[0x0134..=0x014C] {
            (header_checksum, _) = header_checksum.overflowing_sub(*byte);
            (header_checksum, _) = header_checksum.overflowing_sub(1);
        }
        let expected_header_checksum = &rom[0x014d];
        if header_checksum as u8 != *expected_header_checksum {
            return Err(CartError::HeaderCheckSumFailure {
                computed_checksum: header_checksum,
                expected_checksum: expected_header_checksum.clone(),
            });
        }

        for (i, byte) in rom[0x0104..=0x0133].iter().enumerate() {
            if *byte != NINTENDO_LOGO[i] {
                return Err(CartError::InvalidLogo);
            }
        }

        Ok(Self {
            cgb: (rom[0x0143] & 0x80) > 0,
            rom_size,
            rom_banks,
            sgb: rom[0x0146] == 0x03,
            ram_size,
            ram_banks,
            ram: vec![0; ram_size as usize],
            cart_type: CartType::new(&rom[0x0147])?,
            title: title.iter().collect(),
            current_ram_bank: 0,
            current_rom_bank: 0,
            destination: rom[0x014A] == 0x01,
            version: rom[0x014c],
            licensee: Cart::get_licensee(&rom[0x014B], &rom[0x0144], &rom[0x0145]),
            rom,
        })
    }

    pub fn read(&self, addr: &u16) -> Option<u8> {
        match addr {
            // ROM Bank 0
            0x0000..=0x3fff => match self.rom_size < *addr as u32 {
                true => None,
                false => Some(self.rom[*addr as usize]),
            },
            // ROM Bank 1..NN
            0x4000..=0x7fff => {
                let addr = *addr as u32 + (0x4000 as u32 * self.current_rom_bank as u32);
                if self.rom_size < addr {
                    return None;
                };
                return Some(self.rom[addr as usize]);
            }
            0xA000..=0xBFFF => {
                let addr = *addr as u32 + (0x2000 as u32 * self.current_ram_bank as u32);
                if self.ram_size < addr {
                    return None;
                }
                return Some(self.ram[addr as usize]);
            }
            _ => None,
        }
    }

    pub fn write(&mut self, _addr: &u16, _value: u8) {
        todo!()
        //match self.cart_type.mapper {
        //    MapperType::MBC1 => match addr {
        //        0xA000..=0xBFFF => {
        //            let addr = *addr as u32 + (0x2000 as u32 * self.current_ram_bank as u32);
        //            if self.ram_size >= addr {
        //                self.ram[addr as usize] = value;
        //            }
        //        }
        //        0x0000..=0x1fff => {
        //            self.ram_enabled = (value & 0x0a) == 0x0a;
        //        }
        //        0x2000..=0x3fff => {
        //            let mut value = value;
        //            if value == 0x00 || value == 0x20 || value == 0x40 || value == 0x60 {
        //                value += 1;
        //            }
        //            self.current_rom_bank = value;
        //        }
        //    },
        //}
    }

    fn get_rom_size(code: &u8) -> Result<(u32, u8), CartError> {
        match code {
            0x00..=0x08 => Ok((0x8000 * (1 << code), 2u8 << code)),
            _ => Err(CartError::InvalidRomSize(code.clone())),
        }
    }
    fn get_ram_size(code: &u8) -> Result<(u32, u8), CartError> {
        match code {
            0x00 => Ok((0, 0)),
            0x02 => Ok((0x2000, 1)),
            0x03 => Ok((0x8000, 4)),
            0x04 => Ok((0x20000, 16)),
            0x05 => Ok((0x10000, 8)),
            _ => Err(CartError::InvalidRamSize(code.clone())),
        }
    }

    fn get_licensee(code: &u8, one: &u8, two: &u8) -> String {
        let licensee = match code {
            0x00 => "None",
            0x01 => "Nintendo",
            0x08 => "Capcom",
            0x09 => "Hot-B",
            0x0a => "Jaleco",
            0x0b => "Coconuts Japan",
            0x0c => "Elite Systems",
            0x13 => "EA (Electronic Arts)",
            0x18 => "Hudsonsoft",
            0x19 => "ITC Entertainment",
            0x1a => "Yanoman",
            0x1d => "Japan Clary",
            0x1f => "Virgin Interactive",
            0x24 => "PCM Complete",
            0x25 => "San-X",
            0x28 => "Kemco",
            0x29 => "Seta",
            0x30 => "Infogrames",
            0x31 => "Nintendo",
            0x32 => "Bandai",
            0x33 => match [char::from(*one), char::from(*two)] {
                ['0', '1'] => "Nintendo Research & Development 1",
                ['0', '8'] => "Capcom",
                ['1', '3'] => "EA (Electronic Arts)",
                ['1', '8'] => "Hudson Soft",
                ['1', '9'] => "B-AI",
                ['2', '0'] => "KSS",
                ['2', '2'] => "Planning Office WADA",
                ['2', '4'] => "PCM Complete",
                ['2', '5'] => "San-X",
                ['2', '8'] => "Kemco",
                ['2', '9'] => "SETA Corporation",
                ['3', '0'] => "Viacom",
                ['3', '1'] => "Nintendo",
                ['3', '2'] => "Bandai",
                ['3', '3'] => "Ocean Software/Acclaim Entertainment",
                ['3', '4'] => "Konami",
                ['3', '5'] => "HectorSoft",
                ['3', '7'] => "Taito",
                ['3', '8'] => "Hudson Soft",
                ['3', '9'] => "Banpresto",
                ['4', '1'] => "Ubi Soft",
                ['4', '2'] => "Atlus",
                ['4', '4'] => "Malibu Interactive",
                ['4', '6'] => "Angel",
                ['4', '7'] => "Bullet-Proof Software",
                ['4', '9'] => "Irem",
                ['5', '0'] => "Absolute",
                ['5', '1'] => "Acclaim Entertainment",
                ['5', '2'] => "Activision",
                ['5', '3'] => "Sammy USA Corporation",
                ['5', '4'] => "Konami",
                ['5', '5'] => "Hi Tech Expressions",
                ['5', '6'] => "LJN",
                ['5', '7'] => "Matchbox",
                ['5', '8'] => "Mattel",
                ['5', '9'] => "Milton Bradley Company",
                ['6', '0'] => "Titus Interactive",
                ['6', '1'] => "Virgin Games Ltd.",
                ['6', '4'] => "Lucasfilm Games4",
                ['6', '7'] => "Ocean Software",
                ['6', '9'] => "EA (Electronic Arts)",
                ['7', '0'] => "Infogrames5",
                ['7', '1'] => "Interplay Entertainment",
                ['7', '2'] => "Broderbund",
                ['7', '3'] => "Sculptured Software6",
                ['7', '5'] => "The Sales Curve Limited7",
                ['7', '8'] => "THQ",
                ['7', '9'] => "Accolade",
                ['8', '0'] => "Misawa Entertainment",
                ['8', '3'] => "lozc",
                ['8', '6'] => "Tokuma Shoten",
                ['8', '7'] => "Tsukuda Original",
                ['9', '1'] => "Chunsoft Co.8",
                ['9', '2'] => "Video System",
                ['9', '3'] => "Ocean Software/Acclaim Entertainment",
                ['9', '5'] => "Varie",
                ['9', '6'] => "Yonezawa/s'pal",
                ['9', '7'] => "Kaneko",
                ['9', '9'] => "Pack-In-Video",
                ['9', 'H'] => "Bottom Up",
                ['A', '4'] => "Konami (Yu-Gi-Oh!)",
                ['B', 'L'] => "MTO",
                ['D', 'K'] => "Kodansha",
                ['0', '0'] | _ => "None",
            },
            0x34 => "Konami",
            0x35 => "HectorSoft",
            0x38 => "Capcom",
            0x39 => "Banpresto",
            0x3c => ".Entertainment i",
            0x3e => "Gremlin",
            0x41 => "Ubisoft",
            0x42 => "Atlus",
            0x44 => "Malibu",
            0x46 => "Angel",
            0x47 => "Spectrum Holoby",
            0x49 => "Irem",
            0x4a => "Virgin Interactive",
            0x4d => "Malibu",
            0x4f => "U.S. Gold",
            0x50 => "Absolute",
            0x51 => "Acclaim",
            0x52 => "Activision",
            0x53 => "American Sammy",
            0x54 => "GameTek",
            0x55 => "Park Place",
            0x56 => "LJN",
            0x57 => "Matchbox",
            0x59 => "Milton Bradley",
            0x5a => "Mindscape",
            0x5B => "Romstar",
            0x5C => "Naxat Soft",
            0x5D => "Tradewest",
            0x60 => "Titus Interactive",
            0x61 => "Virgin Games Ltd.",
            0x67 => "Ocean Software",
            0x69 => "EA (Electronic Arts)",
            0x6E => "Elite Systems",
            0x6F => "Electro Brain",
            0x70 => "Infogrames",
            0x71 => "Interplay Entertainment",
            0x72 => "Broderbund",
            0x73 => "Sculptured Software",
            0x75 => "The Sales Curve Limited",
            0x78 => "THQ",
            0x79 => "Accolade",
            0x7A => "Triffix Entertainment",
            0x7C => "Microprose",
            0x7F => "Kemco",
            0x80 => "Misawa Entertainment",
            0x83 => "Lozc",
            0x86 => "Tokuma Shoten",
            0x8B => "Bullet-Proof Software",
            0x8C => "Vic Tokai",
            0x8E => "Ape",
            0x8F => "I'Max",
            0x91 => "Chunsoft Co.",
            0x92 => "Video System",
            0x93 => "Tsubaraya Productions",
            0x95 => "Varie",
            0x96 => "Yonezawa/S'Pal",
            0x97 => "Kemco",
            0x99 => "Arc",
            0x9A => "Nihon Bussan",
            0x9B => "Tecmo",
            0x9C => "Imagineer",
            0x9D => "Banpresto",
            0x9F => "Nova",
            0xA1 => "Hori Electric",
            0xA2 => "Bandai",
            0xA4 => "Konami",
            0xA6 => "Kawada",
            0xA7 => "Takara",
            0xA9 => "Technos Japan",
            0xAA => "Broderbund",
            0xAC => "Toei Animation",
            0xAD => "Toho",
            0xAF => "Namco",
            0xB0 => "Acclaim Entertainment",
            0xB1 => "ASCII Corporation or Nexsoft",
            0xB2 => "Bandai",
            0xB4 => "Square Enix",
            0xB6 => "HAL Laboratory",
            0xB7 => "SNK",
            0xB9 => "Pony Canyon",
            0xBA => "Culture Brain",
            0xBB => "Sunsoft",
            0xBD => "Sony Imagesoft",
            0xBF => "Sammy Corporation",
            0xC0 => "Taito",
            0xC2 => "Kemco",
            0xC3 => "Square",
            0xC4 => "Tokuma Shoten",
            0xC5 => "Data East",
            0xC6 => "Tonkinhouse",
            0xC8 => "Koei",
            0xC9 => "UFL",
            0xCA => "Ultra",
            0xCB => "Vap",
            0xCC => "Use Corporation",
            0xCD => "Meldac",
            0xCE => "Pony Canyon",
            0xCF => "Angel",
            0xD0 => "Taito",
            0xD1 => "Sofel",
            0xD2 => "Quest",
            0xD3 => "Sigma Enterprises",
            0xD4 => "ASK Kodansha Co.",
            0xD6 => "Naxat Soft13",
            0xD7 => "Copya System",
            0xD9 => "Banpresto",
            0xDA => "Tomy",
            0xDB => "LJN",
            0xDD => "NCS",
            0xDE => "Human",
            0xDF => "Altron",
            0xE0 => "Jaleco",
            0xE1 => "Towa Chiki",
            0xE2 => "Yutaka",
            0xE3 => "Varie",
            0xE5 => "Epcoh",
            0xE7 => "Athena",
            0xE8 => "Asmik Ace Entertainment",
            0xE9 => "Natsume",
            0xEA => "King Records",
            0xEB => "Atlus",
            0xEC => "Epic/Sony Records",
            0xEE => "IGS",
            0xF0 => "A Wave",
            0xF3 => "Extreme Entertainment",
            0xFF => "LJN",
            _ => "None",
        };

        String::from(licensee)
    }
}
