#[derive(Debug, Clone)]
pub struct MidiEvent {
    pub timestamp: f64, // in seconds
    pub message: Message,
    pub channel: u8,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    NoteOn(u8, u8),
    NoteOff(u8, u8),
    ControlChange(u8, u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_event_creation() {
        let event = MidiEvent {
            timestamp: 1.5,
            message: Message::NoteOn(60, 127),
            channel: 1,
        };
        
        assert_eq!(event.timestamp, 1.5);
        assert_eq!(event.channel, 1);
        match event.message {
            Message::NoteOn(note, velocity) => {
                assert_eq!(note, 60);
                assert_eq!(velocity, 127);
            }
            _ => panic!("Expected NoteOn message"),
        }
    }

    #[test]
    fn test_message_variants() {
        // Test NoteOn
        let note_on = Message::NoteOn(60, 100);
        match note_on {
            Message::NoteOn(note, vel) => {
                assert_eq!(note, 60);
                assert_eq!(vel, 100);
            }
            _ => panic!("Expected NoteOn"),
        }

        // Test NoteOff
        let note_off = Message::NoteOff(60, 0);
        match note_off {
            Message::NoteOff(note, vel) => {
                assert_eq!(note, 60);
                assert_eq!(vel, 0);
            }
            _ => panic!("Expected NoteOff"),
        }

        // Test ControlChange
        let cc = Message::ControlChange(7, 127);
        match cc {
            Message::ControlChange(controller, value) => {
                assert_eq!(controller, 7);
                assert_eq!(value, 127);
            }
            _ => panic!("Expected ControlChange"),
        }
    }

    #[test]
    fn test_midi_event_boundary_values() {
        // Test boundary values for MIDI data
        let event = MidiEvent {
            timestamp: 0.0,
            message: Message::NoteOn(0, 0), // Minimum values
            channel: 1,
        };
        assert_eq!(event.timestamp, 0.0);
        assert_eq!(event.channel, 1);

        let event = MidiEvent {
            timestamp: f64::MAX,
            message: Message::NoteOn(127, 127), // Maximum MIDI values
            channel: 16, // Maximum MIDI channel
        };
        match event.message {
            Message::NoteOn(note, vel) => {
                assert_eq!(note, 127);
                assert_eq!(vel, 127);
            }
            _ => panic!("Expected NoteOn"),
        }
        assert_eq!(event.channel, 16);
    }

    #[test]
    fn test_message_clone() {
        let original = Message::ControlChange(64, 100);
        let cloned = original.clone();
        
        match (original, cloned) {
            (Message::ControlChange(cc1, val1), Message::ControlChange(cc2, val2)) => {
                assert_eq!(cc1, cc2);
                assert_eq!(val1, val2);
            }
            _ => panic!("Clone should produce identical message"),
        }
    }

    #[test]
    fn test_midi_event_clone() {
        let original = MidiEvent {
            timestamp: 42.5,
            message: Message::NoteOn(72, 90),
            channel: 5,
        };
        
        let cloned = original.clone();
        
        assert_eq!(original.timestamp, cloned.timestamp);
        assert_eq!(original.channel, cloned.channel);
        
        match (original.message, cloned.message) {
            (Message::NoteOn(n1, v1), Message::NoteOn(n2, v2)) => {
                assert_eq!(n1, n2);
                assert_eq!(v1, v2);
            }
            _ => panic!("Clone should produce identical message"),
        }
    }
}
