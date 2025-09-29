# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Game Boy emulator written in Rust called `cash-gb`. The project implements a basic Game Boy CPU with instruction parsing, cartridge reading, and display processing (PPU).

## Commands

### Build and Run
- `cargo build` - Build the project
- `cargo run <rom_file>` - Run emulator with a ROM file
- `cargo test` - Run tests
- `cargo check` - Check code without building
- `cargo clippy` - Run linter

### Example Usage
```bash
cargo run /path/to/game.gb
```

## Architecture

The codebase is organized into several key modules:

### Core Components

- **`lib.rs`** - Main library entry point with `read_cart()` function for loading ROM files
- **`main.rs`** - CLI entry point that loads a cart and runs the CPU for a fixed number of steps
- **`cpu/`** - Modular CPU implementation:
  - **`mod.rs`** - Main CPU struct, coordination, and memory management
  - **`instructions.rs`** - All instruction enums and decoding logic
  - **`execute.rs`** - Complete instruction execution implementations
  - **`registers.rs`** - CPU register management (AF, BC, DE, HL pairs with individual byte access)
- **`memory/`** - Memory management system:
  - **`mod.rs`** - Memory bus that routes all memory access with integrated PPU
  - **`cart.rs`** - Cartridge/ROM loading and header parsing
  - **`mbc.rs`** - Memory Bank Controllers (NoMBC, MBC1 implemented)
- **`ppu/`** - Picture Processing Unit (graphics system):
  - **`mod.rs`** - Main PPU coordination with LCD timing state machine
  - **`registers.rs`** - PPU control and status registers (LCDC, STAT, palettes, etc.)
  - **`timing.rs`** - LCD mode timing and memory access permissions
  - **`background.rs`** - Background and window tile rendering
  - **`sprites.rs`** - Sprite (OAM) rendering with priority handling
  - **`display.rs`** - Display output abstraction and color conversion
- **`bus.rs`** - Legacy file (functionality moved to memory module)

### Memory Architecture

The Memory Bus routes all memory access through a centralized system with integrated PPU:
- **ROM banks (0x0000-0x7FFF)** - Handled by Memory Bank Controllers (MBC)
- **VRAM banks (0x8000-0x9FFF)** - Routed through PPU with timing restrictions
- **Cartridge RAM (0xA000-0xBFFF)** - External RAM via MBC
- **Work RAM (0xC000-0xDFFF)** - Internal RAM with banking
- **OAM (0xFE00-0xFE9F)** - Routed through PPU with timing restrictions
- **I/O registers (0xFF00-0xFF7F)** - Hardware control registers (PPU registers 0xFF40-0xFF4B routed to PPU)
- **High RAM (0xFF80-0xFFFE)** - Fast internal RAM

**PPU Integration:**
- PPU is integrated into the MemoryBus and steps with CPU timing
- Memory access restrictions enforced based on LCD modes:
  - Drawing mode blocks VRAM access
  - OAM Scan and Drawing modes block OAM access
- PPU generates VBlank and LCD STAT interrupts handled by CPU

### Instruction System

The CPU uses a comprehensive enum-based instruction system with:
- Detailed instruction parsing (`get_instruction()` and `get_cb_instruction()`)
- Cycle-accurate timing
- Complete flag handling (Z, N, H, C)
- All Game Boy instruction types (load, arithmetic, bitwise, jumps, etc.)

### Current Implementation Status

- ✅ CPU core with all instructions implemented
- ✅ Memory Bus architecture with centralized routing and PPU integration
- ✅ Memory Bank Controllers (NoMBC, MBC1 implemented)
- ✅ Register management
- ✅ PPU with complete graphics pipeline:
  - ✅ LCD timing state machine (OAM Scan, Drawing, HBlank, VBlank)
  - ✅ Background and window tile rendering
  - ✅ Sprite rendering with priorities and transparency
  - ✅ Memory access restrictions based on LCD modes
  - ✅ Frame buffer management (160×144 RGBA output)
  - ✅ Display output abstraction
- ✅ Interrupt handling system (VBlank, LCD STAT, Timer, Serial, Joypad)
- ❌ Advanced MBC support (MBC3, MBC5, etc.)
- ❌ Audio processing
- ❌ Input handling
- ❌ Actual background/sprite rendering implementation (currently test patterns)

### Key Implementation Details

- The project uses extensive pattern matching for instruction decoding
- Memory access is handled through `read()` and `write()` methods on the CPU, routed through MemoryBus
- PPU is fully integrated with CPU timing and memory bus:
  - CPU steps PPU by 4 cycles per CPU step
  - PPU enforces memory access restrictions during Drawing and OAM Scan modes
  - PPU generates interrupts that are handled by CPU interrupt system
- The PPU implements Game Boy's 4-mode LCD timing (OAM Scan, Drawing, HBlank, VBlank)
- Cartridge validation includes header checksum and Nintendo logo verification
- Comprehensive test coverage (49 tests) including CPU, PPU, and integration tests

### Development Notes

- The codebase supports NoMBC and MBC1 cartridge types; advanced types (MBC3, MBC5, etc.) need implementation
- The emulator runs for a fixed number of steps rather than implementing proper frame timing
- Debug output is enabled throughout the CPU execution for development purposes
- PPU currently generates test patterns; actual tile/sprite rendering needs background and sprite renderer integration

## Code Quality Standards

### Required Practices

- **All code must pass `cargo clippy` with zero warnings** - Run clippy before committing any changes
- **Comprehensive testing** - All new features must include unit tests and integration tests where applicable
- **Documentation** - All public APIs should be documented with doc comments
- **Error handling** - Use proper Result types and avoid panics in production code paths

### Architecture Principles

- **Memory safety** - Use Rust's ownership system correctly, avoid unnecessary clones
- **Modularity** - Keep components loosely coupled with clear interfaces
- **Testing** - Maintain high test coverage, especially for core emulation logic
- **Performance** - Code should be efficient but prioritize correctness and maintainability

### Workflow

1. Run `cargo check` to verify compilation
2. Run `cargo test` to ensure all tests pass
3. Run `cargo clippy` to check code quality
4. Only commit code that passes all three checks