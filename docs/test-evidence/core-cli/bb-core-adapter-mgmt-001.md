### Case BB-CORE-ADAPTER-MGMT-001: Core adapter inspection 命令覆盖

Entry:
- `test/smoke/core/cases/config-management.ts > smoke task CORE-ADAPTER-MGMT-001`

Contract:
- `docs/adapter-contract.md` 定义或约束“Core adapter inspection 命令覆盖”所涉及的稳定行为边界。

Proves:
- `doctor` 报告 static registry 和 adapter layer checks。
- `adapter list` 输出 core release static registry 内置 Markdown adapter metadata。
