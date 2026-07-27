### Case BB-CORE-ARGS-001: Core 拒绝缺失的 operation 参数

Entry:
- `test/smoke/core/cases/cli-args.ts > smoke task CORE-ARGS-001`

Contract:
- `docs/cli.md` 定义或约束“Core 拒绝缺失的 operation 参数”所涉及的稳定行为边界。

Proves:
- document command 缺少本 operation 拥有的必需参数时返回稳定 input failure。
- 该 smoke case 代表这一类外部 CLI 错误，不枚举所有 parser 组合。
