# expand-json-adapter-pathname-hints

本 change 在已落地的 JSONC-capable `docnav-json` 基线上扩展高置信度 JSON-family pathname hints。它只扩大 automatic pathname routing 的命中集合；被选中的文档仍使用同一个 JSON adapter、grammar、ref 和 generic structural navigation 契约。

## 目标结果

| Manifest 字段 | Current | 本 change 的 Target |
| --- | --- | --- |
| `extensions[]` | `.json`、`.code-workspace`、`.jsonc` | 保留 Current 顺序，再追加 `.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif` |
| `filenames[]` | `.prettierrc`、`.watchmanconfig` | 保留 Current 顺序，再追加 `Pipfile.lock`、`deno.lock` |
| `content_types[]` | `application/json`、`application/jsonc` | 不变 |

实施后，这九类 pathname 可以在未显式指定 adapter 时自动选择 `docnav-json`，并复用既有 `outline -> ref -> read`、find、info 和 full-read 行为。Hint 命中只表示“应交给 JSON adapter 尝试解析”，不表示文档满足 JSON-LD、GeoJSON、HAR、Notebook、SARIF 或 lockfile 等 profile 规范。

## Current 基线与实施状态

Current code、tests、CLI/release smoke 和主 `json-adapter` spec 已证明 `.jsonc`、`application/jsonc` 与统一 JSONC-capable grammar。Task 0 已基于这些证据完成 change artifact 审计；后续工作从 task 1.1 的 owner 与失败证据开始。

本 change 的九个新增 hints 尚未实施，仍是 Target；只有 owner、实现、tests、CLI 和 release evidence 全部闭合后才能标为 Current。

## 边界

本 change 不增加 adapter、format identity、grammar mode、content type、public input、protocol/schema field、routing algorithm 或 profile-specific navigation。JSON5、NDJSON/JSONL、JSON Text Sequences、模糊 rc names、弱 generic basenames 和 binary JSON-like formats 不在 allowlist 中。

## 阅读与执行顺序

1. 阅读 [proposal.md](proposal.md) 恢复问题、目标和可观察效果。
2. 阅读 [design.md](design.md) 恢复 exact allowlist、owner 边界和验证决策。
3. 以 [specs/json-adapter/spec.md](specs/json-adapter/spec.md) 作为本 change 的完整 `MODIFIED` requirement。
4. 从 [tasks.md](tasks.md) 的 section 1 继续实施；section 0 保存已完成的 Current-baseline 与 artifact 审计记录。
