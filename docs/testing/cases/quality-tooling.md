# quality-tooling

## Case AUX-QUALITY-CACHE-001: Quality measurement cache identity 稳定

Owner: `scripts/tools/quality-core/README.md#use`

Entities:
- `bun|scripts/tools/quality-core/src/measurement/cache.test.ts|quality measurement cache > keys duplicate-code cache by scan identity and strips changed-scope annotations`
- `bun|scripts/tools/quality-core/src/measurement/cache.test.ts|quality measurement cache > reuses baseline snapshots only when identity and snapshot hash match`

Proves:
- duplicate-code cache key changes for tested code area and input fingerprint differences, and cache lookup misses when tool version differs.
- duplicate-code cache entry 使用 `.cache/docnav/quality/<scan_cache_version>/` 作为 owner 目录。
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
- `bun|scripts/quality/config.test.ts|quality code area classification > classifies root workspace crates by Rust source role`
- `bun|scripts/quality/config.test.ts|quality code area classification > discovers representative Rust and TypeScript sources in the root workspace`
- `bun|scripts/quality/config.test.ts|quality code area classification > keeps smoke case and fixture files in the fixtures/examples area`
- `bun|scripts/quality/config.test.ts|quality code area classification > keeps smoke harness infrastructure in the validation/smoke area`

Proves:
- smoke case 和 fixture 文件归入 `fixtures-examples`，不被 `typescript-validation-smoke` 的广泛 globs 遮蔽。
- smoke harness 和 validator infrastructure 仍归入 `typescript-validation-smoke`。
- quality current scan 的实际文件发现包含根 workspace 下原有与迁入 `crates/shared/**` 的 Rust source，以及 TypeScript scripts 和 tests。
- Rust production、tests 和 benches 沿用既有 Rust code areas；examples/fixtures 沿用 `fixtures-examples`。
- TypeScript code area globs 继续将 production scripts 与 validation/smoke TypeScript 分开。

## Case AUX-QUALITY-FINGERPRINT-001: Quality input fingerprint 稳定

Owner: `scripts/tools/quality-core/README.md#use`

Entities:
- `bun|scripts/tools/quality-core/src/input/files.test.ts|quality input fingerprints > uses stable SHA-256 fingerprints for sorted file content`

Proves:
- quality input fingerprint 使用排序后的文件内容生成稳定 SHA-256。
- 文件内容变化会改变 fingerprint，文件顺序变化不会改变 fingerprint。

## Case AUX-QUALITY-GIT-PATHSPEC-001: Quality git pathspec 参数稳定

Owner: `scripts/tools/quality-core/README.md#use`

Entities:
- `bun|scripts/tools/quality-core/src/input/files.test.ts|quality input git pathspecs > builds explicit git pathspec arguments and can omit empty pathspecs`

Proves:
- quality input git pathspec 参数使用显式 `--` 分隔并保留 glob pathspec magic。
- 空 pathspec 可按调用方需要保留 `--` 或完全省略。

## Case AUX-QUALITY-JSCPD-TASK-001: Quality jscpd task planning 稳定

Owner: `scripts/tools/quality-core/README.md#use`

Entities:
- `bun|scripts/tools/quality-core/src/measurement/scanners/jscpd/area-scans.test.ts|jscpd tasks > plans one scan task per code area`
- `bun|scripts/tools/quality-core/src/measurement/scanners/jscpd/area-scans.test.ts|jscpd tasks > records current-scan fatal issues when jscpd output is invalid`

Proves:
- jscpd 每个 code area 生成一个 scan task。
- task id 和文件排序保持可复现。
- current revision area scan 将 execution/report/parse failure 记录为 `fatalIssues` 的 `current-scan` failure channel，不静默降级为空 duplicate result。

## Case AUX-QUALITY-JSCPD-WRAPPER-001: Quality jscpd wrapper failure projection 稳定

Owner: `scripts/tools/quality-core/README.md#use`

Entities:
- `bun|scripts/tools/quality-core/src/measurement/scanners.test.ts|quality jscpd wrapper failure projection > classifies empty jscpd JSON reports as report failures`
- `bun|scripts/tools/quality-core/src/measurement/scanners.test.ts|quality jscpd wrapper failure projection > classifies missing jscpd tools as skipped unavailable scans`
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
- quality scan 的 `--verification-output` flag parsing 保持 opt-in。
- changed file collection 在 CLI defaults 下仍能解析当前 changed scope。

## Case AUX-QUALITY-WARNINGS-001: Quality warning 阈值语义稳定

Owner: `scripts/tools/quality-core/README.md#use`

Entities:
- `bun|scripts/quality/annotate/warnings.test.ts|quality warning annotations > keeps accepted warnings in machine records but selects only unaccepted warnings`
- `bun|scripts/tools/quality-core/src/output/warnings/generator.test.ts|quality warning generation > adds configured accepted reasons without relying on duplicate line numbers`
- `bun|scripts/tools/quality-core/src/output/warnings/generator.test.ts|quality warning generation > uses complexity-aware function code density thresholds`
- `bun|scripts/tools/quality-core/src/output/warnings/generator.test.ts|quality warning generation > uses scc code lines and low decision-token allowance for file-size warnings`
- `bun|scripts/tools/quality-core/src/output/warnings/generator.test.ts|quality warning generation > warns when an accepted warning rule no longer matches any generated warning`

Proves:
- 文件大小 warning 使用 scc `Code` 代码行数，而不是包含注释和空行的总行数。
- 文件大小 warning 根据 scc decision-token count 选择 code-line floor，低 decision-token 文件可使用更高行数阈值。
- warning record 的 rule id、metric、message 和 suggestion 反映代码行数、阈值和 responsibility-focused guidance。
- 函数 warning 使用复杂度感知的代码密度阈值：普通复杂度函数超过 50 行触发，CC < 5 的简单函数超过 150 行才触发。
- 函数代码密度 warning record 的 rule id、metric 和 message 反映组合阈值语义，不再输出单纯函数代码行数规则。
- 配置的已知可接受 warning 保留在 all/changed/regression warning records 中，并通过 `acceptedReason` 字段携带原因。
- GitHub annotation selection 跳过带 `acceptedReason` 的 warning 和 info records，只投影未接受的 warning；完整机器记录保持不变。
- 配置的 accepted warning 匹配不依赖重复片段行号；匹配不到任何 generated warning 的 accepted rule 会生成 `quality-accepted-warning-unmatched` warning。
