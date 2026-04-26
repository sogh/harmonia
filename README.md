# harmonia

Instrument-agnostic music theory primitives in Rust.

`harmonia` gives you typed, parseable, analyzable musical values: pitch
classes, intervals, scales, chords, keys, Roman numerals, and a small
catalogue of analysis routines. There's no fretboard, no keyboard, no
audio — just the theory you'd write on a chalkboard, expressed as Rust
values you can compose into whatever instrument or interface layer you
need on top.

## Why harmonia?

- **It's the in-between layer most music libraries skip.** Audio
  crates handle synthesis. Notation crates handle rendering. `harmonia`
  handles the theory between them: which chords are diatonic, what the
  V7 of `ii` is, whether a progression sits in C major or A minor.

- **Strong types over strings.** A `Chord` is not a `String`; a
  `RomanNumeral` is not a `String`. You can still parse them from
  user input — every public type implements `FromStr` and round-trips
  through `Display` — but once parsed they're values you can match on,
  hash, and pass through type-checked APIs.

- **Real analysis built in.** Detect a key from a chord progression,
  label chords with Roman numerals, find scales that fit over a chord
  transition, and generate context-aware next-chord suggestions across
  six categories (diatonic, resolution, borrowed, secondary, relative,
  chromatic).

- **Pluggable.** Optional `serde` for persistence; zero required
  dependencies otherwise.

The theory model is extracted from [fretboard-explorer], where the
same primitives drove a guitar fretboard, a piano keyboard, and a
trumpet fingering chart. The split was useful in JavaScript; it's
useful in Rust too. Plausible uses: chord-chart generators, practice
trainers, theory-teaching aids, exercise generators, and the brain of
any instrument-specific layer.

[fretboard-explorer]: https://github.com/sogh/fretboard-explorer

## Examples

### Parse a progression and detect the key

```rust
use harmonia::{detect_key, Chord};

let progression: Vec<Chord> = ["C", "Am", "Dm", "G7"]
    .iter().map(|s| s.parse().unwrap()).collect();

let candidates = detect_key(&progression);
let best = &candidates[0];
println!("{}: {}/{} chords fit", best.key, best.matched, best.total);
// → C major: 4/4 chords fit
```

### Roman-numeral analysis in a key

```rust
use harmonia::{Chord, Key, PitchClass};

let c_major = Key::new(PitchClass::C);
for symbol in ["Am", "F", "G7", "C7", "F#"] {
    let chord: Chord = symbol.parse().unwrap();
    let roman = c_major.roman_for(chord)
        .map(|r| r.to_string())
        .unwrap_or_else(|| "—".into());
    println!("{symbol:>4} → {roman}");
}
//   Am → vi
//    F → IV
//   G7 → V7
//   C7 → I7        (the I-as-dom7 fuzzy fallback)
//   F# → —          (out of key)
```

### Spell a scale with diatonic letter names

```rust
use harmonia::Scale;

let g_major: Scale = "G Ionian".parse().unwrap();
let labels: Vec<String> = g_major.spelled().unwrap()
    .iter().map(|n| n.to_string()).collect();
assert_eq!(labels, ["G", "A", "B", "C", "D", "E", "F♯"]);
```

The spelling algorithm picks the right accidentals so each natural
letter A–G appears exactly once — no `A♯` next to an `A`.

### Suggest the next chord

```rust
use harmonia::{suggest_next_chords, Chord};

let history: Vec<Chord> = ["C", "Am", "F"]
    .iter().map(|s| s.parse().unwrap()).collect();

let result = suggest_next_chords(&history, None);
let key = result.key.unwrap();
println!("Key: {}\n", key.key);

for s in result.suggestions.iter().take(6) {
    let roman = s.roman.as_ref().map(|r| r.to_string()).unwrap_or_default();
    println!("  {} ({}) — {}", s.chord, roman, s.reason);
}
// Key: C major
//
//   C (I) — diatonic in C major
//   Dm (ii) — diatonic in C major
//   Em (iii) — diatonic in C major
//   F (IV) — diatonic in C major
//   G (V) — diatonic in C major
//   Am (vi) — diatonic in C major
```

Suggestions also span resolution moves (V→I, ii→V, IV→V), borrowed
chords (♭III, iv, ♭VI, ♭VII), secondary dominants (V7/ii, V7/iii, …),
relative major/minor, and chromatic motion — each tagged with its
category so a UI can group them.

### Find scales that fit over a chord transition

```rust
use harmonia::{suggest_scales_for_bracket, Chord};

let g: Chord = "G".parse().unwrap();
let am: Chord = "Am".parse().unwrap();

let suggestions = suggest_scales_for_bracket(Some(g), Some(am));
for s in suggestions.iter().take(3) {
    println!("{:<22} {}", s.scale.to_string(), s.reasoning);
}
// G Ionian (major)       G and Am are both diatonic to G Ionian (major)
// G Mixolydian           G and Am are both diatonic to G Mixolydian
// A Dorian               G and Am are both diatonic to A Dorian
```

(G-rooted modes win the prev-chord bias; A Dorian sneaks ahead of
C Ionian on the next-chord bias.)

### Persist results (with the `serde` feature)

```rust,ignore
let json = serde_json::to_string(&result)?;
let restored: harmonia::ChordSuggestionResult = serde_json::from_str(&json)?;
```

## What's in

**Foundation** — `PitchClass`, `Interval`, `Note` (`Letter` +
`Accidental`), and `spell_heptatonic` for diatonic letter spelling.

**Catalogue** — `ScaleKind` (16 scales: modes, pentatonics,
harmonic/melodic minor, symmetric) and `ChordQuality` (12 qualities:
6 triads + 6 sevenths), each with `Scale` / `Chord` companions.

**Analysis** — `Key` with diatonic chord templates, `RomanNumeral`
(typed with alteration, degree, quality, and secondary-of),
`detect_key`, `suggest_scales_for_bracket`, `suggest_next_chords`.

Every public data type implements `FromStr`, `Display`, `PartialEq`,
and `Hash`.

## Crate features

- **`serde`** *(off by default)* — derives `Serialize`/`Deserialize`
  on every public data type. `PitchClass` and `Interval` serialize as
  integers; `SuggestionCategory` as a lowercase string. Enable with:

  ```toml
  harmonia = { version = "0.1", features = ["serde"] }
  ```

## License

MIT
