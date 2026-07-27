### Case WB-PROTO-BASIC-008: Generated request id is non empty

Entry:
- `crates/shared/protocol/src/tests/basic.rs > generated_request_id_is_non_empty`

Contract:
- `docs/protocol.md` 定义或约束“Protocol 基础类型和 envelope 规则稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `generated_request_id_is_non_empty` 直接验证“Generated request id is non empty”所描述的结果。
