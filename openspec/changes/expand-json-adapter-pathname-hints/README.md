# expand-json-adapter-pathname-hints

本 change 在已落地的 JSONC-capable `docnav-json` 基线上扩展高置信度 JSON-family pathname hints。它只扩大 automatic pathname routing 的命中集合；被选中的文档仍使用同一个 JSON adapter、grammar、ref 和 generic structural navigation 契约。

## 目标结果

| Manifest 字段 | Pre-change Current | Implemented Current |
| --- | --- | --- |
| `extensions[]` | `.json`、`.code-workspace`、`.jsonc` | `.json`、`.code-workspace`、`.jsonc`、`.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif` |
| `filenames[]` | `.prettierrc`、`.watchmanconfig` | `.prettierrc`、`.watchmanconfig`、`Pipfile.lock`、`deno.lock` |
| `content_types[]` | `application/json`、`application/jsonc` | 不变 |

实施后，这九类 pathname 可以在未显式指定 adapter 时自动选择 `docnav-json`，并复用既有 `outline -> ref -> read`、find、info 和 full-read 行为。Hint 命中只表示“应交给 JSON adapter 尝试解析”，不表示文档满足 JSON-LD、GeoJSON、HAR、Notebook、SARIF 或 lockfile 等 profile 规范。

## Current 基线与实施状态

Pre-change code、tests、CLI/release smoke 和主 `json-adapter` spec 已证明 `.jsonc`、`application/jsonc` 与统一 JSONC-capable grammar。Task 0 基于这些证据完成 change artifact 审计。

本 change 的九个新增 hints 已由单一 manifest owner 实施，22 项 tasks 均已闭合，并同步为 Current owner/spec、semantic Cases、Rust projections、开发 CLI smoke 与 release-package smoke。Required/full workspace verification、strict OpenSpec validation 和 change Markdown 导航检查均已执行；change 已达到归档条件，但尚未 archive。

## 边界

本 change 不增加 adapter、format identity、grammar mode、content type、public input、protocol/schema field、routing algorithm 或 profile-specific navigation。JSON5、NDJSON/JSONL、JSON Text Sequences、模糊 rc names、弱 generic basenames 和 binary JSON-like formats 不在 allowlist 中。

## 阅读与执行顺序

1. 本 README 说明 change 范围、实施状态与阅读入口。
2. [proposal.md](proposal.md) 记录问题、目标和可观察效果；[design.md](design.md) 记录 exact allowlist、owner 边界和验证决策。
3. [specs/json-adapter/spec.md](specs/json-adapter/spec.md) 保存本 change 的完整 `MODIFIED` requirement；[tasks.md](tasks.md) 保存已完成的执行与审计证据清单。
4. Current 产品契约以 [`docs/adapters/json.md`](../../../docs/adapters/json.md) 和 [main `json-adapter` spec](../../specs/json-adapter/spec.md) 为准；代码、tests 与 release artifacts 证明当前实现状态。
