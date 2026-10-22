//! The Mooneye test suite, run headless. Each ROM reports over the serial port; see
//! `tests/roms/mooneye/README.md`.

use std::path::Path;

use cash_gb::{
    headless::{FRAMES_PER_SECOND, Headless, SerialResult},
    read_cart,
};

/// Each test finishes within a second or two of emulated time; this leaves headroom
const MAX_SECONDS: u32 = 20;

/// Run a ROM under `tests/roms/mooneye/` until it reports, and fail with the bytes it sent
/// if it didn't pass.
fn run_mooneye(rom: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/roms/mooneye")
        .join(rom);
    let cart = read_cart(&path).unwrap_or_else(|error| panic!("loading {rom}: {error}"));
    let report = Headless::new(cart).run_mooneye_test(MAX_SECONDS * FRAMES_PER_SECOND);
    assert_eq!(report.result, SerialResult::Passed, "{rom}\n{report}");
}

macro_rules! mooneye_tests {
    ($($(#[$attr:meta])* $name:ident: $rom:literal;)*) => {
        $(
            #[test]
            $(#[$attr])*
            fn $name() {
                run_mooneye($rom);
            }
        )*
    };
}

mooneye_tests! {
    #[ignore = "needs cycle-accurate memory timing"]
    add_sp_e_timing: "acceptance/add_sp_e_timing.gb";
    bits_mem_oam: "acceptance/bits/mem_oam.gb";
    bits_reg_f: "acceptance/bits/reg_f.gb";
    #[ignore = "unused I/O register bits do not read back as 1"]
    bits_unused_hwio_gs: "acceptance/bits/unused_hwio-GS.gb";
    #[ignore = "hardware state after the boot ROM is not matched"]
    boot_div_dmgabcmgb: "acceptance/boot_div-dmgABCmgb.gb";
    #[ignore = "hardware state after the boot ROM is not matched"]
    boot_hwio_dmgabcmgb: "acceptance/boot_hwio-dmgABCmgb.gb";
    boot_regs_dmgabc: "acceptance/boot_regs-dmgABC.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    call_cc_timing: "acceptance/call_cc_timing.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    call_cc_timing2: "acceptance/call_cc_timing2.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    call_timing: "acceptance/call_timing.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    call_timing2: "acceptance/call_timing2.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    di_timing_gs: "acceptance/di_timing-GS.gb";
    div_timing: "acceptance/div_timing.gb";
    ei_sequence: "acceptance/ei_sequence.gb";
    ei_timing: "acceptance/ei_timing.gb";
    halt_ime0_ei: "acceptance/halt_ime0_ei.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    halt_ime0_nointr_timing: "acceptance/halt_ime0_nointr_timing.gb";
    halt_ime1_timing: "acceptance/halt_ime1_timing.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    halt_ime1_timing2_gs: "acceptance/halt_ime1_timing2-GS.gb";
    if_ie_registers: "acceptance/if_ie_registers.gb";
    instr_daa: "acceptance/instr/daa.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    interrupts_ie_push: "acceptance/interrupts/ie_push.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    intr_timing: "acceptance/intr_timing.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    jp_cc_timing: "acceptance/jp_cc_timing.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    jp_timing: "acceptance/jp_timing.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    ld_hl_sp_e_timing: "acceptance/ld_hl_sp_e_timing.gb";
    oam_dma_basic: "acceptance/oam_dma/basic.gb";
    oam_dma_reg_read: "acceptance/oam_dma/reg_read.gb";
    #[ignore = "OAM DMA restart and source edge cases"]
    oam_dma_sources_gs: "acceptance/oam_dma/sources-GS.gb";
    #[ignore = "OAM DMA restart and source edge cases"]
    oam_dma_restart: "acceptance/oam_dma_restart.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    oam_dma_start: "acceptance/oam_dma_start.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    oam_dma_timing: "acceptance/oam_dma_timing.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    pop_timing: "acceptance/pop_timing.gb";
    #[ignore = "needs accurate PPU mode timing"]
    ppu_hblank_ly_scx_timing_gs: "acceptance/ppu/hblank_ly_scx_timing-GS.gb";
    #[ignore = "needs accurate PPU mode timing"]
    ppu_intr_1_2_timing_gs: "acceptance/ppu/intr_1_2_timing-GS.gb";
    #[ignore = "needs accurate PPU mode timing"]
    ppu_intr_2_0_timing: "acceptance/ppu/intr_2_0_timing.gb";
    #[ignore = "needs accurate PPU mode timing"]
    ppu_intr_2_mode0_timing: "acceptance/ppu/intr_2_mode0_timing.gb";
    #[ignore = "needs accurate PPU mode timing"]
    ppu_intr_2_mode0_timing_sprites: "acceptance/ppu/intr_2_mode0_timing_sprites.gb";
    #[ignore = "needs accurate PPU mode timing"]
    ppu_intr_2_mode3_timing: "acceptance/ppu/intr_2_mode3_timing.gb";
    #[ignore = "needs accurate PPU mode timing"]
    ppu_intr_2_oam_ok_timing: "acceptance/ppu/intr_2_oam_ok_timing.gb";
    #[ignore = "needs accurate PPU mode timing"]
    ppu_lcdon_timing_gs: "acceptance/ppu/lcdon_timing-GS.gb";
    #[ignore = "needs accurate PPU mode timing"]
    ppu_lcdon_write_timing_gs: "acceptance/ppu/lcdon_write_timing-GS.gb";
    ppu_stat_irq_blocking: "acceptance/ppu/stat_irq_blocking.gb";
    #[ignore = "needs accurate PPU mode timing"]
    ppu_stat_lyc_onoff: "acceptance/ppu/stat_lyc_onoff.gb";
    #[ignore = "needs accurate PPU mode timing"]
    ppu_vblank_stat_intr_gs: "acceptance/ppu/vblank_stat_intr-GS.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    push_timing: "acceptance/push_timing.gb";
    rapid_di_ei: "acceptance/rapid_di_ei.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    ret_cc_timing: "acceptance/ret_cc_timing.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    ret_timing: "acceptance/ret_timing.gb";
    reti_intr_timing: "acceptance/reti_intr_timing.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    reti_timing: "acceptance/reti_timing.gb";
    #[ignore = "needs cycle-accurate memory timing"]
    rst_timing: "acceptance/rst_timing.gb";
    #[ignore = "hardware state after the boot ROM is not matched"]
    serial_boot_sclk_align_dmgabcmgb: "acceptance/serial/boot_sclk_align-dmgABCmgb.gb";
    timer_div_write: "acceptance/timer/div_write.gb";
    timer_rapid_toggle: "acceptance/timer/rapid_toggle.gb";
    timer_tim00: "acceptance/timer/tim00.gb";
    timer_tim00_div_trigger: "acceptance/timer/tim00_div_trigger.gb";
    timer_tim01: "acceptance/timer/tim01.gb";
    timer_tim01_div_trigger: "acceptance/timer/tim01_div_trigger.gb";
    timer_tim10: "acceptance/timer/tim10.gb";
    timer_tim10_div_trigger: "acceptance/timer/tim10_div_trigger.gb";
    timer_tim11: "acceptance/timer/tim11.gb";
    timer_tim11_div_trigger: "acceptance/timer/tim11_div_trigger.gb";
    #[ignore = "TIMA reload from TMA is not delayed by one cycle"]
    timer_tima_reload: "acceptance/timer/tima_reload.gb";
    #[ignore = "TIMA reload from TMA is not delayed by one cycle"]
    timer_tima_write_reloading: "acceptance/timer/tima_write_reloading.gb";
    #[ignore = "TIMA reload from TMA is not delayed by one cycle"]
    timer_tma_write_reloading: "acceptance/timer/tma_write_reloading.gb";
    mbc1_bits_bank1: "emulator-only/mbc1/bits_bank1.gb";
    mbc1_bits_bank2: "emulator-only/mbc1/bits_bank2.gb";
    mbc1_bits_mode: "emulator-only/mbc1/bits_mode.gb";
    mbc1_bits_ramg: "emulator-only/mbc1/bits_ramg.gb";
    #[ignore = "MBC1 multicarts are not detected"]
    mbc1_multicart_rom_8mb: "emulator-only/mbc1/multicart_rom_8Mb.gb";
    mbc1_ram_256kb: "emulator-only/mbc1/ram_256kb.gb";
    #[ignore = "MBC1 with 64 KiB of RAM fails"]
    mbc1_ram_64kb: "emulator-only/mbc1/ram_64kb.gb";
    mbc1_rom_16mb: "emulator-only/mbc1/rom_16Mb.gb";
    mbc1_rom_1mb: "emulator-only/mbc1/rom_1Mb.gb";
    mbc1_rom_2mb: "emulator-only/mbc1/rom_2Mb.gb";
    mbc1_rom_4mb: "emulator-only/mbc1/rom_4Mb.gb";
    mbc1_rom_512kb: "emulator-only/mbc1/rom_512kb.gb";
    mbc1_rom_8mb: "emulator-only/mbc1/rom_8Mb.gb";
}
