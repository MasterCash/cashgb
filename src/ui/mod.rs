use egui::{Color32, Layout, Rect, RichText, Sense, Vec2, vec2};

use crate::{cpu::Cpu, memory::cart::Cart};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    scale: f32,
    #[serde(skip)]
    cpu: Cpu,
    #[serde(skip)]
    frame_buffer: Vec<u8>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            scale: Default::default(),
            cpu: Cpu::empty(),
            frame_buffer: Vec::new(),
        }
    }
}

impl App {
    pub fn new(cart: Cart) -> Self {
        Self {
            scale: 5.0,
            cpu: Cpu::new(cart),
            frame_buffer: Vec::new(),
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    ui.menu_button("Theme", |ui| {
                        egui::widgets::global_theme_preference_buttons(ui);
                    })
                });
            });
        });

        egui::Panel::left("registers").show(ui, |ui| {
            let step_clicked = ui.button("Step").interact(Sense::click());
            if step_clicked.clicked() {
                self.cpu.step();
            }
            let step_clicked = ui.button("Next Instruction").interact(Sense::click());
            if step_clicked.clicked() {
                self.cpu.step_instruction();
            }
            ui.vertical(|ui| {
                ui.label("instruction: ");
                ui.label(format!("{:#?}", self.cpu.instruction))
            });
            ui.horizontal(|ui| {
                ui.label("Status: ");
                ui.label(format!("{:#?}", self.cpu.status))
            });
            ui.horizontal(|ui| {
                ui.label("Step Count: ");
                ui.label(format!("{}", self.cpu.step_count))
            });
            ui.horizontal(|ui| {
                ui.label("AF: ");
                ui.label(format!("{:b}", self.cpu.get_af()))
            });
            ui.horizontal(|ui| {
                ui.label("BC: ");
                ui.label(format!("{:b}", self.cpu.get_bc()))
            });
            ui.horizontal(|ui| {
                ui.label("DE: ");
                ui.label(format!("{:b}", self.cpu.get_de()))
            });
            ui.horizontal(|ui| {
                ui.label("HL: ");
                ui.label(format!("{:b}", self.cpu.get_hl()))
            });
            ui.horizontal(|ui| {
                ui.label("PC: ");
                ui.label(format!("{:X}", self.cpu.get_pc()))
            });
            ui.horizontal(|ui| {
                ui.label("SP: ");
                ui.label(format!("{:X}", self.cpu.get_sp()))
            });
        });

        egui::Panel::right("memory").show(ui, |ui| {
            ui.label("rom");
            const ROW_SIZE: usize = 32;
            let rom = self.cpu.memory.cart.mbc.rom();
            let text_style = egui::TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);
            let font_id = egui::TextStyle::Body.resolve(ui.style());

            let exact_max_width = ui.fonts_mut(|f| {
                "ABCDEF"
                    .chars()
                    .map(|c| f.glyph_width(&font_id, c))
                    .fold(0.0, f32::max)
            });

            // let row_height = ui.spacing().interact_size.y; // if you are adding buttons instead of labels.
            let total_rows = (rom.len() as f32 / ROW_SIZE as f32).ceil() as usize;
            let total_digits = usize::MAX.leading_ones() - rom.len().leading_zeros();
            let total_digits = total_digits.div_ceil(4) as usize;
            egui::scroll_area::ScrollArea::vertical().show_rows(
                ui,
                row_height,
                total_rows,
                |ui, row_range| {
                    egui::Grid::new("rom")
                        .min_row_height(row_height)
                        .min_col_width(2.0 * exact_max_width)
                        .show(ui, |ui| {
                            for row in row_range {
                                ui.allocate_ui_with_layout(
                                    vec2(total_digits as f32 * exact_max_width, row_height),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(format!(
                                                "{:0width$X}:",
                                                row * ROW_SIZE,
                                                width = total_digits
                                            ))
                                            .color(Color32::GRAY),
                                        );
                                    },
                                );

                                for i in 0..ROW_SIZE {
                                    let i = row * ROW_SIZE + i;
                                    if i >= rom.len() {
                                        break;
                                    }
                                    let datum = rom[i];
                                    ui.with_layout(
                                        Layout::centered_and_justified(
                                            egui::Direction::LeftToRight,
                                        ),
                                        |ui| {
                                            if i == self.cpu.get_pc() as usize {
                                                ui.label(
                                                    RichText::new(format!("{:02X}", datum))
                                                        .color(Color32::GREEN),
                                                );
                                            } else {
                                                ui.label(format!("{:02X}", datum));
                                            }
                                        },
                                    );
                                }
                                ui.end_row();
                            }
                        });
                },
            );
        });

        egui::CentralPanel::default().show(ui, |ui| {
            use egui::Color32;
            const GB_SCREEN_WIDTH: u32 = 160;
            const GB_SCREEN_HEIGHT: u32 = 144;
            let scale: f32 = self.scale;
            let size: Vec2 = (
                (GB_SCREEN_WIDTH) as f32 * scale,
                (GB_SCREEN_HEIGHT) as f32 * scale,
            )
                .into();
            let (response, painter) = ui.allocate_painter(size, Sense::empty());
            let origin = response.rect.min;
            let pixels = if self.frame_buffer.is_empty() {
                vec![]
            } else {
                egui::ColorImage::from_rgba_unmultiplied([160, 144], &self.frame_buffer).pixels
            };
            for row in 0..GB_SCREEN_HEIGHT {
                for col in 0..GB_SCREEN_WIDTH {
                    let rect = Rect::from_min_size(
                        origin + Vec2::new(col as f32 * scale, row as f32 * scale),
                        size,
                    );
                    let pixel = pixels
                        .get((row * GB_SCREEN_WIDTH + col) as usize)
                        .unwrap_or(&Color32::WHITE);
                    painter.rect_filled(rect, 0.0, *pixel);
                }
            }
        });
    }
}
