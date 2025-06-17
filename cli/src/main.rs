use anyhow::{Context, Result};
use clap::Parser;
use midi2stagetraxx_core::{Extractor, Message, MidiFormatter, StageTraxxFormatter};
use midi_file::MidiFile;

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

    #[arg(
        long,
        help = "Skip off notes that arrive at the same time as an ON note (this can help with timing issues when controlling mutually exclusive scenes with lights)"
    )]
    skip_off_note_collisions: bool,

    #[arg(
        long,
        value_delimiter = ',',
        help = "Comma-separated list of MIDI note numbers that should never have their OFF events skipped (e.g., --off-collision-exceptions 48,64)"
    )]
    off_collision_exceptions: Option<Vec<u8>>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let midi_file = MidiFile::load(&args.midi_file).context("load midi file")?;
    let mut extractor = Extractor::new(midi_file, args.override_midi_channel)?;
    let events = extractor.run()?;
    let formatter = StageTraxxFormatter::new();

    let exception_notes: Vec<u8> = args.off_collision_exceptions.unwrap_or_default();

    for (event, next) in events.iter().zip(events.iter().skip(1)) {
        let diff = next.timestamp - event.timestamp;
        if let Message::NoteOff(note, _) = event.message {
            if diff <= 0.01 && args.skip_off_note_collisions {
                // Check if this note is in the exception list
                if exception_notes.contains(&note) {
                    // Don't skip this note - it's an exception
                    println!("{}", formatter.format(event));
                    continue;
                }
                // Skip the note off event to avoid conflicts
                continue;
            }
        }
        println!("{}", formatter.format(event));
    }

    // Handle the last event
    if let Some(last_event) = events.last() {
        println!("{}", formatter.format(last_event));
    }

    Ok(())
}
