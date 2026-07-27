### Case AUX-TEST-EVIDENCE-CATALOG-003: Rejects an invalid case document

Entry:
- `scripts/docs/test-evidence-validation.test.ts > test evidence catalog integration > rejects an invalid case document`

Contract:
- `.codex/skills/test-evidence-review/references/catalog-contract.md` 定义固定目录、case 结构、唯一性和派生索引的新鲜度。

Proves:
- 缺少必需 Contract 的 case 产生阻断的 `catalog.invalid` 诊断。
