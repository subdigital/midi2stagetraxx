use anyhow::{Context, Result};
use eframe::egui;
use egui::RichText;
use midi2stagetraxx_core::{Extractor, Message, MidiEvent, MidiFormatter, StageTraxxFormatter};
use midi_file::MidiFile;
use std::path::PathBuf;

#[derive(Default)]
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

        match self.load_and_process_midi(path) {
            Ok(output) => self.output = output,
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    fn load_and_process_midi(&self, path: &std::path::PathBuf) -> Result<String> {
        let midi_file = MidiFile::load(path).context("Failed to load MIDI file")?;
        self.process_midi_file(midi_file)
    }

    fn process_midi_file(&self, midi_file: MidiFile) -> Result<String> {
        let mut extractor = Extractor::new(midi_file, self.override_midi_channel)
            .context("Failed to create extractor")?;
        let events = extractor.run().context("Failed to extract events")?;
        Ok(self.convert_events(events))
    }

    fn convert_events(&self, events: Vec<MidiEvent>) -> String {
        let formatter = StageTraxxFormatter::new();
        let mut output_lines = Vec::new();

        for (event, next) in events.iter().zip(events.iter().skip(1)) {
            let diff = next.timestamp - event.timestamp;
            if let Message::NoteOff(note, _) = event.message {
                // Sometimes if notes are turned on and then off in quick succession the midi
                // engine in StageTraxx (or the receiving app/device) may not be able to
                // distinguish the messages, leading to a light or setting staying on.
                // If this setting is turned on, we'll proactively filter out these off notes.
                // There is an exception for some notes where this is needed, so we'll check the
                // exceptions before skipping.

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

        output_lines.join("\n")
    }
}

impl eframe::App for MidiConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("MIDI File").strong()
                );
                if let Some(path) = &self.midi_file_path {
                    let filename = path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Unknown file");
                    ui.label(filename).on_hover_text(path.display().to_string());
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
                let override_midi_tooltip = "Assign this if you want to override the midi channel used in the file with something else.";
                ui.label(
                    RichText::new("Override MIDI Channel:").strong()
                )
                .on_hover_text(override_midi_tooltip);
                ui.add(egui::TextEdit::singleline(&mut self.channel_input)
                    .desired_width(40.0))
                    .on_hover_text(override_midi_tooltip);
                if ui.button("Set")
                    .on_hover_text(override_midi_tooltip)
                    .clicked() {
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

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.skip_off_note_collisions, "Skip OFF note collisions")
                .on_hover_text("Skip OFF notes that arrive at the same time as ON notes (helps with timing issues for mutually exclusive scenes)");
            });

            if self.skip_off_note_collisions {
                ui.horizontal(|ui| {
                    ui.label("Exception notes:");
                    ui.add(egui::TextEdit::singleline(&mut self.exceptions_input)
                        .desired_width(100.0));
                    ui.label("?").on_hover_text("Comma-separated MIDI note numbers that should never have their OFF events skipped (e.g., 48,64)");
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
