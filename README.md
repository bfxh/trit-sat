# trit-sat

SAT-based equivalence checking for three-valued logics (Kleene K3 / Łukasiewicz Ł3 / Bochvar B3).

The symbolic engine lands in the first PR. Planned shape:

- `Ternary` values `{False, Unknown, True}` and the three algebras.
- `equivalent_sat(a, b, algebra)` — dual-rail encoding into two Boolean rails per trit,
  per-gate truth-table clauses, UNSAT means equivalent.
- `equivalent_exhaustive(a, b, algebra, max_vars)` — the 3^n reference used to cross-check SAT.
- CLI: `trit-sat equiv [--exhaustive] "a & b" "b & a"`.

The 3^n reference hits its ceiling around 18 variables; the SAT path is the reason this
repository exists.

## License

Dual licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
