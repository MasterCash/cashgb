#[cfg(test)]
mod controller_tests {
    use crate::memory::{
        cart::{
            CartError, MapperType,
            cart_header::{get_rom_size, ram_codes, rom_codes},
            tests::cart_tests::{create_test_cart_data, create_test_cart_sized_data},
        },
        controller::{create_mbc, mbc1},
    };
    use core::assert_matches;

    #[test]
    fn test_no_controller_create() {
        let rom = Vec::from(create_test_cart_data());

        let controller = create_mbc(MapperType::None, rom);
        assert!(controller.is_ok());

        let controller = controller.unwrap();
        assert_eq!(controller.as_ref().get_mapper_type(), MapperType::None);
    }

    #[test]
    fn test_no_controller_read() {
        let mut rom = Vec::from(create_test_cart_data());

        rom[0x154] = 0x01;

        let controller = create_mbc(MapperType::None, rom);
        assert!(controller.is_ok());

        let controller = controller.unwrap();
        assert_eq!(controller.get_mapper_type(), MapperType::None);

        assert_matches!(controller.read(0x154), Some(0x01));
    }

    #[test]
    fn test_no_controller_no_write_rom() {
        let mut rom = Vec::from(create_test_cart_data());

        rom[0x154] = 0x01;
        let controller = create_mbc(MapperType::None, rom);
        assert!(controller.is_ok());

        let mut controller = controller.unwrap();
        assert_eq!(controller.get_mapper_type(), MapperType::None);
        controller.write(0x154, 0x00);

        assert_matches!(controller.read(0x154), Some(0x01));
    }

    #[test]
    fn test_mbc1_create_ram_banks() {
        let rom = create_test_cart_sized_data(
            rom_codes::CODE_512_KILOBYTES,
            Some(ram_codes::CODE_32_KILOBYTES),
        );

        let controller = create_mbc(MapperType::MBC1, rom);

        assert!(controller.is_ok());
    }

    #[test]
    fn test_mbc1_create_fixed_ram() {
        let rom = create_test_cart_sized_data(
            rom_codes::CODE_1_MEGABYTES,
            Some(ram_codes::CODE_32_KILOBYTES),
        );

        let controller = create_mbc(MapperType::MBC1, rom);

        assert!(controller.is_ok());
    }

    #[test]
    fn test_mbc1_create_too_much_rom() {
        let rom =
            create_test_cart_sized_data(rom_codes::CODE_4_MEGABYTES, Some(ram_codes::CODE_NONE));

        let controller = create_mbc(MapperType::MBC1, rom);

        assert_matches!(
            controller,
            Err(CartError::UnsupportedMapperRomSize(
                MapperType::MBC1,
                rom_codes::CODE_4_MEGABYTES
            ))
        );
    }

    #[test]
    fn test_mbc1_create_too_much_ram_original() {
        let rom = create_test_cart_sized_data(
            rom_codes::CODE_512_KILOBYTES,
            Some(ram_codes::CODE_64_KILOBYTES),
        );

        let controller = create_mbc(MapperType::MBC1, rom);

        assert_matches!(
            controller,
            Err(CartError::UnsupportedMapperRamSize(
                MapperType::MBC1,
                ram_codes::CODE_64_KILOBYTES
            ))
        );
    }

    #[test]
    fn test_mbc1_create_too_much_ram_fixed() {
        let rom = create_test_cart_sized_data(
            rom_codes::CODE_2_MEGABYTES,
            Some(ram_codes::CODE_32_KILOBYTES),
        );

        let controller = create_mbc(MapperType::MBC1, rom);

        assert_matches!(
            controller,
            Err(CartError::UnsupportedMapperRamSize(
                MapperType::MBC1,
                ram_codes::CODE_32_KILOBYTES
            ))
        );
    }
    #[test]
    fn test_mbc1_read_first_rom_bank() {
        let (mut rom, _) = create_rom(rom_codes::CODE_2_MEGABYTES, ram_codes::CODE_8_KILOBYTES);

        rom[0x155] = 1;

        let controller = create_mbc(MapperType::MBC1, rom).unwrap();

        let datum = controller.read(0x155);

        assert_matches!(datum, Some(1));
    }

    #[test]
    fn test_mbc1_read_dynamic_rom_bank() {
        let (mut rom, bank_size) =
            create_rom(rom_codes::CODE_2_MEGABYTES, ram_codes::CODE_8_KILOBYTES);

        rom[bank_size] = 1;

        let controller = create_mbc(MapperType::MBC1, rom).unwrap();

        let datum = controller.read(bank_size as u16);

        assert_matches!(datum, Some(1));
    }

    #[test]
    fn test_mbc1_read_dynamic_rom_bank_swap() {
        use mbc1::mbc1_addresses::*;
        let (mut rom, bank_size) =
            create_rom(rom_codes::CODE_2_MEGABYTES, ram_codes::CODE_8_KILOBYTES);

        let actual_location = bank_size * 2;
        rom[actual_location] = 1;

        let mut controller = create_mbc(MapperType::MBC1, rom).unwrap();

        let datum = controller.read(bank_size as u16);

        assert_matches!(datum, Some(0));

        controller.write(ROM_BANK_NUM_START, 2);

        let datum = controller.read(bank_size as u16);

        assert_matches!(datum, Some(1));
    }

    #[test]
    fn test_mbc1_read_dynamic_rom_bank_never_zero() {
        use mbc1::mbc1_addresses::*;
        let (mut rom, bank_size) =
            create_rom(rom_codes::CODE_2_MEGABYTES, ram_codes::CODE_8_KILOBYTES);

        rom[0x00] = 1;
        rom[bank_size] = 2;
        let mut controller = create_mbc(MapperType::MBC1, rom).unwrap();
        let datum = controller.read(bank_size as u16);

        assert_matches!(datum, Some(2));

        controller.write(ROM_BANK_NUM_START, 0x00);

        let datum = controller.read(bank_size as u16);

        assert_matches!(datum, Some(2));
    }

    #[test]
    fn test_mbc1_read_dynamic_rom_bank_extra_bank() {
        use mbc1::mbc1_addresses::*;

        let (mut rom, bank_size) =
            create_rom(rom_codes::CODE_2_MEGABYTES, ram_codes::CODE_8_KILOBYTES);

        let actual_location = bank_size * 0x61;

        rom[actual_location] = 1;

        let mut controller = create_mbc(MapperType::MBC1, rom).unwrap();

        let datum = controller.read(bank_size as u16);

        assert_matches!(datum, Some(0));

        controller.write(ROM_BANK_NUM_START, 0x61);
        controller.write(RAM_BANK_NUM_START, 0x61 >> 5);

        let datum = controller.read(bank_size as u16);

        assert_matches!(datum, Some(1));
    }

    fn create_rom(rom_code: u8, ram_code: u8) -> (Vec<u8>, usize) {
        let rom = create_test_cart_sized_data(rom_code, Some(ram_code));
        let (rom_size, rom_banks) = get_rom_size(&rom_code).unwrap();
        let bank_size = rom_size as usize / rom_banks as usize;

        (rom, bank_size)
    }
}
