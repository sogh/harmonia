//! Chords: 12 qualities and a `Chord` (kind + root) with pitch-class access.
//!
//! Catalogue ports `CHORD_INTERVALS` from `theory.js` — six triads (major,
//! minor, diminished, augmented, sus2, sus4) and six seventh chords (maj7,
//! dominant 7, min7, minor-major 7, dim7, half-diminished 7).

use std::fmt;

use crate::interval::Interval;
use crate::pitch::PitchClass;

/// A chord quality — the harmonic shape (interval pattern) of a chord
/// independent of its root.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    Sus2,
    Sus4,
    Major7,
    Dominant7,
    Minor7,
    MinorMajor7,
    Diminished7,
    HalfDiminished7,
}

const MAJOR: &[Interval] = &[
    Interval::new(0),
    Interval::new(4),
    Interval::new(7),
];
const MINOR: &[Interval] = &[
    Interval::new(0),
    Interval::new(3),
    Interval::new(7),
];
const DIMINISHED: &[Interval] = &[
    Interval::new(0),
    Interval::new(3),
    Interval::new(6),
];
const AUGMENTED: &[Interval] = &[
    Interval::new(0),
    Interval::new(4),
    Interval::new(8),
];
const SUS2: &[Interval] = &[
    Interval::new(0),
    Interval::new(2),
    Interval::new(7),
];
const SUS4: &[Interval] = &[
    Interval::new(0),
    Interval::new(5),
    Interval::new(7),
];
const MAJOR7: &[Interval] = &[
    Interval::new(0),
    Interval::new(4),
    Interval::new(7),
    Interval::new(11),
];
const DOMINANT7: &[Interval] = &[
    Interval::new(0),
    Interval::new(4),
    Interval::new(7),
    Interval::new(10),
];
const MINOR7: &[Interval] = &[
    Interval::new(0),
    Interval::new(3),
    Interval::new(7),
    Interval::new(10),
];
const MINOR_MAJOR7: &[Interval] = &[
    Interval::new(0),
    Interval::new(3),
    Interval::new(7),
    Interval::new(11),
];
const DIMINISHED7: &[Interval] = &[
    Interval::new(0),
    Interval::new(3),
    Interval::new(6),
    Interval::new(9),
];
const HALF_DIMINISHED7: &[Interval] = &[
    Interval::new(0),
    Interval::new(3),
    Interval::new(6),
    Interval::new(10),
];

impl ChordQuality {
    /// Every chord quality, in catalogue order (triads first, then sevenths).
    pub const ALL: &'static [ChordQuality] = &[
        ChordQuality::Major,
        ChordQuality::Minor,
        ChordQuality::Diminished,
        ChordQuality::Augmented,
        ChordQuality::Sus2,
        ChordQuality::Sus4,
        ChordQuality::Major7,
        ChordQuality::Dominant7,
        ChordQuality::Minor7,
        ChordQuality::MinorMajor7,
        ChordQuality::Diminished7,
        ChordQuality::HalfDiminished7,
    ];

    /// Intervals from the root that define this chord quality.
    pub const fn intervals(self) -> &'static [Interval] {
        match self {
            ChordQuality::Major => MAJOR,
            ChordQuality::Minor => MINOR,
            ChordQuality::Diminished => DIMINISHED,
            ChordQuality::Augmented => AUGMENTED,
            ChordQuality::Sus2 => SUS2,
            ChordQuality::Sus4 => SUS4,
            ChordQuality::Major7 => MAJOR7,
            ChordQuality::Dominant7 => DOMINANT7,
            ChordQuality::Minor7 => MINOR7,
            ChordQuality::MinorMajor7 => MINOR_MAJOR7,
            ChordQuality::Diminished7 => DIMINISHED7,
            ChordQuality::HalfDiminished7 => HALF_DIMINISHED7,
        }
    }

    /// Long-form name, e.g. "Dominant 7th", "Half-Diminished 7th".
    pub const fn name(self) -> &'static str {
        match self {
            ChordQuality::Major => "Major",
            ChordQuality::Minor => "Minor",
            ChordQuality::Diminished => "Diminished",
            ChordQuality::Augmented => "Augmented",
            ChordQuality::Sus2 => "Sus2",
            ChordQuality::Sus4 => "Sus4",
            ChordQuality::Major7 => "Major 7th",
            ChordQuality::Dominant7 => "Dominant 7th",
            ChordQuality::Minor7 => "Minor 7th",
            ChordQuality::MinorMajor7 => "Minor-Major 7th",
            ChordQuality::Diminished7 => "Diminished 7th",
            ChordQuality::HalfDiminished7 => "Half-Diminished 7th",
        }
    }

    /// Short symbol appended to a root note for chord names, e.g.
    /// `""` for major, `"m"` for minor, `"m7♭5"` for half-diminished.
    pub const fn symbol(self) -> &'static str {
        match self {
            ChordQuality::Major => "",
            ChordQuality::Minor => "m",
            ChordQuality::Diminished => "dim",
            ChordQuality::Augmented => "aug",
            ChordQuality::Sus2 => "sus2",
            ChordQuality::Sus4 => "sus4",
            ChordQuality::Major7 => "maj7",
            ChordQuality::Dominant7 => "7",
            ChordQuality::Minor7 => "m7",
            ChordQuality::MinorMajor7 => "mM7",
            ChordQuality::Diminished7 => "dim7",
            ChordQuality::HalfDiminished7 => "m7♭5",
        }
    }

    /// True if this is a three-note chord.
    pub const fn is_triad(self) -> bool {
        self.intervals().len() == 3
    }

    /// True if this is a four-note chord built on a 7th.
    pub const fn is_seventh(self) -> bool {
        self.intervals().len() == 4
    }
}

impl fmt::Display for ChordQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A concrete chord: a [`ChordQuality`] anchored at a root [`PitchClass`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Chord {
    pub root: PitchClass,
    pub quality: ChordQuality,
}

impl Chord {
    pub const fn new(root: PitchClass, quality: ChordQuality) -> Self {
        Self { root, quality }
    }

    /// Iterate over the pitch classes of the chord, ascending from the root.
    /// Ports `chordPcs(root, quality)`.
    pub fn pitch_classes(&self) -> impl Iterator<Item = PitchClass> + '_ {
        let root = self.root;
        self.quality.intervals().iter().map(move |&iv| root + iv)
    }

    /// True if `pc` is one of the chord's tones.
    pub fn contains(&self, pc: PitchClass) -> bool {
        self.pitch_classes().any(|p| p == pc)
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.root, self.quality.symbol())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pcs(chord: Chord) -> Vec<u8> {
        chord.pitch_classes().map(|p| p.value()).collect()
    }

    // Tests ported from chordPcs in fretboard-explorer/theory.test.js.

    #[test]
    fn c_major_is_c_e_g() {
        assert_eq!(
            pcs(Chord::new(PitchClass::C, ChordQuality::Major)),
            vec![0, 4, 7]
        );
    }

    #[test]
    fn g_major_is_g_b_d() {
        assert_eq!(
            pcs(Chord::new(PitchClass::G, ChordQuality::Major)),
            vec![7, 11, 2]
        );
    }

    #[test]
    fn a_minor_is_a_c_e() {
        assert_eq!(
            pcs(Chord::new(PitchClass::A, ChordQuality::Minor)),
            vec![9, 0, 4]
        );
    }

    #[test]
    fn catalogue_has_twelve_qualities() {
        assert_eq!(ChordQuality::ALL.len(), 12);
    }

    #[test]
    fn triads_and_sevenths_partition_the_catalogue() {
        let triads = ChordQuality::ALL.iter().filter(|q| q.is_triad()).count();
        let sevenths = ChordQuality::ALL.iter().filter(|q| q.is_seventh()).count();
        assert_eq!(triads, 6);
        assert_eq!(sevenths, 6);
        for q in ChordQuality::ALL {
            assert!(
                q.is_triad() ^ q.is_seventh(),
                "{q:?} should be exactly one of triad/seventh"
            );
        }
    }

    #[test]
    fn dominant7_extends_major_triad_with_minor_seventh() {
        let dom7 = ChordQuality::Dominant7.intervals();
        let major = ChordQuality::Major.intervals();
        // First three intervals of dom7 match the major triad.
        assert_eq!(&dom7[..3], major);
        assert_eq!(dom7[3], Interval::MINOR_SEVENTH);
    }

    #[test]
    fn half_diminished_differs_from_dim7_only_in_seventh() {
        let hdim = ChordQuality::HalfDiminished7.intervals();
        let dim7 = ChordQuality::Diminished7.intervals();
        assert_eq!(&hdim[..3], &dim7[..3]);
        assert_eq!(hdim[3], Interval::MINOR_SEVENTH);
        assert_eq!(dim7[3], Interval::MAJOR_SIXTH);
    }

    #[test]
    fn contains_recognises_chord_tones() {
        let c_maj7 = Chord::new(PitchClass::C, ChordQuality::Major7);
        assert!(c_maj7.contains(PitchClass::C));
        assert!(c_maj7.contains(PitchClass::E));
        assert!(c_maj7.contains(PitchClass::G));
        assert!(c_maj7.contains(PitchClass::B));
        assert!(!c_maj7.contains(PitchClass::D));
        assert!(!c_maj7.contains(PitchClass::A_SHARP));
    }

    #[test]
    fn display_uses_short_symbols() {
        assert_eq!(
            Chord::new(PitchClass::C, ChordQuality::Major).to_string(),
            "C"
        );
        assert_eq!(
            Chord::new(PitchClass::A, ChordQuality::Minor).to_string(),
            "Am"
        );
        assert_eq!(
            Chord::new(PitchClass::G, ChordQuality::Dominant7).to_string(),
            "G7"
        );
        assert_eq!(
            Chord::new(PitchClass::F_SHARP, ChordQuality::Diminished).to_string(),
            "F♯dim"
        );
        assert_eq!(
            Chord::new(PitchClass::B, ChordQuality::HalfDiminished7).to_string(),
            "Bm7♭5"
        );
        assert_eq!(
            Chord::new(PitchClass::C, ChordQuality::Major7).to_string(),
            "Cmaj7"
        );
    }

    #[test]
    fn all_qualities_start_on_root() {
        for q in ChordQuality::ALL {
            assert_eq!(
                q.intervals().first().copied(),
                Some(Interval::UNISON),
                "{q:?} should start on the root"
            );
        }
    }
}
