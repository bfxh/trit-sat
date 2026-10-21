//! SAT-based equivalence checking for three-valued logics.
//!
//! The symbolic engine lands in the first PR; see the repository README.

/// Crate version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
