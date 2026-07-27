### Case AUX-WORKSPACE-PROCESS-001: Shared process wrapper plain-text environment 稳定

Entry:
- `scripts/tools/foundation/test/foundation.test.ts > script foundation > runs child processes with plain text output environment`

Contract:
- `docs/testing.md` 定义或约束“Shared process wrapper plain-text environment 稳定”所涉及的稳定行为边界。

Proves:
- shared process wrapper 在 sync 和 async child process 入口覆盖 caller-provided color env，统一注入 plain-text output environment。
