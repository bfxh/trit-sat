use trit_sat::{
    equivalent_exhaustive, equivalent_sat, eval, parse, Algebra, Expr, Ternary, Verdict,
};

fn check_pair(left: &str, right: &str, alg: Algebra) {
    let a = parse(left).expect("left parses");
    let b = parse(right).expect("right parses");
    let reference = equivalent_exhaustive(&a, &b, alg, 8).expect("small formula");
    let sat = equivalent_sat(&a, &b, alg).expect("SAT runs");
    assert_eq!(
        reference.equivalent, sat.equivalent,
        "{alg:?}: {left} vs {right}"
    );
    assert_eq!(reference.variable_count, sat.variable_count);
    if let Some(counterexample) = &sat.counterexample {
        assert_ne!(
            eval(&a, counterexample, alg),
            eval(&b, counterexample, alg),
            "SAT counterexample must distinguish: {alg:?} {left} vs {right}"
        );
    }
}

#[test]
fn sat_agrees_with_exhaustive() {
    let pairs = [
        ("a & b", "b & a"),
        ("a | b", "b | a"),
        ("a ^ b", "b ^ a"),
        ("a => b", "!a | b"),
        ("a ≡ b", "(a => b) & (b => a)"),
        ("a & (b | c)", "(a & b) | (a & c)"),
        ("a | (b & c)", "(a | b) & (a | c)"),
        ("a & 0", "0"),
        ("a & 1", "a"),
        ("a | X", "X"),
        ("a & X", "0"),
        ("!(a | b)", "!a & !b"),
        ("!(a & b)", "!a | !b"),
        ("a", "b"),
        ("a ^ a", "X"),
    ];
    for alg in Algebra::ALL {
        for (left, right) in pairs {
            check_pair(left, right, alg);
        }
    }
}

#[test]
fn sat_handles_hundred_variables() {
    let vars: Vec<String> = (0..100).map(|i| format!("a{i}")).collect();
    let chain = vars
        .iter()
        .map(|v| Expr::Var(v.clone()))
        .reduce(Expr::and)
        .unwrap();
    let same = equivalent_sat(&chain, &chain, Algebra::Kleene).expect("SAT runs");
    assert!(same.equivalent);

    let shorter = vars[..99]
        .iter()
        .map(|v| Expr::Var(v.clone()))
        .reduce(Expr::and)
        .unwrap();
    let diff = equivalent_sat(&chain, &shorter, Algebra::Kleene).expect("SAT runs");
    assert!(!diff.equivalent);
    let counterexample = diff.counterexample.expect("counterexample");
    assert_eq!(counterexample.len(), 100);
    assert_ne!(
        eval(&chain, &counterexample, Algebra::Kleene),
        eval(&shorter, &counterexample, Algebra::Kleene)
    );
}

#[test]
fn exhaustive_refuses_too_many_variables() {
    let vars: Vec<String> = (0..12).map(|i| format!("a{i}")).collect();
    let chain = vars
        .iter()
        .map(|v| Expr::Var(v.clone()))
        .reduce(Expr::and)
        .unwrap();
    let err = equivalent_exhaustive(&chain, &chain, Algebra::Kleene, 8).unwrap_err();
    assert!(matches!(
        err,
        trit_sat::EvalError::ExhaustiveLimit { vars: 12, max: 8 }
    ));
}

#[test]
fn parser_respects_precedence() {
    let a = parse("a | b & c").unwrap();
    let b = parse("a | (b & c)").unwrap();
    assert_eq!(a, b);

    let a = parse("!a & b").unwrap();
    let b = parse("(!a) & b").unwrap();
    assert_eq!(a, b);

    let a = parse("a => b => c").unwrap();
    let b = parse("a => (b => c)").unwrap();
    assert_eq!(a, b);

    assert_eq!(parse("0"), Ok(Expr::Const(Ternary::False)));
    assert_eq!(parse("X"), Ok(Expr::Const(Ternary::Unknown)));
    assert!(parse("a &").is_err());
}

#[test]
fn verdict_shape_is_stable() {
    let a = parse("a").unwrap();
    let b = parse("a").unwrap();
    let v: Verdict = equivalent_sat(&a, &b, Algebra::Kleene).unwrap();
    assert!(v.equivalent);
    assert!(v.counterexample.is_none());
    assert_eq!(v.variable_count, 1);
}
