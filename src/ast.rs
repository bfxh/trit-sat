//! Expression AST.

use std::collections::HashSet;

use crate::value::Ternary;

/// An expression over three-valued variables.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    /// Constant.
    Const(Ternary),
    /// Variable.
    Var(String),
    /// Negation.
    Not(Box<Expr>),
    /// Conjunction.
    And(Box<Expr>, Box<Expr>),
    /// Disjunction.
    Or(Box<Expr>, Box<Expr>),
    /// Strong addition.
    Xor(Box<Expr>, Box<Expr>),
    /// Implication.
    Implies(Box<Expr>, Box<Expr>),
    /// Equivalence.
    Equiv(Box<Expr>, Box<Expr>),
}

impl Expr {
    /// Negation node.
    pub fn logical_not(a: Expr) -> Self {
        Self::Not(Box::new(a))
    }

    /// Conjunction node.
    pub fn and(a: Expr, b: Expr) -> Self {
        Self::And(Box::new(a), Box::new(b))
    }

    /// Disjunction node.
    pub fn or(a: Expr, b: Expr) -> Self {
        Self::Or(Box::new(a), Box::new(b))
    }

    /// Strong addition node.
    pub fn xor(a: Expr, b: Expr) -> Self {
        Self::Xor(Box::new(a), Box::new(b))
    }

    /// Implication node.
    pub fn implies(a: Expr, b: Expr) -> Self {
        Self::Implies(Box::new(a), Box::new(b))
    }

    /// Equivalence node.
    pub fn equiv(a: Expr, b: Expr) -> Self {
        Self::Equiv(Box::new(a), Box::new(b))
    }

    /// Variables in first-occurrence order.
    pub fn variables(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        self.collect_vars(&mut out, &mut seen);
        out
    }

    fn collect_vars(&self, out: &mut Vec<String>, seen: &mut HashSet<String>) {
        match self {
            Self::Const(_) => {}
            Self::Var(name) => {
                if seen.insert(name.clone()) {
                    out.push(name.clone());
                }
            }
            Self::Not(a) => a.collect_vars(out, seen),
            Self::And(a, b)
            | Self::Or(a, b)
            | Self::Xor(a, b)
            | Self::Implies(a, b)
            | Self::Equiv(a, b) => {
                a.collect_vars(out, seen);
                b.collect_vars(out, seen);
            }
        }
    }
}
