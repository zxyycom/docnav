本 tasks 将 `implement-code-quality-observability` 拆成可执行实现步骤。目标是在不改变 `docnav` CLI、adapter、MCP、schema 或 examples 业务契约的前提下，交付非阻断代码质量观测命令、配置、临时产物和 CI 汇报。

## 1. 实现前对齐

- [ ] 1.1 确认 proposal、design、specs 和 tasks 都围绕“Clippy 保持 Rust 阻断 gate，Lizard/scc/PMD CPD 生成非阻断代码质量快照、warning 和报告”这一核心目标。
- [ ] 1.2 确认 capability ID 使用 `code-quality-observability`，且没有创建一次性、实现型或同义 capability。
- [ ] 1.3 确认实现范围只影响质量观测脚本、配置、package scripts、CI 汇报、测试/fixture 和 OpenSpec artifacts。
- [ ] 1.4 确认现有 `verify:docnav-workspace` 的 Clippy gate 语义保持不变。

## 2. 指标模型与配置

- [ ] 2.1 定义统一 metrics JSON schema 或等价结构，包含 schema version、扫描元数据、工具名称和版本、扫描范围、排除规则、code areas、current/baseline 扫描输入指纹、baseline metadata、baseline status、comparison status、文件指标、函数指标、重复代码指标、聚合指标、趋势比较和 warning records。
- [ ] 2.2 定义规范化路径、语言、代码区域、仓库体量、语言占比、文件行数、文件级复杂度、函数行数、参数数量、函数圈复杂度和重复代码字段。
- [ ] 2.3 定义 metric source、scope 和缺失指标的表达方式，避免 scc、Lizard、PMD CPD 或 Clippy 不支持某个字段时伪造数值。
- [ ] 2.4 定义 Markdown summary 的默认栏目、排序规则、top N 数量、watchlist、changed files、duplicate code 和 trend section。
- [ ] 2.5 定义 warning record 字段，包括 level、rule id、source tool、path、line、metric、value、comparison basis、message 和 suggestion。
- [ ] 2.6 新增质量观测配置文件，定义 include paths、exclude paths、generated files、默认 6 类 code areas、tool options、artifact directory defaults、warning policy 和按 code area 拆分的 PMD CPD `minimum tokens`。
- [ ] 2.7 在配置中定义默认 code areas：`rust-production`、`rust-tests`、`node-production-scripts`、`node-validation-smoke`、`fixtures-examples` 和 `generated`。
- [ ] 2.8 为每个默认 code area 定义初始 warning 策略：production 最严格，tests 和 validation/smoke 放宽，fixtures/examples 默认只进 summary/watchlist，generated 默认排除 warning。
- [ ] 2.9 定义按 code area 的扫描输入 fingerprint 字段，覆盖当前提交和 baseline commit 参与扫描的文件列表、文件内容或 git blob 指纹。
- [ ] 2.10 定义同次运行生成 baseline snapshot 的状态，包括 `generated`、`history-unavailable`、`no-baseline-commit`、`baseline-materialization-failed` 和 `baseline-scan-failed`。
- [ ] 2.11 定义趋势 delta 字段，至少覆盖仓库体量、语言占比、文件数量、行数、文件级复杂度、函数级行数、函数参数数量、函数圈复杂度和重复代码片段数量的 current、baseline 和 delta。
- [ ] 2.12 定义 baseline metadata 字段，包括 baseline commit sha、baseline commit date、baseline selection reason、扫描时使用的当前配置版本和当前 tool metadata。
- [ ] 2.13 定义 comparison status 字段，包括 `compared`、`input-unchanged` 和 `baseline-unavailable`。

## 3. 采集实现

- [ ] 3.1 确认 Clippy 继续由现有 Rust lint workflow 承担阻断职责，质量观测脚本只读取或汇总其 gate 状态，不改变 Clippy 失败语义。
- [ ] 3.2 实现仓库脚本封装 Lizard 函数级指标，并统一输出函数名称、所属文件、函数行数、参数数量、圈复杂度、路径和排序。
- [ ] 3.3 实现仓库脚本封装 scc 仓库/文件级指标，并统一输出仓库体量、语言占比、文件行数、文件级复杂度、路径和排序。
- [ ] 3.4 实现仓库脚本封装 PMD CPD 重复代码指标，并按 code area 传递或应用 `minimum tokens`，统一输出重复片段、token count、涉及文件、起始行、code area 和排序。
- [ ] 3.5 默认扫描配置指定的 Rust crates 和仓库 JavaScript 脚本，并排除 `.git`、`target`、`node_modules`、`.venv`、`dist`、`build`、缓存目录和已标记 generated files。
- [ ] 3.6 生成临时 artifact 目录，例如 `target/docnav-quality/metrics.json`、`target/docnav-quality/report.md`、`target/docnav-quality/warnings.ndjson` 和 `target/docnav-quality/raw/`。
- [ ] 3.7 确认 Lizard/scc/PMD CPD 指标数值不会使命令失败；命令失败仅用于工具不可运行、输入不可读、输出不可写、配置无效或 wrapper 产物不符合内部 schema 等执行错误。
- [ ] 3.8 实现从 git history 定位 previous-code baseline commit：当前 revision 修改扫描输入时选择它之前最近的代码提交；当前 revision 未修改扫描输入时选择最近一次代码提交。
- [ ] 3.9 在临时隔离目录中用当前配置和当前 wrapper/tool 扫描 baseline commit，生成 baseline snapshot、raw outputs 和 baseline metadata。
- [ ] 3.10 实现文本-only 或非扫描输入变更识别；当 current code inputs 与 baseline code inputs 一致时将 `comparison.status` 标记为 `input-unchanged`，并避免生成复杂度或重复代码 CI annotation。

## 4. 报告、CI 与验证

- [ ] 4.1 为配置解析、报告生成和工具输出 normalization 添加单元或 fixture 测试，覆盖排序、分组、缺失指标、source tool 标记、路径过滤、6 类默认 code area 归类、按 code area 的 CPD `minimum tokens`、generated files 过滤、baseline commit 定位、baseline scan 状态和 text-only 变更。
- [ ] 4.2 添加本地脚本入口，例如 `pnpm quality:scan` 或等价命令，并在文档中标注其非阻断性质。
- [ ] 4.3 实现 changed scope、同次运行生成的 baseline delta 和绝对下限组合的 warning 规则，避免对未改动历史热点或 text-only 变更默认刷 CI/PR annotation。
- [ ] 4.4 在 CI 中产出质量 artifact、step summary 和非阻断 warning annotation，不接入阻断式质量门禁。
- [ ] 4.5 验证本地临时目录和 CI artifact 使用同一目录结构。
- [ ] 4.6 运行与脚本、Rust/JavaScript 指标采集、CPD 重复检测和 workspace 范围匹配的验证命令。
- [ ] 4.7 积累报告样本后，另行评估是否需要历史曲线、PR 评论或门禁策略。
