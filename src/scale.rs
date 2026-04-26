//! Scales: a catalogue of 16 scale types and a `Scale` (kind + root) for
//! working with concrete pitch-class sets.

use std::fmt;
use std::str::FromStr;

use crate::interval::Interval;
use crate::note::Note;
use crate::parse::ParseError;
use crate::pitch::PitchClass;
use crate::spelling::spell_heptatonic;

/// One of the 16 scale types in the catalogue.
///
/// Categorized into four groups (see [`ScaleGroup`]):
/// the seven church modes, three pentatonic-family scales, harmonic and
/// melodic minor, and four symmetric scales.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScaleKind {
    Ionian,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Aeolian,
    Locrian,
    MajorPentatonic,
    MinorPentatonic,
    Blues,
    HarmonicMinor,
    MelodicMinor,
    WholeTone,
    DiminishedWholeHalf,
    DiminishedHalfWhole,
    Chromatic,
}

/// High-level grouping used by the original UI to organize the catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScaleGroup {
    Modes,
    Pentatonic,
    HarmonicMelodic,
    Symmetric,
}

const IONIAN_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(2),
    Interval::new(4),
    Interval::new(5),
    Interval::new(7),
    Interval::new(9),
    Interval::new(11),
];
const DORIAN_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(2),
    Interval::new(3),
    Interval::new(5),
    Interval::new(7),
    Interval::new(9),
    Interval::new(10),
];
const PHRYGIAN_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(1),
    Interval::new(3),
    Interval::new(5),
    Interval::new(7),
    Interval::new(8),
    Interval::new(10),
];
const LYDIAN_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(2),
    Interval::new(4),
    Interval::new(6),
    Interval::new(7),
    Interval::new(9),
    Interval::new(11),
];
const MIXOLYDIAN_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(2),
    Interval::new(4),
    Interval::new(5),
    Interval::new(7),
    Interval::new(9),
    Interval::new(10),
];
const AEOLIAN_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(2),
    Interval::new(3),
    Interval::new(5),
    Interval::new(7),
    Interval::new(8),
    Interval::new(10),
];
const LOCRIAN_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(1),
    Interval::new(3),
    Interval::new(5),
    Interval::new(6),
    Interval::new(8),
    Interval::new(10),
];
const MAJOR_PENT_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(2),
    Interval::new(4),
    Interval::new(7),
    Interval::new(9),
];
const MINOR_PENT_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(3),
    Interval::new(5),
    Interval::new(7),
    Interval::new(10),
];
const BLUES_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(3),
    Interval::new(5),
    Interval::new(6),
    Interval::new(7),
    Interval::new(10),
];
const HARM_MIN_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(2),
    Interval::new(3),
    Interval::new(5),
    Interval::new(7),
    Interval::new(8),
    Interval::new(11),
];
const MEL_MIN_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(2),
    Interval::new(3),
    Interval::new(5),
    Interval::new(7),
    Interval::new(9),
    Interval::new(11),
];
const WHOLE_TONE_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(2),
    Interval::new(4),
    Interval::new(6),
    Interval::new(8),
    Interval::new(10),
];
const DIM_WH_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(2),
    Interval::new(3),
    Interval::new(5),
    Interval::new(6),
    Interval::new(8),
    Interval::new(9),
    Interval::new(11),
];
const DIM_HW_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(1),
    Interval::new(3),
    Interval::new(4),
    Interval::new(6),
    Interval::new(7),
    Interval::new(9),
    Interval::new(10),
];
const CHROMATIC_STEPS: &[Interval] = &[
    Interval::new(0),
    Interval::new(1),
    Interval::new(2),
    Interval::new(3),
    Interval::new(4),
    Interval::new(5),
    Interval::new(6),
    Interval::new(7),
    Interval::new(8),
    Interval::new(9),
    Interval::new(10),
    Interval::new(11),
];

impl ScaleKind {
    /// Every scale in the catalogue, in the canonical order used by the UI.
    pub const ALL: &'static [ScaleKind] = &[
        ScaleKind::Ionian,
        ScaleKind::Dorian,
        ScaleKind::Phrygian,
        ScaleKind::Lydian,
        ScaleKind::Mixolydian,
        ScaleKind::Aeolian,
        ScaleKind::Locrian,
        ScaleKind::MajorPentatonic,
        ScaleKind::MinorPentatonic,
        ScaleKind::Blues,
        ScaleKind::HarmonicMinor,
        ScaleKind::MelodicMinor,
        ScaleKind::WholeTone,
        ScaleKind::DiminishedWholeHalf,
        ScaleKind::DiminishedHalfWhole,
        ScaleKind::Chromatic,
    ];

    /// Display name (e.g. "Ionian (major)", "Diminished (W-H)").
    pub const fn name(self) -> &'static str {
        match self {
            ScaleKind::Ionian => "Ionian (major)",
            ScaleKind::Dorian => "Dorian",
            ScaleKind::Phrygian => "Phrygian",
            ScaleKind::Lydian => "Lydian",
            ScaleKind::Mixolydian => "Mixolydian",
            ScaleKind::Aeolian => "Aeolian (minor)",
            ScaleKind::Locrian => "Locrian",
            ScaleKind::MajorPentatonic => "Major Pentatonic",
            ScaleKind::MinorPentatonic => "Minor Pentatonic",
            ScaleKind::Blues => "Blues",
            ScaleKind::HarmonicMinor => "Harmonic Minor",
            ScaleKind::MelodicMinor => "Melodic Minor",
            ScaleKind::WholeTone => "Whole Tone",
            ScaleKind::DiminishedWholeHalf => "Diminished (W-H)",
            ScaleKind::DiminishedHalfWhole => "Diminished (H-W)",
            ScaleKind::Chromatic => "Chromatic",
        }
    }

    /// Intervals from the root, in ascending order. Length varies: 5
    /// (pentatonics), 6 (blues, whole tone), 7 (modes, harmonic/melodic
    /// minor), 8 (diminished), 12 (chromatic).
    pub const fn steps(self) -> &'static [Interval] {
        match self {
            ScaleKind::Ionian => IONIAN_STEPS,
            ScaleKind::Dorian => DORIAN_STEPS,
            ScaleKind::Phrygian => PHRYGIAN_STEPS,
            ScaleKind::Lydian => LYDIAN_STEPS,
            ScaleKind::Mixolydian => MIXOLYDIAN_STEPS,
            ScaleKind::Aeolian => AEOLIAN_STEPS,
            ScaleKind::Locrian => LOCRIAN_STEPS,
            ScaleKind::MajorPentatonic => MAJOR_PENT_STEPS,
            ScaleKind::MinorPentatonic => MINOR_PENT_STEPS,
            ScaleKind::Blues => BLUES_STEPS,
            ScaleKind::HarmonicMinor => HARM_MIN_STEPS,
            ScaleKind::MelodicMinor => MEL_MIN_STEPS,
            ScaleKind::WholeTone => WHOLE_TONE_STEPS,
            ScaleKind::DiminishedWholeHalf => DIM_WH_STEPS,
            ScaleKind::DiminishedHalfWhole => DIM_HW_STEPS,
            ScaleKind::Chromatic => CHROMATIC_STEPS,
        }
    }

    /// Scale-degree labels matching `steps()` index-by-index, e.g.
    /// `["1","2","♭3","4","5","6","♭7"]` for Dorian.
    pub const fn degrees(self) -> &'static [&'static str] {
        match self {
            ScaleKind::Ionian => &["1", "2", "3", "4", "5", "6", "7"],
            ScaleKind::Dorian => &["1", "2", "♭3", "4", "5", "6", "♭7"],
            ScaleKind::Phrygian => &["1", "♭2", "♭3", "4", "5", "♭6", "♭7"],
            ScaleKind::Lydian => &["1", "2", "3", "♯4", "5", "6", "7"],
            ScaleKind::Mixolydian => &["1", "2", "3", "4", "5", "6", "♭7"],
            ScaleKind::Aeolian => &["1", "2", "♭3", "4", "5", "♭6", "♭7"],
            ScaleKind::Locrian => &["1", "♭2", "♭3", "4", "♭5", "♭6", "♭7"],
            ScaleKind::MajorPentatonic => &["1", "2", "3", "5", "6"],
            ScaleKind::MinorPentatonic => &["1", "♭3", "4", "5", "♭7"],
            ScaleKind::Blues => &["1", "♭3", "4", "♭5", "5", "♭7"],
            ScaleKind::HarmonicMinor => &["1", "2", "♭3", "4", "5", "♭6", "7"],
            ScaleKind::MelodicMinor => &["1", "2", "♭3", "4", "5", "6", "7"],
            ScaleKind::WholeTone => &["1", "2", "3", "♯4", "♯5", "♭7"],
            ScaleKind::DiminishedWholeHalf => {
                &["1", "2", "♭3", "4", "♭5", "♭6", "𝄫7", "7"]
            }
            ScaleKind::DiminishedHalfWhole => {
                &["1", "♭2", "♭3", "3", "♭5", "5", "6", "♭7"]
            }
            ScaleKind::Chromatic => &[
                "1", "♭2", "2", "♭3", "3", "4", "♭5", "5", "♭6", "6", "♭7", "7",
            ],
        }
    }

    /// Step-pattern formula in W/H notation, e.g. "W H W W W H W" for Dorian.
    pub const fn formula(self) -> &'static str {
        match self {
            ScaleKind::Ionian => "W W H W W W H",
            ScaleKind::Dorian => "W H W W W H W",
            ScaleKind::Phrygian => "H W W W H W W",
            ScaleKind::Lydian => "W W W H W W H",
            ScaleKind::Mixolydian => "W W H W W H W",
            ScaleKind::Aeolian => "W H W W H W W",
            ScaleKind::Locrian => "H W W H W W W",
            ScaleKind::MajorPentatonic => "W W m3 W m3",
            ScaleKind::MinorPentatonic => "m3 W W m3 W",
            ScaleKind::Blues => "minor pent + ♭5",
            ScaleKind::HarmonicMinor => "W H W W H m3 H",
            ScaleKind::MelodicMinor => "W H W W W W H",
            ScaleKind::WholeTone => "W W W W W W",
            ScaleKind::DiminishedWholeHalf => "W H W H W H W H",
            ScaleKind::DiminishedHalfWhole => "H W H W H W H W",
            ScaleKind::Chromatic => "all semitones",
        }
    }

    /// The catalogue group this scale belongs to.
    pub const fn group(self) -> ScaleGroup {
        match self {
            ScaleKind::Ionian
            | ScaleKind::Dorian
            | ScaleKind::Phrygian
            | ScaleKind::Lydian
            | ScaleKind::Mixolydian
            | ScaleKind::Aeolian
            | ScaleKind::Locrian => ScaleGroup::Modes,
            ScaleKind::MajorPentatonic
            | ScaleKind::MinorPentatonic
            | ScaleKind::Blues => ScaleGroup::Pentatonic,
            ScaleKind::HarmonicMinor | ScaleKind::MelodicMinor => {
                ScaleGroup::HarmonicMelodic
            }
            ScaleKind::WholeTone
            | ScaleKind::DiminishedWholeHalf
            | ScaleKind::DiminishedHalfWhole
            | ScaleKind::Chromatic => ScaleGroup::Symmetric,
        }
    }

    /// Number of distinct pitches in the scale.
    pub const fn note_count(self) -> usize {
        self.steps().len()
    }

    /// True if the scale has exactly seven notes — required for diatonic
    /// letter spelling via [`spell_heptatonic`].
    pub const fn is_heptatonic(self) -> bool {
        self.note_count() == 7
    }
}

impl fmt::Display for ScaleKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Parse a [`ScaleKind`] by name. Accepts the canonical [`ScaleKind::name`]
/// form (`"Ionian (major)"`, `"Diminished (W-H)"`) and the short variant
/// (`"Ionian"`, `"Aeolian"`).
///
/// # Examples
///
/// ```
/// use harmonia::ScaleKind;
///
/// assert_eq!("Ionian".parse::<ScaleKind>().unwrap(), ScaleKind::Ionian);
/// assert_eq!("Ionian (major)".parse::<ScaleKind>().unwrap(), ScaleKind::Ionian);
/// assert_eq!("Harmonic Minor".parse::<ScaleKind>().unwrap(), ScaleKind::HarmonicMinor);
/// assert_eq!("Diminished (W-H)".parse::<ScaleKind>().unwrap(), ScaleKind::DiminishedWholeHalf);
/// ```
impl FromStr for ScaleKind {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Ionian" | "Ionian (major)" => ScaleKind::Ionian,
            "Dorian" => ScaleKind::Dorian,
            "Phrygian" => ScaleKind::Phrygian,
            "Lydian" => ScaleKind::Lydian,
            "Mixolydian" => ScaleKind::Mixolydian,
            "Aeolian" | "Aeolian (minor)" => ScaleKind::Aeolian,
            "Locrian" => ScaleKind::Locrian,
            "Major Pentatonic" => ScaleKind::MajorPentatonic,
            "Minor Pentatonic" => ScaleKind::MinorPentatonic,
            "Blues" => ScaleKind::Blues,
            "Harmonic Minor" => ScaleKind::HarmonicMinor,
            "Melodic Minor" => ScaleKind::MelodicMinor,
            "Whole Tone" => ScaleKind::WholeTone,
            "Diminished (W-H)" => ScaleKind::DiminishedWholeHalf,
            "Diminished (H-W)" => ScaleKind::DiminishedHalfWhole,
            "Chromatic" => ScaleKind::Chromatic,
            other => {
                return Err(ParseError::new(format!(
                    "unknown scale kind: {other:?}"
                )))
            }
        })
    }
}

impl ScaleGroup {
    pub const ALL: &'static [ScaleGroup] = &[
        ScaleGroup::Modes,
        ScaleGroup::Pentatonic,
        ScaleGroup::HarmonicMelodic,
        ScaleGroup::Symmetric,
    ];

    /// Display label matching the JS `SCALE_GROUPS` array.
    pub const fn label(self) -> &'static str {
        match self {
            ScaleGroup::Modes => "Modes",
            ScaleGroup::Pentatonic => "Pentatonic",
            ScaleGroup::HarmonicMelodic => "Harm./Melodic",
            ScaleGroup::Symmetric => "Symmetric",
        }
    }

    /// Scales that belong to this group, in catalogue order.
    pub const fn scales(self) -> &'static [ScaleKind] {
        match self {
            ScaleGroup::Modes => &[
                ScaleKind::Ionian,
                ScaleKind::Dorian,
                ScaleKind::Phrygian,
                ScaleKind::Lydian,
                ScaleKind::Mixolydian,
                ScaleKind::Aeolian,
                ScaleKind::Locrian,
            ],
            ScaleGroup::Pentatonic => &[
                ScaleKind::MajorPentatonic,
                ScaleKind::MinorPentatonic,
                ScaleKind::Blues,
            ],
            ScaleGroup::HarmonicMelodic => {
                &[ScaleKind::HarmonicMinor, ScaleKind::MelodicMinor]
            }
            ScaleGroup::Symmetric => &[
                ScaleKind::WholeTone,
                ScaleKind::DiminishedWholeHalf,
                ScaleKind::DiminishedHalfWhole,
                ScaleKind::Chromatic,
            ],
        }
    }
}

impl fmt::Display for ScaleGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// A concrete scale: a [`ScaleKind`] anchored at a root [`PitchClass`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Scale {
    pub root: PitchClass,
    pub kind: ScaleKind,
}

impl Scale {
    pub const fn new(root: PitchClass, kind: ScaleKind) -> Self {
        Self { root, kind }
    }

    /// Iterate over the pitch classes of this scale, ascending from the root.
    pub fn pitch_classes(&self) -> impl Iterator<Item = PitchClass> + '_ {
        let root = self.root;
        self.kind.steps().iter().map(move |&iv| root + iv)
    }

    /// True if `pc` is one of the scale's pitch classes.
    pub fn contains(&self, pc: PitchClass) -> bool {
        self.pitch_classes().any(|p| p == pc)
    }

    /// Diatonic spelling of the scale tones, with each natural letter A–G
    /// appearing exactly once. Returns `None` for non-heptatonic scales —
    /// fall back to [`PitchClass::default_name`] for sharp-only labels.
    pub fn spelled(&self) -> Option<[Note; 7]> {
        if !self.kind.is_heptatonic() {
            return None;
        }
        let steps = self.kind.steps();
        let array: [Interval; 7] = [
            steps[0], steps[1], steps[2], steps[3], steps[4], steps[5], steps[6],
        ];
        spell_heptatonic(self.root, &array)
    }
}

impl fmt::Display for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.root, self.kind)
    }
}

/// Parse a scale from `"<root> <kind>"`, splitting on the first space.
/// The kind may contain spaces (`"Major Pentatonic"`, `"Diminished (W-H)"`).
///
/// # Examples
///
/// ```
/// use harmonia::{PitchClass, Scale, ScaleKind};
///
/// let g_major: Scale = "G Ionian".parse().unwrap();
/// assert_eq!(g_major, Scale::new(PitchClass::G, ScaleKind::Ionian));
///
/// let pent: Scale = "C Major Pentatonic".parse().unwrap();
/// assert_eq!(pent.kind, ScaleKind::MajorPentatonic);
/// ```
impl FromStr for Scale {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (root_str, kind_str) = s
            .split_once(' ')
            .ok_or_else(|| ParseError::new(format!("scale missing kind: {s:?}")))?;
        let root: PitchClass = root_str.parse()?;
        let kind: ScaleKind = kind_str.parse()?;
        Ok(Scale::new(root, kind))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_has_all_sixteen_scales() {
        assert_eq!(ScaleKind::ALL.len(), 16);
    }

    #[test]
    fn group_membership_partitions_the_catalogue() {
        // Every kind appears in exactly one group.
        for kind in ScaleKind::ALL {
            let group = kind.group();
            assert!(
                group.scales().contains(kind),
                "{kind:?} missing from its own group {group:?}"
            );
            // Not in any other group.
            for other in ScaleGroup::ALL {
                if *other == group {
                    continue;
                }
                assert!(
                    !other.scales().contains(kind),
                    "{kind:?} appears in extra group {other:?}"
                );
            }
        }
        // And every group's scales sum to the full catalogue.
        let total: usize = ScaleGroup::ALL.iter().map(|g| g.scales().len()).sum();
        assert_eq!(total, ScaleKind::ALL.len());
    }

    #[test]
    fn note_counts_match_scale_families() {
        assert_eq!(ScaleKind::Ionian.note_count(), 7);
        assert_eq!(ScaleKind::MajorPentatonic.note_count(), 5);
        assert_eq!(ScaleKind::Blues.note_count(), 6);
        assert_eq!(ScaleKind::WholeTone.note_count(), 6);
        assert_eq!(ScaleKind::DiminishedWholeHalf.note_count(), 8);
        assert_eq!(ScaleKind::Chromatic.note_count(), 12);
    }

    #[test]
    fn is_heptatonic_matches_modes_and_minor_variants() {
        for kind in [
            ScaleKind::Ionian,
            ScaleKind::Dorian,
            ScaleKind::Phrygian,
            ScaleKind::Lydian,
            ScaleKind::Mixolydian,
            ScaleKind::Aeolian,
            ScaleKind::Locrian,
            ScaleKind::HarmonicMinor,
            ScaleKind::MelodicMinor,
        ] {
            assert!(kind.is_heptatonic(), "{kind:?} should be heptatonic");
        }
        for kind in [
            ScaleKind::MajorPentatonic,
            ScaleKind::MinorPentatonic,
            ScaleKind::Blues,
            ScaleKind::WholeTone,
            ScaleKind::DiminishedWholeHalf,
            ScaleKind::DiminishedHalfWhole,
            ScaleKind::Chromatic,
        ] {
            assert!(!kind.is_heptatonic(), "{kind:?} should not be heptatonic");
        }
    }

    #[test]
    fn degrees_align_with_steps() {
        // For every scale, degrees and steps must have equal length.
        for kind in ScaleKind::ALL {
            assert_eq!(
                kind.degrees().len(),
                kind.steps().len(),
                "{kind:?} degree/step length mismatch"
            );
        }
    }

    #[test]
    fn c_major_is_white_keys() {
        let scale = Scale::new(PitchClass::C, ScaleKind::Ionian);
        let pcs: Vec<u8> = scale.pitch_classes().map(|p| p.value()).collect();
        assert_eq!(pcs, vec![0, 2, 4, 5, 7, 9, 11]);
    }

    #[test]
    fn a_minor_is_also_white_keys() {
        let scale = Scale::new(PitchClass::A, ScaleKind::Aeolian);
        let mut pcs: Vec<u8> = scale.pitch_classes().map(|p| p.value()).collect();
        pcs.sort();
        assert_eq!(pcs, vec![0, 2, 4, 5, 7, 9, 11]);
    }

    #[test]
    fn contains_recognises_in_and_out_of_scale_pcs() {
        let g_major = Scale::new(PitchClass::G, ScaleKind::Ionian);
        assert!(g_major.contains(PitchClass::G));
        assert!(g_major.contains(PitchClass::F_SHARP));
        assert!(!g_major.contains(PitchClass::F));
        assert!(!g_major.contains(PitchClass::A_SHARP));
    }

    #[test]
    fn chromatic_contains_every_pc() {
        let scale = Scale::new(PitchClass::C, ScaleKind::Chromatic);
        for v in 0..12 {
            assert!(scale.contains(PitchClass::new(v)));
        }
    }

    #[test]
    fn spelled_g_major_has_distinct_letters() {
        let scale = Scale::new(PitchClass::G, ScaleKind::Ionian);
        let spelled = scale.spelled().expect("heptatonic spells");
        let labels: Vec<String> = spelled.iter().map(|n| n.to_string()).collect();
        assert_eq!(
            labels,
            vec!["G", "A", "B", "C", "D", "E", "F♯"]
        );
    }

    #[test]
    fn spelled_returns_none_for_non_heptatonic_scales() {
        assert!(
            Scale::new(PitchClass::C, ScaleKind::MajorPentatonic)
                .spelled()
                .is_none()
        );
        assert!(
            Scale::new(PitchClass::C, ScaleKind::Blues).spelled().is_none()
        );
        assert!(
            Scale::new(PitchClass::C, ScaleKind::DiminishedWholeHalf)
                .spelled()
                .is_none()
        );
        assert!(
            Scale::new(PitchClass::C, ScaleKind::Chromatic)
                .spelled()
                .is_none()
        );
    }

    #[test]
    fn display_combines_root_and_name() {
        let scale = Scale::new(PitchClass::F_SHARP, ScaleKind::Lydian);
        assert_eq!(scale.to_string(), "F♯ Lydian");
    }

    #[test]
    fn parse_kind_round_trips_via_display() {
        for kind in ScaleKind::ALL {
            let parsed: ScaleKind = kind.to_string().parse().unwrap();
            assert_eq!(parsed, *kind);
        }
    }

    #[test]
    fn parse_kind_accepts_short_forms_for_modes() {
        assert_eq!("Ionian".parse::<ScaleKind>().unwrap(), ScaleKind::Ionian);
        assert_eq!("Aeolian".parse::<ScaleKind>().unwrap(), ScaleKind::Aeolian);
    }

    #[test]
    fn parse_scale_round_trips_via_display() {
        for kind in ScaleKind::ALL {
            for pc_value in 0..12 {
                let scale = Scale::new(PitchClass::new(pc_value), *kind);
                let parsed: Scale = scale.to_string().parse().unwrap();
                assert_eq!(parsed, scale);
            }
        }
    }

    #[test]
    fn parse_scale_handles_multi_word_kinds() {
        let s: Scale = "C Major Pentatonic".parse().unwrap();
        assert_eq!(s.kind, ScaleKind::MajorPentatonic);
        let d: Scale = "F♯ Diminished (W-H)".parse().unwrap();
        assert_eq!(d.root, PitchClass::F_SHARP);
        assert_eq!(d.kind, ScaleKind::DiminishedWholeHalf);
    }

    #[test]
    fn parse_scale_rejects_malformed_input() {
        assert!("".parse::<Scale>().is_err());
        assert!("Cmajor".parse::<Scale>().is_err()); // no space
        assert!("C Banana".parse::<Scale>().is_err()); // unknown kind
        assert!("H Ionian".parse::<Scale>().is_err()); // bad root letter
    }
}
