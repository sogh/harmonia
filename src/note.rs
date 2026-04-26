//! Named notes: a letter A–G plus an accidental.

use std::fmt;
use std::str::FromStr;

use crate::parse::ParseError;
use crate::pitch::PitchClass;

/// One of the seven natural letters used to name notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

/// Parse a note like `"C"`, `"F#"`, `"Bb"`, `"C♯"`, `"E𝄫"`.
///
/// Letter is a single uppercase character A–G. Accidental is one of:
/// `""` (natural), `"#"`/`"♯"`, `"b"`/`"♭"`, `"x"`/`"𝄪"`, or
/// `"bb"`/`"♭♭"`/`"𝄫"`.
///
/// # Examples
///
/// ```
/// use harmonia::{Accidental, Letter, Note};
///
/// let f_sharp: Note = "F#".parse().unwrap();
/// assert_eq!(f_sharp, Note::new(Letter::F, Accidental::Sharp));
///
/// // Round-trips through Display.
/// let n = Note::new(Letter::B, Accidental::Flat);
/// assert_eq!(n.to_string().parse::<Note>().unwrap(), n);
/// ```
impl FromStr for Note {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let letter = match chars.next() {
            Some('C') => Letter::C,
            Some('D') => Letter::D,
            Some('E') => Letter::E,
            Some('F') => Letter::F,
            Some('G') => Letter::G,
            Some('A') => Letter::A,
            Some('B') => Letter::B,
            Some(c) => {
                return Err(ParseError::new(format!(
                    "invalid note letter: {c:?}"
                )))
            }
            None => return Err(ParseError::new("empty note name")),
        };
        let rest = chars.as_str();
        let accidental = match rest {
            "" => Accidental::Natural,
            "#" | "♯" => Accidental::Sharp,
            "b" | "♭" => Accidental::Flat,
            "x" | "𝄪" => Accidental::DoubleSharp,
            "bb" | "♭♭" | "𝄫" => Accidental::DoubleFlat,
            other => {
                return Err(ParseError::new(format!(
                    "invalid accidental: {other:?}"
                )))
            }
        };
        Ok(Note::new(letter, accidental))
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

    #[test]
    fn parse_naturals() {
        for letter in [Letter::C, Letter::D, Letter::E, Letter::F, Letter::G, Letter::A, Letter::B] {
            let s = letter.to_string();
            let parsed: Note = s.parse().unwrap();
            assert_eq!(parsed, Note::natural(letter));
        }
    }

    #[test]
    fn parse_accepts_ascii_and_unicode_accidentals() {
        let f_sharp = Note::new(Letter::F, Accidental::Sharp);
        assert_eq!("F#".parse::<Note>().unwrap(), f_sharp);
        assert_eq!("F♯".parse::<Note>().unwrap(), f_sharp);

        let b_flat = Note::new(Letter::B, Accidental::Flat);
        assert_eq!("Bb".parse::<Note>().unwrap(), b_flat);
        assert_eq!("B♭".parse::<Note>().unwrap(), b_flat);

        let e_dflat = Note::new(Letter::E, Accidental::DoubleFlat);
        assert_eq!("Ebb".parse::<Note>().unwrap(), e_dflat);
        assert_eq!("E♭♭".parse::<Note>().unwrap(), e_dflat);
        assert_eq!("E𝄫".parse::<Note>().unwrap(), e_dflat);

        let g_dsharp = Note::new(Letter::G, Accidental::DoubleSharp);
        assert_eq!("Gx".parse::<Note>().unwrap(), g_dsharp);
        assert_eq!("G𝄪".parse::<Note>().unwrap(), g_dsharp);
    }

    #[test]
    fn parse_round_trips_for_every_letter_accidental_pair() {
        let letters = [Letter::C, Letter::D, Letter::E, Letter::F, Letter::G, Letter::A, Letter::B];
        let accidentals = [
            Accidental::DoubleFlat,
            Accidental::Flat,
            Accidental::Natural,
            Accidental::Sharp,
            Accidental::DoubleSharp,
        ];
        for letter in letters {
            for accidental in accidentals {
                let note = Note::new(letter, accidental);
                let parsed: Note = note.to_string().parse().unwrap();
                assert_eq!(parsed, note);
            }
        }
    }

    #[test]
    fn parse_rejects_invalid_inputs() {
        assert!("".parse::<Note>().is_err());
        assert!("H".parse::<Note>().is_err());
        assert!("c".parse::<Note>().is_err()); // lowercase letter
        assert!("C?".parse::<Note>().is_err()); // bogus accidental
        assert!("C#x".parse::<Note>().is_err()); // multi-accidental mix
    }
}
