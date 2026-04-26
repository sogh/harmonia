//! Pitch classes — the twelve octave-equivalent tones of equal temperament.

use std::fmt;
use std::ops::{Add, Sub};

use crate::interval::Interval;

/// A pitch class, an integer in `0..12` representing a tone independent of
/// octave. Index 0 is C, 1 is C♯/D♭, …, 11 is B.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
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

impl fmt::Display for PitchClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.default_name())
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
}
