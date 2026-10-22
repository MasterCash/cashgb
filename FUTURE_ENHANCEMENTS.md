# Future Enhancements

Features that would make cash-gb competitive with established Game Boy emulators such as SameBoy, BGB, Gambatte, mGBA and Emulicious.

The tiers are in a suggested build order: each one builds on the one before.

**Current state:** the CPU, PPU (scanline renderer), joypad, OAM DMA, timer, NoMBC and MBC1 cartridges, and an egui debugger UI work. There is no audio, no save support, and only two cartridge types.

## Suggested next steps

1. Fix the ignored Mooneye tests (`cargo test --release --test mooneye -- --ignored`), starting with cycle-accurate memory timing.
2. MBC3 and MBC5, plus battery saves.
3. Audio.
4. Save states, which make rewind and reverse stepping possible.
5. Disassembler and breakpoints, extending the existing debugger.

## Tier 0: Measurable accuracy

Mature emulators prove correctness with community test ROMs. They are the cheapest way to find CPU, timer and PPU bugs.

- [x] **Headless test-ROM runner** (`cash_gb::headless`): runs a ROM with no window, reads its serial output, and compares screenshots. GitHub Actions runs `cargo fmt --check`, `cargo clippy` and `cargo test` on every push and pull request (`.github/workflows/ci.yml`).
  - [x] Serial port output (`0xFF01`/`0xFF02`), which Blargg's tests use to report results
  - [x] Blargg `cpu_instrs` and `instr_timing` (`tests/blargg.rs`)
  - [x] dmg-acid2: one reference screenshot that covers most PPU rendering rules (`tests/dmg_acid2.rs`, passes)
  - [x] Mooneye test suite: timing, interrupt and MBC edge cases (`tests/mooneye.rs`). 36 of 79 pass; the rest are `#[ignore]`d with a reason.
- [ ] **Pass the ignored Mooneye tests.** Remove each `#[ignore]` as its test starts passing:
  - [ ] 21 CPU timing tests (`call_timing`, `push_timing`, `intr_timing`, `oam_dma_timing`, ...): need cycle-accurate memory timing, below
  - [ ] 11 PPU timing tests (`ppu/*`): need accurate mode timing and variable-length mode 3, below
  - [ ] 3 timer reload tests (`tima_reload`, `tima_write_reloading`, `tma_write_reloading`): the TMA reload delay, below
  - [ ] 3 post-boot state tests (`boot_div`, `boot_hwio`, `serial/boot_sclk_align`): DIV, I/O registers and the serial clock should start as the boot ROM leaves them
  - [ ] `bits/unused_hwio`: unused bits of I/O registers should read back as 1
  - [ ] `oam_dma_restart` and `oam_dma/sources`: OAM DMA restarting mid-transfer, and DMA from every source region
  - [ ] `mbc1/ram_64kb`: MBC1 with 64 KiB of cart RAM
  - [ ] `mbc1/multicart_rom_8Mb`: detect MBC1 multicarts (MBC1M), which wire the bank bits differently
- [ ] **Timer accuracy:**
  - [x] Writing DIV resets the internal counter that TIMA is derived from
  - [ ] TIMA reloads from TMA one cycle after it overflows.
- [ ] **HALT bug:** when HALT runs with IME off and an interrupt pending, the next opcode byte is read twice.
- [ ] **Cycle-accurate memory timing:** memory reads and writes happen on the M-cycle they occur in, rather than when the instruction finishes.
- [ ] **Variable-length PPU mode 3**, depending on SCX, the window and sprites.
- [ ] **Boot ROM support:** run the real DMG boot ROM, which scrolls the Nintendo logo in. `dmg_boot.bin` is currently an empty file.

## Tier 1: Playing the games people own

- [ ] **MBC3**, including the real-time clock (Pokémon Red/Blue/Gold/Silver and others).
- [ ] **MBC5** (most late Game Boy games and nearly all Game Boy Color games).
- [ ] **Battery saves:** write cart RAM to a `.sav` file next to the ROM and load it on start. `InternalMemoryBankController::ram()` is kept for this.
- [ ] **Audio (APU):** the four sound channels, mixed and played through `cpal`.
- [ ] **Gamepad support** with `gilrs`.
- [ ] **Key rebinding:** bindings are currently fixed in `ui/debug.rs` (`KEY_BINDINGS`).

## Tier 2: Features players expect

- [ ] **Save states:** save and restore the whole machine, with multiple slots.
- [ ] **Fast-forward and slow motion.**
- [ ] **Rewind:** a ring buffer of save states taken every few frames.
- [ ] **Display options:**
  - [ ] Colour palettes: DMG green, grey, Game Boy Pocket, or user-defined
  - [ ] LCD ghosting or pixel-grid effect
  - [ ] Integer and fractional scaling
  - [ ] Fullscreen
- [ ] **Convenience:**
  - [ ] Recent ROMs list
  - [ ] Drag-and-drop a ROM onto the window
  - [ ] Load ROMs directly from `.zip` files
  - [ ] Remember settings per game (eframe's `persistence` feature)
- [ ] **Capture:** screenshots, and GIF or video recording.
- [ ] **Cheats:** Game Genie and GameShark codes.

## Tier 3: A debugger that stands out

BGB and Emulicious are the debugger-first Game Boy emulators most homebrew developers use, and neither is open source. A modern, open, cross-platform debugger fills a clear gap.

- [ ] **Disassembly view** around PC.
  - [ ] Symbol names loaded from RGBDS `.sym` files
- [ ] **Stepping:** step into, over and out, with a call stack.
- [ ] **Breakpoints:**
  - [ ] Execute, read and write breakpoints
  - [ ] Conditional breakpoints (for example `A == 3`)
  - [ ] Watchpoints on memory ranges
- [ ] **VRAM viewer:** the tile set, both tile maps with the visible scroll area drawn on top, the 40 sprites in OAM, and the palettes.
- [ ] **Memory editing** in the hex viewer, which is currently read-only.
- [ ] **Profiling:**
  - [ ] Execution trace log
  - [ ] Cycle counter between two points
  - [ ] Event timeline per scanline: interrupts, mode changes and register writes
- [ ] **Reverse stepping:** step backwards through instructions using the rewind buffer. Few Game Boy emulators offer this.

## Tier 4: Bigger features for later

- [ ] **Game Boy Color:** double-speed mode, VRAM and WRAM banking, colour palettes, and HDMA.
- [ ] **Super Game Boy:** borders and colour palettes.
- [ ] **Link cable:**
  - [ ] Two emulator instances in one process
  - [ ] Over a network, for trading and battles
- [ ] **Web version:** eframe can compile to WebAssembly, so the emulator could run in a browser with nothing to install.
- [ ] **libretro core**, so cash-gb runs inside RetroArch.
