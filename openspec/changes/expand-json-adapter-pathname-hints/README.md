# expand-json-adapter-pathname-hints

本临时 change 计划在 JSONC predecessor 成为 Current 后扩展 `docnav-json` 的强 JSON-family pathname hints，完整保留其两个 descriptor content types，同时保持 generic structural navigation 与现有 routing、grammar owner 边界。

## Goal

在既有 `json-adapter` capability 的 registry-facing manifest requirement 中增加 `.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif` suffixes，以及 `Pipfile.lock`、`deno.lock` exact filenames。

## Boundary

这些名称只作为 pathname selection hints；本 change 不承诺 profile validity、domain semantics、remote resolution、profile-specific navigation、新 grammar、新 adapter/format identity、content-type 变化或通用 routing algorithm 变化。JSON5、NDJSON/JSONL、JSON Text Sequences、模糊 rc names、弱 generic basenames 和 binary JSON-like formats 明确排除。

## Sequencing and status

`support-jsonc-in-json-adapter` 是 sequencing predecessor，尤其因为 `.code-snippets` 常见内容依赖其 grammar。该 predecessor 的 Target descriptor 是一个 `json` identity、`.json` / `.code-workspace` / `.jsonc` suffixes、`.prettierrc` / `.watchmanconfig` exact filenames，以及 `application/json` / `application/jsonc` content types；本 change 只在该 Target 上追加九个 hints，不新增或解释 content type。OpenSpec 的 proposal/spec/design/tasks artifacts 已生成，但 task 0 的 predecessor/current-baseline 与完整 artifact 审计尚未执行；该门禁完成前不得开始 production 实施，也不得把上述 predecessor Target 或本 change 描述为已经 Current。

## Reading path

从 [proposal.md](proposal.md) 了解目标，从 [design.md](design.md) 了解 numbered Decisions，从 [specs/json-adapter/spec.md](specs/json-adapter/spec.md) 查看 combined target delta，并从 [tasks.md](tasks.md) 的 blocking task 0 开始后续工作。
