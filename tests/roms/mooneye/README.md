# Mooneye test suite

Hardware test ROMs written by Joonas Javanainen (Gekkio), used by `tests/mooneye.rs` to check
the emulator against real Game Boy behaviour, mostly timing.

- Source: <https://github.com/Gekkio/mooneye-test-suite>
- Copied from the prebuilt release `mts-20240926-1737-443f6e1`:
  <https://gekkio.fi/files/mooneye-test-suite/>
- MIT licensed; see `LICENSE`. The ROM and `.sym` files are unmodified and keep their
  original names and folders.

When a test finishes, it sends the Fibonacci bytes `3, 5, 8, 13, 21, 34` over the serial port
if it passed, or six `0x42` bytes if it failed.

Only the tests that apply to a DMG are copied:

| Folder | What it checks |
|---|---|
| `acceptance/` | CPU, interrupt, timer, OAM DMA and PPU behaviour, much of it to the exact cycle |
| `emulator-only/mbc1/` | MBC1 banking and RAM |

Of the `acceptance/` ROMs, those for other models (`-S`, `-sgb`, `-mgb`, `-dmg0`) are left
out. A `-GS` suffix means the test passes on a DMG and a Super Game Boy.

Tests that cash-gb doesn't pass yet are marked `#[ignore]` in `tests/mooneye.rs`, with the
reason. Run them with:

```bash
cargo test --release --test mooneye -- --ignored
```
