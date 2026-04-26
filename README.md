# harmonia

Instrument-agnostic music theory primitives in Rust. A port of the theory layer from [fretboard-explorer](https://github.com/sogh/fretboard-explorer): pitch classes, intervals, scales, chords, keys, and progressions — without any instrument-specific layout code.

## Status

Early — scaffolding only. The crate is being grown out of the JS abstractions in `fretboard-explorer`.

## Planned scope

- Pitch classes and enharmonic spelling
- Intervals and chord qualities
- Scales (modes, pentatonics, harmonic/melodic minor, symmetric)
- Key detection and Roman numeral analysis
- Chord-progression suggestions (diatonic, borrowed, secondary, resolution, chromatic)
- Voicing data structures (close, open, split) — independent of any instrument
- Sequence/step models for practice routines

## License

MIT
