### Case AUX-TEST-EVIDENCE-CATALOG-005: Rejects a stale derived index

Entry:
- `scripts/docs/test-evidence-validation.test.ts > test evidence catalog integration > rejects a stale derived index`

Contract:
- `.codex/skills/test-evidence-review/references/catalog-contract.md` 定义固定目录、case 结构、唯一性和派生索引的新鲜度。

Proves:
- 权威 case 变化后旧索引产生阻断的 `state-index.index-stale` 诊断。
