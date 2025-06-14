#[derive(Debug)]
pub struct MidiEvent {
    pub timestamp: f64, // in seconds
    pub message: Message,
    pub channel: u8,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Message {
    NoteOn(u8, u8),
    NoteOff(u8, u8),
    ControlChange(u8, u8),
}
