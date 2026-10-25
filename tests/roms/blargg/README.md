# Blargg's Game Boy test ROMs

Hardware test ROMs written by Shay Green (blargg), used by `tests/blargg.rs` to check the
emulator against real Game Boy behaviour.

- Original site: <http://blargg.8bitalley.com/parodius/gb-tests/>
- Copied from the community mirror: <https://github.com/retrio/gb-test-roms>
- Each suite's `readme.txt` is blargg's original documentation, unchanged.

The ROMs report results over the serial port: each character is written to `SB` (`0xFF01`),
then `0x81` is written to `SC` (`0xFF02`). A test finishes by printing `Passed` or `Failed`.
The ROM files are unmodified and keep their original names.

| Suite | What it checks |
|---|---|
| `cpu_instrs/cpu_instrs.gb` | Every CPU instruction except STOP and the illegal opcodes, all 11 groups in one ROM |
| `cpu_instrs/individual/*.gb` | The same 11 groups as separate ROMs, for narrowing down a failure |
| `instr_timing/instr_timing.gb` | How many cycles each instruction takes |
| `halt_bug/halt_bug.gb` | The HALT bug: HALT with IME off and an interrupt pending reads the next byte twice |

`halt_bug.gb` doesn't use the serial port: it prints `Passed` or `Failed` only on screen. Its
font tiles are numbered by ASCII code, so the test reads the text from the background tile
map (`Headless::run_screen_text_test`).
