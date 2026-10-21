# Contributing

1. 从 `main` 开分支：`feat/<topic>` 或 `fix/<topic>`。
2. 提交前跑：
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```
3. 开 PR，按模板填判据与回滚方式；CI 的 4 个 check 必须全绿。
4. `main` 受保护：只接受 PR + squash/rebase，禁止 force push 与删除。

性能类改动必须给出前后对照读数（命令 + 机器 + 变量数），不接受“应该更快”。
