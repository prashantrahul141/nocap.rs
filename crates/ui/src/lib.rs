use eframe::egui;

pub struct UI {}

impl Default for UI {
    fn default() -> Self {
        Self {}
    }
}

impl eframe::App for UI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("nocap-rs");
        });
    }
}
