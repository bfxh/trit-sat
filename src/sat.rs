//! Dual-rail encoding + SAT equivalence check.
//!
//! Each trit becomes two Boolean rails `(t, x)` with the constraint `¬(t ∧ x)`:
//! `False=(0,0)`, `True=(1,0)`, `Unknown=(0,1)`. Every operator adds clauses from
//! its truth table under the chosen algebra, so K3 / Ł3 / B3 share one encoder.
//!
//! Two roots are asserted to be unequal; UNSAT means equivalent. A SAT model is
//! turned back into a counterexample assignment.

use std::collections::{HashMap, HashSet};

use varisat::{ExtendFormula, Lit, Solver, Var};

use crate::algebra::Algebra;
use crate::ast::Expr;
use crate::eval::{union_vars, Assignment, Verdict};
use crate::value::Ternary;

/// SAT encoding errors.
#[derive(Debug, thiserror::Error)]
pub enum SatError {
    /// The solver returned an error.
    #[error("SAT 求解失败: {0}")]
    Solver(String),
}

type Rails = (Var, Var);

struct Encoder {
    solver: Solver<'static>,
    next: usize,
    alg: Algebra,
    vars: HashMap<String, Rails>,
}

impl Encoder {
    fn new(alg: Algebra) -> Self {
        Self {
            solver: Solver::new(),
            next: 0,
            alg,
            vars: HashMap::new(),
        }
    }

    fn lit(var: Var, positive: bool) -> Lit {
        Lit::from_var(var, positive)
    }

    fn fresh(&mut self) -> Rails {
        let t = Var::from_index(self.next);
        let x = Var::from_index(self.next + 1);
        self.next += 2;
        self.solver
            .add_clause(&[Self::lit(t, false), Self::lit(x, false)]);
        (t, x)
    }

    fn fix(&mut self, rails: Rails, value: Ternary) {
        let (t, x) = rails;
        let (t_pos, x_pos) = match value {
            Ternary::False => (false, false),
            Ternary::True => (true, false),
            Ternary::Unknown => (false, true),
        };
        self.solver.add_clause(&[Self::lit(t, t_pos)]);
        self.solver.add_clause(&[Self::lit(x, x_pos)]);
    }

    /// Add the truth table of `f` for one gate.
    ///
    /// For each input combination: if the inputs match, both output rails are
    /// forced. "Output equals a value" is a conjunction of two rails, so each
    /// rail needs its own clause; a single clause would let the other value slip.
    fn table(&mut self, ins: &[Rails], out: Rails, f: impl Fn(&[Ternary]) -> Ternary) {
        let combos = 3usize.pow(ins.len() as u32);
        for idx in 0..combos {
            let mut rest = idx;
            let mut values = Vec::with_capacity(ins.len());
            let mut base = Vec::with_capacity(ins.len() * 2);
            for &(t, x) in ins {
                let value = match rest % 3 {
                    0 => Ternary::False,
                    1 => Ternary::True,
                    _ => Ternary::Unknown,
                };
                rest /= 3;
                values.push(value);
                match value {
                    Ternary::False => {
                        base.push(Self::lit(t, true));
                        base.push(Self::lit(x, true));
                    }
                    Ternary::True => {
                        base.push(Self::lit(t, false));
                        base.push(Self::lit(x, true));
                    }
                    Ternary::Unknown => {
                        base.push(Self::lit(t, true));
                        base.push(Self::lit(x, false));
                    }
                }
            }
            let (t_pos, x_pos) = match f(&values) {
                Ternary::False => (false, false),
                Ternary::True => (true, false),
                Ternary::Unknown => (false, true),
            };
            let mut t_clause = base.clone();
            t_clause.push(Self::lit(out.0, t_pos));
            self.solver.add_clause(&t_clause);
            let mut x_clause = base;
            x_clause.push(Self::lit(out.1, x_pos));
            self.solver.add_clause(&x_clause);
        }
    }

    fn encode(&mut self, expr: &Expr) -> Rails {
        let alg = self.alg;
        match expr {
            Expr::Const(v) => {
                let rails = self.fresh();
                self.fix(rails, *v);
                rails
            }
            Expr::Var(name) => {
                if let Some(rails) = self.vars.get(name) {
                    *rails
                } else {
                    let rails = self.fresh();
                    self.vars.insert(name.clone(), rails);
                    rails
                }
            }
            Expr::Not(a) => {
                let ra = self.encode(a);
                let out = self.fresh();
                self.table(&[ra], out, |v| alg.not(v[0]));
                out
            }
            Expr::And(a, b) => {
                let ra = self.encode(a);
                let rb = self.encode(b);
                let out = self.fresh();
                self.table(&[ra, rb], out, |v| alg.and(v[0], v[1]));
                out
            }
            Expr::Or(a, b) => {
                let ra = self.encode(a);
                let rb = self.encode(b);
                let out = self.fresh();
                self.table(&[ra, rb], out, |v| alg.or(v[0], v[1]));
                out
            }
            Expr::Xor(a, b) => {
                let ra = self.encode(a);
                let rb = self.encode(b);
                let out = self.fresh();
                self.table(&[ra, rb], out, |v| alg.strong_add(v[0], v[1]));
                out
            }
            Expr::Implies(a, b) => {
                let ra = self.encode(a);
                let rb = self.encode(b);
                let out = self.fresh();
                self.table(&[ra, rb], out, |v| alg.implies(v[0], v[1]));
                out
            }
            Expr::Equiv(a, b) => {
                let ra = self.encode(a);
                let rb = self.encode(b);
                let out = self.fresh();
                self.table(&[ra, rb], out, |v| alg.equiv(v[0], v[1]));
                out
            }
        }
    }
}

/// Decide equivalence with a SAT solver.
pub fn equivalent_sat(a: &Expr, b: &Expr, alg: Algebra) -> Result<Verdict, SatError> {
    let vars = union_vars(a, b);
    let mut enc = Encoder::new(alg);
    for name in &vars {
        let rails = enc.fresh();
        enc.vars.insert(name.clone(), rails);
    }
    let ra = enc.encode(a);
    let rb = enc.encode(b);

    // Not equal: forbid both False, both True, and both Unknown.
    let l = |v: Var, p: bool| Encoder::lit(v, p);
    enc.solver
        .add_clause(&[l(ra.0, true), l(ra.1, true), l(rb.0, true), l(rb.1, true)]);
    enc.solver
        .add_clause(&[l(ra.0, false), l(ra.1, true), l(rb.0, false), l(rb.1, true)]);
    enc.solver
        .add_clause(&[l(ra.0, true), l(ra.1, false), l(rb.0, true), l(rb.1, false)]);

    let sat = enc
        .solver
        .solve()
        .map_err(|e| SatError::Solver(e.to_string()))?;
    if !sat {
        return Ok(Verdict {
            equivalent: true,
            counterexample: None,
            variable_count: vars.len(),
        });
    }

    let model = enc
        .solver
        .model()
        .ok_or_else(|| SatError::Solver("SAT 但没有模型".to_string()))?;
    let trues: HashSet<Var> = model
        .iter()
        .filter(|lit| lit.is_positive())
        .map(|lit| lit.var())
        .collect();
    let mut assignment = Assignment::new();
    for name in &vars {
        let (t, x) = enc.vars[name];
        let value = if trues.contains(&t) {
            Ternary::True
        } else if trues.contains(&x) {
            Ternary::Unknown
        } else {
            Ternary::False
        };
        assignment.insert(name.clone(), value);
    }
    Ok(Verdict {
        equivalent: false,
        counterexample: Some(assignment),
        variable_count: vars.len(),
    })
}
