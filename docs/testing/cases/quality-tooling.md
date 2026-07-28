# quality-tooling

## Case AUX-QUALITY-CACHE-001: Quality measurement cache identity 稳定

Owner: `scripts/tools/quality-core/README.md#use`

Entities:
- `bun|scripts/tools/quality-core/src/measurement/cache.test.ts|quality measurement cache > keys duplicate-code cache by scan identity and strips changed-scope annotations`
- `bun|scripts/tools/quality-core/src/measurement/cache.test.ts|quality measurement cache > reuses baseline snapshots only when identity and snapshot hash match`

Proves:
- code area、revision、input fingerprint、tool arguments 或 tool version 等 public input 变化时，duplicate-code cache identity 随之变化。
- cache hit 返回不带 changed-scope annotation 的 metric，保持复用扫描与当前 diff 语义分离。
- baseline snapshot cache key changes for tested tool version differences，命中时通过 snapshot hash 防止错读缓存内容。

## Case AUX-QUALITY-CHANGED-FILES-001: Quality revision inputs 保持 current/changed/baseline 一致

Owner: `scripts/tools/quality-core/README.md#use`

Entities:
- `bun|scripts/tools/quality-core/src/input/files.test.ts|quality changed file input > fails fast when an explicit changed-files list cannot be read`
- `bun|scripts/tools/quality-core/src/input/files.test.ts|quality changed file input > keeps current, changed, and baseline repository files aligned`

Proves:
- quality changed-file input 将 unreadable explicit `--changed-files` path 映射为 thrown diagnostic，错误文本保留 flag 名称和请求的文件路径。
- 一个普通本地仓库代表证明 current scan 收集 tracked 与 untracked files，committed 与 working-tree changes 返回同一组根相对路径，并且 materialized baseline 使用 selected repository revision 的文件内容。

## Case AUX-QUALITY-CODE-AREAS-001: Quality code area 分类稳定

Owner: `docs/tooling.md#验证入口集成`

Entities:
- `bun|scripts/quality/config.test.ts|quality code area classification > classifies representative smoke files by responsibility`
- `bun|scripts/quality/config.test.ts|quality code area classification > classifies root workspace crates by Rust source role`
- `bun|scripts/quality/config.test.ts|quality code area classification > discovers representative Rust and TypeScript sources in the root workspace`

Proves:
- 代表性 smoke case/fixture files 归入 `fixtures-examples`；smoke harness/validator infrastructure 归入 `typescript-validation-smoke`。
- quality current scan 的实际文件发现包含根 workspace 下原有与迁入 `crates/shared/**` 的 Rust source，以及 TypeScript scripts 和 tests。
- Rust production、tests 和 benches 沿用既有 Rust code areas；examples/fixtures 沿用 `fixtures-examples`。
- TypeScript code area globs 继续将 production scripts 与 validation/smoke TypeScript 分开。

## Case AUX-QUALITY-JSCPD-TASK-001: Quality jscpd task 保留 current-scan failure projection

Owner: `scripts/tools/quality-core/README.md#use`

Entities:
- `bun|scripts/tools/quality-core/src/measurement/scanners/jscpd/area-scans.test.ts|jscpd tasks > records current-scan fatal issues when jscpd output is invalid`

Proves:
- current revision area scan 将 execution/report/parse failure 记录为 `fatalIssues` 的 `current-scan` failure channel，不静默降级为空 duplicate result。

## Case AUX-QUALITY-JSCPD-WRAPPER-001: Quality jscpd wrapper failure projection 稳定

Owner: `scripts/tools/quality-core/README.md#use`

Entities:
- `bun|scripts/tools/quality-core/src/measurement/scanners.test.ts|quality jscpd wrapper failure projection > classifies non-zero jscpd exits as execution failures, not skipped scans`
- `bun|scripts/tools/quality-core/src/measurement/scanners.test.ts|quality jscpd wrapper failure projection > classifies unavailable jscpd dependency binaries in tool availability`
- `bun|scripts/tools/quality-core/src/measurement/scanners.test.ts|quality jscpd wrapper failure projection > does not treat a successful jscpd run without JSON as a successful empty scan`
- `bun|scripts/tools/quality-core/src/measurement/scanners.test.ts|quality jscpd wrapper failure projection > keeps real duplicate findings non-fatal and normalizes jscpd JSON`

Proves:
- jscpd wrapper 将 successful process without JSON report 映射为 `jscpd-report-failure` scan failure diagnostic，不把缺失或空 JSON 当作 successful empty duplicate-code result。
- jscpd wrapper 使用真实 `jscpd` duplicate scan 证明发现重复代码时仍解析 JSON 并生成 `DuplicateCodeFragment`，不让第三方 threshold 决定扫描失败。
- jscpd tool availability check 将 missing dependency 或 unavailable binary 映射为 `tool-unavailable`。
- jscpd wrapper 将 non-zero execution 映射为 `jscpd-execution-error`，不把执行失败标成 skipped scan。

## Case AUX-QUALITY-PARSER-001: Quality scanner parser fixtures 稳定

Owner: `scripts/tools/quality-core/README.md#use`

Entities:
- `bun|scripts/tools/quality-core/src/measurement/scanners.test.ts|quality scanner output parsing > classifies invalid jscpd JSON and duplicate items as parse failures`
- `bun|scripts/tools/quality-core/src/measurement/scanners.test.ts|quality scanner output parsing > parses Lizard 1.23 function rows`
- `bun|scripts/tools/quality-core/src/measurement/scanners.test.ts|quality scanner output parsing > parses jscpd version and JSON output`
- `bun|scripts/tools/quality-core/src/measurement/scanners.test.ts|quality scanner output parsing > parses scc 3.7 Provider paths and rejects unknown CSV headers`

Proves:
- scc 3.7 by-file CSV 解析 Provider path 和 `Complexity` decision-token value，并将未知 header 投影为 parser failure。
- Lizard 1.23 CSV row 解析 function name、file path、line range、NLOC、parameter count 和 cyclomatic complexity。
- jscpd parser helpers 解析 code-area format、version output 和 JSON duplicate fragment locations/token count，并把 invalid JSON 或 invalid duplicate item 映射为 `jscpd-parse-failure`。

## Case AUX-QUALITY-PUBLIC-API-001: Quality core applies caller-owned areas and thresholds

Owner: `scripts/tools/quality-core/README.md#use`

Entities:
- `bun|scripts/tools/quality-core/test/quality-core.test.ts|script quality core > classifies files using caller-provided code areas`
- `bun|scripts/tools/quality-core/test/quality-core.test.ts|script quality core > generates warning channels from caller-provided thresholds`
- `bun|scripts/tools/quality-core/test/quality-core.test.ts|script quality core > rejects a metrics envelope without metadata`

Proves:
- The quality-core facade classifies files and generates warning channels from caller-provided code areas and thresholds.
- Malformed metrics envelopes without required metadata are rejected at the facade boundary.

## Case AUX-QUALITY-REPORT-001: Quality report 排名和 changed-file 摘要稳定

Owner: `scripts/tools/quality-core/README.md#use`

Entities:
- `bun|scripts/tools/quality-core/src/output/report/markdown-report.test.ts|quality report > keeps changed-file watchlist useful without baseline annotations`
- `bun|scripts/tools/quality-core/src/output/report/markdown-report.test.ts|quality report > labels scc file Complexity as decision-token count and shows total-token share`
- `bun|scripts/tools/quality-core/src/output/report/markdown-report.test.ts|quality report > shows accepted reasons next to warning records`
- `bun|scripts/tools/quality-core/src/output/report/markdown-report.test.ts|quality report > shows code-area decision-token hotspots by total-token share`
- `bun|scripts/tools/quality-core/src/output/report/markdown-report.test.ts|quality report > sorts rankings by metric without mutating scanner output order`

Proves:
- baseline unavailable 时 changed-file watchlist 仍按风险展示有用文件。
- rankings 排序不修改 scanner output 原始顺序。
- scc `Complexity` 文件列在人类报告中展示为 decision-token count，并补充 `file-decision-tokens / total-file-decision-tokens` 热点占比。
- Code Area 汇总表展示 decision-token count 和总量占比，用于定位热点区域。
- 带 `acceptedReason` 的 warning 在报告中贴近对应 warning 展示原因，不从单独质量扫描中消失。

## Case AUX-QUALITY-SCAN-CLI-001: Quality scan CLI 默认值稳定

Owner: `docs/tooling.md#验证入口集成`

Entities:
- `bun|scripts/quality/args.test.ts|quality scan CLI args > keeps quick quality checks baseline-free and explicit`
- `bun|scripts/quality/args.test.ts|quality scan CLI args > skips baseline by default and keeps baseline generation opt-in`

Proves:
- quality scan 默认跳过 baseline，baseline generation 保持 opt-in。
- quality scan profile 默认为 full；quick profile 固定跳过 baseline，并拒绝 baseline 参数。

## Case AUX-QUALITY-WARNINGS-001: Quality warning 阈值语义稳定

Owner: `scripts/tools/quality-core/README.md#use`

Entities:
- `bun|scripts/quality/annotate/warnings.test.ts|quality warning annotations > keeps accepted warnings in machine records but selects only unaccepted warnings`
- `bun|scripts/tools/quality-core/test/quality-core.test.ts|script quality core > generates warning channels from caller-provided thresholds`
- `bun|scripts/tools/quality-core/src/output/warnings/generator.test.ts|quality warning generation > adds configured accepted reasons without relying on duplicate line numbers`
- `bun|scripts/tools/quality-core/src/output/warnings/generator.test.ts|quality warning generation > warns when an accepted warning rule no longer matches any generated warning`

Proves:
- quality-core facade 使用 caller 提供的 warning thresholds，不自行采用 repository-owned defaults。
- 配置的已知可接受 warning 保留在 all/changed/regression warning records 中，并通过 `acceptedReason` 字段携带原因。
- GitHub annotation selection 跳过带 `acceptedReason` 的 warning 和 info records，只投影未接受的 warning；完整机器记录保持不变。
- 配置的 accepted warning 匹配不依赖重复片段行号；匹配不到任何 generated warning 的 accepted rule 会生成 `quality-accepted-warning-unmatched` warning。
