# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-17

### Added
- Initial release of MIDI to StageTraxx converter
- Command-line interface for batch processing
- Graphical user interface with file picker and real-time conversion
- Support for MIDI channel override
- Configurable OFF note collision filtering with per-note exceptions
- Cross-platform support (macOS, Linux, Windows)
- Comprehensive unit tests for core functionality

### Features
- Convert MIDI files to StageTraxx format
- Handle tempo changes and timing drift
- Filter out simultaneous note on/off collisions
- Export results to clipboard or file