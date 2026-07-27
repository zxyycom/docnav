### Case AUX-TEST-EVIDENCE-CATALOG-004: Rejects duplicate case ids across topics

Entry:
- `scripts/docs/test-evidence-validation.test.ts > test evidence catalog integration > rejects duplicate case ids across topics`

Contract:
- `.codex/skills/test-evidence-review/references/catalog-contract.md` 定义固定目录、case 结构、唯一性和派生索引的新鲜度。

Proves:
- 跨 topic 重复 case ID 产生阻断的 `catalog.case-id-duplicate` 诊断。
