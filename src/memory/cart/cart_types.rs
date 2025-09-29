use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
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
pub struct CartType {
    pub mapper: MapperType,
    pub ram: bool,
    pub battery: bool,
    pub timer: bool,
    pub rumble: bool,
    pub sensor: bool,
}

#[derive(Debug, Clone)]
pub enum CartError {
    MissingCart(String),
    InvalidRomSize(u8),
    InvalidRamSize(u8),
    InvalidCartType(u8),
    UnsupportedMapper(MapperType),
    InvalidLogo,
    HeaderCheckSumFailure {
        computed_checksum: u8,
        expected_checksum: u8,
    },
    GlobalCheckSumFailure {
        computed_checksum: u8,
        expected_checksum: u8,
    },
    LoadError,
}

impl CartType {
    pub fn new(code: &u8) -> Result<Self, CartError> {
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
            0x01 => cart_type.mapper = MapperType::MBC1,
            0x02 => {
                cart_type.mapper = MapperType::MBC1;
                cart_type.ram = true;
            }
            0x03 => {
                cart_type.mapper = MapperType::MBC1;
                cart_type.ram = true;
                cart_type.battery = true;
            }
            0x05 => cart_type.mapper = MapperType::MBC2,
            0x06 => {
                cart_type.mapper = MapperType::MBC2;
                cart_type.battery = true;
            }
            0x08 => cart_type.ram = true,
            0x09 => {
                cart_type.ram = true;
                cart_type.battery = true;
            }
            0x0B => cart_type.mapper = MapperType::MMM01,
            0x0C => {
                cart_type.mapper = MapperType::MMM01;
                cart_type.ram = true;
            }
            0x0D => {
                cart_type.mapper = MapperType::MMM01;
                cart_type.ram = true;
                cart_type.battery = true;
            }
            0x0F => {
                cart_type.mapper = MapperType::MBC3;
                cart_type.timer = true;
                cart_type.battery = true;
            }
            0x10 => {
                cart_type.mapper = MapperType::MBC3;
                cart_type.timer = true;
                cart_type.ram = true;
                cart_type.battery = true;
            }
            0x11 => cart_type.mapper = MapperType::MBC3,
            0x12 => {
                cart_type.mapper = MapperType::MBC3;
                cart_type.ram = true;
            }
            0x13 => {
                cart_type.mapper = MapperType::MBC3;
                cart_type.ram = true;
                cart_type.battery = true;
            }
            0x19 => cart_type.mapper = MapperType::MBC5,
            0x1A => {
                cart_type.mapper = MapperType::MBC5;
                cart_type.ram = true;
            }
            0x1B => {
                cart_type.mapper = MapperType::MBC5;
                cart_type.ram = true;
                cart_type.battery = true;
            }
            0x1C => {
                cart_type.mapper = MapperType::MBC5;
                cart_type.rumble = true;
            }
            0x1D => {
                cart_type.mapper = MapperType::MBC5;
                cart_type.rumble = true;
                cart_type.ram = true;
            }
            0x1E => {
                cart_type.mapper = MapperType::MBC5;
                cart_type.rumble = true;
                cart_type.ram = true;
                cart_type.battery = true;
            }
            0x20 => cart_type.mapper = MapperType::MBC6,
            0x22 => {
                cart_type.mapper = MapperType::MBC7;
                cart_type.sensor = true;
                cart_type.rumble = true;
                cart_type.ram = true;
                cart_type.battery = true;
            }
            0xFC => cart_type.mapper = MapperType::PocketCamera,
            0xFD => cart_type.mapper = MapperType::BandaiTama5,
            0xFE => cart_type.mapper = MapperType::HuC3,
            0xFF => {
                cart_type.mapper = MapperType::HuC1;
                cart_type.ram = true;
                cart_type.battery = true;
            }
            _ => return Err(CartError::InvalidCartType(*code)),
        }
        Ok(cart_type)
    }
}

impl Display for CartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.mapper)?;
        if self.ram {
            write!(f, "+RAM")?;
        }
        if self.battery {
            write!(f, "+BATTERY")?;
        }
        if self.timer {
            write!(f, "+TIMER")?;
        }
        if self.rumble {
            write!(f, "+RUMBLE")?;
        }
        if self.sensor {
            write!(f, "+SENSOR")?;
        }
        Ok(())
    }
}

impl Display for CartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CartError::MissingCart(path) => write!(f, "Missing cart: {}", path),
            CartError::InvalidRomSize(size) => write!(f, "Invalid ROM size: {}", size),
            CartError::InvalidRamSize(size) => write!(f, "Invalid RAM size: {}", size),
            CartError::InvalidCartType(cart_type) => write!(f, "Invalid cart type: {}", cart_type),
            CartError::UnsupportedMapper(mapper) => write!(f, "Unsupported mapper: {:?}", mapper),
            CartError::InvalidLogo => write!(f, "Invalid Nintendo logo"),
            CartError::HeaderCheckSumFailure { computed_checksum, expected_checksum } => {
                write!(f, "Header checksum failure: computed {}, expected {}", computed_checksum, expected_checksum)
            }
            CartError::GlobalCheckSumFailure { computed_checksum, expected_checksum } => {
                write!(f, "Global checksum failure: computed {}, expected {}", computed_checksum, expected_checksum)
            }
            CartError::LoadError => write!(f, "Load error"),
        }
    }
}

impl std::error::Error for CartError {}