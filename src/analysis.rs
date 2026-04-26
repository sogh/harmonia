//! Higher-level inference functions over chords, scales, and keys.

use crate::chord::Chord;
use crate::key::Key;
use crate::pitch::PitchClass;
use crate::scale::{Scale, ScaleGroup};

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

// ── Bracket scale suggestions ─────────────────────────────────────────

/// One suggestion from [`suggest_scales_for_bracket`]: a candidate scale
/// for a lead-line gap between two chords, with which side(s) of the
/// bracket it fits and a human-readable reasoning string.
#[derive(Clone, Debug)]
pub struct ScaleSuggestion {
    pub scale: Scale,
    /// True iff a previous chord was supplied **and** every one of its
    /// pitch classes lies in `scale`.
    pub fits_prev: bool,
    /// True iff a next chord was supplied **and** every one of its
    /// pitch classes lies in `scale`.
    pub fits_next: bool,
    /// One-line explanation of why this scale was suggested, suitable for
    /// rendering directly in a UI.
    pub reasoning: String,
}

impl ScaleSuggestion {
    /// True if both chords were supplied and both are covered by the scale.
    pub fn fits_both(&self) -> bool {
        self.fits_prev && self.fits_next
    }
}

fn scale_covers_chord(scale: &Scale, chord: &Chord) -> bool {
    chord.pitch_classes().all(|pc| scale.contains(pc))
}

/// Suggest scales for a lead-line gap bracketed by two chords.
///
/// Searches every (root, mode) pair across the seven church modes and
/// returns those that contain the chord tones of `prev`, `next`, or
/// both. Results are ranked by:
///
/// 1. Scales that fit *both* chords beat scales that fit only one.
/// 2. Scales rooted on the previous chord's root are preferred (a strong
///    bias toward the most idiomatic mode), then on the next chord's root.
/// 3. Within ties, simpler modes win — Ionian/Dorian over Locrian.
///
/// Returns an empty vector if both `prev` and `next` are `None`.
///
/// Ports `suggestScalesForBracket` from `theory.js`.
pub fn suggest_scales_for_bracket(
    prev: Option<Chord>,
    next: Option<Chord>,
) -> Vec<ScaleSuggestion> {
    if prev.is_none() && next.is_none() {
        return Vec::new();
    }

    let modes = ScaleGroup::Modes.scales();
    let mut scored: Vec<(i32, ScaleSuggestion)> = Vec::new();

    for tonic_pc in 0..12u8 {
        let tonic = PitchClass::new(tonic_pc);
        for (mode_rank, &kind) in modes.iter().enumerate() {
            let scale = Scale::new(tonic, kind);

            let fits_prev = prev.is_some_and(|c| scale_covers_chord(&scale, &c));
            let fits_next = next.is_some_and(|c| scale_covers_chord(&scale, &c));

            // Keep a scale only if it covers a chord that was actually
            // supplied. (When both chords are supplied, fitting either
            // side is enough; one-sided suggestions still help bridging.)
            let keep = match (prev.is_some(), next.is_some()) {
                (true, true) => fits_prev || fits_next,
                (true, false) => fits_prev,
                (false, true) => fits_next,
                (false, false) => false,
            };
            if !keep {
                continue;
            }

            // For ranking, a missing side counts as "satisfied" — that's
            // what gives the prev-only / next-only cases base priority 0.
            let satisfied_both =
                (prev.is_none() || fits_prev) && (next.is_none() || fits_next);
            let mut priority: i32 = if satisfied_both { 0 } else { 100 };
            if prev.is_some_and(|c| c.root == tonic) {
                priority -= 10;
            } else if next.is_some_and(|c| c.root == tonic) {
                priority -= 5;
            }
            priority += mode_rank as i32;

            let reasoning = bracket_reasoning(&scale, prev, next, fits_prev, fits_next);

            scored.push((
                priority,
                ScaleSuggestion {
                    scale,
                    fits_prev,
                    fits_next,
                    reasoning,
                },
            ));
        }
    }

    scored.sort_by_key(|(p, _)| *p);
    scored.into_iter().map(|(_, s)| s).collect()
}

fn bracket_reasoning(
    scale: &Scale,
    prev: Option<Chord>,
    next: Option<Chord>,
    fits_prev: bool,
    fits_next: bool,
) -> String {
    let scale_name = scale.to_string();
    match (prev, next, fits_prev, fits_next) {
        (Some(p), Some(n), true, true) if p.root == n.root => {
            format!("Both chords are diatonic to {scale_name}")
        }
        (Some(p), Some(n), true, true) => {
            format!("{p} and {n} are both diatonic to {scale_name}")
        }
        (Some(p), Some(_), true, false) => {
            format!("Fits the previous chord ({p}); bridge to the next")
        }
        (Some(_), Some(n), false, true) => {
            format!("Fits the next chord ({n}); approach from the previous")
        }
        (Some(p), None, true, false) => format!("Fits {p}"),
        (None, Some(n), false, true) => format!("Fits {n}"),
        _ => String::new(),
    }
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

    // ── suggest_scales_for_bracket ────────────────────────────────

    use crate::scale::ScaleKind;

    #[test]
    fn bracket_g_to_am_picks_a_diatonic_mode() {
        // Ported from theory.test.js. G major → A minor — both share
        // the white-key diatonic set, so the top suggestion should be
        // G Ionian, G Mixolydian, or C Ionian (all valid choices).
        let suggestions = suggest_scales_for_bracket(
            Some(chord(PitchClass::G, ChordQuality::Major)),
            Some(chord(PitchClass::A, ChordQuality::Minor)),
        );
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].fits_both());
        let top = suggestions[0].scale;
        let acceptable = [
            Scale::new(PitchClass::G, ScaleKind::Ionian),
            Scale::new(PitchClass::G, ScaleKind::Mixolydian),
            Scale::new(PitchClass::C, ScaleKind::Ionian),
        ];
        assert!(
            acceptable.contains(&top),
            "top suggestion {top} should be one of the diatonic modes"
        );
        assert!(!suggestions[0].reasoning.is_empty());
    }

    #[test]
    fn bracket_same_chord_on_both_sides() {
        let g = chord(PitchClass::G, ChordQuality::Major);
        let suggestions = suggest_scales_for_bracket(Some(g), Some(g));
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].fits_both());
        assert!(suggestions[0].reasoning.starts_with("Both chords"));
    }

    #[test]
    fn bracket_with_prev_only() {
        let suggestions = suggest_scales_for_bracket(
            Some(chord(PitchClass::G, ChordQuality::Major)),
            None,
        );
        assert!(!suggestions.is_empty());
        // Every suggestion really covers the previous chord, and no
        // suggestion claims to fit a next chord that wasn't supplied.
        assert!(suggestions.iter().all(|s| s.fits_prev));
        assert!(suggestions.iter().all(|s| !s.fits_next));
        assert!(suggestions[0].reasoning.starts_with("Fits "));
    }

    #[test]
    fn bracket_with_next_only() {
        let suggestions = suggest_scales_for_bracket(
            None,
            Some(chord(PitchClass::A, ChordQuality::Minor)),
        );
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().all(|s| !s.fits_prev));
        assert!(suggestions.iter().all(|s| s.fits_next));
    }

    #[test]
    fn bracket_with_no_chords_returns_empty() {
        assert!(suggest_scales_for_bracket(None, None).is_empty());
    }

    #[test]
    fn bracket_distant_chords_still_returns_partial_matches() {
        // C major → F♯ major — no major key contains both, but each
        // chord individually fits several modes, so partial-fit
        // suggestions should still appear.
        let suggestions = suggest_scales_for_bracket(
            Some(chord(PitchClass::C, ChordQuality::Major)),
            Some(chord(PitchClass::F_SHARP, ChordQuality::Major)),
        );
        assert!(!suggestions.is_empty());
        // Each suggestion fits at least one side.
        assert!(suggestions.iter().all(|s| s.fits_prev || s.fits_next));
    }

    #[test]
    fn bracket_results_sorted_by_priority() {
        // For G→Am, fits_both suggestions must come before fits-one-only
        // suggestions in the output.
        let suggestions = suggest_scales_for_bracket(
            Some(chord(PitchClass::G, ChordQuality::Major)),
            Some(chord(PitchClass::A, ChordQuality::Minor)),
        );
        let first_partial = suggestions.iter().position(|s| !s.fits_both());
        if let Some(idx) = first_partial {
            // Everything before that index must fit both.
            assert!(suggestions[..idx].iter().all(|s| s.fits_both()));
        }
    }

    #[test]
    fn bracket_prev_rooted_mode_beats_next_rooted() {
        // G major → A minor: the -10 prev bonus should rank a G-rooted
        // mode above an A-rooted mode that also fits both, all else equal.
        let suggestions = suggest_scales_for_bracket(
            Some(chord(PitchClass::G, ChordQuality::Major)),
            Some(chord(PitchClass::A, ChordQuality::Minor)),
        );
        let g_pos = suggestions.iter().position(|s| {
            s.scale == Scale::new(PitchClass::G, ScaleKind::Ionian)
        });
        let a_pos = suggestions.iter().position(|s| {
            s.scale == Scale::new(PitchClass::A, ScaleKind::Aeolian)
        });
        assert!(g_pos.is_some() && a_pos.is_some());
        assert!(
            g_pos.unwrap() < a_pos.unwrap(),
            "G Ionian should rank above A Aeolian (prev-rooted bonus)"
        );
    }
}
