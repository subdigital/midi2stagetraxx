use crate::formatter::MidiFormatter;
use crate::midi_event::{Message, MidiEvent};
use std::time::Duration;

pub struct StageTraxxFormatter {}

impl StageTraxxFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl MidiFormatter for StageTraxxFormatter {
    fn format(&self, event: &MidiEvent) -> String {
        // [midi@00:46.70: CC1.62@4]
        let params: (&str, u8, u8) = match event.message {
            Message::NoteOn(note, velocity) => ("N", note, velocity),
            Message::NoteOff(note, _) => ("N", note, 0),
            Message::ControlChange(num, val) => ("CC", num, val),
        };
        format!(
            "[midi@{timestamp}: {msg}{arg1}.{arg2}@{channel}]",
            timestamp = format_midi_time(event.timestamp),
            msg = params.0,
            arg1 = params.1,
            arg2 = params.2,
            channel = event.channel
        )
    }
}

fn format_midi_time(seconds: f64) -> String {
    let duration = Duration::from_secs_f64(seconds);
    let minutes = duration.as_secs() / 60;
    let seconds = duration.as_secs() % 60;
    let fractional = duration.subsec_millis();
    format!("{:02}:{:02}.{:03}", minutes, seconds, fractional)
}
