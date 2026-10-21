//! `trit-sat` command line.

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use trit_sat::{equivalent_exhaustive, equivalent_sat, eval, parse, Algebra, Assignment, Ternary};

#[derive(Debug, Parser)]
#[command(
    name = "trit-sat",
    version,
    about = "SAT-based equivalence checking for three-valued logics"
)]
struct Cli {
    /// Semantics: kleene (default), lukasiewicz, bochvar.
    #[arg(long, short = 'a', global = true, default_value = "kleene")]
    algebra: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Evaluate an expression under an assignment.
    Eval {
        /// Expression, e.g. `a & b | !c`.
        expression: String,
        /// Assignment, e.g. `a=1,b=X`.
        #[arg(long, short = 's', default_value = "")]
        assign: String,
    },
    /// Check whether two expressions are equivalent.
    Equiv {
        /// Left expression.
        left: String,
        /// Right expression.
        right: String,
        /// Use the 3^n reference instead of SAT.
        #[arg(long)]
        exhaustive: bool,
        /// Variable limit for the exhaustive path.
        #[arg(long, default_value_t = 18)]
        max_vars: usize,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let Some(alg) = Algebra::parse(&cli.algebra) else {
        eprintln!(
            "未知语义 {:?}；可选 kleene / lukasiewicz / bochvar",
            cli.algebra
        );
        return ExitCode::from(1);
    };
    match cli.command {
        Command::Eval { expression, assign } => cmd_eval(&expression, &assign, alg),
        Command::Equiv {
            left,
            right,
            exhaustive,
            max_vars,
        } => cmd_equiv(&left, &right, alg, exhaustive, max_vars),
    }
}

fn cmd_eval(expression: &str, assign: &str, alg: Algebra) -> ExitCode {
    let expr = match parse(expression) {
        Ok(expr) => expr,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let assignment = match parse_assignment(assign) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    println!("{}  [{}]", eval(&expr, &assignment, alg), alg.as_str());
    ExitCode::SUCCESS
}

fn cmd_equiv(left: &str, right: &str, alg: Algebra, exhaustive: bool, max_vars: usize) -> ExitCode {
    let a = match parse(left) {
        Ok(expr) => expr,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let b = match parse(right) {
        Ok(expr) => expr,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let method = if exhaustive {
        match equivalent_exhaustive(&a, &b, alg, max_vars) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::from(1);
            }
        }
    } else {
        match equivalent_sat(&a, &b, alg) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::from(1);
            }
        }
    };
    let engine = if exhaustive { "exhaustive" } else { "sat" };
    if method.equivalent {
        println!("等价  [{} / {engine}]", alg.as_str());
        ExitCode::SUCCESS
    } else {
        println!("不等价  [{} / {engine}]", alg.as_str());
        if let Some(assignment) = &method.counterexample {
            let detail = assignment
                .iter()
                .map(|(name, value)| format!("{name}={value}"))
                .collect::<Vec<_>>()
                .join(", ");
            println!("反例：{detail}");
        }
        ExitCode::from(2)
    }
}

fn parse_assignment(src: &str) -> Result<Assignment, String> {
    let mut assignment = Assignment::new();
    for part in src.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (name, value) = part
            .split_once('=')
            .ok_or_else(|| format!("赋值片段缺少 '='：{part}"))?;
        let c = value
            .trim()
            .chars()
            .next()
            .ok_or_else(|| format!("赋值片段缺少值：{part}"))?;
        let value = Ternary::parse_char(c).ok_or_else(|| format!("非法三值 {c:?}"))?;
        assignment.insert(name.trim().to_string(), value);
    }
    Ok(assignment)
}
