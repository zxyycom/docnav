# Claim CLAIM-CLI-REQUIRED-ARGUMENT-001: Core 拒绝缺失的 operation 参数

Topic: `core-cli`
Owner ref: `docs/cli.md#document-operation-执行`

Statement:
- A document command missing an operation-owned required argument fails at the stable input boundary.

Observations:
- document command 缺少本 operation 拥有的必需参数时返回稳定 input failure。
- 该 smoke case 代表这一类外部 CLI 错误，不枚举所有 parser 组合。

Supported by:
- `smoke|core:cli-argument-failure|CORE-ARGS-001`
