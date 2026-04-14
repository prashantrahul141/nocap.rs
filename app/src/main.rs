use nocaprs_ui::UI;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

fn main() -> eframe::Result {
    /* setup tracing subscriber */
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let pretty_layer = fmt::layer().pretty();
    tracing_subscriber::registry()
        .with(env_filter)
        .with(pretty_layer)
        .init();

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
