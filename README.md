# harmonia

Instrument-agnostic music theory primitives in Rust. A port of the theory layer from [fretboard-explorer](https://github.com/sogh/fretboard-explorer): pitch classes, intervals, scales, chords, keys, and progressions — without any instrument-specific layout code.

## What's in

**Foundation**
- `PitchClass` (mod-12 arithmetic) and `Interval` with named constants
- `Note` (`Letter` + `Accidental`) preserving enharmonic spelling
- `spell_heptatonic` — diatonic letter spelling for seven-note scales

**Catalogue**
- `ScaleKind` and `Scale` — 16 scales (modes, pentatonics, harmonic/melodic minor, symmetric)
- `ChordQuality` and `Chord` — 12 chord qualities (6 triads + 6 sevenths)

**Analysis**
- `Key` with diatonic chord templates and Roman-numeral labels
- `RomanNumeral` — typed numeral with alteration, degree, quality, and secondary-of
- `detect_key` — score a progression against every major key
- `suggest_scales_for_bracket` — rank scales over a chord transition
- `suggest_next_chords` — context-aware chord recommender (diatonic / resolution / borrowed / secondary / relative / chromatic)

Every data type implements `FromStr` so you can parse `"Cm7"`, `"G Ionian"`, `"♭III"`, etc., and round-trip through `Display`.

## Quick example

```rust
use harmonia::{Chord, Key, PitchClass, suggest_next_chords};

let history: Vec<Chord> = ["C", "Am", "F", "G7"]
    .iter().map(|s| s.parse().unwrap()).collect();
let result = suggest_next_chords(&history, None);

let key = result.key.unwrap();
assert_eq!(key.key, Key::new(PitchClass::C));

let c: Chord = "C".parse().unwrap();
let next = result.suggestions.iter().find(|s| s.chord == c).unwrap();
assert_eq!(next.roman.as_ref().map(|r| r.to_string()).as_deref(), Some("I"));
```

## Crate features

- **`serde`** *(off by default)* — derives `Serialize`/`Deserialize` on every public data type. `PitchClass` and `Interval` serialize as integers; `SuggestionCategory` as a lowercase string. Enable with:

  ```toml
  harmonia = { version = "0.1", features = ["serde"] }
  ```

## License

MIT
