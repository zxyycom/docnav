# code-quality-observability Specification

## Purpose
定义 Docnav 仓库如何采集、保存和汇报非阻断代码质量观测快照，并明确这些指标与阻断式质量门禁的边界。

## Requirements
### Requirement: 代码质量观测必须生成非阻断快照
Docnav 仓库的代码质量观测能力 MUST 生成当前源码状态的指标快照，并 MUST 将 Lizard、scc 和 PMD CPD 指标结果作为信息性报告处理。指标值本身 MUST NOT 使现有 workspace 验证、CLI 行为、adapter 行为、MCP 行为或 OpenSpec 验证失败。Clippy MAY 继续作为 Rust 阻断式 lint gate，因为它保护 Rust correctness 和 lint policy，不属于非阻断观测快照。

#### Scenario: 本地生成快照
- **WHEN** 维护者运行未来的代码质量观测命令
- **THEN** 命令生成当前仓库源码的质量指标快照
- **THEN** 命令输出或写入机器可读、人工可读和 warning record 报告位置
- **THEN** Lizard、scc 或 PMD CPD 产生的行数、复杂度、参数数量、趋势或重复代码数值不作为失败条件

#### Scenario: Clippy gate 与观测快照分离
- **WHEN** Clippy 发现 Rust lint policy 失败
- **THEN** Rust quality gate 可以按现有 lint policy 失败
- **WHEN** Lizard、scc 或 PMD CPD 报告高复杂度、大文件、语言占比变化、趋势变化或重复代码
- **THEN** 观测命令记录 warning data，但不把该指标值作为阻断条件

### Requirement: 代码质量工具职责必须分层
代码质量观测能力 MUST 将 Clippy、Lizard、scc 和 PMD CPD 作为独立信号源处理。Clippy MUST 表示 Rust 阻断式 lint gate。Lizard MUST 提供函数级复杂度指标和 warning candidates。scc MUST 提供仓库体量、语言占比、文件级复杂度、趋势和报告输入。PMD CPD MUST 提供重复代码检测信号。仓库 wrapper MUST 拥有长期输出形状，并 MUST NOT 将第三方原始输出暴露为稳定 contract。

#### Scenario: Clippy 作为 Rust gate
- **WHEN** repository quality workflow 检查 Rust code
- **THEN** Clippy 保持 Rust lint gate 职责，覆盖 correctness、style、complexity 和 performance lints
- **THEN** Clippy 结果不并入 Lizard 或 scc 的 metric semantics

#### Scenario: Lizard 作为函数级信号源
- **WHEN** 观测命令使用 Lizard 扫描支持的 Rust 或 JavaScript files
- **THEN** 快照记录 function 或 method name、owning file、function lines、parameter count 和可用的 cyclomatic complexity
- **THEN** 这些函数级指标可以为 changed functions 或 changed files 生成 warning candidates

#### Scenario: scc 作为仓库级和文件级信号源
- **WHEN** 观测命令使用 scc 扫描仓库
- **THEN** 快照记录可用的 repository size、language distribution 和 file-level metrics
- **THEN** 这些仓库级和文件级指标可以输入 summary、trend、watchlist 和 file-level warning records

#### Scenario: PMD CPD 作为重复代码信号源
- **WHEN** 观测命令使用 PMD CPD 扫描受支持的源码区域
- **THEN** 快照记录重复代码片段、token count、涉及文件、起始行和 code area
- **THEN** CPD findings 可以生成 duplicate-code warning records、summary sections 和 watchlist entries
- **THEN** CPD violation count 不作为观测命令失败条件

### Requirement: 代码质量快照必须覆盖首期核心指标
代码质量观测快照 MUST 至少包含仓库体量、语言占比、文件级行数、文件级复杂度、函数级行数、函数参数数量、函数圈复杂度和重复代码片段。报告 MAY 包含额外指标，但额外指标 MUST 不改变首期核心指标的字段语义。

#### Scenario: 文件级指标
- **WHEN** 观测命令扫描一个纳入范围的源码文件
- **THEN** 快照记录该文件的规范化路径
- **THEN** 快照记录该文件的行数
- **THEN** 快照记录该文件所属语言和代码区域
- **THEN** 快照记录 scc 提供的文件级复杂度或明确标记该指标不可用

#### Scenario: 函数级指标
- **WHEN** 观测命令识别一个纳入范围的函数、方法或等价可调用单元
- **THEN** 快照记录该单元所属文件和名称
- **THEN** 快照记录该单元的行数
- **THEN** 快照记录该单元的参数数量
- **THEN** 快照记录 Lizard 提供的圈复杂度或明确标记该指标不可用

#### Scenario: 仓库级指标
- **WHEN** 观测命令完成扫描
- **THEN** 快照记录可用的按语言聚合 files、lines、code、comments 和 blanks
- **THEN** 快照记录可用的按 code area 聚合文件数量、行数和复杂度

#### Scenario: 重复代码指标
- **WHEN** 观测命令识别重复代码片段
- **THEN** 快照记录重复片段所属 source tool、token count、参与文件、起始行和 code area
- **THEN** 快照记录该重复片段是否命中 changed scope
- **THEN** 快照保留足够信息支持本地复现第三方检测结果

### Requirement: 代码质量报告必须同时支持机器和人工消费
代码质量观测能力 MUST 从同一指标模型生成机器可读 JSON、人类可读 summary、CI/PR warning records 和必要的 raw tool outputs。JSON MUST 保留完整明细和聚合字段；summary MUST 展示最需要人工关注的排名、趋势、分组和重复代码信息；warning records MUST 保留可定位、可过滤、可复现的警告数据。

#### Scenario: 生成 JSON 明细
- **WHEN** 观测命令完成扫描
- **THEN** 机器可读输出包含 schema version、扫描元数据、工具名称和版本、扫描范围、排除规则、code areas、current/baseline 扫描输入指纹、baseline metadata、baseline status、comparison status、文件指标、函数指标、重复代码指标、聚合指标、baseline delta records 和 warning channels
- **THEN** 输出路径使用仓库相对路径或规范化路径
- **THEN** 输出不包含依赖目录、构建产物或缓存目录的指标记录
- **THEN** 输出显式记录每个 code area 的 current snapshot、同次运行生成的 baseline snapshot、delta 可用性、`baseline.status` 和 `comparison.status`
- **THEN** 输出将 warning records 分为 `warnings.all[]`、`warnings.changed[]` 和 `warnings.regressions[]`
- **THEN** `warnings.all[]` 包含当前快照中满足绝对阈值和 code area policy 的全量质量债记录
- **THEN** `warnings.changed[]` 只包含可用于 CI annotation 的 changed-scope warning records
- **THEN** `warnings.regressions[]` 只包含 changed warnings 中有 baseline delta basis 的退化记录

#### Scenario: 生成 Markdown summary
- **WHEN** 观测命令完成扫描
- **THEN** 人类可读 summary 展示仓库体量和语言占比
- **THEN** 人类可读 summary 展示按行数排序的文件列表
- **THEN** summary 展示按文件级复杂度排序的文件列表
- **THEN** summary 展示按圈复杂度排序的函数列表
- **THEN** summary 展示按函数行数或参数数量排序的函数列表
- **THEN** summary 展示重复代码片段摘要和按 code area 分组的重复代码信息
- **THEN** summary 在数据可用时展示 risk-ranked Changed Files Watchlist 和 Warnings sections
- **THEN** Changed Files Watchlist MUST 只展示命中风险条件的 changed files
- **THEN** 风险条件 MUST 至少包含触发 changed warning、有行数或复杂度 delta、或命中重复代码
- **THEN** Changed Files Watchlist MUST 限制展示数量，并说明 changed files 总数和展示数量，例如 `Changed files: 63 total, 10 shown by risk ranking`
- **THEN** summary MUST 展示全量 warnings 摘要，并单独展示 changed warnings
- **THEN** summary 中展示 current commit 或 baseline commit 时 MUST 同时展示 commit hash 和 commit title/subject

#### Scenario: 生成 warning records
- **WHEN** 观测命令生成 warning records
- **THEN** 每条 warning record 包含 level、rule id、source tool、normalized path、code area、changed scope、optional line、metric name、metric value、comparison basis、baseline value、delta value 和 message
- **THEN** changed warning records 可以渲染为 CI annotations、PR summaries 或 local report rows，且不需要重新解析 Markdown
- **THEN** full warning records 可以用于本地报告、长期治理和排序
- **THEN** CI annotation 输入 MUST 使用 changed warning records

#### Scenario: 生成临时产物目录
- **WHEN** 观测命令完成扫描
- **THEN** 默认在临时 artifact 目录写入 `metrics.json`、`report.md`、`warnings.ndjson` 和 `warnings-all.ndjson`
- **THEN** `warnings.ndjson` 的记录来源 MUST 是 `warnings.changed[]`，用于 CI annotation
- **THEN** `warnings-all.ndjson` 的记录来源 MUST 是 `warnings.all[]`，用于本地报告或长期治理场景
- **THEN** 默认在 raw output 子目录保留必要的 Lizard、scc 和 PMD CPD 原始输出或规范化前中间文件
- **THEN** 本地和 CI 可以通过同一目录结构查看结果和中间过程

### Requirement: 代码质量趋势必须基于同次运行生成的 previous-code baseline
代码质量观测能力 MUST 将趋势定义为 current snapshot 与同次运行生成的 previous-code baseline snapshot 的 delta。previous-code baseline MUST 来自 current revision 附近最近一次影响扫描输入的代码提交，并 MUST 使用当前质量观测配置、当前 wrapper 和当前工具版本重新扫描生成。缺少 git history、找不到 baseline commit 或 baseline scan 失败 MUST 不阻断 current snapshot 生成，也 MUST NOT 伪造 delta。

#### Scenario: 记录扫描输入指纹
- **WHEN** 观测命令完成扫描
- **THEN** 快照记录 current snapshot 每个 code area 的参与扫描文件数量、规范化文件列表摘要和 input fingerprint
- **THEN** 快照在 baseline scan 成功时记录 baseline snapshot 每个 code area 的参与扫描文件数量、规范化文件列表摘要和 input fingerprint
- **THEN** 快照记录用于本次 current 和 baseline 扫描的配置来源、wrapper 版本和 Lizard、scc、PMD CPD tool metadata
- **THEN** 快照记录本次 changed scope 是否命中纳入扫描的 code inputs

#### Scenario: 选择 baseline commit
- **WHEN** 观测命令需要生成 baseline delta records
- **THEN** 命令先根据当前配置确定纳入扫描的 code inputs
- **WHEN** current revision 修改了任一纳入扫描的 code input
- **THEN** baseline commit 是 current revision 之前最近一个影响 code inputs 的代码提交
- **WHEN** current revision 没有修改任何纳入扫描的 code input
- **THEN** baseline commit 是当前历史中最近一个影响 code inputs 的代码提交

#### Scenario: 生成 baseline snapshot
- **WHEN** 观测命令已经定位 baseline commit
- **THEN** 命令在临时隔离目录中检出、导出或等价读取该 commit 的文件内容
- **THEN** 命令使用当前配置、当前 wrapper 和当前工具版本重新扫描该 commit，生成 baseline snapshot
- **THEN** baseline raw outputs 保存在临时 artifact 目录的 raw 或 baseline 子目录中

#### Scenario: 文本-only 变更不产生动态 warning
- **WHEN** 本次变更没有修改任何纳入扫描的 code input
- **THEN** 当前快照仍然可以生成
- **THEN** baseline snapshot 来自最近一次影响 code inputs 的代码提交
- **THEN** current 与 baseline 的 code input fingerprint 相同
- **THEN** `comparison.status` 标记为 `input-unchanged`
- **THEN** summary 明确说明代码输入未变化
- **THEN** CI 默认不为复杂度、函数指标或重复代码生成 annotation

#### Scenario: baseline commit 缺失或不可生成
- **WHEN** 观测命令无法从 git history 定位上一次影响扫描输入的代码提交
- **OR** CI shallow clone 没有足够历史
- **OR** baseline checkout、导出或扫描失败
- **THEN** 当前快照仍然可以生成
- **THEN** `baseline.status` 标记为 `history-unavailable`、`no-baseline-commit`、`baseline-materialization-failed` 或 `baseline-scan-failed`
- **THEN** `comparison.status` 标记为 `baseline-unavailable`
- **THEN** report 可以展示当前 top N、watchlist 和聚合指标，但不得展示伪造 delta

#### Scenario: baseline scan 被显式跳过
- **WHEN** 维护者运行观测命令并显式跳过 baseline scan
- **THEN** 当前快照仍然可以生成
- **THEN** `baseline.status` 标记为 `baseline-skipped`
- **THEN** `comparison.status` 标记为 `baseline-unavailable`
- **THEN** report 明确说明 baseline scan was skipped，不得把它描述为 Git history 不足

#### Scenario: baseline scan 成功
- **WHEN** 观测命令成功生成 current snapshot 和 baseline snapshot
- **AND** 至少一个纳入扫描的 code area 相对 baseline 发生 code input 变化
- **THEN** `baseline.status` 标记为 `generated`
- **THEN** `comparison.status` 标记为 `compared`
- **THEN** 快照记录整体和按 code area 的 current value、baseline value 和 delta value
- **THEN** delta 至少覆盖仓库体量、语言占比、文件数量、行数、文件级复杂度、函数级行数、函数参数数量、函数圈复杂度和重复代码片段数量中可生成 delta 的字段
- **THEN** warning records 可以引用该 delta 作为 comparison basis

#### Scenario: baseline 使用当前配置和工具
- **WHEN** 当前提交修改了质量观测配置、wrapper 或工具安装版本
- **THEN** baseline snapshot 仍使用当前提交中的配置和当前运行环境中的工具重新生成
- **THEN** report 明确说明 delta 表示当前规则和当前工具下 current snapshot 相对 previous-code baseline snapshot 的差异

### Requirement: 代码质量观测必须由配置文件驱动扫描和汇报
代码质量观测能力 MUST 通过仓库配置文件定义扫描范围、排除规则、generated files、默认 code areas、tool options、按 code area 拆分的 warning policy、PMD CPD `minimum tokens` 和 artifact directory defaults。配置文件 MUST 是质量观测行为的 owner，脚本实现 MUST NOT 将这些规则散落为不可发现的硬编码逻辑。

#### Scenario: 配置扫描范围
- **WHEN** 观测命令启动
- **THEN** 命令读取仓库质量观测配置文件
- **THEN** 命令根据配置解析 include paths、exclude paths、generated files 和 code areas
- **THEN** 配置错误产生可读诊断并导致执行错误

#### Scenario: 默认 code areas
- **WHEN** 配置文件定义初始 code areas
- **THEN** 配置 MUST 至少定义 `rust-production`、`rust-tests`、`node-production-scripts`、`node-validation-smoke`、`fixtures-examples` 和 `generated`
- **THEN** 快照、summary 和 warning records 保留每条指标所属 code area
- **THEN** 未匹配到具体 code area 的纳入文件 MUST 使用清晰诊断或显式 fallback policy，避免静默进入错误阈值组

#### Scenario: code area 语义
- **WHEN** 配置定义 `rust-production`
- **THEN** 该区域覆盖 `crates/*/src/**` 中非 tests、fixtures、generated 的 Rust production code，并使用最严格 warning policy
- **WHEN** 配置定义 `rust-tests`
- **THEN** 该区域覆盖 `crates/*/tests/**`、`crates/*/src/tests/**` 和 `tests.rs`，并使用比 production 放宽的 warning policy
- **WHEN** 配置定义 `node-production-scripts`
- **THEN** 该区域覆盖 `scripts/*.ts` 和 `scripts/tools/**` 下非测试、fixture、generated 的脚本模块，并使用中等严格 warning policy
- **WHEN** 配置定义 `node-validation-smoke`
- **THEN** 该区域覆盖 `scripts/tools/validators/**`、脚本单测、`test/smoke/**`、`test/tools/**` 和类似 validation/smoke case modules，并使用比 production scripts 放宽的 warning policy
- **WHEN** 配置定义 `fixtures-examples`
- **THEN** 该区域覆盖 fixtures、cases、测试数据和示例输入输出，并默认只进入 summary 或 watchlist，不默认生成 PR annotation
- **WHEN** 配置定义 `generated`
- **THEN** 该区域覆盖 generated files 和配置显式标记的生成物，并默认排除 warning，只保留体量统计或完全排除

#### Scenario: 按 code area 配置 warning 和 CPD
- **WHEN** 配置文件定义 warning policy
- **THEN** Lizard、scc 和 PMD CPD warning thresholds 可以按 code area 拆分
- **THEN** PMD CPD `minimum tokens` MUST 按 code area 配置，而不是使用单一全仓库默认值
- **THEN** production 区域使用更敏感的 duplicate-code policy，tests 和 validation/smoke 区域使用更宽的 duplicate-code policy，fixtures/examples 和 generated 默认不发 PR annotation

### Requirement: 代码质量观测必须限定扫描边界
代码质量观测能力 MUST 默认扫描仓库维护的 Rust 源码和 JavaScript 脚本，并 MUST 排除依赖、构建产物、虚拟环境、发布产物、生成物和缓存目录。未来扩展到文档、schema 或其它语言时，MUST 在配置文件和指标模型中明确新增路径范围和指标语义。

#### Scenario: 默认排除非源码产物
- **WHEN** 观测命令遍历仓库
- **THEN** 默认排除 `.git`、`target`、`node_modules`、`.venv`、`dist`、`build`、缓存目录和已标记的 generated files
- **THEN** 报告不把这些目录中的文件计入文件排名、函数排名或聚合指标

#### Scenario: 按语言、路径和代码区域分组
- **WHEN** 快照包含多种语言或多个代码区域
- **THEN** 报告保留 language、path group 和 code area 信息
- **THEN** 汇总结果能区分 `rust-production`、`rust-tests`、`node-production-scripts`、`node-validation-smoke`、`fixtures-examples`、`generated` 和其它未来纳入范围

### Requirement: 代码质量观测必须为未来报告和门禁保留演进边界
代码质量观测能力 MUST 将当前快照、动态 warning、同次运行生成的 baseline delta、未来自动报告和未来门禁策略分离。阻断阈值、PR 差异阻断或豁免机制 MUST 由后续 change 明确定义，不得从首期观测结果隐式推导。

#### Scenario: 自动报告可复用快照
- **WHEN** 未来 CI 或本地工作流需要生成自动报告
- **THEN** 它可以复用当前观测快照的 JSON 数据模型
- **THEN** 自动报告可以展示趋势、top N 或 changed files 指标
- **THEN** 自动报告不得在未定义门禁策略时因指标值失败

#### Scenario: CI 正常产出非阻断报告
- **WHEN** CI 运行质量观测命令
- **THEN** CI 上传临时 artifact 目录或其关键产物
- **THEN** CI 写入 step summary，展示 `report.md` 的核心内容或链接
- **THEN** CI 可以根据 warning records 发出非阻断 annotation
- **THEN** Lizard、scc 或 PMD CPD 的指标值不使 CI job 失败

#### Scenario: 动态 warning 聚焦 changed scope
- **WHEN** changed file 或 changed function 命中配置的 warning rule
- **THEN** warning 可以输出到 local output、CI annotation 或 PR summary
- **THEN** warning 使用 changed scope、同次运行生成的 baseline delta 和 absolute floor 来避免报告噪音型历史债务
- **THEN** warning rule 使用命中文件所属 code area 的阈值、展示策略和 CPD minimum tokens
- **THEN** unchanged historical hotspots 保留在 summary 或 watchlist，除非未来 gate policy 另行定义
- **THEN** 当 `comparison.status` 为 `input-unchanged` 或 `baseline-unavailable` 时，CI annotation 必须降级为 summary/watchlist 或明确说明 comparison basis 不可用

#### Scenario: 未来门禁需要单独定义
- **WHEN** 后续 change 提议让代码质量指标阻断合并或验证
- **THEN** 该 change MUST 明确阈值或 baseline 来源
- **THEN** 该 change MUST 明确豁免、历史债务处理和 changed files 策略
- **THEN** 该 change MUST 明确失败输出和修复指引
