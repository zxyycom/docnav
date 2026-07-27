### Case WB-TYPED-FIELDS-PROCESSING-006: Set build rejects missing processing strategy

Entry:
- `crates/shared/typed-fields/src/tests/processing.rs > set_build_rejects_missing_processing_strategy`

Contract:
- `docs/architecture.md` 定义或约束“Typed field processing build 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `set_build_rejects_missing_processing_strategy` 直接验证“Set build rejects missing processing strategy”所描述的结果。
