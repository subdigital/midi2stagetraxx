use anyhow::{Context, Result};
use clap::arg;
use clap::Parser;

use formatter::MidiFormatter;
use midi_file::MidiFile;
mod extractor;
use extractor::Extractor;

mod formatter;
mod midi_event;

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
}

fn main() -> Result<()> {
    let args = Args::parse();
    let midi_file = MidiFile::load(args.midi_file).context("load midi file")?;
    let mut extractor = Extractor::new(midi_file, args.override_midi_channel)?;
    let events = extractor.run()?;
    let formatter = formatter::StageTraxxFormatter::new();

    for (event, next) in events.iter().zip(events.iter().skip(1)) {
        if event.timestamp == next.timestamp && args.skip_off_note_collisions {
            // drop the note off event to avoid conflicts
            continue;
        }
        println!("{}", formatter.format(event));
    }

    Ok(())
}
