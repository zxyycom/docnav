### Case WB-MD-LINK-003: Find to read roundtrip with canonical ref

Entry:
- `crates/adapters/markdown/src/markdown/tests.rs > find_to_read_roundtrip_with_canonical_ref`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown outline/find ref 可通过 read roundtrip”所涉及的稳定行为边界。

Proves:
- 原生入口 `find_to_read_roundtrip_with_canonical_ref` 直接验证“Find to read roundtrip with canonical ref”所描述的结果。
