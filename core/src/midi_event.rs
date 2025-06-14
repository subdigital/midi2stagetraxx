#[derive(Debug, Clone)]
pub struct MidiEvent {
    pub timestamp: f64, // in seconds
    pub message: Message,
    pub channel: u8,
}

#[derive(Debug, Clone)]
pub enum Message {
    NoteOn(u8, u8),
    NoteOff(u8, u8),
    ControlChange(u8, u8),
}
