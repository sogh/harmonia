//! # harmonia
//!
//! Instrument-agnostic music theory primitives.
//!
//! This crate is being grown out of the JavaScript theory layer in
//! [fretboard-explorer](https://github.com/sogh/fretboard-explorer). The
//! design separates abstract theory (pitch classes, intervals, scales,
//! chords, keys) from any instrument-specific layout (frets, keys, valves).
//!
//! ## Quick tour
//!
//! ```
//! use harmonia::{Chord, ChordQuality, Key, PitchClass, Scale, ScaleKind};
//!
//! // Pitch-class arithmetic.
//! let g = PitchClass::G;
//! let v_chord = Chord::new(g, ChordQuality::Dominant7);
//! assert_eq!(v_chord.to_string(), "G7");
//!
//! // Roman-numeral analysis in a key.
//! let c_major = Key::new(PitchClass::C);
//! let roman = c_major.roman_for(v_chord).unwrap();
//! assert_eq!(roman.to_string(), "V7");
//!
//! // Diatonic spelling.
//! let g_major: Scale = "G Ionian".parse().unwrap();
//! let spelled = g_major.spelled().unwrap();
//! let labels: Vec<String> = spelled.iter().map(|n| n.to_string()).collect();
//! assert_eq!(labels, vec!["G", "A", "B", "C", "D", "E", "F♯"]);
//!
//! // Parse chord progressions and analyse them.
//! let progression: Vec<Chord> = ["C", "Am", "F", "G7"]
//!     .iter()
//!     .map(|s| s.parse().unwrap())
//!     .collect();
//! let detected = harmonia::detect_key(&progression);
//! assert_eq!(detected[0].key, Key::new(PitchClass::C));
//! ```
//!
//! ## Foundation layer
//!
//! - [`PitchClass`] — one of the twelve octave-equivalent tones (`0..12`).
//! - [`Interval`] — a non-negative number of semitones, with named constants.
//! - [`Note`] — a [`Letter`] plus an [`Accidental`]; carries enharmonic
//!   spelling on top of a pitch class.
//! - [`spell_heptatonic`] — assigns natural letters A–G across a seven-
//!   note scale so each appears exactly once.
//!
//! ## Catalogue
//!
//! - [`ScaleKind`] — the 16 scales from `theory.js` (modes, pentatonics,
//!   harmonic/melodic minor, symmetric scales).
//! - [`Scale`] — a [`ScaleKind`] anchored at a root [`PitchClass`].
//! - [`ChordQuality`] — 12 chord qualities (6 triads + 6 sevenths).
//! - [`Chord`] — a [`ChordQuality`] anchored at a root [`PitchClass`].
//!
//! ## Analysis
//!
//! - [`Key`] — a (currently always major) key. Provides diatonic chord
//!   templates and Roman-numeral labels via [`Key::roman_for`].
//! - [`detect_key`] — score a chord progression against every major key,
//!   returning candidates ranked by fit.
//! - [`suggest_scales_for_bracket`] — rank scales that fit a lead-line
//!   gap between two chords.
//! - [`suggest_next_chords`] — context-aware chord recommender with
//!   diatonic, resolution, borrowed, secondary, relative, and chromatic
//!   categories.
//!
//! ## Crate features
//!
//! - **`serde`** *(off by default)* — derives `Serialize` and `Deserialize`
//!   on every public data type. `PitchClass` and `Interval` serialize as
//!   integers; `SuggestionCategory` as a lowercase string; everything else
//!   uses the default `serde` representation.

pub mod analysis;
pub mod chord;
pub mod interval;
pub mod key;
pub mod note;
pub mod parse;
pub mod pitch;
pub mod roman;
pub mod scale;
pub mod spelling;

pub use analysis::{
    detect_key, suggest_next_chords, suggest_scales_for_bracket, ChordSuggestion,
    ChordSuggestionResult, KeyMatch, ScaleSuggestion, SuggestionCategory,
};
pub use chord::{Chord, ChordQuality};
pub use interval::Interval;
pub use key::{DiatonicChord, Key};
pub use note::{Accidental, Letter, Note};
pub use parse::ParseError;
pub use pitch::PitchClass;
pub use roman::{Alteration, RomanNumeral};
pub use scale::{Scale, ScaleGroup, ScaleKind};
pub use spelling::spell_heptatonic;
