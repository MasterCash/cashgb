#[cfg(test)]
pub mod cart_tests {
    use core::assert_matches;
    const TITLE: &str = "CASH TEST GAME 1";
    use crate::memory::cart::{
        Cart, CartError, MapperType,
        cart_header::{NINTENDO_LOGO, addresses, get_ram_size, validate_nintendo_logo},
    };

    pub fn create_test_cart_sized_data(rom_code: u8, ram_code: Option<u8>) -> Vec<u8> {
        let mut data = Vec::from(create_test_cart_data());
        use crate::memory::cart::cart_header::get_rom_size;
        if let Ok((rom_size, _)) = get_rom_size(&rom_code)
            && rom_code != 0x00
        {
            data[addresses::ROM_SIZE as usize] = rom_code;
            data.resize(rom_size as usize, 0x00);
            if let Some(ram_code) = ram_code
                && let Ok(_) = get_ram_size(&ram_code)
            {
                data[addresses::RAM_SIZE as usize] = ram_code;
            }
        };

        data
    }

    pub fn create_test_cart_data() -> [u8; 0x8000] {
        let mut data = [0; 0x8000];
        // setup logo
        for (i, bit) in NINTENDO_LOGO.iter().enumerate() {
            data[addresses::NINTENDO_LOGO_START as usize + i] = *bit;
        }

        for (i, bit) in TITLE.as_bytes().iter().enumerate() {
            data[0x134 + i] = *bit;
        }

        // cartidge type
        data[addresses::CARTRIDGE_TYPE as usize] = 0x08;
        // rom size
        data[addresses::ROM_SIZE as usize] = 0x00;
        // ram size
        data[addresses::RAM_SIZE as usize] = 0x02;

        // destination code
        data[addresses::DESTINATION_CODE as usize] = 0x01;

        // licensee code
        data[addresses::LICENSEE_CODE as usize] = 0x01;

        // ROM version number
        data[addresses::ROM_VERSION as usize] = 0x01;

        data[addresses::HEADER_CHECKSUM as usize] = 208;

        data
    }

    pub fn _create_test_cart() -> Cart {
        Cart::new(&create_test_cart_data()).expect("failed to make test cart")
    }

    #[test]
    fn test_cart_validate_nintendo_logo_invalid() {
        let mut data = create_test_cart_data();
        data[addresses::NINTENDO_LOGO_START as usize] = 0xff;

        let result = validate_nintendo_logo(&data);

        assert_matches!(result, Err(CartError::InvalidLogo));
    }

    #[test]
    fn test_cart_validate_nintendo_logo() {
        let data = create_test_cart_data();

        let result = validate_nintendo_logo(&data);

        assert_matches!(result, Ok(_));
    }

    #[test]
    fn test_cart_create() {
        let data = create_test_cart_data();

        let result = Cart::new(&data);

        assert_matches!(result, Ok(_));

        let result = result.unwrap();

        assert_eq!(result.cart_type.mapper, MapperType::None);
        assert!(!result.cart_type.battery);
        assert!(result.cart_type.ram);
        assert_eq!(result.licensee, "Nintendo");
        assert!(result.destination);
        assert_eq!(result.ram_banks, 1);
        assert_eq!(result.rom_banks, 2);
        assert_eq!(result.title, TITLE);
    }

    #[test]
    fn test_cart_new_empty() {
        let data: [u8; 4] = [0; 4];
        let result = Cart::new(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_cart_rom_size_invalid_size() {
        use crate::memory::cart::cart_header::get_rom_size;
        let result = get_rom_size(&0x09);

        assert!(result.is_err());
        if let Err(result) = result {
            assert_matches!(result, CartError::InvalidRomSize(0x09))
        }
    }

    #[test]
    fn test_cart_rom_size_valid_sizes() {
        use crate::memory::cart::cart_header::get_rom_size;
        const BANK_SIZE: u32 = 0x4000;
        for code in 0x00u8..=0x08 {
            let banks = 2u16 << code;
            let size = BANK_SIZE * banks as u32;
            let result = get_rom_size(&code);
            assert_matches!(result, Ok((actual_size, _)) if actual_size == size);
            assert_matches!(result, Ok((_, actual_banks)) if actual_banks == banks);
        }
    }

    #[test]
    fn test_cart_ram_size_valid_sizes() {
        use crate::memory::cart::cart_header::get_ram_size;
        const BANK_SIZE: u32 = 0x2000;
        let result = get_ram_size(&0x00);
        assert_matches!(result, Ok((size, banks)) if size == 0 && banks == 0);

        let result = get_ram_size(&0x02);
        assert_matches!(result, Ok((size, banks)) if size == BANK_SIZE && banks == 1);

        let result = get_ram_size(&0x03);
        assert_matches!(result, Ok((size, banks)) if size == BANK_SIZE * 4 && banks == 4);

        let result = get_ram_size(&0x04);
        assert_matches!(result, Ok((size, banks)) if size == BANK_SIZE * 16 && banks == 16);

        let result = get_ram_size(&0x05);
        assert_matches!(result, Ok((size, banks)) if size == BANK_SIZE * 8 && banks == 8);
    }

    #[test]
    fn test_cart_ram_size_invalid_size() {
        use crate::memory::cart::cart_header::get_ram_size;
        let result = get_ram_size(&0x01);
        assert_matches!(result, Err(CartError::InvalidRamSize(0x01)));
        let result = get_ram_size(&0x08);
        assert_matches!(result, Err(CartError::InvalidRamSize(0x08)));
    }
}
