## Why

`.log/` 当前同时承载审计日志、可复用缓存、临时工作区和人工质量扫描报告，导致目录语义不清，也让同一系列输出只能靠前缀区分。该 change 将运行期输出按 owner 和删除语义分组，降低验证脚本维护和人工排查成本。

## What Changes

- `.log/` 只承载可审计文本日志，例如 verifier 和 smoke 的 latest/timestamp logs。
- `.tmp/` 承载一次性运行临时工作区，例如 smoke 测试 scratch workspace。
- `.cache/docnav/` 承载可复用缓存和运行中间状态，例如 quality scan cache 和 verifier dev binary env file。
- `artifacts/docnav-quality/` 继续承载 quality scan 报告、metrics 和 raw outputs。
- 同一系列输出使用 owner-first 分组，例如 `.log/smoke/markdown/`、`.log/smoke/core/` 和 `.log/verify/workspace/`。
- 同步更新验证脚本、smoke 配置、文档、测试期望和 OpenSpec spec delta。
- Non-goal: 不改变 Docnav CLI、adapter protocol、schema、readable output 或 MCP 映射。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `code-quality-observability`: quality scan cache 使用 `.cache/docnav/quality/<scan_cache_version>/`，quality reports 继续归属 artifacts。
- `markdown-navigation`: Markdown CLI smoke audit log 使用 `.log/smoke/markdown/`。

## Impact

- Scripts: workspace verifier path handling, docnav smoke configs, quality scan cache root handling, and defaults that expose artifact directories.
- Tests: workspace verifier tests, quality cache tests, smoke harness output expectations, and docs validators for updated path references.
- Docs/specs: `docs/testing.md`, Markdown navigation requirement, code quality observability requirement, `.gitignore`, lint ignores, and quality scan excludes.
- Compatibility: current scripts use the new runtime layout. Existing ignored local `.log/` history can remain in place or be deleted; deleting old cache may slow the next quality scan but must not affect correctness.
