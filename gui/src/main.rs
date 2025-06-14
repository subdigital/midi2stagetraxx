mod app;

use anyhow::Result;
use app::MidiConverterApp;

fn main() -> Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "MIDI to StageTraxx Converter",
        native_options,
        Box::new(|cc| Ok(Box::new(MidiConverterApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))
}