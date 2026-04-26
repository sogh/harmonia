//! Enharmonic spelling for diatonic scales.
//!
//! A pitch class on its own is ambiguous — `pc = 6` could be `F♯` or `G♭`,
//! and the right choice depends on key context. For seven-note scales, the
//! standard convention is that each natural letter A–G appears exactly once;
//! the spelling problem is then to assign the right accidental to each
//! letter. This module ports `spellScale()` from `theory.js`.

use crate::interval::Interval;
use crate::note::{Accidental, Letter, Note};
use crate::pitch::PitchClass;

const LETTER_ORDER: [Letter; 7] = [
    Letter::C,
    Letter::D,
    Letter::E,
    Letter::F,
    Letter::G,
    Letter::A,
    Letter::B,
];

/// Penalty per single accidental.
const SINGLE_ACCIDENTAL_COST: i32 = 1;
/// Heavy penalty per double accidental — disprefers, but allows.
const DOUBLE_ACCIDENTAL_COST: i32 = 100;

/// Candidate root letters for spelling a scale rooted on `pc`.
///
/// For natural roots (C, D, E, F, G, A, B) the only candidate is the
/// matching letter. For the five enharmonic roots (C♯/D♭, D♯/E♭, F♯/G♭,
/// G♯/A♭, A♯/B♭) we try both letter spellings and pick whichever scores
/// lower; ties favor the sharp spelling.
fn candidate_root_letters(pc: PitchClass) -> &'static [Letter] {
    match pc.value() {
        0 => &[Letter::C],
        1 => &[Letter::C, Letter::D],
        2 => &[Letter::D],
        3 => &[Letter::D, Letter::E],
        4 => &[Letter::E],
        5 => &[Letter::F],
        6 => &[Letter::F, Letter::G],
        7 => &[Letter::G],
        8 => &[Letter::G, Letter::A],
        9 => &[Letter::A],
        10 => &[Letter::A, Letter::B],
        11 => &[Letter::B],
        _ => unreachable!("PitchClass invariant: value < 12"),
    }
}

/// Spell a seven-note scale so each natural letter A–G appears exactly once.
///
/// Returns the spelled note at each scale degree (index 0 = root). Returns
/// `None` if no valid spelling exists with at most double accidentals — in
/// practice this only happens for inputs that aren't really diatonic.
///
/// For non-heptatonic scales (pentatonic, blues, diminished, chromatic),
/// fall back to [`PitchClass::default_name`] for sharp-only labels.
pub fn spell_heptatonic(root: PitchClass, steps: &[Interval; 7]) -> Option<[Note; 7]> {
    let root_pc = root.value() as i16;
    let candidates = candidate_root_letters(root);

    let mut best: Option<[Note; 7]> = None;
    let mut best_score: i32 = i32::MAX;

    for &root_letter in candidates {
        let r_idx = LETTER_ORDER
            .iter()
            .position(|&l| l == root_letter)
            .expect("LETTER_ORDER contains every Letter");

        let mut notes = [Note::natural(Letter::C); 7];
        let mut score: i32 = 0;
        let mut ok = true;

        for i in 0..7 {
            let pc = (root_pc + steps[i].semitones() as i16).rem_euclid(12);
            let letter = LETTER_ORDER[(r_idx + i) % 7];
            let letter_pc = letter.pitch_class().value() as i16;
            let mut diff = (pc - letter_pc).rem_euclid(12);
            if diff > 6 {
                diff -= 12;
            }

            let acc = match diff {
                0 => Accidental::Natural,
                1 => {
                    score += SINGLE_ACCIDENTAL_COST;
                    Accidental::Sharp
                }
                -1 => {
                    score += SINGLE_ACCIDENTAL_COST;
                    Accidental::Flat
                }
                2 => {
                    score += DOUBLE_ACCIDENTAL_COST;
                    Accidental::DoubleSharp
                }
                -2 => {
                    score += DOUBLE_ACCIDENTAL_COST;
                    Accidental::DoubleFlat
                }
                _ => {
                    ok = false;
                    break;
                }
            };
            notes[i] = Note::new(letter, acc);
        }

        if ok && score < best_score {
            best = Some(notes);
            best_score = score;
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spelled(root: PitchClass, steps: [u16; 7]) -> Option<String> {
        let intervals: [Interval; 7] = steps.map(Interval::new);
        spell_heptatonic(root, &intervals).map(|notes| {
            notes
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        })
    }

    // Step patterns for each mode (semitones from root).
    const IONIAN: [u16; 7] = [0, 2, 4, 5, 7, 9, 11];
    const DORIAN: [u16; 7] = [0, 2, 3, 5, 7, 9, 10];
    const PHRYGIAN: [u16; 7] = [0, 1, 3, 5, 7, 8, 10];
    const LYDIAN: [u16; 7] = [0, 2, 4, 6, 7, 9, 11];
    const AEOLIAN: [u16; 7] = [0, 2, 3, 5, 7, 8, 10];
    const HARMONIC_MINOR: [u16; 7] = [0, 2, 3, 5, 7, 8, 11];

    // Tests ported from fretboard-explorer/theory.test.js.

    #[test]
    fn seven_note_scales_have_distinct_natural_letters() {
        assert_eq!(
            spelled(PitchClass::G, DORIAN).as_deref(),
            Some("G A B♭ C D E F")
        );
        assert_eq!(
            spelled(PitchClass::G, IONIAN).as_deref(),
            Some("G A B C D E F♯")
        );
        assert_eq!(
            spelled(PitchClass::D, PHRYGIAN).as_deref(),
            Some("D E♭ F G A B♭ C")
        );
        assert_eq!(
            spelled(PitchClass::F, LYDIAN).as_deref(),
            Some("F G A B C D E")
        );
        assert_eq!(
            spelled(PitchClass::F, IONIAN).as_deref(),
            Some("F G A B♭ C D E")
        );
        assert_eq!(
            spelled(PitchClass::G, AEOLIAN).as_deref(),
            Some("G A B♭ C D E♭ F")
        );
        assert_eq!(
            spelled(PitchClass::G, HARMONIC_MINOR).as_deref(),
            Some("G A B♭ C D E♭ F♯")
        );
    }

    #[test]
    fn enharmonic_root_picks_cleaner_flat_spelling() {
        // A♯ Dorian should resolve to B♭ Dorian.
        assert_eq!(
            spelled(PitchClass::A_SHARP, DORIAN).as_deref(),
            Some("B♭ C D♭ E♭ F G A♭")
        );
        // F♯ Lydian should resolve to G♭ Lydian.
        assert_eq!(
            spelled(PitchClass::F_SHARP, LYDIAN).as_deref(),
            Some("G♭ A♭ B♭ C D♭ E♭ F")
        );
    }

    #[test]
    fn sharp_leaning_enharmonic_roots_stay_sharp() {
        // C♯ Dorian: sharp letters score better than D♭ Dorian (which
        // would need C♭ as the seventh).
        assert_eq!(
            spelled(PitchClass::C_SHARP, DORIAN).as_deref(),
            Some("C♯ D♯ E F♯ G♯ A♯ B")
        );
    }

    #[test]
    fn g_dorian_avoids_a_sharp() {
        let notes = spell_heptatonic(
            PitchClass::G,
            &DORIAN.map(Interval::new),
        )
        .expect("G Dorian must spell");
        let labels: Vec<String> = notes.iter().map(|n| n.to_string()).collect();
        assert!(
            !labels.iter().any(|s| s == "A♯"),
            "G Dorian should not contain A♯, got {labels:?}"
        );
        assert!(labels.iter().any(|s| s == "A"));
        assert!(labels.iter().any(|s| s == "B♭"));
    }

    #[test]
    fn spelled_notes_have_matching_pitch_classes() {
        // Whatever spelling we pick, each spelled note must have the same
        // pitch class as the corresponding scale tone.
        let intervals = DORIAN.map(Interval::new);
        let notes = spell_heptatonic(PitchClass::G, &intervals).unwrap();
        for (i, note) in notes.iter().enumerate() {
            assert_eq!(
                note.pitch_class(),
                PitchClass::G + intervals[i],
                "degree {i} pitch class mismatch"
            );
        }
    }
}
