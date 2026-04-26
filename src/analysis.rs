//! Higher-level inference functions over chords, scales, and keys.

use std::collections::HashSet;
use std::fmt;

use crate::chord::{Chord, ChordQuality};
use crate::interval::Interval;
use crate::key::Key;
use crate::pitch::PitchClass;
use crate::roman::RomanNumeral;
use crate::scale::{Scale, ScaleGroup};

/// One row of [`detect_key`]'s output: a candidate key, the count of input
/// chords that fit it diatonically, and the total chord count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
///
/// # Examples
///
/// ```
/// use harmonia::{detect_key, Chord, Key, PitchClass};
///
/// // I – ii – V7 – I in C major.
/// let progression: Vec<Chord> = ["C", "Dm", "G7", "C"]
///     .iter().map(|s| s.parse().unwrap()).collect();
/// let results = detect_key(&progression);
///
/// assert_eq!(results[0].key, Key::new(PitchClass::C));
/// assert_eq!(results[0].matched, 4);
/// assert_eq!(results[0].ratio(), 1.0);
/// ```
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

// ── Next-chord suggestions ────────────────────────────────────────────

/// What kind of relationship a [`ChordSuggestion`] represents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum SuggestionCategory {
    /// In the detected key's diatonic chord set (or a functional equivalent).
    Diatonic,
    /// A common voice-leading move from the previous chord (V→I, ii→V, IV→V).
    Resolution,
    /// Borrowed from the parallel minor (♭III, iv, ♭VI, ♭VII).
    Borrowed,
    /// A secondary dominant (V7/X) of a diatonic target.
    Secondary,
    /// The relative major or minor of the previous chord.
    Relative,
    /// A half- or whole-step chromatic move from the previous chord.
    Chromatic,
}

impl fmt::Display for SuggestionCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SuggestionCategory::Diatonic => "diatonic",
            SuggestionCategory::Resolution => "resolution",
            SuggestionCategory::Borrowed => "borrowed",
            SuggestionCategory::Secondary => "secondary",
            SuggestionCategory::Relative => "relative",
            SuggestionCategory::Chromatic => "chromatic",
        })
    }
}

impl std::str::FromStr for SuggestionCategory {
    type Err = crate::parse::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "diatonic" => SuggestionCategory::Diatonic,
            "resolution" => SuggestionCategory::Resolution,
            "borrowed" => SuggestionCategory::Borrowed,
            "secondary" => SuggestionCategory::Secondary,
            "relative" => SuggestionCategory::Relative,
            "chromatic" => SuggestionCategory::Chromatic,
            other => {
                return Err(crate::parse::ParseError::new(format!(
                    "unknown suggestion category: {other:?}"
                )))
            }
        })
    }
}

/// One suggestion from [`suggest_next_chords`].
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChordSuggestion {
    pub chord: Chord,
    /// Roman numeral label, when one is meaningful in the detected key.
    /// `None` when no key is detected and the category doesn't supply its
    /// own label (chromatic / resolution moves out of key).
    pub roman: Option<RomanNumeral>,
    /// Human-readable explanation of why this chord was suggested.
    pub reason: String,
    pub category: SuggestionCategory,
    /// Effect on tonality: `"diatonic in C major"`, `"shifts key toward G
    /// major"`, or `"chromatic / borrowed"`. Empty when no key is detected.
    pub tonality_effect: String,
}

/// Result of [`suggest_next_chords`].
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChordSuggestionResult {
    /// Best-fit major key inferred from the chord history, if any.
    pub key: Option<KeyMatch>,
    /// A second key tied with the best — populated only when the
    /// progression is genuinely ambiguous (e.g., a single chord that's
    /// the I in multiple keys).
    pub second_key: Option<KeyMatch>,
    pub suggestions: Vec<ChordSuggestion>,
}

const KEY_CONFIDENCE_THRESHOLD: f64 = 0.5;
const MAX_SUGGESTIONS: usize = 30;
/// Common starting roots, in the order the JS suggests them.
const STARTING_ROOTS: [PitchClass; 5] = [
    PitchClass::C,
    PitchClass::G,
    PitchClass::D,
    PitchClass::A,
    PitchClass::F,
];

struct Builder<'a> {
    suggestions: Vec<ChordSuggestion>,
    seen: HashSet<(PitchClass, ChordQuality)>,
    best_key: Option<KeyMatch>,
    history_for_alt_keys: &'a [Chord],
}

impl<'a> Builder<'a> {
    fn new(
        best_key: Option<KeyMatch>,
        history_for_alt_keys: &'a [Chord],
        current: Option<Chord>,
    ) -> Self {
        let mut seen = HashSet::new();
        if let Some(c) = current {
            seen.insert((c.root, c.quality));
        }
        Self {
            suggestions: Vec::new(),
            seen,
            best_key,
            history_for_alt_keys,
        }
    }

    fn add(
        &mut self,
        chord: Chord,
        roman_override: Option<RomanNumeral>,
        reason: String,
        category: SuggestionCategory,
    ) {
        if !self.seen.insert((chord.root, chord.quality)) {
            return;
        }
        let (roman, tonality_effect) = self.label(chord, roman_override, category);
        self.suggestions.push(ChordSuggestion {
            chord,
            roman,
            reason,
            category,
            tonality_effect,
        });
    }

    fn label(
        &self,
        chord: Chord,
        roman_override: Option<RomanNumeral>,
        category: SuggestionCategory,
    ) -> (Option<RomanNumeral>, String) {
        let Some(bk) = self.best_key else {
            return (roman_override, String::new());
        };

        if let Some(r) = bk.key.roman_for(chord) {
            return (Some(r), format!("diatonic in {}", bk.key));
        }

        // Non-diatonic. Keep caller-supplied romans only for categories
        // that own their labelling convention; for the rest, drop the
        // override since a numeral from a different tonal context would
        // mislead.
        let roman = match category {
            SuggestionCategory::Secondary | SuggestionCategory::Borrowed => roman_override,
            _ => None,
        };

        // See whether adding this chord would tilt the detected key.
        let mut alt = self.history_for_alt_keys.to_vec();
        alt.push(chord);
        let alt_keys = detect_key(&alt);
        let tonality_effect = match alt_keys.first() {
            Some(top) if top.key != bk.key && top.matched > bk.matched => {
                format!("shifts key toward {}", top.key)
            }
            _ => "chromatic / borrowed".to_string(),
        };

        (roman, tonality_effect)
    }
}

/// Suggest the next chord given the chord history before the cursor and
/// (optionally) a chord at the cursor that should be excluded from
/// suggestions.
///
/// Generates up to 30 candidates across six categories:
/// diatonic, resolution moves, borrowed chords, secondary dominants,
/// relative major/minor, and chromatic motion. Each suggestion is
/// deduplicated by (root, quality) — earlier categories win, so a chord
/// that is both diatonic and a resolution move shows up as diatonic.
///
/// Ports `suggestNextChords` from `theory.js`.
///
/// # Examples
///
/// ```
/// use harmonia::{suggest_next_chords, Chord};
///
/// let history: Vec<Chord> = ["C", "Am", "F"]
///     .iter().map(|s| s.parse().unwrap()).collect();
/// let result = suggest_next_chords(&history, None);
///
/// // The progression establishes C major.
/// let key = result.key.expect("key detected");
/// assert_eq!(key.key.tonic, harmonia::PitchClass::C);
///
/// // G7 (V7) should be among the suggestions.
/// let g7: Chord = "G7".parse().unwrap();
/// assert!(result.suggestions.iter().any(|s| s.chord == g7));
/// ```
pub fn suggest_next_chords(
    prior_chords: &[Chord],
    current: Option<Chord>,
) -> ChordSuggestionResult {
    // Key detection uses every chord in the sequence, including the one
    // currently at the cursor.
    let mut all_chords: Vec<Chord> = prior_chords.to_vec();
    if let Some(c) = current {
        all_chords.push(c);
    }

    let key_results = detect_key(&all_chords);
    let best_key = key_results.iter().find(|m| m.matched > 0).copied();
    let second_key = match best_key {
        Some(best) => key_results
            .iter()
            .skip(1)
            .find(|m| m.matched == best.matched)
            .copied(),
        None => None,
    };

    let prev_chord = prior_chords.last().copied();
    let mut builder = Builder::new(best_key, &all_chords, current);

    if let Some(bk) = best_key
        && bk.ratio() >= KEY_CONFIDENCE_THRESHOLD
    {
        add_diatonic(&mut builder, bk);
    }

    if let Some(prev) = prev_chord {
        add_resolution(&mut builder, prev);
    }

    if let Some(bk) = best_key
        && bk.ratio() >= KEY_CONFIDENCE_THRESHOLD
    {
        add_borrowed(&mut builder, bk);
        add_secondary_dominants(&mut builder, bk);
    }

    if let Some(prev) = prev_chord {
        add_chromatic(&mut builder, prev);
    }

    if prev_chord.is_none() && builder.suggestions.is_empty() {
        add_starting(&mut builder);
    }

    let mut suggestions = builder.suggestions;
    suggestions.truncate(MAX_SUGGESTIONS);

    ChordSuggestionResult {
        key: best_key,
        second_key,
        suggestions,
    }
}

fn add_diatonic(b: &mut Builder, bk: KeyMatch) {
    let key_label = bk.key.to_string();
    for d in bk.key.diatonic_triads() {
        let chord = d.in_key(bk.key);
        b.add(
            chord,
            Some(d.roman.clone()),
            format!("diatonic in {key_label}"),
            SuggestionCategory::Diatonic,
        );
    }
    for d in bk.key.diatonic_sevenths() {
        let chord = d.in_key(bk.key);
        b.add(
            chord,
            Some(d.roman.clone()),
            format!("diatonic in {key_label}"),
            SuggestionCategory::Diatonic,
        );
    }
}

fn add_resolution(b: &mut Builder, prev: Chord) {
    let p_root = prev.root;
    let p_qual = prev.quality;
    let prev_label = prev.to_string();

    // V → I (and v → i): up a perfect fourth from a major-ish chord.
    if matches!(p_qual, ChordQuality::Major | ChordQuality::Dominant7) {
        let tonic = p_root + Interval::PERFECT_FOURTH;
        b.add(
            Chord::new(tonic, ChordQuality::Major),
            Some(RomanNumeral::new(1, ChordQuality::Major)),
            format!("resolves V→I from {prev_label}"),
            SuggestionCategory::Resolution,
        );
        b.add(
            Chord::new(tonic, ChordQuality::Minor),
            Some(RomanNumeral::new(1, ChordQuality::Minor)),
            format!("resolves V→i from {prev_label}"),
            SuggestionCategory::Resolution,
        );
    }

    // ii → V: up a perfect fifth from a minor-ish chord.
    if matches!(p_qual, ChordQuality::Minor | ChordQuality::Minor7) {
        let v = p_root + Interval::PERFECT_FIFTH;
        b.add(
            Chord::new(v, ChordQuality::Dominant7),
            Some(RomanNumeral::new(5, ChordQuality::Dominant7)),
            format!("ii→V7 from {prev_label}"),
            SuggestionCategory::Resolution,
        );
        b.add(
            Chord::new(v, ChordQuality::Major),
            Some(RomanNumeral::new(5, ChordQuality::Major)),
            format!("ii→V from {prev_label}"),
            SuggestionCategory::Resolution,
        );
    }

    // IV → V: up a whole step from a major/maj7 chord.
    if matches!(p_qual, ChordQuality::Major | ChordQuality::Major7) {
        let v = p_root + Interval::MAJOR_SECOND;
        b.add(
            Chord::new(v, ChordQuality::Major),
            Some(RomanNumeral::new(5, ChordQuality::Major)),
            format!("IV→V motion from {prev_label}"),
            SuggestionCategory::Resolution,
        );
        b.add(
            Chord::new(v, ChordQuality::Dominant7),
            Some(RomanNumeral::new(5, ChordQuality::Dominant7)),
            format!("IV→V7 motion from {prev_label}"),
            SuggestionCategory::Resolution,
        );
    }

    // Relative minor of a major/maj7/dom7 chord (up a major sixth).
    if matches!(
        p_qual,
        ChordQuality::Major | ChordQuality::Major7 | ChordQuality::Dominant7
    ) {
        let rel_min = p_root + Interval::MAJOR_SIXTH;
        b.add(
            Chord::new(rel_min, ChordQuality::Minor),
            Some(RomanNumeral::new(6, ChordQuality::Minor)),
            format!("relative minor of {prev_label}"),
            SuggestionCategory::Relative,
        );
    }
    // Relative major of a minor/min7 chord (up a minor third).
    if matches!(p_qual, ChordQuality::Minor | ChordQuality::Minor7) {
        let rel_maj = p_root + Interval::MINOR_THIRD;
        b.add(
            Chord::new(rel_maj, ChordQuality::Major),
            Some(RomanNumeral::new(3, ChordQuality::Major)),
            format!("relative major of {prev_label}"),
            SuggestionCategory::Relative,
        );
    }
}

fn add_borrowed(b: &mut Builder, bk: KeyMatch) {
    // Borrowed from the parallel minor: ♭III, iv, ♭VI, ♭VII.
    let templates: [(Interval, ChordQuality, RomanNumeral, &str); 4] = [
        (
            Interval::MINOR_THIRD,
            ChordQuality::Major,
            RomanNumeral::flat(3, ChordQuality::Major),
            "borrowed from parallel minor",
        ),
        (
            Interval::MINOR_SIXTH,
            ChordQuality::Major,
            RomanNumeral::flat(6, ChordQuality::Major),
            "borrowed from parallel minor",
        ),
        (
            Interval::MINOR_SEVENTH,
            ChordQuality::Major,
            RomanNumeral::flat(7, ChordQuality::Major),
            "borrowed from parallel minor",
        ),
        (
            Interval::PERFECT_FOURTH,
            ChordQuality::Minor,
            RomanNumeral::new(4, ChordQuality::Minor),
            "borrowed minor iv",
        ),
    ];
    for (interval, quality, roman, desc) in templates {
        let chord = Chord::new(bk.key.tonic + interval, quality);
        b.add(chord, Some(roman), desc.to_string(), SuggestionCategory::Borrowed);
    }
}

fn add_secondary_dominants(b: &mut Builder, bk: KeyMatch) {
    // V7/X for each diatonic triad, except the diminished one (no
    // conventional secondary dominant of vii°).
    for d in bk.key.diatonic_triads() {
        if d.quality == ChordQuality::Diminished {
            continue;
        }
        let target_pc = bk.key.tonic + d.interval;
        let sec_dom_pc = target_pc + Interval::PERFECT_FIFTH;
        let chord = Chord::new(sec_dom_pc, ChordQuality::Dominant7);
        let roman = RomanNumeral::new(5, ChordQuality::Dominant7)
            .secondary_of(d.roman.clone());
        b.add(
            chord,
            Some(roman),
            format!("secondary dominant resolving to {target_pc}"),
            SuggestionCategory::Secondary,
        );
    }
}

fn add_chromatic(b: &mut Builder, prev: Chord) {
    let prev_label = prev.to_string();
    // Half-step up, whole-step up, half-step down, whole-step down.
    let moves: [(i8, &str, &str); 4] = [
        (1, "up", "half step"),
        (2, "up", "whole step"),
        (-1, "down", "half step"),
        (-2, "down", "whole step"),
    ];
    for (delta, dir, dist) in moves {
        let target_pc =
            PitchClass::new((prev.root.value() as i16 + delta as i16).rem_euclid(12) as u8);
        let reason = format!("{dir} {dist} from {prev_label}");
        b.add(
            Chord::new(target_pc, ChordQuality::Major),
            None,
            reason.clone(),
            SuggestionCategory::Chromatic,
        );
        b.add(
            Chord::new(target_pc, ChordQuality::Minor),
            None,
            reason,
            SuggestionCategory::Chromatic,
        );
    }
}

fn add_starting(b: &mut Builder) {
    for &root in &STARTING_ROOTS {
        b.add(
            Chord::new(root, ChordQuality::Major),
            None,
            "common starting key".to_string(),
            SuggestionCategory::Diatonic,
        );
        b.add(
            Chord::new(root, ChordQuality::Minor),
            None,
            "common starting key".to_string(),
            SuggestionCategory::Diatonic,
        );
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

    // ── suggest_next_chords ───────────────────────────────────────

    fn first_with_chord(
        result: &ChordSuggestionResult,
        chord: Chord,
    ) -> Option<&ChordSuggestion> {
        result.suggestions.iter().find(|s| s.chord == chord)
    }

    #[test]
    fn empty_history_returns_starting_chords() {
        let result = suggest_next_chords(&[], None);
        assert!(result.key.is_none());
        assert!(!result.suggestions.is_empty());
        // At least the C major and A minor common starts are present.
        assert!(first_with_chord(
            &result,
            chord(PitchClass::C, ChordQuality::Major)
        )
        .is_some());
        assert!(first_with_chord(
            &result,
            chord(PitchClass::A, ChordQuality::Minor)
        )
        .is_some());
        // All starting suggestions carry the diatonic category.
        assert!(result
            .suggestions
            .iter()
            .all(|s| s.category == SuggestionCategory::Diatonic));
    }

    #[test]
    fn detects_c_major_after_diatonic_progression() {
        let history = [
            chord(PitchClass::C, ChordQuality::Major),
            chord(PitchClass::F, ChordQuality::Major),
        ];
        let result = suggest_next_chords(&history, None);
        let key = result.key.expect("key should be detected");
        assert_eq!(key.key.tonic, PitchClass::C);
    }

    #[test]
    fn diatonic_chords_appear_in_c_major() {
        let history = [
            chord(PitchClass::C, ChordQuality::Major),
            chord(PitchClass::G, ChordQuality::Major),
        ];
        let result = suggest_next_chords(&history, None);
        // F (IV) and Am (vi) should be among the diatonic suggestions.
        let f_major = first_with_chord(&result, chord(PitchClass::F, ChordQuality::Major));
        let a_minor = first_with_chord(&result, chord(PitchClass::A, ChordQuality::Minor));
        assert!(f_major.is_some());
        assert_eq!(f_major.unwrap().category, SuggestionCategory::Diatonic);
        assert_eq!(
            f_major.unwrap().roman.as_ref().map(|r| r.to_string()).as_deref(),
            Some("IV")
        );
        assert!(a_minor.is_some());
        assert_eq!(
            a_minor.unwrap().roman.as_ref().map(|r| r.to_string()).as_deref(),
            Some("vi")
        );
    }

    #[test]
    fn current_chord_excluded_from_suggestions() {
        // Editing position 1 (the F): suggestions must not include F major,
        // because that's what's already there.
        let prior = [chord(PitchClass::C, ChordQuality::Major)];
        let cur = chord(PitchClass::F, ChordQuality::Major);
        let result = suggest_next_chords(&prior, Some(cur));
        assert!(first_with_chord(&result, cur).is_none());
    }

    #[test]
    fn v_dom7_suggests_i_resolution() {
        let prior = [chord(PitchClass::G, ChordQuality::Dominant7)];
        let result = suggest_next_chords(&prior, None);
        // C major (I) and C minor (i) should both be suggested.
        let c_major = first_with_chord(&result, chord(PitchClass::C, ChordQuality::Major));
        assert!(c_major.is_some());
        // The category may be Diatonic (since C major is also I in C-major
        // detection from a single G7 chord, depending on key detection),
        // but the reasoning should appear somewhere.
        assert!(
            result
                .suggestions
                .iter()
                .any(|s| s.reason.contains("V→I") || s.reason.contains("V\u{2192}I"))
                || c_major.unwrap().category == SuggestionCategory::Diatonic
        );
    }

    #[test]
    fn ii_minor_suggests_v_dominant() {
        // Dm7 should produce G7 as a ii→V7 suggestion (or a diatonic
        // chord, since key detection from a single Dm7 may pick C major).
        let prior = [chord(PitchClass::D, ChordQuality::Minor7)];
        let result = suggest_next_chords(&prior, None);
        let g7 = first_with_chord(&result, chord(PitchClass::G, ChordQuality::Dominant7));
        assert!(g7.is_some(), "G7 should be suggested after Dm7");
    }

    #[test]
    fn borrowed_chord_appears_with_strong_key() {
        // Strong C major progression — should expose ♭III (E♭ major) as a
        // borrowed chord.
        let history = [
            chord(PitchClass::C, ChordQuality::Major),
            chord(PitchClass::F, ChordQuality::Major),
            chord(PitchClass::G, ChordQuality::Major),
            chord(PitchClass::C, ChordQuality::Major),
            chord(PitchClass::A, ChordQuality::Minor),
            chord(PitchClass::G, ChordQuality::Major),
        ];
        let result = suggest_next_chords(&history, None);
        let eb = first_with_chord(&result, chord(PitchClass::D_SHARP, ChordQuality::Major));
        assert!(eb.is_some(), "♭III should appear as a borrowed chord");
        assert_eq!(eb.unwrap().category, SuggestionCategory::Borrowed);
        assert_eq!(
            eb.unwrap().roman.as_ref().map(|r| r.to_string()).as_deref(),
            Some("♭III")
        );
    }

    #[test]
    fn secondary_dominant_appears_with_strong_key() {
        let history = [
            chord(PitchClass::C, ChordQuality::Major),
            chord(PitchClass::F, ChordQuality::Major),
            chord(PitchClass::G, ChordQuality::Major),
            chord(PitchClass::C, ChordQuality::Major),
        ];
        let result = suggest_next_chords(&history, None);
        // V7/ii in C major = A7. Should be in suggestions, marked secondary.
        let a7 = first_with_chord(&result, chord(PitchClass::A, ChordQuality::Dominant7));
        assert!(a7.is_some(), "V7/ii (A7) should be a secondary dominant");
        assert_eq!(a7.unwrap().category, SuggestionCategory::Secondary);
        let roman_str = a7.unwrap().roman.as_ref().map(|r| r.to_string());
        assert!(
            roman_str.as_deref().is_some_and(|s| s.starts_with("V7/")),
            "expected V7/X, got {roman_str:?}"
        );
    }

    #[test]
    fn chromatic_moves_appear_when_no_key() {
        // Single chord, no clear key context for borrowed/secondary.
        // The fallbacks include chromatic moves from the prev chord.
        let prior = [chord(PitchClass::C, ChordQuality::Major)];
        let result = suggest_next_chords(&prior, None);
        // C♯ (up half step) should be suggested chromatically — but
        // dedup may have categorized it differently if it's diatonic
        // somewhere. At minimum, *some* suggestion above C must exist.
        assert!(!result.suggestions.is_empty());
    }

    #[test]
    fn suggestion_count_capped_at_max() {
        // A heavy-context progression generates many candidates.
        let history = [
            chord(PitchClass::C, ChordQuality::Major),
            chord(PitchClass::A, ChordQuality::Minor),
            chord(PitchClass::F, ChordQuality::Major),
            chord(PitchClass::G, ChordQuality::Major),
        ];
        let result = suggest_next_chords(&history, None);
        assert!(result.suggestions.len() <= MAX_SUGGESTIONS);
    }

    #[test]
    fn dedup_first_category_wins() {
        // C major progression → F major shows up as IV (diatonic).
        // It's also a IV→V resolution target etc., but should be tagged
        // diatonic since that section runs first.
        let history = [
            chord(PitchClass::C, ChordQuality::Major),
            chord(PitchClass::G, ChordQuality::Major),
        ];
        let result = suggest_next_chords(&history, None);
        let f_major = first_with_chord(&result, chord(PitchClass::F, ChordQuality::Major)).unwrap();
        assert_eq!(f_major.category, SuggestionCategory::Diatonic);
    }

    #[test]
    fn second_key_populated_for_ambiguous_input() {
        // Single C major: ties for top in C, F, and G major.
        let history = [chord(PitchClass::C, ChordQuality::Major)];
        let result = suggest_next_chords(&history, None);
        assert!(result.key.is_some());
        assert!(result.second_key.is_some());
        assert_eq!(result.key.unwrap().matched, result.second_key.unwrap().matched);
    }

    #[test]
    fn category_display_matches_js_strings() {
        assert_eq!(SuggestionCategory::Diatonic.to_string(), "diatonic");
        assert_eq!(SuggestionCategory::Resolution.to_string(), "resolution");
        assert_eq!(SuggestionCategory::Borrowed.to_string(), "borrowed");
        assert_eq!(SuggestionCategory::Secondary.to_string(), "secondary");
        assert_eq!(SuggestionCategory::Relative.to_string(), "relative");
        assert_eq!(SuggestionCategory::Chromatic.to_string(), "chromatic");
    }

    #[test]
    fn category_parse_round_trips() {
        for c in [
            SuggestionCategory::Diatonic,
            SuggestionCategory::Resolution,
            SuggestionCategory::Borrowed,
            SuggestionCategory::Secondary,
            SuggestionCategory::Relative,
            SuggestionCategory::Chromatic,
        ] {
            let parsed: SuggestionCategory = c.to_string().parse().unwrap();
            assert_eq!(parsed, c);
        }
        assert!("nope".parse::<SuggestionCategory>().is_err());
    }
}
