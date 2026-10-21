//! Kleene / Łukasiewicz / Bochvar semantics.

use crate::value::Ternary;

/// The three-valued semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algebra {
    /// Kleene strong three-valued logic: `X` is "unknown", the lattice midpoint.
    Kleene,
    /// Łukasiewicz three-valued logic: `X` is a value between `0` and `1`.
    Lukasiewicz,
    /// Bochvar three-valued logic: `X` is poisonous and absorbs everything.
    Bochvar,
}

impl Algebra {
    /// All algebras.
    pub const ALL: [Self; 3] = [Self::Kleene, Self::Lukasiewicz, Self::Bochvar];

    /// Parse `kleene`/`k3`, `lukasiewicz`/`l3`, `bochvar`/`b3`.
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "kleene" | "k3" => Some(Self::Kleene),
            "lukasiewicz" | "l3" => Some(Self::Lukasiewicz),
            "bochvar" | "b3" => Some(Self::Bochvar),
            _ => None,
        }
    }

    /// Stable name used in output.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Kleene => "kleene",
            Self::Lukasiewicz => "lukasiewicz",
            Self::Bochvar => "bochvar",
        }
    }

    /// Negation.
    pub const fn not(self, a: Ternary) -> Ternary {
        match a {
            Ternary::False => Ternary::True,
            Ternary::Unknown => Ternary::Unknown,
            Ternary::True => Ternary::False,
        }
    }

    /// Conjunction.
    pub const fn and(self, a: Ternary, b: Ternary) -> Ternary {
        match self {
            Self::Bochvar => {
                if a.is_unknown() || b.is_unknown() {
                    Ternary::Unknown
                } else if a.is_true() && b.is_true() {
                    Ternary::True
                } else {
                    Ternary::False
                }
            }
            Self::Kleene | Self::Lukasiewicz => match (a, b) {
                (Ternary::False, _) | (_, Ternary::False) => Ternary::False,
                (Ternary::True, Ternary::True) => Ternary::True,
                _ => Ternary::Unknown,
            },
        }
    }

    /// Disjunction.
    pub const fn or(self, a: Ternary, b: Ternary) -> Ternary {
        match self {
            Self::Bochvar => {
                if a.is_unknown() || b.is_unknown() {
                    Ternary::Unknown
                } else if a.is_true() || b.is_true() {
                    Ternary::True
                } else {
                    Ternary::False
                }
            }
            Self::Kleene | Self::Lukasiewicz => match (a, b) {
                (Ternary::True, _) | (_, Ternary::True) => Ternary::True,
                (Ternary::False, Ternary::False) => Ternary::False,
                _ => Ternary::Unknown,
            },
        }
    }

    /// Strong addition `⊕`.
    ///
    /// Łukasiewicz: `min(1, a + b)` with `X = 0.5`, hence `X ⊕ X = 1`.
    /// Kleene / Bochvar: `X` propagates.
    pub const fn strong_add(self, a: Ternary, b: Ternary) -> Ternary {
        match self {
            Self::Lukasiewicz => {
                let sum = a.as_index() + b.as_index();
                if sum >= 2 {
                    Ternary::True
                } else if sum == 0 {
                    Ternary::False
                } else {
                    Ternary::Unknown
                }
            }
            Self::Kleene | Self::Bochvar => {
                if a.is_unknown() || b.is_unknown() {
                    Ternary::Unknown
                } else if a.is_true() && b.is_true() {
                    Ternary::True
                } else {
                    Ternary::False
                }
            }
        }
    }

    /// Implication.
    ///
    /// Kleene: material implication `¬a ∨ b`.
    /// Łukasiewicz: `min(1, 1 - a + b)`, hence `X ⇒ X = 1`.
    /// Bochvar: `X` poisons.
    pub const fn implies(self, a: Ternary, b: Ternary) -> Ternary {
        match self {
            Self::Kleene => self.or(self.not(a), b),
            Self::Bochvar => {
                if a.is_unknown() || b.is_unknown() {
                    Ternary::Unknown
                } else if a.is_true() {
                    b
                } else {
                    Ternary::True
                }
            }
            Self::Lukasiewicz => {
                let diff = 2 + b.as_index() as i8 - a.as_index() as i8;
                if diff >= 2 {
                    Ternary::True
                } else if diff <= 0 {
                    Ternary::False
                } else {
                    Ternary::Unknown
                }
            }
        }
    }

    /// Equivalence: `(a ⇒ b) ∧ (b ⇒ a)`.
    pub const fn equiv(self, a: Ternary, b: Ternary) -> Ternary {
        self.and(self.implies(a, b), self.implies(b, a))
    }
}
