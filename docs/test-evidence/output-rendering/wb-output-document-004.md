### Case WB-OUTPUT-DOCUMENT-004: Render failure happens before stdout and strategy runs once

Entry:
- `crates/shared/output/src/tests.rs > render_failure_happens_before_stdout_and_strategy_runs_once`

Contract:
- `docs/output.md` 定义或约束“共享 document output facade 分层”所涉及的稳定行为边界。

Proves:
- 原生入口 `render_failure_happens_before_stdout_and_strategy_runs_once` 直接验证“Render failure happens before stdout and strategy runs once”所描述的结果。
