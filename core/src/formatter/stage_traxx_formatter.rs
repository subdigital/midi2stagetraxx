use crate::formatter::MidiFormatter;
use crate::midi_event::{Message, MidiEvent};
use std::time::Duration;

pub struct StageTraxxFormatter {}

impl StageTraxxFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for StageTraxxFormatter {
    fn default() -> Self {
        Self::new()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_midi_time_basic() {
        // Test basic time formatting
        assert_eq!(format_midi_time(0.0), "00:00.000");
        assert_eq!(format_midi_time(1.0), "00:01.000");
        assert_eq!(format_midi_time(60.0), "01:00.000");
        assert_eq!(format_midi_time(61.5), "01:01.500");
    }

    #[test]
    fn test_format_midi_time_precise() {
        // Test fractional seconds
        assert_eq!(format_midi_time(0.123), "00:00.123");
        assert_eq!(format_midi_time(1.456), "00:01.456");
        assert_eq!(format_midi_time(59.999), "00:59.999");

        // Test minute rollover
        assert_eq!(format_midi_time(60.001), "01:00.001");
        assert_eq!(format_midi_time(120.123), "02:00.123");
    }

    #[test]
    fn test_format_midi_time_large_values() {
        // Test larger time values
        assert_eq!(format_midi_time(3661.5), "61:01.500"); // 1 hour, 1 minute, 1.5 seconds
        assert_eq!(format_midi_time(7200.0), "120:00.000"); // 2 hours
    }

    #[test]
    fn test_format_note_on() {
        let formatter = StageTraxxFormatter::new();
        let event = MidiEvent {
            timestamp: 1.5,
            message: Message::NoteOn(60, 127), // Middle C, max velocity
            channel: 1,
        };

        let result = formatter.format(&event);
        assert_eq!(result, "[midi@00:01.500: N60.127@1]");
    }

    #[test]
    fn test_format_note_off() {
        let formatter = StageTraxxFormatter::new();
        let event = MidiEvent {
            timestamp: 2.0,
            message: Message::NoteOff(60, 64), // Note: velocity is ignored for note off
            channel: 2,
        };

        let result = formatter.format(&event);
        assert_eq!(result, "[midi@00:02.000: N60.0@2]"); // Note: velocity becomes 0
    }

    #[test]
    fn test_format_control_change() {
        let formatter = StageTraxxFormatter::new();
        let event = MidiEvent {
            timestamp: 0.123,
            message: Message::ControlChange(7, 100), // Volume control
            channel: 16,
        };

        let result = formatter.format(&event);
        assert_eq!(result, "[midi@00:00.123: CC7.100@16]");
    }

    #[test]
    fn test_format_edge_cases() {
        let formatter = StageTraxxFormatter::new();

        // Zero values
        let event = MidiEvent {
            timestamp: 0.0,
            message: Message::NoteOn(0, 0),
            channel: 1,
        };
        assert_eq!(formatter.format(&event), "[midi@00:00.000: N0.0@1]");

        // Maximum MIDI values
        let event = MidiEvent {
            timestamp: 0.0,
            message: Message::NoteOn(127, 127),
            channel: 16,
        };
        assert_eq!(formatter.format(&event), "[midi@00:00.000: N127.127@16]");

        // Maximum CC values
        let event = MidiEvent {
            timestamp: 0.0,
            message: Message::ControlChange(127, 127),
            channel: 16,
        };
        assert_eq!(formatter.format(&event), "[midi@00:00.000: CC127.127@16]");
    }

    #[test]
    fn test_format_timing_precision() {
        let formatter = StageTraxxFormatter::new();

        // Test millisecond precision
        let event = MidiEvent {
            timestamp: 46.70100001, // Should round to 46.701
            message: Message::ControlChange(1, 62),
            channel: 4,
        };

        let result = formatter.format(&event);
        assert_eq!(result, "[midi@00:46.701: CC1.62@4]");
    }

    #[test]
    fn test_format_multiple_minutes() {
        let formatter = StageTraxxFormatter::new();

        let event = MidiEvent {
            timestamp: 123.456, // 2 minutes, 3.456 seconds
            message: Message::NoteOn(48, 64),
            channel: 8,
        };

        let result = formatter.format(&event);
        assert_eq!(result, "[midi@02:03.456: N48.64@8]");
    }
}
