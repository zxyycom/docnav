### Case WB-READABLE-VIEW-003: Built in renderer maps find response

Entry:
- `crates/shared/output/src/tests.rs > built_in_renderer_maps_find_response`

Contract:
- `docs/output.md` 定义或约束“内置 readable-view 从 ProtocolResponse 派生”所涉及的稳定行为边界。

Proves:
- 原生入口 `built_in_renderer_maps_find_response` 直接验证“Built in renderer maps find response”所描述的结果。
