use egui::{Button, RichText};

pub struct AppErrorShow {
    error: String,
}

impl AppErrorShow {
    pub fn new(error: String) -> Self {
        Self { error }
    }
}

impl eframe::App for AppErrorShow {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ROG ERROR");

            ui.centered_and_justified(|ui| {
                ui.label(RichText::new(format!("The error was: {:?}", self.error)).size(22.0));
            });

            egui::TopBottomPanel::bottom("error_bar_2")
                .default_height(26.0)
                .show(ctx, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if ui
                            .add(Button::new(RichText::new("Okay").size(20.0)))
                            .clicked()
                        {
                            frame.close();
                        }
                    });
                });
        });
    }
}
