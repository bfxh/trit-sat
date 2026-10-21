# trit-sat

[![CI](https://github.com/bfxh/trit-sat/actions/workflows/ci.yml/badge.svg)](https://github.com/bfxh/trit-sat/actions/workflows/ci.yml)

SAT-based equivalence checking for three-valued logics (Kleene K3 / Łukasiewicz Ł3 / Bochvar B3).

## 为什么

真值表等价判定是 `3^n` 穷举：18 个变量约 3.9 亿组，20 个变量已经不可用。
`trit-sat` 把每个 trit 编成两根布尔导轨，用 SAT 判定“两个根不相等”是否可满足——
UNSAT 即等价，SAT 还能给出反例。变量数不再受真值表规模限制。

## 快速开始

```bash
cargo run --release -- equiv "a & (b | c)" "(a & b) | (a & c)"
cargo run --release -- -a lukasiewicz equiv "X => X" "1"
cargo run --release -- eval "a & b" -s "a=1,b=X"
```

不等价时退出码为 `2`，便于脚本判断；`--exhaustive` 切回 `3^n` 参考实现（默认上限 18 变量，
`--max-vars N` 可调），`--algebra` / `-a` 选 `kleene`（默认）、`lukasiewicz`、`bochvar`。

## 编码

每个三值量用两根导轨 `(t, x)`：

| 值 | t | x |
|---|---|---|
| `False` | 0 | 0 |
| `True` | 1 | 0 |
| `Unknown` | 0 | 1 |

并加约束 `¬(t ∧ x)`。每个运算符按所选代数的真值表逐门加子句（每条输入组合两根输出轨道各一条），
所以 K3 / Ł3 / B3 共用同一套编码，包括 `X ⊕ X = 1`（Ł3）与 `X` 毒值（B3）这类差异。

## 测速

同一台机器，单次，`n` 个变量的 AND 链与自身比较：

| 变量数 | 穷举 `--exhaustive` | SAT（默认） |
|---|---|---|
| 12 | 0.33 s | 0.02 s |
| 16 | 3.4 s | 0.02 s |
| 18 | 35.3 s | 0.02 s |
| 20 | 护栏报错 | 0.018 s |
| 50 | 不可行 | 0.020 s |
| 100 | 不可行 | 0.025 s |

## API

```rust
use trit_sat::{equivalent_sat, equivalent_exhaustive, parse, Algebra};

let a = parse("a & (b | c)")?;
let b = parse("(a & b) | (a & c)")?;
let v = equivalent_sat(&a, &b, Algebra::Kleene)?;
assert!(v.equivalent);

// 参考实现，默认调用方自己设上限
let v = equivalent_exhaustive(&a, &b, Algebra::Kleene, 12)?;
```

反例是 `BTreeMap<String, Ternary>`，可直接用 `eval(&expr, &counterexample, alg)` 复核。

## 范围与限制

- 只判定等价；不做化简、网表 X 传播或时序分析。
- 是**真值函数**三值逻辑，不是硬件四态（0/1/X/Z）仿真；X-optimism 那一类问题不在范围内。
- SAT 是 NP 困难的：结构化公式（AND/OR 链、De Morgan 等）很快，随机难公式仍可能慢。
- 穷举参考是逐赋值的，18 变量约 35 s，仅用于小规模对拍。

## 许可

双许可：[MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE)。
