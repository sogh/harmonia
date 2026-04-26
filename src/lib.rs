//! # harmonia
//!
//! Instrument-agnostic music theory primitives.
//!
//! This crate is being grown out of the JavaScript theory layer in
//! [fretboard-explorer](https://github.com/sogh/fretboard-explorer). The
//! design separates abstract theory (pitch classes, intervals, scales,
//! chords, keys) from any instrument-specific layout (frets, keys, valves).
//!
//! ## Foundation layer
//!
//! - [`PitchClass`] — one of the twelve octave-equivalent tones (`0..12`).
//! - [`Interval`] — a non-negative number of semitones, with named constants.
//! - [`Note`] — a [`Letter`] plus an [`Accidental`]; carries enharmonic
//!   spelling on top of a pitch class.
//! - [`spell_heptatonic`](spelling::spell_heptatonic) — assigns natural
//!   letters A–G across a seven-note scale so each appears exactly once.
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
//! - [`detect_key`](analysis::detect_key) — score a chord progression
//!   against every major key, returning candidates ranked by fit.
//! - [`suggest_scales_for_bracket`](analysis::suggest_scales_for_bracket)
//!   — rank scales that fit a lead-line gap between two chords.
//! - [`suggest_next_chords`](analysis::suggest_next_chords) — context-
//!   aware chord recommender with diatonic, resolution, borrowed,
//!   secondary, relative, and chromatic categories.

pub mod analysis;
pub mod chord;
pub mod interval;
pub mod key;
pub mod note;
pub mod pitch;
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
pub use pitch::PitchClass;
pub use scale::{Scale, ScaleGroup, ScaleKind};
pub use spelling::spell_heptatonic;
