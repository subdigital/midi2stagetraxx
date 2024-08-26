use std::mem;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::arg;
use clap::Parser;

use midi_file::core::NoteMessage;
use midi_file::file::Format;
use midi_file::file::MetaEvent;
use midi_file::{
    core::Message,
    file::{Event, QuarterNoteDivision, SmpteOffsetValue},
    MidiFile,
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    midi_file: String,

    #[arg(
        short,
        long,
        help = "Override the MIDI channel for all notes and CC changes"
    )]
    override_midi_channel: Option<u8>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("MIDI FILE: {}", args.midi_file);

    let file = MidiFile::load(args.midi_file).context("load midi file")?;
    let fmt = file.header().format();
    match fmt {
        Format::Single => (),
        Format::Multi => unimplemented!("Multi Track"),
        Format::Sequential => println!("Sequential"),
    }

    let div = file.header().division();
    let pulses_per_qn = match div {
        midi_file::file::Division::QuarterNote(qtr) => {
            println!("Quarter Note Division: {}", qtr);
            qtr
        }
        midi_file::file::Division::Smpte(smpte) => {
            println!("SMPTE Division: {:?}", smpte);
            unimplemented!("SMPTE division")
        }
    };

    // microseconds per second
    let micros_per_s = 1_000_000.0;

    // this is a guess until we get a tempo event
    let starting_bpm = 120.0;
    let mut tempo_micros_per_q: u32 = (micros_per_s / (starting_bpm / 60.0)) as u32;
    let mut ticks = 0;
    let mut last_tempo_change_tick: u32 = 0;
    let mut ellapsed_seconds = 0.0;
    let mut last_midi_event_ts = 0.0;

    for track in file.tracks() {
        for event in track.events() {
            let dt = event.delta_time();
            let e = event.event();
            ticks += dt;
            match e {
                Event::Midi(msg) => {
                    let ticks_since_last_tempo_change = ticks - last_tempo_change_tick;
                    let mut timestamp = ellapsed_seconds
                        + ticks_to_seconds(
                            ticks_since_last_tempo_change,
                            pulses_per_qn,
                            tempo_micros_per_q,
                        );

                    // TODO: this is a hack to avoid multiple events at the same time,
                    // but produces inconsistent results
                    // if timestamp - last_midi_event_ts < 0.01 {
                    //     timestamp += 0.01;
                    // }
                    last_midi_event_ts = timestamp;
                    handle_midi_msg(
                        msg,
                        timestamp,
                        ticks_since_last_tempo_change,
                        args.override_midi_channel,
                    )
                }

                Event::Meta(midi_file::file::MetaEvent::SetTempo(new_tempo)) => {
                    let bpm = micros_per_s / new_tempo.get() as f64 * 60.0;
                    eprintln!("-- Tempo change: {}", bpm);
                    let ticks_since_last_tempo_change = ticks - last_tempo_change_tick;
                    last_tempo_change_tick = ticks;
                    ellapsed_seconds += ticks_to_seconds(
                        ticks_since_last_tempo_change,
                        pulses_per_qn,
                        tempo_micros_per_q,
                    );
                    tempo_micros_per_q = new_tempo.get()
                }

                Event::Meta(midi_file::file::MetaEvent::SmpteOffset(smpte_offset)) => {
                    let (frame_rate, hr) = extract_frame_rate_hrs(smpte_offset);
                    eprintln!(
                        "-- SMPTE OFFSET: ({:?}) frame: {}, hr: {}",
                        smpte_offset, frame_rate, hr
                    );
                }

                Event::Meta(MetaEvent::TimeSignature(sig)) => {
                    eprintln!("-- TIME SIGNATURE: {:?}", sig);
                }

                _ => eprintln!("-- EVENT: {:?} {:?}", dt, e),
            }
        }
    }

    Ok(())
}

fn handle_note(note: &NoteMessage, timestamp: f64, on: bool, ch: Option<u8>) {
    // [midi@00:48.50: N122.127@4]
    // off note has velocity 0
    let velocity = if on { note.velocity().get() } else { 0 };
    // if !on {
    //     return;
    // }
    let msg = format!(
        "[midi@{}: N{:?}.{:?}@{:?}]",
        format_midi_time(timestamp),
        note.note_number().get(),
        velocity,
        ch.unwrap_or(note.channel().get() + 1), // midi_file is 0-based
    );
    println!("{}", msg);
}

fn handle_midi_msg(msg: &Message, timestamp: f64, dt: u32, ch: Option<u8>) {
    match msg {
        Message::NoteOn(note) => handle_note(note, timestamp, true, ch),
        Message::NoteOff(note) => handle_note(note, timestamp, false, ch),
        Message::Control(cc) => {
            // [midi@00:46.70: CC1.62@4]
            let msg = format!(
                "[midi@{}: CC{:?}.{:?}@{:?}]",
                format_midi_time(timestamp),
                cc.control() as u8,
                cc.value().get(),
                ch.unwrap_or(cc.channel().get() + 1), // midi_file is 0-based
            );
            println!("{}", msg);
        }
        _ => println!("MIDI: {:?} {:?}", dt, msg),
    }
}

fn ticks_to_seconds(ticks: u32, pulses_per_qn: &QuarterNoteDivision, tempo: u32) -> f64 {
    // MIDI tempo is in microseconds per quarter note
    let tempo_in_secs = tempo as f64 / 1_000_000.0;
    let beats = ticks as f64 / pulses_per_qn.get() as f64;
    beats * tempo_in_secs
}

fn format_midi_time(seconds: f64) -> String {
    let duration = Duration::from_secs_f64(seconds);
    let minutes = duration.as_secs() / 60;
    let seconds = duration.as_secs() % 60;
    let fractional = duration.subsec_millis();
    format!("{:02}:{:02}.{:03}", minutes, seconds, fractional)
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
