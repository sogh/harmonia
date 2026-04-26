//! Typed Roman numerals for harmonic analysis.
//!
//! Roman numerals encode three things at once: a scale degree (1–7),
//! an optional chromatic alteration (`♭`/`♯`), and a chord quality.
//! Chord quality controls both the case of the numeral (uppercase for
//! major-ish, lowercase for minor-ish) and the suffix (`°`, `7`, `maj7`,
//! `ø7`, …). [`RomanNumeral`] models all three plus an optional
//! `secondary_of` for tonicization (`V7/ii`).

use std::fmt;
use std::str::FromStr;

use crate::chord::ChordQuality;
use crate::parse::ParseError;

/// A chromatic alteration applied to a scale-degree numeral.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Alteration {
    Flat,
    Sharp,
}

impl Alteration {
    pub const fn symbol(self) -> &'static str {
        match self {
            Alteration::Flat => "♭",
            Alteration::Sharp => "♯",
        }
    }
}

impl fmt::Display for Alteration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}

/// A Roman-numeral chord label.
///
/// Construct with [`RomanNumeral::new`] (clean degree), [`RomanNumeral::flat`]
/// or [`RomanNumeral::sharp`] (altered degree), and chain
/// [`RomanNumeral::secondary_of`] to mark a secondary dominant.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RomanNumeral {
    pub alteration: Option<Alteration>,
    /// Scale degree, `1..=7`.
    pub degree: u8,
    /// Chord quality — drives both the case of the numeral and the suffix.
    pub quality: ChordQuality,
    /// When present, this label refers to *self* applied to the named
    /// target — e.g. `V7/ii` is `V7` with `secondary_of = Some(ii)`.
    pub secondary_of: Option<Box<RomanNumeral>>,
}

impl RomanNumeral {
    /// Build a plain numeral with no alteration and no secondary target.
    /// `degree` must be in `1..=7` (asserted in const context).
    pub const fn new(degree: u8, quality: ChordQuality) -> Self {
        assert!(
            degree >= 1 && degree <= 7,
            "RomanNumeral degree must be in 1..=7"
        );
        Self {
            alteration: None,
            degree,
            quality,
            secondary_of: None,
        }
    }

    /// Build a flat-altered numeral (e.g. `♭III`).
    pub const fn flat(degree: u8, quality: ChordQuality) -> Self {
        assert!(degree >= 1 && degree <= 7);
        Self {
            alteration: Some(Alteration::Flat),
            degree,
            quality,
            secondary_of: None,
        }
    }

    /// Build a sharp-altered numeral (e.g. `♯iv`).
    pub const fn sharp(degree: u8, quality: ChordQuality) -> Self {
        assert!(degree >= 1 && degree <= 7);
        Self {
            alteration: Some(Alteration::Sharp),
            degree,
            quality,
            secondary_of: None,
        }
    }

    /// Mark this numeral as a secondary chord targeting `target`.
    /// Consuming builder method: `RomanNumeral::new(5, Dominant7).secondary_of(...)`.
    pub fn secondary_of(mut self, target: RomanNumeral) -> Self {
        self.secondary_of = Some(Box::new(target));
        self
    }

    /// Return a copy of this numeral with a different chord quality —
    /// useful for the "fuzzy" fallback in [`crate::Key::roman_for`] that
    /// turns a major-degree triad into the corresponding `<degree>7`.
    pub fn with_quality(mut self, quality: ChordQuality) -> Self {
        self.quality = quality;
        self
    }

    /// True if this numeral renders in uppercase letters
    /// (major-family qualities).
    pub const fn is_uppercase(&self) -> bool {
        is_uppercase_quality(self.quality)
    }
}

const fn is_uppercase_quality(q: ChordQuality) -> bool {
    matches!(
        q,
        ChordQuality::Major
            | ChordQuality::Major7
            | ChordQuality::Dominant7
            | ChordQuality::Augmented
            | ChordQuality::Sus2
            | ChordQuality::Sus4
    )
}

const fn quality_suffix(q: ChordQuality) -> &'static str {
    match q {
        ChordQuality::Major => "",
        ChordQuality::Minor => "",
        ChordQuality::Diminished => "°",
        ChordQuality::Augmented => "+",
        ChordQuality::Sus2 => "sus2",
        ChordQuality::Sus4 => "sus4",
        ChordQuality::Major7 => "maj7",
        ChordQuality::Dominant7 => "7",
        ChordQuality::Minor7 => "7",
        ChordQuality::MinorMajor7 => "M7",
        ChordQuality::Diminished7 => "°7",
        ChordQuality::HalfDiminished7 => "ø7",
    }
}

const fn degree_letters(degree: u8, uppercase: bool) -> &'static str {
    match (degree, uppercase) {
        (1, true) => "I",
        (1, false) => "i",
        (2, true) => "II",
        (2, false) => "ii",
        (3, true) => "III",
        (3, false) => "iii",
        (4, true) => "IV",
        (4, false) => "iv",
        (5, true) => "V",
        (5, false) => "v",
        (6, true) => "VI",
        (6, false) => "vi",
        (7, true) => "VII",
        (7, false) => "vii",
        // Out-of-range degree — Display shouldn't be lossy, but the
        // const constructors already assert this.
        _ => "?",
    }
}

impl fmt::Display for RomanNumeral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(a) = self.alteration {
            f.write_str(a.symbol())?;
        }
        f.write_str(degree_letters(self.degree, self.is_uppercase()))?;
        f.write_str(quality_suffix(self.quality))?;
        if let Some(target) = &self.secondary_of {
            write!(f, "/{}", target)?;
        }
        Ok(())
    }
}

/// Roman digits in descending length order, so a longest-prefix match picks
/// the right value (e.g. `"VII"` beats `"V"`).
const UPPER_ROMANS: &[(&str, u8)] = &[
    ("VII", 7),
    ("VI", 6),
    ("V", 5),
    ("IV", 4),
    ("III", 3),
    ("II", 2),
    ("I", 1),
];
const LOWER_ROMANS: &[(&str, u8)] = &[
    ("vii", 7),
    ("vi", 6),
    ("v", 5),
    ("iv", 4),
    ("iii", 3),
    ("ii", 2),
    ("i", 1),
];

fn parse_roman_digits(s: &str) -> Option<(u8, bool, &str)> {
    for (digits, degree) in UPPER_ROMANS {
        if let Some(rest) = s.strip_prefix(*digits) {
            return Some((*degree, true, rest));
        }
    }
    for (digits, degree) in LOWER_ROMANS {
        if let Some(rest) = s.strip_prefix(*digits) {
            return Some((*degree, false, rest));
        }
    }
    None
}

fn quality_for(uppercase: bool, suffix: &str) -> Option<ChordQuality> {
    Some(match (uppercase, suffix) {
        (true, "") => ChordQuality::Major,
        (false, "") => ChordQuality::Minor,
        (false, "°") => ChordQuality::Diminished,
        (true, "+") => ChordQuality::Augmented,
        (true, "sus2") => ChordQuality::Sus2,
        (true, "sus4") => ChordQuality::Sus4,
        (true, "maj7") => ChordQuality::Major7,
        (true, "7") => ChordQuality::Dominant7,
        (false, "7") => ChordQuality::Minor7,
        (false, "M7") => ChordQuality::MinorMajor7,
        (false, "°7") => ChordQuality::Diminished7,
        (false, "ø7") => ChordQuality::HalfDiminished7,
        _ => return None,
    })
}

/// Parse a Roman numeral. Inverse of [`RomanNumeral`]'s
/// [`Display`](fmt::Display) impl.
///
/// Accepts the standard chromatic alterations (`♭`/`♯`, with `b`/`#` as
/// ASCII alternatives), the seven Roman digits in either case, the
/// quality suffixes the `Display` impl emits, and `/<target>` for
/// secondary chords (the target is parsed recursively).
///
/// # Examples
///
/// ```
/// use harmonia::{ChordQuality, RomanNumeral};
///
/// let v7: RomanNumeral = "V7".parse().unwrap();
/// assert_eq!(v7, RomanNumeral::new(5, ChordQuality::Dominant7));
///
/// let flat_three: RomanNumeral = "♭III".parse().unwrap();
/// assert_eq!(flat_three, RomanNumeral::flat(3, ChordQuality::Major));
///
/// let secondary: RomanNumeral = "V7/ii".parse().unwrap();
/// assert_eq!(
///     secondary,
///     RomanNumeral::new(5, ChordQuality::Dominant7)
///         .secondary_of(RomanNumeral::new(2, ChordQuality::Minor))
/// );
///
/// // Round-trips through Display.
/// let half: RomanNumeral = "viiø7".parse().unwrap();
/// assert_eq!(half.to_string(), "viiø7");
/// ```
impl FromStr for RomanNumeral {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut rest = s;

        // 1. Optional alteration prefix.
        let alteration = if let Some(r) = rest.strip_prefix('♭').or_else(|| rest.strip_prefix('b')) {
            rest = r;
            Some(Alteration::Flat)
        } else if let Some(r) = rest.strip_prefix('♯').or_else(|| rest.strip_prefix('#')) {
            rest = r;
            Some(Alteration::Sharp)
        } else {
            None
        };

        // 2. Roman digits.
        let (degree, uppercase, after_digits) = parse_roman_digits(rest).ok_or_else(|| {
            ParseError::new(format!("expected Roman digit in {s:?}"))
        })?;

        // 3. Suffix and optional /secondary, splitting at the first '/'.
        let (suffix, secondary) = match after_digits.find('/') {
            Some(idx) => {
                let suf = &after_digits[..idx];
                let sec_str = &after_digits[idx + 1..];
                let sec: RomanNumeral = sec_str.parse()?;
                (suf, Some(Box::new(sec)))
            }
            None => (after_digits, None),
        };

        // 4. Resolve quality from (case, suffix).
        let quality = quality_for(uppercase, suffix).ok_or_else(|| {
            let case = if uppercase { "uppercase" } else { "lowercase" };
            ParseError::new(format!(
                "no chord quality matches {case} numeral with suffix {suffix:?}"
            ))
        })?;

        Ok(Self {
            alteration,
            degree,
            quality,
            secondary_of: secondary,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(r: RomanNumeral) -> String {
        r.to_string()
    }

    #[test]
    fn diatonic_triads_render_with_correct_case() {
        assert_eq!(s(RomanNumeral::new(1, ChordQuality::Major)), "I");
        assert_eq!(s(RomanNumeral::new(2, ChordQuality::Minor)), "ii");
        assert_eq!(s(RomanNumeral::new(3, ChordQuality::Minor)), "iii");
        assert_eq!(s(RomanNumeral::new(4, ChordQuality::Major)), "IV");
        assert_eq!(s(RomanNumeral::new(5, ChordQuality::Major)), "V");
        assert_eq!(s(RomanNumeral::new(6, ChordQuality::Minor)), "vi");
        assert_eq!(s(RomanNumeral::new(7, ChordQuality::Diminished)), "vii°");
    }

    #[test]
    fn diatonic_sevenths_render_with_correct_suffixes() {
        assert_eq!(s(RomanNumeral::new(1, ChordQuality::Major7)), "Imaj7");
        assert_eq!(s(RomanNumeral::new(2, ChordQuality::Minor7)), "ii7");
        assert_eq!(s(RomanNumeral::new(5, ChordQuality::Dominant7)), "V7");
        assert_eq!(
            s(RomanNumeral::new(7, ChordQuality::HalfDiminished7)),
            "viiø7"
        );
    }

    #[test]
    fn alterations_render_with_unicode_symbols() {
        assert_eq!(s(RomanNumeral::flat(3, ChordQuality::Major)), "♭III");
        assert_eq!(s(RomanNumeral::flat(6, ChordQuality::Major)), "♭VI");
        assert_eq!(s(RomanNumeral::flat(7, ChordQuality::Major)), "♭VII");
        assert_eq!(s(RomanNumeral::sharp(4, ChordQuality::Minor)), "♯iv");
    }

    #[test]
    fn secondary_dominants_render_with_slash() {
        let v7_of_ii = RomanNumeral::new(5, ChordQuality::Dominant7)
            .secondary_of(RomanNumeral::new(2, ChordQuality::Minor));
        assert_eq!(s(v7_of_ii), "V7/ii");

        let v7_of_iv = RomanNumeral::new(5, ChordQuality::Dominant7)
            .secondary_of(RomanNumeral::new(4, ChordQuality::Major));
        assert_eq!(s(v7_of_iv), "V7/IV");
    }

    #[test]
    fn fuzzy_fallback_with_quality() {
        // The "I7" / "IV7" pattern: take a major-degree triad and swap
        // its quality to dom7.
        let i = RomanNumeral::new(1, ChordQuality::Major);
        assert_eq!(s(i.with_quality(ChordQuality::Dominant7)), "I7");
        let iv = RomanNumeral::new(4, ChordQuality::Major);
        assert_eq!(s(iv.with_quality(ChordQuality::Dominant7)), "IV7");
    }

    #[test]
    fn case_classification() {
        assert!(RomanNumeral::new(1, ChordQuality::Major).is_uppercase());
        assert!(RomanNumeral::new(5, ChordQuality::Dominant7).is_uppercase());
        assert!(!RomanNumeral::new(2, ChordQuality::Minor).is_uppercase());
        assert!(!RomanNumeral::new(7, ChordQuality::Diminished).is_uppercase());
        assert!(!RomanNumeral::new(7, ChordQuality::HalfDiminished7).is_uppercase());
    }

    #[test]
    fn parse_basic_diatonic_triads() {
        let cases = [
            ("I", 1, ChordQuality::Major),
            ("ii", 2, ChordQuality::Minor),
            ("iii", 3, ChordQuality::Minor),
            ("IV", 4, ChordQuality::Major),
            ("V", 5, ChordQuality::Major),
            ("vi", 6, ChordQuality::Minor),
            ("vii°", 7, ChordQuality::Diminished),
        ];
        for (input, degree, quality) in cases {
            let parsed: RomanNumeral = input.parse().unwrap();
            assert_eq!(parsed, RomanNumeral::new(degree, quality), "input {input:?}");
        }
    }

    #[test]
    fn parse_seventh_chords() {
        assert_eq!(
            "V7".parse::<RomanNumeral>().unwrap(),
            RomanNumeral::new(5, ChordQuality::Dominant7)
        );
        assert_eq!(
            "ii7".parse::<RomanNumeral>().unwrap(),
            RomanNumeral::new(2, ChordQuality::Minor7)
        );
        assert_eq!(
            "Imaj7".parse::<RomanNumeral>().unwrap(),
            RomanNumeral::new(1, ChordQuality::Major7)
        );
        assert_eq!(
            "viiø7".parse::<RomanNumeral>().unwrap(),
            RomanNumeral::new(7, ChordQuality::HalfDiminished7)
        );
    }

    #[test]
    fn parse_alterations() {
        assert_eq!(
            "♭III".parse::<RomanNumeral>().unwrap(),
            RomanNumeral::flat(3, ChordQuality::Major)
        );
        // ASCII alternatives.
        assert_eq!(
            "bIII".parse::<RomanNumeral>().unwrap(),
            RomanNumeral::flat(3, ChordQuality::Major)
        );
        assert_eq!(
            "♯iv".parse::<RomanNumeral>().unwrap(),
            RomanNumeral::sharp(4, ChordQuality::Minor)
        );
        assert_eq!(
            "#iv".parse::<RomanNumeral>().unwrap(),
            RomanNumeral::sharp(4, ChordQuality::Minor)
        );
    }

    #[test]
    fn parse_secondary_dominants() {
        let r: RomanNumeral = "V7/ii".parse().unwrap();
        assert_eq!(
            r,
            RomanNumeral::new(5, ChordQuality::Dominant7)
                .secondary_of(RomanNumeral::new(2, ChordQuality::Minor))
        );

        // Targets can themselves carry alteration.
        let r: RomanNumeral = "V7/♭III".parse().unwrap();
        assert_eq!(
            r,
            RomanNumeral::new(5, ChordQuality::Dominant7)
                .secondary_of(RomanNumeral::flat(3, ChordQuality::Major))
        );
    }

    #[test]
    fn parse_round_trips_for_every_degree_and_quality() {
        // Without alteration or secondary.
        for degree in 1..=7 {
            for q in ChordQuality::ALL {
                let original = RomanNumeral::new(degree, *q);
                let parsed: RomanNumeral = original.to_string().parse().unwrap();
                assert_eq!(parsed, original, "{degree} {q:?}");
            }
        }
    }

    #[test]
    fn parse_round_trips_with_alteration() {
        for degree in 1..=7 {
            for q in ChordQuality::ALL {
                for r in [
                    RomanNumeral::flat(degree, *q),
                    RomanNumeral::sharp(degree, *q),
                ] {
                    let parsed: RomanNumeral = r.to_string().parse().unwrap();
                    assert_eq!(parsed, r);
                }
            }
        }
    }

    #[test]
    fn parse_round_trips_with_secondary() {
        // V7/X for every diatonic-triad target.
        for (target_degree, target_quality) in [
            (1, ChordQuality::Major),
            (2, ChordQuality::Minor),
            (3, ChordQuality::Minor),
            (4, ChordQuality::Major),
            (5, ChordQuality::Major),
            (6, ChordQuality::Minor),
            (7, ChordQuality::Diminished),
        ] {
            let original = RomanNumeral::new(5, ChordQuality::Dominant7)
                .secondary_of(RomanNumeral::new(target_degree, target_quality));
            let parsed: RomanNumeral = original.to_string().parse().unwrap();
            assert_eq!(parsed, original);
        }
    }

    #[test]
    fn parse_rejects_invalid_inputs() {
        assert!("".parse::<RomanNumeral>().is_err());
        assert!("Z".parse::<RomanNumeral>().is_err()); // not a roman digit
        assert!("Vfoo".parse::<RomanNumeral>().is_err()); // unknown suffix
        assert!("I°".parse::<RomanNumeral>().is_err()); // ° on uppercase has no quality
        assert!("V7/".parse::<RomanNumeral>().is_err()); // trailing slash, no target
        assert!("viii".parse::<RomanNumeral>().is_err()); // garbage after vii
    }
}
