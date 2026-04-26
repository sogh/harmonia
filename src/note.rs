//! Named notes: a letter A–G plus an accidental.

use std::fmt;

use crate::pitch::PitchClass;

/// One of the seven natural letters used to name notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Letter {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl Letter {
    /// The natural pitch class of this letter (no accidental applied).
    pub const fn pitch_class(self) -> PitchClass {
        match self {
            Letter::C => PitchClass::C,
            Letter::D => PitchClass::D,
            Letter::E => PitchClass::E,
            Letter::F => PitchClass::F,
            Letter::G => PitchClass::G,
            Letter::A => PitchClass::A,
            Letter::B => PitchClass::B,
        }
    }

    /// The single uppercase character used to display this letter.
    pub const fn symbol(self) -> char {
        match self {
            Letter::C => 'C',
            Letter::D => 'D',
            Letter::E => 'E',
            Letter::F => 'F',
            Letter::G => 'G',
            Letter::A => 'A',
            Letter::B => 'B',
        }
    }
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Letter::C => "C",
            Letter::D => "D",
            Letter::E => "E",
            Letter::F => "F",
            Letter::G => "G",
            Letter::A => "A",
            Letter::B => "B",
        })
    }
}

/// An accidental applied to a letter: from double-flat to double-sharp.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Accidental {
    DoubleFlat,
    Flat,
    #[default]
    Natural,
    Sharp,
    DoubleSharp,
}

impl Accidental {
    /// Semitone offset applied to the natural letter (`-2`..=`+2`).
    pub const fn offset(self) -> i8 {
        match self {
            Accidental::DoubleFlat => -2,
            Accidental::Flat => -1,
            Accidental::Natural => 0,
            Accidental::Sharp => 1,
            Accidental::DoubleSharp => 2,
        }
    }

    /// Unicode glyph for the accidental. Natural renders as the empty string
    /// so a [`Note`] like `C♮` displays simply as `C`.
    pub const fn symbol(self) -> &'static str {
        match self {
            Accidental::DoubleFlat => "𝄫",
            Accidental::Flat => "♭",
            Accidental::Natural => "",
            Accidental::Sharp => "♯",
            Accidental::DoubleSharp => "𝄪",
        }
    }
}

impl fmt::Display for Accidental {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}

/// A named note: a letter plus an accidental. Two `Note`s with different
/// spellings of the same pitch class (e.g. `F♯` vs `G♭`) are not equal as
/// `Note`s but have the same [`PitchClass`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Note {
    pub letter: Letter,
    pub accidental: Accidental,
}

impl Note {
    pub const fn new(letter: Letter, accidental: Accidental) -> Self {
        Self { letter, accidental }
    }

    /// A natural (no-accidental) note.
    pub const fn natural(letter: Letter) -> Self {
        Self::new(letter, Accidental::Natural)
    }

    /// The pitch class of this spelled note.
    pub fn pitch_class(self) -> PitchClass {
        let base = self.letter.pitch_class().value() as i16;
        PitchClass::new(((base + self.accidental.offset() as i16).rem_euclid(12)) as u8)
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.letter, self.accidental)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letter_pitch_classes_match_diatonic_steps() {
        assert_eq!(Letter::C.pitch_class(), PitchClass::C);
        assert_eq!(Letter::E.pitch_class(), PitchClass::E);
        assert_eq!(Letter::F.pitch_class(), PitchClass::F);
        assert_eq!(Letter::B.pitch_class(), PitchClass::B);
    }

    #[test]
    fn note_pitch_class_applies_accidental() {
        assert_eq!(
            Note::new(Letter::F, Accidental::Sharp).pitch_class(),
            PitchClass::F_SHARP
        );
        assert_eq!(
            Note::new(Letter::G, Accidental::Flat).pitch_class(),
            PitchClass::F_SHARP
        );
        assert_eq!(
            Note::new(Letter::C, Accidental::Flat).pitch_class(),
            PitchClass::B
        );
        assert_eq!(
            Note::new(Letter::B, Accidental::Sharp).pitch_class(),
            PitchClass::C
        );
    }

    #[test]
    fn note_display_uses_unicode_accidentals() {
        assert_eq!(Note::natural(Letter::G).to_string(), "G");
        assert_eq!(
            Note::new(Letter::F, Accidental::Sharp).to_string(),
            "F♯"
        );
        assert_eq!(
            Note::new(Letter::B, Accidental::Flat).to_string(),
            "B♭"
        );
        assert_eq!(
            Note::new(Letter::E, Accidental::DoubleFlat).to_string(),
            "E𝄫"
        );
    }
}
