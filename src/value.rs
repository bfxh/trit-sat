//! Three-valued values.

use std::fmt;

/// A three-valued logic value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ternary {
    /// False (0).
    False,
    /// Unknown (X).
    Unknown,
    /// True (1).
    True,
}

impl Ternary {
    /// All values in lattice order.
    pub const ALL: [Self; 3] = [Self::False, Self::Unknown, Self::True];

    /// Whether this is `False`.
    pub const fn is_false(self) -> bool {
        matches!(self, Self::False)
    }

    /// Whether this is `Unknown`.
    pub const fn is_unknown(self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// Whether this is `True`.
    pub const fn is_true(self) -> bool {
        matches!(self, Self::True)
    }

    /// `False -> 0`, `Unknown -> 1`, `True -> 2`.
    ///
    /// Also the digit order used by counterexample indices.
    pub const fn as_index(self) -> u8 {
        match self {
            Self::False => 0,
            Self::Unknown => 1,
            Self::True => 2,
        }
    }

    /// Inverse of [`Self::as_index`].
    pub const fn from_index(index: u8) -> Self {
        match index {
            0 => Self::False,
            2 => Self::True,
            _ => Self::Unknown,
        }
    }

    /// Parse `0`, `1`, `X` (case-insensitive) or `?`.
    pub fn parse_char(c: char) -> Option<Self> {
        match c {
            '0' => Some(Self::False),
            '1' => Some(Self::True),
            'X' | 'x' | '?' => Some(Self::Unknown),
            _ => None,
        }
    }

    /// The compact character form.
    pub const fn as_char(self) -> char {
        match self {
            Self::False => '0',
            Self::Unknown => 'X',
            Self::True => '1',
        }
    }
}

impl fmt::Display for Ternary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::False => "0",
            Self::Unknown => "X",
            Self::True => "1",
        })
    }
}
