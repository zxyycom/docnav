### Case WB-TYPED-FIELDS-PROCESSING-002: Processing build returns caller processed value for typed raw input

Entry:
- `crates/shared/typed-fields/src/tests/processing.rs > processing_build_returns_caller_processed_value_for_typed_raw_input`

Contract:
- `docs/architecture.md` 定义或约束“Typed field processing build 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `processing_build_returns_caller_processed_value_for_typed_raw_input` 直接验证“Processing build returns caller processed value for typed raw input”所描述的结果。
