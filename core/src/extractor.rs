use anyhow::Result;

use crate::midi_event;
use midi_file::core::{ControlChangeValue, NoteMessage, ProgramChangeValue};
use midi_file::file::SmpteOffsetValue;
use midi_file::file::TrackEvent;
use midi_file::file::{Division, MetaEvent};
use midi_file::{core::Message, file::Event, MidiFile};
use std::mem;

pub struct Extractor {
    midi_file: MidiFile,
    override_midi_channel: Option<u8>,
    pulses_per_qn: u16,
    ticks: u32,
    last_tempo_change_ticks: u32,
    elapsed_sec: f64,
    last_midi_event_ts: f64,
    current_tempo_micros_per_qn: u32,
}

// microseconds per second
const MICROS_PER_SEC: f64 = 1_000_000.0;
const DEFAULT_BPM: f64 = 120.0;

impl Extractor {
    pub fn new(midi_file: MidiFile, override_midi_channel: Option<u8>) -> Result<Self> {
        // read division to get pulses per quarter note
        let div = midi_file.header().division();

        let pulses_per_qn: u16 = match div {
            Division::QuarterNote(qtr) => {
                println!("Quarter Note Division: {}", qtr);
                qtr.get()
            }
            Division::Smpte(smpte) => {
                // don't think we need this for now, but we can add it later
                println!("SMPTE Division: {:?}", smpte);
                unimplemented!("SMPTE division")
            }
        };

        Ok(Self {
            midi_file,
            override_midi_channel,
            pulses_per_qn,
            ticks: 0,
            last_tempo_change_ticks: 0,
            elapsed_sec: 0.0,
            last_midi_event_ts: 0.0,
            current_tempo_micros_per_qn: (MICROS_PER_SEC / (DEFAULT_BPM / 60.0)) as u32,
        })
    }

    pub fn run(&mut self) -> Result<Vec<midi_event::MidiEvent>> {
        let tracks = self.midi_file.tracks();
        let track_events: Vec<TrackEvent> = tracks.flat_map(|t| t.events().cloned()).collect();

        let mut results: Vec<midi_event::MidiEvent> = Vec::new();
        for track_event in track_events {
            if let Some(event) = self.process_event(&track_event) {
                results.push(event);
            }
        }

        Ok(results)
    }

    fn process_event(&mut self, track_event: &TrackEvent) -> Option<midi_event::MidiEvent> {
        let dt = track_event.delta_time();
        let event = track_event.event();
        self.ticks += dt;
        match event {
            Event::Midi(msg) => {
                let ticks_since_last_tempo_change = self.ticks - self.last_tempo_change_ticks;
                let timestamp = self.elapsed_sec
                    + ticks_to_seconds(
                        ticks_since_last_tempo_change,
                        self.pulses_per_qn,
                        self.current_tempo_micros_per_qn,
                    );
                self.last_midi_event_ts = timestamp;
                self.handle_midi_msg(msg, timestamp, ticks_since_last_tempo_change)
            }

            Event::Meta(MetaEvent::SetTempo(new_tempo)) => {
                self.handle_tempo_change(new_tempo.get());
                None
            }

            Event::Meta(MetaEvent::SmpteOffset(smpte_offset)) => {
                self.handle_smpte_offset(smpte_offset);
                None
            }

            Event::Meta(MetaEvent::TimeSignature(sig)) => {
                eprintln!("-- TIME SIGNATURE: {:?}", sig);
                None
            }

            _ => {
                eprintln!("-- EVENT: {:?} {:?}", dt, event);
                None
            }
        }
    }

    fn handle_midi_msg(
        &self,
        msg: &Message,
        timestamp: f64,
        dt: u32,
    ) -> Option<midi_event::MidiEvent> {
        match msg {
            Message::NoteOn(note) => Some(self.handle_note(note, timestamp, true)),
            Message::NoteOff(note) => Some(self.handle_note(note, timestamp, false)),
            Message::Control(cc) => Some(self.handle_control_change(cc, timestamp)),
            Message::ProgramChange(pc) => Some(self.handle_program_change(pc, timestamp)),
            _ => {
                eprintln!("Unhandled MIDI: {:?} {:?}", dt, msg);
                None
            }
        }
    }

    fn handle_note(&self, note: &NoteMessage, timestamp: f64, on: bool) -> midi_event::MidiEvent {
        let velocity = if on { note.velocity().get() } else { 0 };
        let message = if on {
            midi_event::Message::NoteOn(note.note_number().get(), velocity)
        } else {
            midi_event::Message::NoteOff(note.note_number().get(), velocity)
        };

        midi_event::MidiEvent {
            timestamp,
            message,
            channel: self
                .override_midi_channel
                .unwrap_or(note.channel().get() + 1), // midi_file is 0-based
        }
    }

    fn handle_control_change(
        &self,
        cc: &ControlChangeValue,
        timestamp: f64,
    ) -> midi_event::MidiEvent {
        midi_event::MidiEvent {
            timestamp,
            message: midi_event::Message::ControlChange(cc.control() as u8, cc.value().get()),
            channel: self.override_midi_channel.unwrap_or(cc.channel().get() + 1), // midi_file is 0-based
        }
    }

    fn handle_tempo_change(&mut self, new_tempo_micros_per_qn: u32) {
        let bpm = MICROS_PER_SEC / new_tempo_micros_per_qn as f64 * 60.0;
        eprintln!("-- Tempo change: {}", bpm);

        let ticks_since_last_tempo_change = self.ticks - self.last_tempo_change_ticks;
        self.last_tempo_change_ticks = self.ticks;

        self.elapsed_sec += ticks_to_seconds(
            ticks_since_last_tempo_change,
            self.pulses_per_qn,
            self.current_tempo_micros_per_qn,
        );
        self.current_tempo_micros_per_qn = new_tempo_micros_per_qn;
    }

    fn handle_smpte_offset(&self, smpte_offset: &SmpteOffsetValue) {
        eprintln!("-- SMPTE offset: {:?}", smpte_offset);
        let (frame_rate, hr) = extract_frame_rate_hrs(smpte_offset);
        eprintln!(
            "-- SMPTE OFFSET: ({:?}) frame: {}, hr: {}",
            smpte_offset, frame_rate, hr
        );
    }

    fn handle_program_change(
        &self,
        pc: &ProgramChangeValue,
        timestamp: f64,
    ) -> midi_event::MidiEvent {
        midi_event::MidiEvent {
            timestamp,
            message: midi_event::Message::ProgramChange(pc.program().get()),
            channel: self.override_midi_channel.unwrap_or(pc.channel().get() + 1), // midi_file is 0-based
        }
    }
}

pub fn ticks_to_seconds(ticks: u32, pulses_per_qn: u16, tempo: u32) -> f64 {
    // MIDI tempo is in microseconds per quarter note
    let tempo_in_secs = tempo as f64 / 1_000_000.0;
    let beats = ticks as f64 / pulses_per_qn as f64;
    beats * tempo_in_secs
}

#[allow(dead_code)]
struct SmpteOffsetValueLayout {
    // TODO - these are held as raw bytes for now without caring about their meaning or signedness.
    hr: u8,
    mn: u8,
    se: u8,
    fr: u8,
    ff: u8,
}

enum SmpteFrameSpec {
    F24 = 0,
    F25 = 1,
    F2997 = 2,
    F30 = 3,
}

impl SmpteFrameSpec {
    fn frame_rate(&self) -> f64 {
        match self {
            SmpteFrameSpec::F24 => 24.0,
            SmpteFrameSpec::F25 => 25.0,
            SmpteFrameSpec::F2997 => 29.97,
            SmpteFrameSpec::F30 => 30.0,
        }
    }
}

impl From<u8> for SmpteFrameSpec {
    fn from(val: u8) -> Self {
        match val {
            0 => SmpteFrameSpec::F24,
            1 => SmpteFrameSpec::F25,
            2 => SmpteFrameSpec::F2997,
            3 => SmpteFrameSpec::F30,
            _ => panic!("Invalid SMPTE frame rate"),
        }
    }
}

fn extract_frame_rate_hrs(smpte_offset: &SmpteOffsetValue) -> (f64, u8) {
    unsafe {
        let smpte_layout =
            mem::transmute::<SmpteOffsetValue, SmpteOffsetValueLayout>(*smpte_offset);
        // shift off the last 6  bits to get the frame rate
        let mask = 0b0000_0011;
        let frame_rate_spec = (smpte_layout.hr >> 6) & mask;
        let fr = SmpteFrameSpec::from(frame_rate_spec).frame_rate();

        let hr_mask = 0b0001_1111;
        let hr = smpte_layout.hr & hr_mask;

        (fr, hr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticks_to_seconds_basic() {
        // Test basic conversion: 480 ticks per quarter note at 120 BPM
        let pulses_per_qn = 480;
        let tempo_120_bpm = 500_000; // 120 BPM = 500,000 microseconds per quarter note

        // 480 ticks = 1 quarter note = 0.5 seconds at 120 BPM
        let result = ticks_to_seconds(480, pulses_per_qn, tempo_120_bpm);
        assert!(
            (result - 0.5).abs() < 0.001,
            "Expected ~0.5s, got {}",
            result
        );

        // 960 ticks = 2 quarter notes = 1.0 seconds at 120 BPM
        let result = ticks_to_seconds(960, pulses_per_qn, tempo_120_bpm);
        assert!(
            (result - 1.0).abs() < 0.001,
            "Expected ~1.0s, got {}",
            result
        );
    }

    #[test]
    fn test_ticks_to_seconds_different_tempos() {
        let pulses_per_qn = 480;

        // 60 BPM = 1,000,000 microseconds per quarter note
        let tempo_60_bpm = 1_000_000;
        let result = ticks_to_seconds(480, pulses_per_qn, tempo_60_bpm);
        assert!(
            (result - 1.0).abs() < 0.001,
            "Expected ~1.0s at 60 BPM, got {}",
            result
        );

        // 240 BPM = 250,000 microseconds per quarter note
        let tempo_240_bpm = 250_000;
        let result = ticks_to_seconds(480, pulses_per_qn, tempo_240_bpm);
        assert!(
            (result - 0.25).abs() < 0.001,
            "Expected ~0.25s at 240 BPM, got {}",
            result
        );
    }

    #[test]
    fn test_ticks_to_seconds_edge_cases() {
        let pulses_per_qn = 480;
        let tempo_120_bpm = 500_000;

        // Zero ticks should give zero time
        let result = ticks_to_seconds(0, pulses_per_qn, tempo_120_bpm);
        assert_eq!(result, 0.0);

        // Fractional beats
        let result = ticks_to_seconds(240, pulses_per_qn, tempo_120_bpm); // Half a quarter note
        assert!(
            (result - 0.25).abs() < 0.001,
            "Expected ~0.25s for half beat, got {}",
            result
        );
    }

    #[test]
    fn test_ticks_to_seconds_different_resolutions() {
        let tempo_120_bpm = 500_000;

        // Different MIDI resolutions
        assert!((ticks_to_seconds(96, 96, tempo_120_bpm) - 0.5).abs() < 0.001);
        assert!((ticks_to_seconds(192, 192, tempo_120_bpm) - 0.5).abs() < 0.001);
        assert!((ticks_to_seconds(960, 960, tempo_120_bpm) - 0.5).abs() < 0.001);
    }
}
