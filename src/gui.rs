use crate::extractor::Extractor;
use crate::formatter::{MidiFormatter, StageTraxxFormatter};
use crate::midi_event::Message;
use eframe::egui;
use midi_file::MidiFile;
use std::path::PathBuf;

pub struct MidiConverterApp {
    midi_file_path: Option<PathBuf>,
    override_midi_channel: Option<u8>,
    skip_off_note_collisions: bool,
    off_collision_exceptions: Vec<u8>,
    exceptions_input: String,
    output: String,
    error: Option<String>,
    channel_input: String,
}

impl Default for MidiConverterApp {
    fn default() -> Self {
        Self {
            midi_file_path: None,
            override_midi_channel: None,
            skip_off_note_collisions: false,
            off_collision_exceptions: Vec::new(),
            exceptions_input: String::new(),
            output: String::new(),
            error: None,
            channel_input: String::new(),
        }
    }
}

impl MidiConverterApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn convert_midi(&mut self) {
        self.error = None;
        self.output.clear();

        let Some(path) = &self.midi_file_path else {
            self.error = Some("No MIDI file selected".to_string());
            return;
        };

        match MidiFile::load(path) {
            Ok(midi_file) => {
                match Extractor::new(midi_file, self.override_midi_channel) {
                    Ok(mut extractor) => {
                        match extractor.run() {
                            Ok(events) => {
                                let formatter = StageTraxxFormatter::new();
                                let mut output_lines = Vec::new();

                                for (event, next) in events.iter().zip(events.iter().skip(1)) {
                                    let diff = next.timestamp - event.timestamp;
                                    if let Message::NoteOff(note, _) = event.message {
                                        if diff <= 0.01 && self.skip_off_note_collisions {
                                            if self.off_collision_exceptions.contains(&note) {
                                                // Don't skip this note - it's an exception
                                                output_lines.push(formatter.format(event));
                                                continue;
                                            }
                                            // Skip the note off event
                                            continue;
                                        }
                                    }
                                    output_lines.push(formatter.format(event));
                                }

                                if let Some(last_event) = events.last() {
                                    output_lines.push(formatter.format(last_event));
                                }

                                self.output = output_lines.join("\n");
                            }
                            Err(e) => self.error = Some(format!("Failed to extract events: {}", e)),
                        }
                    }
                    Err(e) => self.error = Some(format!("Failed to create extractor: {}", e)),
                }
            }
            Err(e) => self.error = Some(format!("Failed to load MIDI file: {}", e)),
        }
    }
}

impl eframe::App for MidiConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MIDI to StageTraxx Converter");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("MIDI File:");
                if let Some(path) = &self.midi_file_path {
                    ui.label(path.display().to_string());
                } else {
                    ui.label("No file selected");
                }
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("MIDI files", &["mid", "midi"])
                        .pick_file()
                    {
                        self.midi_file_path = Some(path);
                    }
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Override MIDI Channel:");
                ui.text_edit_singleline(&mut self.channel_input);
                if ui.button("Set").clicked() {
                    match self.channel_input.parse::<u8>() {
                        Ok(channel) if channel <= 16 && channel > 0 => {
                            self.override_midi_channel = Some(channel);
                            self.error = None;
                        }
                        _ => {
                            self.error = Some("Channel must be between 1 and 16".to_string());
                        }
                    }
                }
                if self.override_midi_channel.is_some() && ui.button("Clear").clicked() {
                    self.override_midi_channel = None;
                    self.channel_input.clear();
                }
                if let Some(channel) = self.override_midi_channel {
                    ui.label(format!("(Currently: {})", channel));
                }
            });

            ui.checkbox(&mut self.skip_off_note_collisions, "Skip OFF note collisions");
            ui.small_button("?").on_hover_text("Skip OFF notes that arrive at the same time as ON notes (helps with timing issues for mutually exclusive scenes)");

            if self.skip_off_note_collisions {
                ui.horizontal(|ui| {
                    ui.label("Exception notes:");
                    ui.text_edit_singleline(&mut self.exceptions_input);
                    ui.small_button("?").on_hover_text("Comma-separated MIDI note numbers that should never have their OFF events skipped (e.g., 48,64)");
                    if ui.button("Set").clicked() {
                        self.off_collision_exceptions.clear();
                        for part in self.exceptions_input.split(',') {
                            if let Ok(note) = part.trim().parse::<u8>() {
                                if note <= 127 {
                                    self.off_collision_exceptions.push(note);
                                }
                            }
                        }
                        self.error = None;
                    }
                    if !self.off_collision_exceptions.is_empty() {
                        ui.label(format!("(Active: {:?})", self.off_collision_exceptions));
                    }
                });
            }

            ui.separator();

            if ui.button("Convert").clicked() {
                self.convert_midi();
            }

            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            }

            ui.separator();

            ui.label("Output:");
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.output)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace)
                    );
                });

            if !self.output.is_empty() {
                ui.horizontal(|ui| {
                    if ui.button("Copy to Clipboard").clicked() {
                        ui.output_mut(|o| o.copied_text = self.output.clone());
                    }
                    if ui.button("Save to File").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Text files", &["txt"])
                            .save_file()
                        {
                            match std::fs::write(path, &self.output) {
                                Ok(_) => self.error = None,
                                Err(e) => self.error = Some(format!("Failed to save: {}", e)),
                            }
                        }
                    }
                });
            }
        });
    }
}