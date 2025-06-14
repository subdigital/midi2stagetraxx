mod app;

use anyhow::Result;
use app::MidiConverterApp;
use egui::{FontData, FontDefinitions};
use std::fs;

fn main() -> Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "MIDI to StageTraxx Converter",
        native_options,
        Box::new(|cc| Ok(Box::new(MidiConverterApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))
}

#[cfg(target_os = "macos")]
fn system_font_path() -> Option<&str> {
    "/System/Library/Fonts/SFNS.ttf"
}

#[cfg(target_os = "linux")]
fn system_font_path() -> Option<&str> {
    None
}
#[cfg(target_os = "windows")]
fn system_font_path() -> Option<&str> {
    None
}

fn load_system_font() -> Result<()> {
    let bytes = system_font_path()?;
    let font_data = FontData::from_static(bytes);
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("SF".to_owned(), std::sync::Arc::new(font_data));
}

