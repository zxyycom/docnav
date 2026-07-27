### Case BB-CORE-TOOLS-001: Core 非 document 命令保持可用

Entry:
- `test/smoke/core/cases/config-management.ts > smoke task CORE-TOOLS-001`

Contract:
- `docs/cli.md` 定义或约束“Core 非 document 命令保持可用”所涉及的稳定行为边界。

Proves:
- `init` 通过真实 CLI 创建 project config。
- `version` 输出 crate version，document help 暴露 output/pagination CLI options。
