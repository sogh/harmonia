//! Intervals measured in semitones.

use std::fmt;

/// A musical interval, expressed as a non-negative number of semitones.
///
/// Compound intervals (greater than an octave) are permitted; values are not
/// reduced modulo 12. Use [`PitchClass`](crate::PitchClass) arithmetic when
/// you want octave-equivalent behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Interval(u16);

impl Interval {
    pub const UNISON: Self = Self(0);
    pub const MINOR_SECOND: Self = Self(1);
    pub const MAJOR_SECOND: Self = Self(2);
    pub const MINOR_THIRD: Self = Self(3);
    pub const MAJOR_THIRD: Self = Self(4);
    pub const PERFECT_FOURTH: Self = Self(5);
    pub const TRITONE: Self = Self(6);
    pub const PERFECT_FIFTH: Self = Self(7);
    pub const MINOR_SIXTH: Self = Self(8);
    pub const MAJOR_SIXTH: Self = Self(9);
    pub const MINOR_SEVENTH: Self = Self(10);
    pub const MAJOR_SEVENTH: Self = Self(11);
    pub const OCTAVE: Self = Self(12);

    pub const fn new(semitones: u16) -> Self {
        Self(semitones)
    }

    pub const fn semitones(self) -> u16 {
        self.0
    }
}

impl From<u16> for Interval {
    fn from(semitones: u16) -> Self {
        Self::new(semitones)
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}st", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_constants_have_expected_semitones() {
        assert_eq!(Interval::UNISON.semitones(), 0);
        assert_eq!(Interval::MAJOR_THIRD.semitones(), 4);
        assert_eq!(Interval::PERFECT_FIFTH.semitones(), 7);
        assert_eq!(Interval::OCTAVE.semitones(), 12);
    }

    #[test]
    fn ordering_is_by_semitones() {
        assert!(Interval::MINOR_THIRD < Interval::MAJOR_THIRD);
        assert!(Interval::OCTAVE > Interval::MAJOR_SEVENTH);
    }
}
