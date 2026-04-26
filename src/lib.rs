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

pub mod interval;
pub mod note;
pub mod pitch;
pub mod spelling;

pub use interval::Interval;
pub use note::{Accidental, Letter, Note};
pub use pitch::PitchClass;
pub use spelling::spell_heptatonic;
