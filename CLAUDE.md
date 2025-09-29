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
  - **`mod.rs`** - Memory bus that routes all memory access
  - **`cart.rs`** - Cartridge/ROM loading and header parsing
  - **`mbc.rs`** - Memory Bank Controllers (NoMBC, MBC1 implemented)
- **`bus.rs`** - Legacy file (functionality moved to memory module)

### Memory Architecture

The Memory Bus routes all memory access through a centralized system:
- **ROM banks (0x0000-0x7FFF)** - Handled by Memory Bank Controllers (MBC)
- **VRAM banks (0x8000-0x9FFF)** - Video RAM with banking support
- **Cartridge RAM (0xA000-0xBFFF)** - External RAM via MBC
- **Work RAM (0xC000-0xDFFF)** - Internal RAM with banking
- **OAM (0xFE00-0xFE9F)** - Object Attribute Memory for sprites
- **I/O registers (0xFF00-0xFF7F)** - Hardware control registers
- **High RAM (0xFF80-0xFFFE)** - Fast internal RAM

### Instruction System

The CPU uses a comprehensive enum-based instruction system with:
- Detailed instruction parsing (`get_instruction()` and `get_cb_instruction()`)
- Cycle-accurate timing
- Complete flag handling (Z, N, H, C)
- All Game Boy instruction types (load, arithmetic, bitwise, jumps, etc.)

### Current Implementation Status

- ✅ CPU core with all instructions implemented
- ✅ Memory Bus architecture with centralized routing
- ✅ Memory Bank Controllers (NoMBC, MBC1 implemented)
- ✅ Register management
- ❌ PPU with basic scanline rendering
- ❌ Advanced MBC support (MBC3, MBC5, etc.)
- ❌ Audio processing
- ❌ Input handling
- ❌ Interrupt handling system

### Key Implementation Details

- The project uses extensive pattern matching for instruction decoding
- Memory access is handled through `read()` and `write()` methods on the CPU
- The PPU implements Game Boy's 4-mode LCD timing (OAM Search, Pixel Transfer, HBlank, VBlank)
- Cartridge validation includes header checksum and Nintendo logo verification

### Development Notes

- The codebase currently only supports basic ROM cartridges without memory bank controllers
- Most advanced cartridge types (MBC1-7, etc.) return `UnsupportedMapper` errors
- The emulator runs for a fixed number of steps rather than implementing proper frame timing
- Debug output is enabled throughout the CPU execution for development purposes