//! Emulator window: the screen, run controls, keyboard input, and the debugger panels.

mod debug;
mod memory_view;

use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use egui::{Color32, ColorImage, TextureHandle, TextureOptions, vec2};

use crate::{
    cpu::{Cpu, instructions::CpuStatus},
    memory::{cart::Cart, joypad::Button},
    read_cart,
};
use memory_view::MemoryView;

const GB_SCREEN_WIDTH: usize = 160;
const GB_SCREEN_HEIGHT: usize = 144;

/// One Game Boy frame: 70224 dots at 4.194304 MHz (about 59.73 frames per second)
const FRAME_DURATION: Duration = Duration::from_nanos(16_742_706);

/// Most emulated frames per UI update, so a slow UI drops frames instead of falling further behind
const MAX_FRAMES_PER_UPDATE: u32 = 3;

const WINDOW_TITLE: &str = "cash-gb";

pub struct App {
    cpu: Cpu,
    /// File the current ROM was loaded from, or `None` before a ROM is loaded
    rom_path: Option<PathBuf>,
    /// Message for the "couldn't load ROM" dialog
    load_error: Option<String>,
    /// Window title to apply on the next frame (loading happens outside the UI pass)
    pending_title: Option<String>,
    /// Whether the emulator is running continuously (as opposed to single-stepping)
    running: bool,
    /// GPU texture holding the last presented frame
    screen: Option<TextureHandle>,
    last_update: Instant,
    /// Emulated time owed to real time while running
    frame_debt: Duration,
    memory_view: MemoryView,
    show_registers: bool,
    show_memory: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// An emulator window with no ROM loaded yet.
    pub fn new() -> Self {
        Self {
            cpu: Cpu::empty(),
            rom_path: None,
            load_error: None,
            pending_title: None,
            running: false,
            screen: None,
            last_update: Instant::now(),
            frame_debt: Duration::ZERO,
            memory_view: MemoryView::new(),
            show_registers: true,
            show_memory: true,
        }
    }

    /// Replace the running game with `cart` and start it.
    pub fn load_cart(&mut self, path: PathBuf, cart: Cart) {
        let title = cart.title().trim().to_string();
        self.pending_title = Some(if title.is_empty() {
            WINDOW_TITLE.to_string()
        } else {
            format!("{WINDOW_TITLE} - {title}")
        });
        self.cpu = Cpu::new(cart);
        self.rom_path = Some(path);
        self.screen = None; // back to a blank screen until the new game draws a frame
        self.running = true;
        self.frame_debt = Duration::ZERO;
    }

    /// Load a ROM file, showing an error dialog if it can't be loaded.
    fn load_from_path(&mut self, path: PathBuf) {
        match read_cart(&path) {
            Ok(cart) => self.load_cart(path, cart),
            Err(error) => self.load_error = Some(format!("{}\n\n{error}", path.display())),
        }
    }

    /// Ask for a ROM file with the native file dialog. Blocks the UI while the dialog is open.
    fn open_rom_dialog(&mut self) {
        let mut dialog = rfd::FileDialog::new()
            .set_title("Open Game Boy ROM")
            .add_filter("Game Boy ROM", &["gb", "gbc"])
            .add_filter("All files", &["*"]);
        if let Some(folder) = self.rom_path.as_ref().and_then(|path| path.parent()) {
            dialog = dialog.set_directory(folder);
        }
        if let Some(path) = dialog.pick_file() {
            self.load_from_path(path);
        }
    }

    /// Run one frame. A STOPped CPU keeps "running" so a button press can wake it; an error
    /// pauses.
    fn run_frame(&mut self) {
        self.cpu.run_frame();
        if self.cpu.status == CpuStatus::Errored {
            self.running = false;
        }
    }

    /// Advance emulation to keep pace with real time.
    fn run_for_elapsed_time(&mut self, elapsed: Duration) {
        self.frame_debt += elapsed;
        let mut frames = 0;
        while self.running && self.frame_debt >= FRAME_DURATION && frames < MAX_FRAMES_PER_UPDATE {
            self.run_frame();
            self.frame_debt -= FRAME_DURATION;
            frames += 1;
        }
        // Too far behind to catch up: drop the backlog rather than spiral
        if self.frame_debt >= FRAME_DURATION {
            self.frame_debt = Duration::ZERO;
        }
    }

    /// Copy the PPU framebuffer into the screen texture.
    fn present_frame(&mut self, ctx: &egui::Context) {
        let image = ColorImage::from_rgba_unmultiplied(
            [GB_SCREEN_WIDTH, GB_SCREEN_HEIGHT],
            self.cpu.get_framebuffer(),
        );
        match &mut self.screen {
            Some(texture) => texture.set(image, TextureOptions::NEAREST),
            None => self.screen = Some(ctx.load_texture("screen", image, TextureOptions::NEAREST)),
        }
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open ROM…").clicked() {
                    ui.close();
                    self.open_rom_dialog();
                }
                if ui
                    .add_enabled(self.rom_path.is_some(), egui::Button::new("Reload ROM"))
                    .clicked()
                    && let Some(path) = self.rom_path.clone()
                {
                    self.load_from_path(path);
                }
                ui.separator();
                if ui.button("Quit").clicked() {
                    ui.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button("View", |ui| {
                ui.checkbox(&mut self.show_registers, "Registers");
                ui.checkbox(&mut self.show_memory, "Memory");
                ui.separator();
                ui.menu_button("Theme", |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            });
        });
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        let loaded = self.rom_path.is_some();
        ui.horizontal(|ui| {
            let label = if self.running { "Pause" } else { "Run" };
            if toolbar_button(ui, loaded, label) {
                self.running = !self.running;
                self.frame_debt = Duration::ZERO;
            }

            // Stepping only makes sense while paused. Each step shows the partly drawn frame.
            let can_step = loaded && !self.running;
            let mut stepped = false;
            if toolbar_button(ui, can_step, "Step cycle") {
                self.cpu.step();
                stepped = true;
            }
            if toolbar_button(ui, can_step, "Step instruction") {
                self.cpu.step_instruction();
                stepped = true;
            }
            if toolbar_button(ui, can_step, "Step frame") {
                self.run_frame();
                stepped = true;
            }
            if stepped {
                self.present_frame(ui.ctx());
            }

            if loaded {
                ui.separator();
                ui.label(format!("{:?}", self.cpu.status));
                ui.label(egui::RichText::new(format!("PC {:04X}", self.cpu.get_pc())).monospace());
            }
        });
    }

    fn screen_panel(&mut self, ui: &mut egui::Ui) {
        if self.rom_path.is_none() {
            ui.centered_and_justified(|ui| ui.label("No ROM loaded. Use File → Open ROM…"));
            return;
        }
        if self.screen.is_none() {
            let blank = ColorImage::filled([GB_SCREEN_WIDTH, GB_SCREEN_HEIGHT], Color32::WHITE);
            self.screen = Some(
                ui.ctx()
                    .load_texture("screen", blank, TextureOptions::NEAREST),
            );
        }
        let Some(texture) = &self.screen else {
            return;
        };

        // Largest whole-number scale that fits, so every Game Boy pixel is the same size
        let available = ui.available_size();
        let scale = (available.x / GB_SCREEN_WIDTH as f32)
            .min(available.y / GB_SCREEN_HEIGHT as f32)
            .floor()
            .max(1.0);
        let size = vec2(GB_SCREEN_WIDTH as f32, GB_SCREEN_HEIGHT as f32) * scale;
        ui.centered_and_justified(|ui| ui.image((texture.id(), size)));
    }

    fn load_error_dialog(&mut self, ctx: &egui::Context) {
        let Some(message) = &self.load_error else {
            return;
        };
        let mut dismissed = false;
        egui::Window::new("Couldn't load ROM")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(message);
                dismissed = ui.button("OK").clicked();
            });
        if dismissed {
            self.load_error = None;
        }
    }
}

/// A toolbar button that gives up keyboard focus after a click, so Enter and the arrow keys
/// keep going to the Game Boy instead of re-activating the button.
fn toolbar_button(ui: &mut egui::Ui, enabled: bool, label: &str) -> bool {
    let response = ui.add_enabled(enabled, egui::Button::new(label));
    if response.clicked() {
        response.surrender_focus();
        true
    } else {
        false
    }
}

fn button_for_key(key: egui::Key) -> Option<Button> {
    debug::KEY_BINDINGS
        .iter()
        .find(|(bound, _)| *bound == key)
        .map(|(_, button)| *button)
}

/// Apply game-key presses and releases to the joypad, and remove those key events so egui
/// never sees them.
///
/// Every key event makes egui repaint immediately, and on Windows a held key repeats about 30
/// times a second. Letting those repeats through made each one cost two full frames, which
/// starved the window of mouse input while paused. Removing them also stops game keys from
/// moving widget focus (arrows) or activating buttons (Enter).
///
/// While a text box has focus the keys are left for it, but releases still reach the joypad
/// so no button gets stuck down.
fn take_game_keys(cpu: &mut Cpu, typing: bool, events: &mut Vec<egui::Event>) {
    events.retain(|event| match event {
        egui::Event::Key {
            key,
            pressed,
            repeat,
            ..
        } => {
            let Some(button) = button_for_key(*key) else {
                return true;
            };
            if !pressed {
                cpu.set_button(button, false);
            } else if !typing && !repeat {
                cpu.set_button(button, true);
            }
            typing
        }
        // Keys released while the window is in the background never arrive; release them all
        egui::Event::WindowFocused(false) => {
            for button in Button::ALL {
                cpu.set_button(button, false);
            }
            true
        }
        _ => true,
    });
}

impl eframe::App for App {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        take_game_keys(
            &mut self.cpu,
            ctx.text_edit_focused(),
            &mut raw_input.events,
        );
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let elapsed = now - self.last_update;
        self.last_update = now;

        if let Some(title) = self.pending_title.take() {
            ui.send_viewport_cmd(egui::ViewportCommand::Title(title));
        }
        if self.running {
            self.run_for_elapsed_time(elapsed);
            // Wake up again when the next frame is due
            ui.ctx()
                .request_repaint_after(FRAME_DURATION.saturating_sub(self.frame_debt));
        }
        if self.cpu.is_frame_ready() {
            self.present_frame(ui.ctx());
        }

        egui::Panel::top("top_panel").show(ui, |ui| {
            self.menu_bar(ui);
            self.toolbar(ui);
        });

        if self.show_registers {
            egui::Panel::left("registers")
                .resizable(true)
                .default_size(260.0)
                .min_size(200.0)
                .show(ui, |ui| debug::show(ui, &self.cpu));
        }
        if self.show_memory {
            egui::Panel::right("memory")
                .resizable(true)
                .default_size(460.0)
                .min_size(240.0)
                .show(ui, |ui| self.memory_view.show(ui, &self.cpu));
        }
        egui::CentralPanel::default().show(ui, |ui| self.screen_panel(ui));
        self.load_error_dialog(ui.ctx());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::cart::cart_header::NINTENDO_LOGO;

    /// `JR -2`: spin in place at the entry point, like a game idling between frames
    const IDLE_LOOP: [u8; 2] = [0x18, 0xFE];

    /// A 32KB ROM-only image with `code` at the 0x0100 entry point.
    /// `code` must not run into the header at 0x0104, which isn't valid code.
    fn rom_with_code(code: &[u8]) -> Vec<u8> {
        let mut rom = vec![0u8; 0x8000];
        rom[0x100..0x100 + code.len()].copy_from_slice(code);
        rom[0x104..0x104 + NINTENDO_LOGO.len()].copy_from_slice(&NINTENDO_LOGO);
        rom[0x134..0x138].copy_from_slice(b"TEST");
        let mut checksum = 0u8;
        for byte in &rom[0x134..=0x14C] {
            checksum = checksum.wrapping_sub(*byte).wrapping_sub(1);
        }
        rom[0x14D] = checksum;
        rom
    }

    fn app_with_code(code: &[u8]) -> App {
        let mut app = App::new();
        app.load_cart("test.gb".into(), Cart::new(&rom_with_code(code)).unwrap());
        app.running = false;
        app
    }

    fn key(key: egui::Key, pressed: bool, repeat: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat,
            modifiers: egui::Modifiers::NONE,
        }
    }

    /// Run one headless egui frame and return the repaint delay egui asks for afterwards.
    fn run_ui(
        ctx: &egui::Context,
        input: egui::RawInput,
        add: impl FnMut(&mut egui::Ui),
    ) -> Duration {
        let mut output = ctx.run_ui(input, add);
        output.textures_delta.clear(); // no renderer here to upload the font atlas to
        output.viewport_output[&egui::ViewportId::ROOT].repaint_delay
    }

    // --- Run loop ----------------------------------------------------------------------

    #[test]
    fn run_frame_stops_at_a_completed_frame() {
        let mut app = app_with_code(&IDLE_LOOP);
        app.running = true;

        app.run_frame();

        assert!(app.cpu.is_frame_ready());
        assert!(app.running);
    }

    #[test]
    fn run_for_elapsed_time_keeps_the_remainder() {
        let mut app = app_with_code(&IDLE_LOOP);
        app.running = true;

        app.run_for_elapsed_time(FRAME_DURATION + FRAME_DURATION / 2);

        assert_eq!(app.frame_debt, FRAME_DURATION / 2);
    }

    #[test]
    fn run_for_elapsed_time_drops_a_large_backlog() {
        let mut app = app_with_code(&IDLE_LOOP);
        app.running = true;

        app.run_for_elapsed_time(Duration::from_secs(1));

        assert_eq!(app.frame_debt, Duration::ZERO);
    }

    #[test]
    fn run_frame_keeps_running_while_the_cpu_is_stopped() {
        let mut app = app_with_code(&[0x10, 0x00, 0x18, 0xFE]); // STOP; JR -2
        app.running = true;

        app.run_frame();

        assert_eq!(app.cpu.status, CpuStatus::Stopped);
        assert!(app.running, "STOP waits for a button instead of pausing");
    }

    #[test]
    fn button_press_resumes_a_stopped_cpu() {
        let mut app = app_with_code(&[0x10, 0x00, 0x18, 0xFE]); // STOP; JR -2
        app.running = true;
        app.run_frame();

        app.cpu.set_button(Button::Start, true);
        app.run_frame();

        assert_eq!(app.cpu.status, CpuStatus::Running);
        assert!(app.cpu.is_frame_ready());
    }

    #[test]
    fn run_frame_pauses_on_error() {
        let mut app = app_with_code(&[0xD3]); // illegal opcode
        app.running = true;

        app.run_frame();

        assert_eq!(app.cpu.status, CpuStatus::Errored);
        assert!(!app.running);
    }

    // --- Rendering ---------------------------------------------------------------------

    #[test]
    fn present_frame_creates_the_screen_texture() {
        let mut app = app_with_code(&IDLE_LOOP);
        let ctx = egui::Context::default();

        app.present_frame(&ctx);

        let texture = app.screen.as_ref().expect("texture created");
        assert_eq!(texture.size(), [GB_SCREEN_WIDTH, GB_SCREEN_HEIGHT]);
    }

    #[test]
    fn debugger_panels_render_every_region() {
        let mut app = app_with_code(&IDLE_LOOP);
        app.run_frame();
        let ctx = egui::Context::default();

        for region in memory_view::MemoryRegion::ALL {
            app.memory_view = MemoryView::with_region(region);
            run_ui(&ctx, egui::RawInput::default(), |ui| {
                debug::show(ui, &app.cpu);
                app.memory_view.show(ui, &app.cpu);
            });
        }
    }

    #[test]
    fn whole_window_renders_without_a_rom() {
        let mut app = App::new();
        let ctx = egui::Context::default();

        run_ui(&ctx, egui::RawInput::default(), |ui| {
            app.menu_bar(ui);
            app.toolbar(ui);
            debug::show(ui, &app.cpu);
            app.memory_view.show(ui, &app.cpu);
            app.screen_panel(ui);
        });
    }

    // --- Keyboard input ----------------------------------------------------------------

    #[test]
    fn game_key_press_and_release_reach_the_joypad() {
        let mut cpu = app_with_code(&IDLE_LOOP).cpu;

        let mut events = vec![key(egui::Key::X, true, false)];
        take_game_keys(&mut cpu, false, &mut events);
        assert!(cpu.memory.joypad.is_pressed(Button::A));
        assert!(!cpu.memory.joypad.is_pressed(Button::B));

        let mut events = vec![key(egui::Key::X, false, false)];
        take_game_keys(&mut cpu, false, &mut events);
        assert!(!cpu.memory.joypad.is_pressed(Button::A));
    }

    #[test]
    fn game_keys_and_their_repeats_are_hidden_from_egui() {
        let mut cpu = app_with_code(&IDLE_LOOP).cpu;
        let mut events = vec![
            key(egui::Key::ArrowUp, true, false),
            key(egui::Key::ArrowUp, true, true),
            key(egui::Key::Enter, true, false),
        ];

        take_game_keys(&mut cpu, false, &mut events);

        assert!(events.is_empty());
        assert!(cpu.memory.joypad.is_pressed(Button::Up));
        assert!(cpu.memory.joypad.is_pressed(Button::Start));
    }

    #[test]
    fn other_keys_are_left_for_egui() {
        let mut cpu = app_with_code(&IDLE_LOOP).cpu;
        let mut events = vec![
            key(egui::Key::A, true, false),
            key(egui::Key::Tab, true, false),
        ];

        take_game_keys(&mut cpu, false, &mut events);

        assert_eq!(events.len(), 2);
    }

    #[test]
    fn typing_keeps_keys_for_the_text_box_but_still_releases() {
        let mut cpu = app_with_code(&IDLE_LOOP).cpu;
        cpu.set_button(Button::Select, true);
        let mut events = vec![
            key(egui::Key::ArrowLeft, true, false),
            key(egui::Key::Backspace, false, false),
        ];

        take_game_keys(&mut cpu, true, &mut events);

        assert_eq!(events.len(), 2, "the text box still gets its keys");
        assert!(
            !cpu.memory.joypad.is_pressed(Button::Left),
            "no press while typing"
        );
        assert!(
            !cpu.memory.joypad.is_pressed(Button::Select),
            "release still applies"
        );
    }

    #[test]
    fn losing_window_focus_releases_every_button() {
        let mut cpu = app_with_code(&IDLE_LOOP).cpu;
        cpu.set_button(Button::A, true);
        cpu.set_button(Button::Down, true);
        let mut events = vec![egui::Event::WindowFocused(false)];

        take_game_keys(&mut cpu, false, &mut events);

        assert!(
            Button::ALL
                .iter()
                .all(|b| !cpu.memory.joypad.is_pressed(*b))
        );
    }

    /// Regression test for the paused-window freeze: a held game key must not make egui ask
    /// for an immediate extra repaint on every key repeat.
    #[test]
    fn held_game_key_does_not_request_immediate_repaints() {
        let mut app = app_with_code(&IDLE_LOOP);
        let ctx = egui::Context::default();
        let repeat = || egui::RawInput {
            events: vec![key(egui::Key::X, true, true)],
            ..Default::default()
        };
        let frame = |app: &mut App, input: egui::RawInput| {
            run_ui(&ctx, input, |ui| {
                app.toolbar(ui);
                debug::show(ui, &app.cpu);
            })
        };
        // egui repaints right away while first laying out new widgets; let that settle
        for _ in 0..3 {
            frame(&mut app, egui::RawInput::default());
        }
        assert!(
            frame(&mut app, egui::RawInput::default()) > Duration::ZERO,
            "sanity: an idle frame doesn't ask for a repaint"
        );
        assert_eq!(
            frame(&mut app, repeat()),
            Duration::ZERO,
            "sanity: an unfiltered key repeat does ask for one, which is the bug"
        );
        frame(&mut app, egui::RawInput::default());

        let mut input = repeat();
        eframe::App::raw_input_hook(&mut app, &ctx, &mut input);

        assert!(
            frame(&mut app, input) > Duration::ZERO,
            "egui asked to repaint again immediately"
        );
    }

    // --- Loading ROMs ------------------------------------------------------------------

    #[test]
    fn app_starts_without_a_rom() {
        let app = App::new();
        assert!(app.rom_path.is_none());
        assert!(!app.running);
    }

    #[test]
    fn loading_a_rom_file_starts_it() {
        let path = std::env::temp_dir().join(format!("cash-gb-test-{}.gb", std::process::id()));
        std::fs::write(&path, rom_with_code(&IDLE_LOOP)).unwrap();
        let mut app = App::new();

        app.load_from_path(path.clone());
        std::fs::remove_file(&path).ok();

        assert_eq!(app.rom_path.as_ref(), Some(&path));
        assert!(app.running);
        assert!(app.load_error.is_none());
        assert_eq!(app.pending_title.as_deref(), Some("cash-gb - TEST"));
        assert_eq!(app.cpu.get_pc(), 0x0100);
    }

    #[test]
    fn loading_a_missing_file_shows_an_error_and_keeps_the_current_game() {
        let mut app = app_with_code(&IDLE_LOOP);
        let missing = std::env::temp_dir().join("cash-gb-no-such-rom.gb");

        app.load_from_path(missing);

        assert!(app.load_error.is_some());
        assert_eq!(
            app.rom_path.as_deref(),
            Some(std::path::Path::new("test.gb"))
        );
    }
}
