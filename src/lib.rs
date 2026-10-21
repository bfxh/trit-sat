//! SAT-based equivalence checking for three-valued logics.
//!
//! Two engines are exposed:
//!
//! - [`equivalent_exhaustive`] enumerates all `3^n` assignments (the reference,
//!   capped at a configurable variable count);
//! - [`equivalent_sat`] encodes the problem with dual rails and asks a SAT solver,
//!   which is the reason this crate exists.
//!
//! ```
//! use trit_sat::{equivalent_sat, parse, Algebra};
//! let a = parse("a & (b | c)").unwrap();
//! let b = parse("(a & b) | (a & c)").unwrap();
//! assert!(equivalent_sat(&a, &b, Algebra::Kleene).unwrap().equivalent);
//! ```

pub mod algebra;
pub mod ast;
pub mod eval;
pub mod parser;
pub mod sat;
pub mod value;

pub use algebra::Algebra;
pub use ast::Expr;
pub use eval::{
    decode_assignment, equivalent_exhaustive, eval, union_vars, Assignment, EvalError, Verdict,
};
pub use parser::{parse, ParseError};
pub use sat::{equivalent_sat, SatError};
pub use value::Ternary;

/// Crate version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
