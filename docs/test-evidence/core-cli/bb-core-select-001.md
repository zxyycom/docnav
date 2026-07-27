### Case BB-CORE-SELECT-001: 显式 adapter 失败返回 selection diagnostic

Entry:
- `test/smoke/core/cases/adapter-selection.ts > smoke task CORE-SELECT-001`

Contract:
- `docs/adapter-contract.md` 定义或约束“显式 adapter 失败返回 selection diagnostic”所涉及的稳定行为边界。

Proves:
- 显式 CLI 或 project config 选择的 adapter 不存在时返回 adapter selection diagnostic，不隐藏为 registry fallback。
- 显式 adapter id 不存在时，即使同一请求携带 invalid-looking native option，也返回 adapter selection diagnostic，而不是 option validation error。
