//! Round-trip serialization tests, gated behind the `serde` feature.
//!
//! Run with: `cargo test --features serde`.

#![cfg(feature = "serde")]

use harmonia::{
    detect_key, suggest_next_chords, suggest_scales_for_bracket, Accidental, Alteration, Chord,
    ChordQuality, ChordSuggestion, ChordSuggestionResult, DiatonicChord, Interval, Key, KeyMatch,
    Letter, Note, PitchClass, RomanNumeral, Scale, ScaleGroup, ScaleKind, ScaleSuggestion,
    SuggestionCategory,
};
use serde::{de::DeserializeOwned, Serialize};

fn roundtrip<T>(value: &T) -> T
where
    T: Serialize + DeserializeOwned,
{
    let json = serde_json::to_string(value).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

fn assert_roundtrip<T>(value: T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug + Clone,
{
    let result: T = roundtrip(&value);
    assert_eq!(result, value);
}

#[test]
fn pitch_class_serializes_as_integer() {
    let json = serde_json::to_string(&PitchClass::G).unwrap();
    assert_eq!(json, "7");
    let parsed: PitchClass = serde_json::from_str("7").unwrap();
    assert_eq!(parsed, PitchClass::G);
}

#[test]
fn pitch_class_deserialization_normalizes_out_of_range() {
    // Mirrors PitchClass::new — wraps mod 12.
    let parsed: PitchClass = serde_json::from_str("19").unwrap();
    assert_eq!(parsed.value(), 7);
}

#[test]
fn interval_serializes_as_integer() {
    let json = serde_json::to_string(&Interval::PERFECT_FIFTH).unwrap();
    assert_eq!(json, "7");
    let parsed: Interval = serde_json::from_str("7").unwrap();
    assert_eq!(parsed, Interval::PERFECT_FIFTH);
}

#[test]
fn note_round_trips() {
    let n = Note::new(Letter::F, Accidental::Sharp);
    assert_roundtrip(n);
}

#[test]
fn every_letter_and_accidental_round_trips() {
    for letter in [
        Letter::C,
        Letter::D,
        Letter::E,
        Letter::F,
        Letter::G,
        Letter::A,
        Letter::B,
    ] {
        assert_roundtrip(letter);
    }
    for acc in [
        Accidental::DoubleFlat,
        Accidental::Flat,
        Accidental::Natural,
        Accidental::Sharp,
        Accidental::DoubleSharp,
    ] {
        assert_roundtrip(acc);
    }
}

#[test]
fn chord_round_trips() {
    for q in ChordQuality::ALL {
        for pc in 0..12 {
            assert_roundtrip(Chord::new(PitchClass::new(pc), *q));
        }
    }
}

#[test]
fn scale_round_trips() {
    for kind in ScaleKind::ALL {
        for pc in 0..12 {
            assert_roundtrip(Scale::new(PitchClass::new(pc), *kind));
        }
    }
}

#[test]
fn scale_group_round_trips() {
    for g in ScaleGroup::ALL {
        assert_roundtrip(*g);
    }
}

#[test]
fn key_round_trips() {
    assert_roundtrip(Key::new(PitchClass::C));
    assert_roundtrip(Key::new(PitchClass::F_SHARP));
}

#[test]
fn diatonic_chord_round_trips() {
    let dc = DiatonicChord::new(
        Interval::PERFECT_FIFTH,
        ChordQuality::Dominant7,
        RomanNumeral::new(5, ChordQuality::Dominant7),
    );
    assert_roundtrip(dc);
}

#[test]
fn roman_numeral_round_trips_basic() {
    assert_roundtrip(RomanNumeral::new(1, ChordQuality::Major));
    assert_roundtrip(RomanNumeral::flat(3, ChordQuality::Major));
    assert_roundtrip(RomanNumeral::sharp(4, ChordQuality::Minor));
    assert_roundtrip(RomanNumeral::new(7, ChordQuality::HalfDiminished7));
}

#[test]
fn roman_numeral_round_trips_secondary() {
    let v7_of_ii = RomanNumeral::new(5, ChordQuality::Dominant7)
        .secondary_of(RomanNumeral::new(2, ChordQuality::Minor));
    assert_roundtrip(v7_of_ii);
}

#[test]
fn alteration_round_trips() {
    assert_roundtrip(Alteration::Flat);
    assert_roundtrip(Alteration::Sharp);
}

#[test]
fn suggestion_category_serializes_as_lowercase() {
    let json = serde_json::to_string(&SuggestionCategory::Diatonic).unwrap();
    assert_eq!(json, "\"diatonic\"");
    let parsed: SuggestionCategory = serde_json::from_str("\"borrowed\"").unwrap();
    assert_eq!(parsed, SuggestionCategory::Borrowed);
}

#[test]
fn key_match_round_trips() {
    let progression: Vec<Chord> = ["C", "G", "F"]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();
    let results = detect_key(&progression);
    let original: KeyMatch = results[0];
    let json = serde_json::to_string(&original).unwrap();
    let parsed: KeyMatch = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, original);
}

#[test]
fn scale_suggestion_round_trips() {
    let g: Chord = "G".parse().unwrap();
    let am: Chord = "Am".parse().unwrap();
    let suggestions = suggest_scales_for_bracket(Some(g), Some(am));
    let original = &suggestions[0];

    let json = serde_json::to_string(original).unwrap();
    let parsed: ScaleSuggestion = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.scale, original.scale);
    assert_eq!(parsed.fits_prev, original.fits_prev);
    assert_eq!(parsed.fits_next, original.fits_next);
    assert_eq!(parsed.reasoning, original.reasoning);
}

#[test]
fn chord_suggestion_result_round_trips() {
    let history: Vec<Chord> = ["C", "Am", "F", "G7"]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();
    let result = suggest_next_chords(&history, None);

    let json = serde_json::to_string(&result).unwrap();
    let parsed: ChordSuggestionResult = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.key, result.key);
    assert_eq!(parsed.second_key, result.second_key);
    assert_eq!(parsed.suggestions.len(), result.suggestions.len());
    for (a, b) in parsed.suggestions.iter().zip(result.suggestions.iter()) {
        assert_chord_suggestion_eq(a, b);
    }
}

fn assert_chord_suggestion_eq(a: &ChordSuggestion, b: &ChordSuggestion) {
    assert_eq!(a.chord, b.chord);
    assert_eq!(a.roman, b.roman);
    assert_eq!(a.reason, b.reason);
    assert_eq!(a.category, b.category);
    assert_eq!(a.tonality_effect, b.tonality_effect);
}
