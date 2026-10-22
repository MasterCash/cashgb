//! CPU and I/O register inspector.

use egui::{Color32, RichText};

use crate::{cpu::Cpu, memory::joypad::Button};

/// Keyboard keys mapped to Game Boy buttons, shown in the Joypad section
pub const KEY_BINDINGS: [(egui::Key, Button); 8] = [
    (egui::Key::ArrowRight, Button::Right),
    (egui::Key::ArrowLeft, Button::Left),
    (egui::Key::ArrowUp, Button::Up),
    (egui::Key::ArrowDown, Button::Down),
    (egui::Key::X, Button::A),
    (egui::Key::Z, Button::B),
    (egui::Key::Backspace, Button::Select),
    (egui::Key::Enter, Button::Start),
];

/// Interrupt sources in IE/IF bit order
const INTERRUPTS: [&str; 5] = ["VBlank", "LCD STAT", "Timer", "Serial", "Joypad"];

/// Show every register section. All values are read through `Cpu::read_memory`, which has
/// no side effects, so inspecting never changes emulator state.
pub fn show(ui: &mut egui::Ui, cpu: &Cpu) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            section(ui, "CPU", |ui| cpu_section(ui, cpu));
            section(ui, "Interrupts", |ui| interrupt_section(ui, cpu));
            section(ui, "PPU", |ui| ppu_section(ui, cpu));
            section(ui, "Timer", |ui| timer_section(ui, cpu));
            section(ui, "Joypad", |ui| joypad_section(ui, cpu));
            section(ui, "Cartridge", |ui| cartridge_section(ui, cpu));
        });
}

/// A collapsible section holding a two-column name/value grid.
fn section(ui: &mut egui::Ui, title: &str, add_rows: impl FnOnce(&mut egui::Ui)) {
    egui::CollapsingHeader::new(title)
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new(title)
                .num_columns(2)
                .striped(true)
                .spacing([16.0, 2.0])
                .show(ui, add_rows);
        });
}

fn row(ui: &mut egui::Ui, name: &str, value: impl Into<RichText>) {
    ui.label(name);
    ui.label(value.into().monospace());
    ui.end_row();
}

/// A byte as hex followed by its bits, for bit-field registers
fn hex_and_bits(value: u8) -> String {
    format!("{value:02X}  {value:08b}")
}

/// Highlighted when set, dimmed when clear
fn bit_text(label: &str, set: bool) -> RichText {
    let text = RichText::new(label).monospace();
    if set {
        text.color(Color32::LIGHT_GREEN).strong()
    } else {
        text.weak()
    }
}

fn cpu_section(ui: &mut egui::Ui, cpu: &Cpu) {
    for (name, value) in [
        ("AF", cpu.get_af()),
        ("BC", cpu.get_bc()),
        ("DE", cpu.get_de()),
        ("HL", cpu.get_hl()),
        ("SP", cpu.get_sp()),
        ("PC", cpu.get_pc()),
    ] {
        row(ui, name, format!("{value:04X}"));
    }

    let (z, n, h, c) = cpu.get_flags();
    ui.label("Flags");
    ui.horizontal(|ui| {
        for (label, set) in [("Z", z), ("N", n), ("H", h), ("C", c)] {
            ui.label(bit_text(label, set));
        }
    });
    ui.end_row();

    row(ui, "IME", if cpu.ime { "enabled" } else { "disabled" });
    row(ui, "Status", format!("{:?}", cpu.status));
    row(ui, "Instruction", cpu.get_current_instruction().to_string());
    row(ui, "Steps left", cpu.step_count.to_string());
}

fn interrupt_section(ui: &mut egui::Ui, cpu: &Cpu) {
    let enabled = cpu.read_memory(0xFFFF);
    let requested = cpu.read_memory(0xFF0F);
    row(ui, "IE (FFFF)", hex_and_bits(enabled));
    row(ui, "IF (FF0F)", hex_and_bits(requested));

    for (bit, name) in INTERRUPTS.iter().enumerate() {
        let mask = 1 << bit;
        ui.label(*name);
        ui.horizontal(|ui| {
            ui.label(bit_text("enabled", enabled & mask != 0));
            ui.label(bit_text("requested", requested & mask != 0));
        });
        ui.end_row();
    }
}

fn ppu_section(ui: &mut egui::Ui, cpu: &Cpu) {
    let ppu = cpu.memory.get_ppu();
    row(ui, "Mode", format!("{:?}", ppu.get_current_mode()));
    for (name, addr) in [("LCDC (FF40)", 0xFF40), ("STAT (FF41)", 0xFF41)] {
        row(ui, name, hex_and_bits(cpu.read_memory(addr)));
    }
    for (name, addr) in [
        ("LY", 0xFF44),
        ("LYC", 0xFF45),
        ("SCY", 0xFF42),
        ("SCX", 0xFF43),
        ("WY", 0xFF4A),
        ("WX", 0xFF4B),
    ] {
        let value = cpu.read_memory(addr);
        row(ui, name, format!("{value:02X}  ({value})"));
    }
    for (name, addr) in [("BGP", 0xFF47), ("OBP0", 0xFF48), ("OBP1", 0xFF49)] {
        row(ui, name, hex_and_bits(cpu.read_memory(addr)));
    }
    row(ui, "DMA (FF46)", format!("{:02X}", cpu.read_memory(0xFF46)));
}

fn timer_section(ui: &mut egui::Ui, cpu: &Cpu) {
    for (name, addr) in [
        ("DIV (FF04)", 0xFF04),
        ("TIMA (FF05)", 0xFF05),
        ("TMA (FF06)", 0xFF06),
    ] {
        row(ui, name, format!("{:02X}", cpu.read_memory(addr)));
    }
    row(ui, "TAC (FF07)", hex_and_bits(cpu.read_memory(0xFF07)));
}

fn joypad_section(ui: &mut egui::Ui, cpu: &Cpu) {
    row(ui, "P1 (FF00)", hex_and_bits(cpu.read_memory(0xFF00)));
    for (key, button) in KEY_BINDINGS {
        ui.label(format!("{button:?}"));
        ui.horizontal(|ui| {
            ui.label(bit_text("held", cpu.memory.joypad.is_pressed(button)));
            ui.label(RichText::new(format!("key: {}", key.name())).weak());
        });
        ui.end_row();
    }
}

fn cartridge_section(ui: &mut egui::Ui, cpu: &Cpu) {
    let cart = &cpu.memory.cart;
    row(ui, "Title", cart.title());
    row(ui, "Mapper", format!("{:?}", cart.mbc.get_mapper_type()));
    row(ui, "ROM size", format!("{} KiB", cart.rom_size() / 1024));
    row(ui, "RAM size", format!("{} KiB", cart.ram_size() / 1024));
    if let Some(offset) = cart.mbc.rom_offset(0x4000) {
        row(ui, "ROM bank at 4000", format!("{}", offset / 0x4000));
    }
}
