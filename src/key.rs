//! Keys and Roman-numeral analysis.
//!
//! Currently only **major keys** are modeled, matching the scope of
//! `theory.js`. A future minor-key extension will add a `mode` field to
//! [`Key`]; downstream code that constructs keys via [`Key::new`] is
//! already implicitly major.

use std::fmt;

use crate::chord::{Chord, ChordQuality};
use crate::interval::Interval;
use crate::pitch::PitchClass;
use crate::scale::{Scale, ScaleKind};

/// One row of a key's diatonic chord template: the interval from the
/// tonic, the chord quality, and the Roman-numeral label.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DiatonicChord {
    pub interval: Interval,
    pub quality: ChordQuality,
    pub roman: &'static str,
}

impl DiatonicChord {
    pub const fn new(
        interval: Interval,
        quality: ChordQuality,
        roman: &'static str,
    ) -> Self {
        Self {
            interval,
            quality,
            roman,
        }
    }

    /// The concrete chord at this scale degree, anchored at `key.tonic`.
    pub fn in_key(&self, key: Key) -> Chord {
        Chord::new(key.tonic + self.interval, self.quality)
    }
}

/// Diatonic triads of a major key, indexed I..vii°.
pub const MAJOR_KEY_TRIADS: &[DiatonicChord] = &[
    DiatonicChord::new(Interval::UNISON, ChordQuality::Major, "I"),
    DiatonicChord::new(Interval::MAJOR_SECOND, ChordQuality::Minor, "ii"),
    DiatonicChord::new(Interval::MAJOR_THIRD, ChordQuality::Minor, "iii"),
    DiatonicChord::new(Interval::PERFECT_FOURTH, ChordQuality::Major, "IV"),
    DiatonicChord::new(Interval::PERFECT_FIFTH, ChordQuality::Major, "V"),
    DiatonicChord::new(Interval::MAJOR_SIXTH, ChordQuality::Minor, "vi"),
    DiatonicChord::new(Interval::MAJOR_SEVENTH, ChordQuality::Diminished, "vii°"),
];

/// Diatonic seventh chords of a major key, indexed Imaj7..viiø7.
pub const MAJOR_KEY_SEVENTHS: &[DiatonicChord] = &[
    DiatonicChord::new(Interval::UNISON, ChordQuality::Major7, "Imaj7"),
    DiatonicChord::new(Interval::MAJOR_SECOND, ChordQuality::Minor7, "ii7"),
    DiatonicChord::new(Interval::MAJOR_THIRD, ChordQuality::Minor7, "iii7"),
    DiatonicChord::new(Interval::PERFECT_FOURTH, ChordQuality::Major7, "IVmaj7"),
    DiatonicChord::new(Interval::PERFECT_FIFTH, ChordQuality::Dominant7, "V7"),
    DiatonicChord::new(Interval::MAJOR_SIXTH, ChordQuality::Minor7, "vi7"),
    DiatonicChord::new(
        Interval::MAJOR_SEVENTH,
        ChordQuality::HalfDiminished7,
        "viiø7",
    ),
];

/// A key — currently always major. Holds only the tonic [`PitchClass`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub tonic: PitchClass,
}

impl Key {
    /// A major key with the given tonic.
    pub const fn new(tonic: PitchClass) -> Self {
        Self { tonic }
    }

    /// The Ionian scale anchored at this key's tonic.
    pub const fn scale(self) -> Scale {
        Scale::new(self.tonic, ScaleKind::Ionian)
    }

    /// Diatonic triads I..vii° in this key.
    pub const fn diatonic_triads(self) -> &'static [DiatonicChord] {
        MAJOR_KEY_TRIADS
    }

    /// Diatonic seventh chords Imaj7..viiø7 in this key.
    pub const fn diatonic_sevenths(self) -> &'static [DiatonicChord] {
        MAJOR_KEY_SEVENTHS
    }

    /// True if `chord` is one of the diatonic triads or sevenths of this key.
    pub fn contains(self, chord: Chord) -> bool {
        let interval = chord.root - self.tonic;
        self.diatonic_triads()
            .iter()
            .chain(self.diatonic_sevenths().iter())
            .any(|d| d.interval == interval && d.quality == chord.quality)
    }

    /// Roman-numeral label for `chord` in this key.
    ///
    /// Returns the diatonic label when `chord` matches a triad or seventh
    /// template exactly. As a fallback (ported from `theory.js`), labels
    /// non-diatonic dominant 7ths sitting on a major degree as `<roman>7`
    /// (e.g. C7 in C major → `"I7"`) and minor 7ths on a minor degree
    /// likewise. Returns `None` when no plausible label exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use harmonia::{Chord, Key, PitchClass};
    ///
    /// let c_major = Key::new(PitchClass::C);
    ///
    /// // Diatonic chords get exact Roman labels.
    /// let g: Chord = "G".parse().unwrap();
    /// let g7: Chord = "G7".parse().unwrap();
    /// assert_eq!(c_major.roman_for(g).as_deref(),  Some("V"));
    /// assert_eq!(c_major.roman_for(g7).as_deref(), Some("V7"));
    ///
    /// // Non-diatonic dom7 on a major degree gets the fuzzy label.
    /// let c7: Chord = "C7".parse().unwrap();
    /// assert_eq!(c_major.roman_for(c7).as_deref(), Some("I7"));
    ///
    /// // Truly out-of-key chords return None.
    /// let f_sharp: Chord = "F#".parse().unwrap();
    /// assert!(c_major.roman_for(f_sharp).is_none());
    /// ```
    pub fn roman_for(self, chord: Chord) -> Option<String> {
        let interval = chord.root - self.tonic;

        for d in self.diatonic_triads() {
            if d.interval == interval && d.quality == chord.quality {
                return Some(d.roman.to_string());
            }
        }
        for d in self.diatonic_sevenths() {
            if d.interval == interval && d.quality == chord.quality {
                return Some(d.roman.to_string());
            }
        }

        for d in self.diatonic_triads() {
            if d.interval != interval {
                continue;
            }
            match (d.quality, chord.quality) {
                (ChordQuality::Major, ChordQuality::Dominant7) => {
                    return Some(format!("{}7", d.roman));
                }
                (ChordQuality::Minor, ChordQuality::Minor7) => {
                    return Some(format!("{}7", d.roman));
                }
                _ => {}
            }
        }

        None
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} major", self.tonic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_major_diatonic_triads_are_c_dm_em_f_g_am_bdim() {
        let key = Key::new(PitchClass::C);
        let chords: Vec<String> = key
            .diatonic_triads()
            .iter()
            .map(|d| d.in_key(key).to_string())
            .collect();
        assert_eq!(
            chords,
            vec!["C", "Dm", "Em", "F", "G", "Am", "Bdim"]
        );
    }

    #[test]
    fn c_major_diatonic_sevenths() {
        let key = Key::new(PitchClass::C);
        let chords: Vec<String> = key
            .diatonic_sevenths()
            .iter()
            .map(|d| d.in_key(key).to_string())
            .collect();
        assert_eq!(
            chords,
            vec![
                "Cmaj7", "Dm7", "Em7", "Fmaj7", "G7", "Am7", "Bm7♭5"
            ]
        );
    }

    #[test]
    fn d_major_v7_is_a7() {
        let key = Key::new(PitchClass::D);
        let v7 = key.diatonic_sevenths()[4].in_key(key);
        assert_eq!(v7.to_string(), "A7");
    }

    #[test]
    fn roman_for_diatonic_triads_in_c_major() {
        let key = Key::new(PitchClass::C);
        let cases = [
            (PitchClass::C, ChordQuality::Major, "I"),
            (PitchClass::D, ChordQuality::Minor, "ii"),
            (PitchClass::E, ChordQuality::Minor, "iii"),
            (PitchClass::F, ChordQuality::Major, "IV"),
            (PitchClass::G, ChordQuality::Major, "V"),
            (PitchClass::A, ChordQuality::Minor, "vi"),
            (PitchClass::B, ChordQuality::Diminished, "vii°"),
        ];
        for (root, q, expected) in cases {
            assert_eq!(
                key.roman_for(Chord::new(root, q)).as_deref(),
                Some(expected),
                "{root:?} {q:?}"
            );
        }
    }

    #[test]
    fn roman_for_seventh_chords_in_c_major() {
        let key = Key::new(PitchClass::C);
        assert_eq!(
            key.roman_for(Chord::new(PitchClass::G, ChordQuality::Dominant7))
                .as_deref(),
            Some("V7")
        );
        assert_eq!(
            key.roman_for(Chord::new(PitchClass::B, ChordQuality::HalfDiminished7))
                .as_deref(),
            Some("viiø7")
        );
        assert_eq!(
            key.roman_for(Chord::new(PitchClass::C, ChordQuality::Major7))
                .as_deref(),
            Some("Imaj7")
        );
    }

    #[test]
    fn roman_for_non_diatonic_dom7_on_major_degree() {
        // C7 in C major: I is major, dom7 doesn't match exactly → fuzzy "I7".
        let key = Key::new(PitchClass::C);
        assert_eq!(
            key.roman_for(Chord::new(PitchClass::C, ChordQuality::Dominant7))
                .as_deref(),
            Some("I7")
        );
        // F7 in C major → "IV7".
        assert_eq!(
            key.roman_for(Chord::new(PitchClass::F, ChordQuality::Dominant7))
                .as_deref(),
            Some("IV7")
        );
    }

    #[test]
    fn roman_for_returns_none_for_unrelated_chord() {
        let key = Key::new(PitchClass::C);
        // A major in C major: vi position is minor; A major doesn't match
        // any template, and the fuzzy fallback only handles dom7/min7.
        assert!(
            key.roman_for(Chord::new(PitchClass::A, ChordQuality::Major))
                .is_none()
        );
        // F# anything is far afield.
        assert!(
            key.roman_for(Chord::new(PitchClass::F_SHARP, ChordQuality::Major))
                .is_none()
        );
    }

    #[test]
    fn contains_recognises_diatonic_chords() {
        let key = Key::new(PitchClass::C);
        assert!(key.contains(Chord::new(PitchClass::C, ChordQuality::Major)));
        assert!(key.contains(Chord::new(PitchClass::A, ChordQuality::Minor)));
        assert!(key.contains(Chord::new(PitchClass::G, ChordQuality::Dominant7)));
        assert!(!key.contains(Chord::new(PitchClass::F_SHARP, ChordQuality::Major)));
        assert!(!key.contains(Chord::new(PitchClass::C, ChordQuality::Dominant7)));
    }

    #[test]
    fn key_scale_is_ionian_at_tonic() {
        let key = Key::new(PitchClass::G);
        assert_eq!(key.scale(), Scale::new(PitchClass::G, ScaleKind::Ionian));
    }

    #[test]
    fn display_includes_mode() {
        assert_eq!(Key::new(PitchClass::C).to_string(), "C major");
        assert_eq!(Key::new(PitchClass::F_SHARP).to_string(), "F♯ major");
    }

    #[test]
    fn template_lengths_are_seven_each() {
        assert_eq!(MAJOR_KEY_TRIADS.len(), 7);
        assert_eq!(MAJOR_KEY_SEVENTHS.len(), 7);
    }
}
