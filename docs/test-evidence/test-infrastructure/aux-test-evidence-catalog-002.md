### Case AUX-TEST-EVIDENCE-CATALOG-002: Rejects an unknown topic directory

Entry:
- `scripts/docs/test-evidence-validation.test.ts > test evidence catalog integration > rejects an unknown topic directory`

Contract:
- `.codex/skills/test-evidence-review/references/catalog-contract.md` 定义固定目录、case 结构、唯一性和派生索引的新鲜度。

Proves:
- 未知 topic 目录产生阻断的 `catalog.topic-unknown` 诊断。
