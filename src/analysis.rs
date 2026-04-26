//! Higher-level inference functions over chords, scales, and keys.

use crate::chord::Chord;
use crate::key::Key;
use crate::pitch::PitchClass;

/// One row of [`detect_key`]'s output: a candidate key, the count of input
/// chords that fit it diatonically, and the total chord count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyMatch {
    pub key: Key,
    /// Number of input chords that have a Roman-numeral role in `key`,
    /// counting both strict diatonic chords and the major-degree dominant
    /// 7th functional equivalent (e.g. C7 in C major → I7).
    pub matched: usize,
    /// Total number of input chords. Useful for computing a ratio.
    pub total: usize,
}

impl KeyMatch {
    /// Fraction of input chords that fit this key, in `0.0..=1.0`.
    /// Returns `0.0` when there are no input chords.
    pub fn ratio(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.matched as f64 / self.total as f64
        }
    }
}

/// Score every major key by how many of `chords` fit diatonically.
///
/// Returns all 12 major keys sorted by `matched` descending, ties broken
/// by ascending tonic pitch class. The "fit" check uses
/// [`Key::roman_for`], which accepts both strict diatonic chords and the
/// major-degree dominant-7th functional equivalent.
///
/// Returns an empty vector when `chords` is empty. Otherwise the result
/// always has exactly 12 entries — callers typically take the first
/// element (best fit) and treat ties (`results[0].matched ==
/// results[1].matched`) as ambiguous.
///
/// Ports `detectKey` from `theory.js`.
pub fn detect_key(chords: &[Chord]) -> Vec<KeyMatch> {
    if chords.is_empty() {
        return Vec::new();
    }
    let total = chords.len();

    let mut results: Vec<KeyMatch> = (0..12)
        .map(|tonic_pc| {
            let key = Key::new(PitchClass::new(tonic_pc));
            let matched = chords
                .iter()
                .filter(|c| key.roman_for(**c).is_some())
                .count();
            KeyMatch {
                key,
                matched,
                total,
            }
        })
        .collect();

    results.sort_by(|a, b| {
        b.matched
            .cmp(&a.matched)
            .then_with(|| a.key.tonic.value().cmp(&b.key.tonic.value()))
    });
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chord::ChordQuality;

    fn chord(root: PitchClass, quality: ChordQuality) -> Chord {
        Chord::new(root, quality)
    }

    #[test]
    fn empty_input_returns_empty_vec() {
        assert!(detect_key(&[]).is_empty());
    }

    #[test]
    fn full_results_have_twelve_entries() {
        let progression = [chord(PitchClass::C, ChordQuality::Major)];
        assert_eq!(detect_key(&progression).len(), 12);
    }

    #[test]
    fn classic_c_major_progression_picks_c() {
        // I – ii – V7 – I in C major.
        let progression = [
            chord(PitchClass::C, ChordQuality::Major),
            chord(PitchClass::D, ChordQuality::Minor),
            chord(PitchClass::G, ChordQuality::Dominant7),
            chord(PitchClass::C, ChordQuality::Major),
        ];
        let results = detect_key(&progression);
        assert_eq!(results[0].key, Key::new(PitchClass::C));
        assert_eq!(results[0].matched, 4);
        assert_eq!(results[0].total, 4);
        assert_eq!(results[0].ratio(), 1.0);
    }

    #[test]
    fn results_are_sorted_descending_by_match_count() {
        let progression = [
            chord(PitchClass::C, ChordQuality::Major),
            chord(PitchClass::F, ChordQuality::Major),
            chord(PitchClass::G, ChordQuality::Major),
        ];
        let results = detect_key(&progression);
        for window in results.windows(2) {
            assert!(window[0].matched >= window[1].matched);
        }
    }

    #[test]
    fn ties_broken_by_ascending_tonic() {
        // A single C major chord is the I in C, IV in G, V in F — all
        // score 1. The sort tie-breaks by ascending tonic pc, so C wins.
        let progression = [chord(PitchClass::C, ChordQuality::Major)];
        let results = detect_key(&progression);
        // Take all entries that tied for top.
        let top_score = results[0].matched;
        let tied_tonics: Vec<u8> = results
            .iter()
            .take_while(|m| m.matched == top_score)
            .map(|m| m.key.tonic.value())
            .collect();
        // Order should be ascending pc.
        let mut sorted = tied_tonics.clone();
        sorted.sort();
        assert_eq!(tied_tonics, sorted);
        // Top-scoring keys are exactly C(0), F(5), G(7) — the keys in
        // which C major appears.
        assert_eq!(tied_tonics, vec![0, 5, 7]);
    }

    #[test]
    fn dom7_on_i_counts_as_diatonic() {
        // C7 isn't strictly diatonic in C major (Imaj7 is), but the
        // detector's functional matching counts it via the I7 fallback.
        let progression = [chord(PitchClass::C, ChordQuality::Dominant7)];
        let results = detect_key(&progression);
        // C major must score 1 here (and tie with F and G as the V7s).
        let c_match = results.iter().find(|m| m.key.tonic == PitchClass::C).unwrap();
        assert_eq!(c_match.matched, 1);
    }

    #[test]
    fn out_of_key_chord_lowers_score() {
        // Same C-Dm-G7-C as before but with a non-diatonic F♯ major
        // dropped in. C major should still win, but no longer at 1.0.
        let progression = [
            chord(PitchClass::C, ChordQuality::Major),
            chord(PitchClass::D, ChordQuality::Minor),
            chord(PitchClass::F_SHARP, ChordQuality::Major),
            chord(PitchClass::G, ChordQuality::Dominant7),
            chord(PitchClass::C, ChordQuality::Major),
        ];
        let results = detect_key(&progression);
        assert_eq!(results[0].key, Key::new(PitchClass::C));
        assert_eq!(results[0].matched, 4);
        assert_eq!(results[0].total, 5);
        assert!(results[0].ratio() < 1.0);
    }

    #[test]
    fn purely_chromatic_input_has_zero_top_score() {
        // No major key contains all of C, C♯, D simultaneously; the best
        // matched count should still be > 0 (since each chord fits some
        // keys), but no key fits all three.
        let progression = [
            chord(PitchClass::C, ChordQuality::Major),
            chord(PitchClass::C_SHARP, ChordQuality::Major),
            chord(PitchClass::D, ChordQuality::Major),
        ];
        let results = detect_key(&progression);
        assert!(results[0].matched < 3);
    }

    #[test]
    fn ratio_is_zero_when_total_is_zero() {
        let m = KeyMatch {
            key: Key::new(PitchClass::C),
            matched: 0,
            total: 0,
        };
        assert_eq!(m.ratio(), 0.0);
    }
}
