//! Evaluation and the exhaustive reference.

use std::collections::BTreeMap;

use crate::algebra::Algebra;
use crate::ast::Expr;
use crate::value::Ternary;

/// A variable assignment.
pub type Assignment = BTreeMap<String, Ternary>;

/// Result of an equivalence check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// Whether the two expressions are equivalent.
    pub equivalent: bool,
    /// A distinguishing assignment when they are not.
    pub counterexample: Option<Assignment>,
    /// Number of variables in the union.
    pub variable_count: usize,
}

/// Exhaustive evaluation errors.
#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    /// The exhaustive path refuses to enumerate more than `max` variables.
    #[error("变量数 {vars} 超过穷举上限 {max}；改用 SAT 路径")]
    ExhaustiveLimit {
        /// Actual variable count.
        vars: usize,
        /// Configured limit.
        max: usize,
    },
}

/// Evaluate an expression under an assignment.
pub fn eval(expr: &Expr, assignment: &Assignment, alg: Algebra) -> Ternary {
    match expr {
        Expr::Const(v) => *v,
        Expr::Var(name) => assignment.get(name).copied().unwrap_or(Ternary::Unknown),
        Expr::Not(a) => alg.not(eval(a, assignment, alg)),
        Expr::And(a, b) => alg.and(eval(a, assignment, alg), eval(b, assignment, alg)),
        Expr::Or(a, b) => alg.or(eval(a, assignment, alg), eval(b, assignment, alg)),
        Expr::Xor(a, b) => alg.strong_add(eval(a, assignment, alg), eval(b, assignment, alg)),
        Expr::Implies(a, b) => alg.implies(eval(a, assignment, alg), eval(b, assignment, alg)),
        Expr::Equiv(a, b) => alg.equiv(eval(a, assignment, alg), eval(b, assignment, alg)),
    }
}

/// Variables of both expressions, first-occurrence order.
pub fn union_vars(a: &Expr, b: &Expr) -> Vec<String> {
    let mut vars = a.variables();
    for v in b.variables() {
        if !vars.contains(&v) {
            vars.push(v);
        }
    }
    vars
}

/// Decode a base-3 assignment index into an assignment.
///
/// Digit order: `0 = False`, `1 = True`, `2 = Unknown`; variable `i` uses the
/// `i`-th digit, matching [`Ternary::as_index`].
pub fn decode_assignment(index: u64, vars: &[String]) -> Assignment {
    let mut assignment = Assignment::new();
    let mut rest = index;
    for name in vars {
        assignment.insert(name.clone(), Ternary::from_index((rest % 3) as u8));
        rest /= 3;
    }
    assignment
}

/// The 3^n reference equivalence check.
///
/// Variables are compiled to array indices and the assignment is advanced in
/// place (base-3 odometer), so a full sweep does not allocate per assignment.
pub fn equivalent_exhaustive(
    a: &Expr,
    b: &Expr,
    alg: Algebra,
    max_vars: usize,
) -> Result<Verdict, EvalError> {
    let vars = union_vars(a, b);
    let n = vars.len();
    if n > max_vars {
        return Err(EvalError::ExhaustiveLimit {
            vars: n,
            max: max_vars,
        });
    }
    let index: std::collections::HashMap<String, usize> = vars
        .iter()
        .enumerate()
        .map(|(i, name)| (name.clone(), i))
        .collect();
    let ca = compile(a, &index);
    let cb = compile(b, &index);
    let mut values = vec![Ternary::False; n];
    let total = 3u64.pow(n as u32);
    for _ in 0..total {
        if eval_compiled(&ca, &values, alg) != eval_compiled(&cb, &values, alg) {
            let mut counterexample = Assignment::new();
            for (name, value) in vars.iter().zip(&values) {
                counterexample.insert(name.clone(), *value);
            }
            return Ok(Verdict {
                equivalent: false,
                counterexample: Some(counterexample),
                variable_count: n,
            });
        }
        if !bump(&mut values) {
            break;
        }
    }
    Ok(Verdict {
        equivalent: true,
        counterexample: None,
        variable_count: n,
    })
}

/// Advance a base-3 odometer (`False -> True -> Unknown -> False`).
fn bump(values: &mut [Ternary]) -> bool {
    for value in values {
        match value {
            Ternary::False => {
                *value = Ternary::True;
                return true;
            }
            Ternary::True => {
                *value = Ternary::Unknown;
                return true;
            }
            Ternary::Unknown => *value = Ternary::False,
        }
    }
    false
}

enum Compiled {
    Const(Ternary),
    Var(usize),
    Not(Box<Compiled>),
    And(Box<Compiled>, Box<Compiled>),
    Or(Box<Compiled>, Box<Compiled>),
    Xor(Box<Compiled>, Box<Compiled>),
    Implies(Box<Compiled>, Box<Compiled>),
    Equiv(Box<Compiled>, Box<Compiled>),
}

fn compile(expr: &Expr, index: &std::collections::HashMap<String, usize>) -> Compiled {
    match expr {
        Expr::Const(v) => Compiled::Const(*v),
        Expr::Var(name) => Compiled::Var(*index.get(name).expect("variable collected")),
        Expr::Not(a) => Compiled::Not(Box::new(compile(a, index))),
        Expr::And(a, b) => Compiled::And(Box::new(compile(a, index)), Box::new(compile(b, index))),
        Expr::Or(a, b) => Compiled::Or(Box::new(compile(a, index)), Box::new(compile(b, index))),
        Expr::Xor(a, b) => Compiled::Xor(Box::new(compile(a, index)), Box::new(compile(b, index))),
        Expr::Implies(a, b) => {
            Compiled::Implies(Box::new(compile(a, index)), Box::new(compile(b, index)))
        }
        Expr::Equiv(a, b) => {
            Compiled::Equiv(Box::new(compile(a, index)), Box::new(compile(b, index)))
        }
    }
}

fn eval_compiled(expr: &Compiled, values: &[Ternary], alg: Algebra) -> Ternary {
    match expr {
        Compiled::Const(v) => *v,
        Compiled::Var(i) => values[*i],
        Compiled::Not(a) => alg.not(eval_compiled(a, values, alg)),
        Compiled::And(a, b) => {
            alg.and(eval_compiled(a, values, alg), eval_compiled(b, values, alg))
        }
        Compiled::Or(a, b) => alg.or(eval_compiled(a, values, alg), eval_compiled(b, values, alg)),
        Compiled::Xor(a, b) => {
            alg.strong_add(eval_compiled(a, values, alg), eval_compiled(b, values, alg))
        }
        Compiled::Implies(a, b) => {
            alg.implies(eval_compiled(a, values, alg), eval_compiled(b, values, alg))
        }
        Compiled::Equiv(a, b) => {
            alg.equiv(eval_compiled(a, values, alg), eval_compiled(b, values, alg))
        }
    }
}
