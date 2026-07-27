### Case WB-READABLE-VIEW-005: Built in renderer maps read response and preserves block framing

Entry:
- `crates/shared/output/src/tests.rs > built_in_renderer_maps_read_response_and_preserves_block_framing`

Contract:
- `docs/output.md` 定义或约束“内置 readable-view 从 ProtocolResponse 派生”所涉及的稳定行为边界。

Proves:
- 原生入口 `built_in_renderer_maps_read_response_and_preserves_block_framing` 直接验证“Built in renderer maps read response and preserves block framing”所描述的结果。
