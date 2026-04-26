//! Typed Roman numerals for harmonic analysis.
//!
//! Roman numerals encode three things at once: a scale degree (1–7),
//! an optional chromatic alteration (`♭`/`♯`), and a chord quality.
//! Chord quality controls both the case of the numeral (uppercase for
//! major-ish, lowercase for minor-ish) and the suffix (`°`, `7`, `maj7`,
//! `ø7`, …). [`RomanNumeral`] models all three plus an optional
//! `secondary_of` for tonicization (`V7/ii`).

use std::fmt;

use crate::chord::ChordQuality;

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
}
