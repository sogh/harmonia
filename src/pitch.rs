//! Pitch classes — the twelve octave-equivalent tones of equal temperament.

use std::fmt;
use std::ops::{Add, Sub};
use std::str::FromStr;

use crate::interval::Interval;
use crate::note::Note;
use crate::parse::ParseError;

/// A pitch class, an integer in `0..12` representing a tone independent of
/// octave. Index 0 is C, 1 is C♯/D♭, …, 11 is B.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "u8", from = "u8"))]
pub struct PitchClass(u8);

const SHARP_NAMES: [&str; 12] = [
    "C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B",
];

impl PitchClass {
    pub const C: Self = Self(0);
    pub const C_SHARP: Self = Self(1);
    pub const D: Self = Self(2);
    pub const D_SHARP: Self = Self(3);
    pub const E: Self = Self(4);
    pub const F: Self = Self(5);
    pub const F_SHARP: Self = Self(6);
    pub const G: Self = Self(7);
    pub const G_SHARP: Self = Self(8);
    pub const A: Self = Self(9);
    pub const A_SHARP: Self = Self(10);
    pub const B: Self = Self(11);

    /// Construct a pitch class from any `u8`, reducing modulo 12.
    pub const fn new(value: u8) -> Self {
        Self(value % 12)
    }

    /// The underlying index in `0..12`.
    pub const fn value(self) -> u8 {
        self.0
    }

    /// Default sharp-only spelling — useful when you have no key context.
    /// Use [`crate::spelling::spell_heptatonic`] when you need diatonic
    /// spelling that respects key signature.
    pub fn default_name(self) -> &'static str {
        SHARP_NAMES[self.0 as usize]
    }
}

impl From<u8> for PitchClass {
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

impl From<PitchClass> for u8 {
    fn from(pc: PitchClass) -> u8 {
        pc.0
    }
}

impl Add<Interval> for PitchClass {
    type Output = PitchClass;
    fn add(self, rhs: Interval) -> PitchClass {
        let total = self.0 as u16 + rhs.semitones();
        PitchClass((total % 12) as u8)
    }
}

/// Difference between two pitch classes, going upward, in `0..12`.
impl Sub for PitchClass {
    type Output = Interval;
    fn sub(self, rhs: PitchClass) -> Interval {
        let diff = (self.0 as i16 - rhs.0 as i16).rem_euclid(12);
        Interval::new(diff as u16)
    }
}

/// Pitch class shifted **down** by an interval. Wraps modulo 12.
impl Sub<Interval> for PitchClass {
    type Output = PitchClass;
    fn sub(self, rhs: Interval) -> PitchClass {
        let total = self.0 as i32 - rhs.semitones() as i32;
        PitchClass(total.rem_euclid(12) as u8)
    }
}

impl fmt::Display for PitchClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.default_name())
    }
}

/// Parse a pitch class from a note name, ignoring spelling.
///
/// Accepts the same syntax as [`Note::from_str`]; the result is the
/// pitch class of the named note, so `"C#"`, `"Db"`, `"B##"` (if it
/// were spelled that way) all map to the same `PitchClass`.
///
/// # Examples
///
/// ```
/// use harmonia::PitchClass;
///
/// let c_sharp: PitchClass = "C#".parse().unwrap();
/// let d_flat:  PitchClass = "Db".parse().unwrap();
/// assert_eq!(c_sharp, d_flat);
/// assert_eq!(c_sharp, PitchClass::C_SHARP);
/// ```
impl FromStr for PitchClass {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let note: Note = s.parse()?;
        Ok(note.pitch_class())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_reduces_mod_12() {
        assert_eq!(PitchClass::new(12).value(), 0);
        assert_eq!(PitchClass::new(13).value(), 1);
        assert_eq!(PitchClass::new(255).value(), 255 % 12);
    }

    #[test]
    fn default_name_uses_sharps() {
        assert_eq!(PitchClass::C.default_name(), "C");
        assert_eq!(PitchClass::C_SHARP.default_name(), "C♯");
        assert_eq!(PitchClass::A_SHARP.default_name(), "A♯");
    }

    #[test]
    fn add_interval_wraps_octave() {
        assert_eq!(PitchClass::G + Interval::PERFECT_FIFTH, PitchClass::D);
        assert_eq!(PitchClass::C + Interval::OCTAVE, PitchClass::C);
        assert_eq!(PitchClass::B + Interval::MINOR_SECOND, PitchClass::C);
    }

    #[test]
    fn subtraction_returns_upward_interval() {
        assert_eq!(PitchClass::G - PitchClass::C, Interval::PERFECT_FIFTH);
        assert_eq!(PitchClass::C - PitchClass::G, Interval::PERFECT_FOURTH);
        assert_eq!(PitchClass::C - PitchClass::C, Interval::UNISON);
    }

    #[test]
    fn subtract_interval_wraps_octave() {
        // G - P5 = C
        assert_eq!(PitchClass::G - Interval::PERFECT_FIFTH, PitchClass::C);
        // C - octave = C
        assert_eq!(PitchClass::C - Interval::OCTAVE, PitchClass::C);
        // C - half step wraps to B
        assert_eq!(PitchClass::C - Interval::MINOR_SECOND, PitchClass::B);
    }

    #[test]
    fn add_and_sub_are_inverse() {
        for v in 0..12 {
            let pc = PitchClass::new(v);
            for semitones in 0..24 {
                let iv = Interval::new(semitones);
                assert_eq!(pc + iv - iv, pc, "pc {v}, iv {semitones}");
            }
        }
    }

    #[test]
    fn parse_accepts_sharp_and_flat_names() {
        assert_eq!("C".parse::<PitchClass>().unwrap(), PitchClass::C);
        assert_eq!("C#".parse::<PitchClass>().unwrap(), PitchClass::C_SHARP);
        assert_eq!("Db".parse::<PitchClass>().unwrap(), PitchClass::C_SHARP);
        assert_eq!("F♯".parse::<PitchClass>().unwrap(), PitchClass::F_SHARP);
        assert_eq!("G♭".parse::<PitchClass>().unwrap(), PitchClass::F_SHARP);
        // B♯ wraps to C, C♭ wraps to B.
        assert_eq!("B#".parse::<PitchClass>().unwrap(), PitchClass::C);
        assert_eq!("Cb".parse::<PitchClass>().unwrap(), PitchClass::B);
    }

    #[test]
    fn parse_round_trips_default_names() {
        for v in 0..12 {
            let pc = PitchClass::new(v);
            let parsed: PitchClass = pc.to_string().parse().unwrap();
            assert_eq!(parsed, pc);
        }
    }
}
