//! Parser error type shared by every [`FromStr`](std::str::FromStr)
//! implementation in the crate.

use std::fmt;

/// Error returned when parsing a music-theory value (a note, chord, scale,
/// etc.) fails. The message is human-readable but its exact text is not
/// part of the stable API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    /// Construct a [`ParseError`] from an arbitrary message.
    /// Crate-internal — external callers receive `ParseError` as the
    /// `Err` variant of `FromStr` impls.
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseError {}
