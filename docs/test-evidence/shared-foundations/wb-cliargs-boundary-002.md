### Case WB-CLIARGS-BOUNDARY-002: Unknown flag does not consume following positional

Entry:
- `crates/shared/cli-args/src/tests.rs > unknown_flag_does_not_consume_following_positional`

Contract:
- `docs/cli.md` 定义或约束“Strict CLI 参数扫描保持输入边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `unknown_flag_does_not_consume_following_positional` 直接验证“Unknown flag does not consume following positional”所描述的结果。
