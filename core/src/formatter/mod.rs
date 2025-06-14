use crate::midi_event::MidiEvent;

mod stage_traxx_formatter;

pub use stage_traxx_formatter::StageTraxxFormatter;

pub trait MidiFormatter {
    fn format(&self, event: &MidiEvent) -> String;
}
