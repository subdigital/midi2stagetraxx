# MIDI to StageTraxx Converter

A tool for converting MIDI files into StageTraxx-compatible format.

![the midi2stagetraxx gui app running on macOS](assets/gui.png)

## Two Flavors

- A gui application that runs on macOS, Linux, or Windows.
- A command-line application that runs in a terminal.

If you aren't a developer or aren't familiar with the Rust toolchain,
check the [Releases](https://github.com/subdigital/midi2stagetraxx/releases) page
and download the latest release for your operating system.

If you'd like to build from source, read on...

## Project Structure

This is a [Rust](https://www.rust-lang.org/) workspace with three crates:

- **`core/`** - Shared library with MIDI processing logic
- **`cli/`** - Command-line interface
- **`gui/`** - Native GUI application (using egui)

## Building

```bash
# Build everything
cargo build

# Build release versions
cargo build --release
```

## Running

### CLI Version
```bash
# Run from workspace root
cargo run -p midi2stagetraxx-cli -- -m your_file.mid

# Or directly
./target/debug/midi2stagetraxx -m your_file.mid

# With options
./target/debug/midi2stagetraxx -m your_file.mid --skip-off-note-collisions --off-collision-exceptions 48,64
```

### GUI Version
```bash
# Run from workspace root
cargo run -p midi2stagetraxx-gui

# Or directly
./target/debug/midi2stagetraxx-gui
```

## CLI Options

- `-m, --midi-file <FILE>` - MIDI file to convert (required)
- `-o, --override-midi-channel <CHANNEL>` - Override MIDI channel for all notes and CC changes
- `--skip-off-note-collisions` - Skip OFF notes that arrive at the same time as ON notes
- `--off-collision-exceptions <NOTES>` - Comma-separated list of MIDI note numbers that should never have their OFF events skipped

## Development

To work on a specific crate:
```bash
cd core  # or cli, gui
cargo build
cargo test
```
