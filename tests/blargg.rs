//! Blargg's hardware test ROMs, run headless. Each ROM reports over the serial port; see
//! `tests/roms/blargg/README.md`.

use std::path::Path;

use cash_gb::{
    headless::{FRAMES_PER_SECOND, Headless, SerialResult},
    read_cart,
};

/// Run a ROM under `tests/roms/blargg/` until it reports, and fail with its serial output
/// if it didn't pass.
fn run_blargg(rom: &str, max_seconds: u32) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/roms/blargg")
        .join(rom);
    let cart = read_cart(&path).unwrap_or_else(|error| panic!("loading {rom}: {error}"));
    let report = Headless::new(cart).run_serial_test(max_seconds * FRAMES_PER_SECOND);
    assert_eq!(report.result, SerialResult::Passed, "{rom}\n{report}");
}

macro_rules! blargg_tests {
    ($($name:ident: $rom:literal, $max_seconds:literal;)*) => {
        $(
            #[test]
            fn $name() {
                run_blargg($rom, $max_seconds);
            }
        )*
    };
}

// The individual cpu_instrs groups run in a few emulated seconds each; the limits leave
// plenty of headroom so only a hang or a wrong result fails them.
blargg_tests! {
    cpu_instrs_01_special: "cpu_instrs/individual/01-special.gb", 30;
    cpu_instrs_02_interrupts: "cpu_instrs/individual/02-interrupts.gb", 30;
    cpu_instrs_03_op_sp_hl: "cpu_instrs/individual/03-op sp,hl.gb", 30;
    cpu_instrs_04_op_r_imm: "cpu_instrs/individual/04-op r,imm.gb", 30;
    cpu_instrs_05_op_rp: "cpu_instrs/individual/05-op rp.gb", 30;
    cpu_instrs_06_ld_r_r: "cpu_instrs/individual/06-ld r,r.gb", 30;
    cpu_instrs_07_jr_jp_call_ret_rst: "cpu_instrs/individual/07-jr,jp,call,ret,rst.gb", 30;
    cpu_instrs_08_misc_instrs: "cpu_instrs/individual/08-misc instrs.gb", 30;
    cpu_instrs_09_op_r_r: "cpu_instrs/individual/09-op r,r.gb", 30;
    cpu_instrs_10_bit_ops: "cpu_instrs/individual/10-bit ops.gb", 30;
    cpu_instrs_11_op_a_hl: "cpu_instrs/individual/11-op a,(hl).gb", 30;
    cpu_instrs_all: "cpu_instrs/cpu_instrs.gb", 120;
    instr_timing: "instr_timing/instr_timing.gb", 30;
}
