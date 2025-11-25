use ui::UI;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "nocap-rs",
        options,
        Box::new(|cc| {
            // image support:
            // egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<UI>::default())
        }),
    )
}
