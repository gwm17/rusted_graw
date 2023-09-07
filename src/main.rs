mod ui;
mod merger;

use ui::app::MergerApp;

#[allow(unreachable_code, dead_code)]
fn main() {
    simplelog::TermLogger::init(simplelog::LevelFilter::Info, 
                                simplelog::Config::default(),
                                simplelog::TerminalMode::Mixed,
                            simplelog::ColorChoice::Auto)
                            .unwrap();

    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(eframe::epaint::vec2(600.0, 275.0));
    match eframe::run_native("rusted_graw", native_options, Box::new(|cc| Box::new(MergerApp::new(cc)))) {
        Ok(()) => (),
        Err(e) => log::error!("Eframe error: {}", e)
    }
    return;
}
