## MODIFIED Requirements

### Requirement: 代码质量 baseline delta 必须是显式 opt-in
代码质量观测能力 MUST 默认生成 current snapshot，并 MUST 默认跳过 previous-code baseline 定位、物化、扫描和 delta 生成。baseline delta MAY 作为显式诊断入口启用；启用时，previous-code baseline MUST 来自 current revision 附近最近一次影响扫描输入的代码提交或调用方指定的 commit，并 MUST 使用当前质量观测配置、当前 wrapper 和当前工具版本扫描生成，或从能够证明扫描输入、工具版本、wrapper/config version、code area fingerprint 和工具参数等价的缓存条目加载。缺少 git history、找不到 baseline commit、baseline scan 失败或 baseline cache 无法安全加载 MUST 不阻断 current snapshot 生成，也 MUST NOT 伪造 delta。

#### Scenario: 记录扫描输入指纹
- **WHEN** 观测命令完成扫描
- **THEN** 快照记录 current snapshot 每个 code area 的参与扫描文件数量、规范化文件列表摘要和 input fingerprint
- **THEN** 快照在 baseline scan 成功时记录 baseline snapshot 每个 code area 的参与扫描文件数量、规范化文件列表摘要和 input fingerprint
- **THEN** 快照记录用于本次 current 扫描的配置来源、wrapper 版本和 Lizard、scc、PMD CPD tool metadata
- **THEN** 显式启用 baseline comparison 时，快照记录 baseline 扫描使用的配置来源、wrapper 版本和 Lizard、scc、PMD CPD tool metadata
- **THEN** 显式启用 baseline comparison 时，快照记录本次 changed scope 是否命中纳入扫描的 code inputs

#### Scenario: 选择 baseline commit
- **WHEN** 维护者显式要求生成 baseline delta records
- **THEN** 命令先根据当前配置确定纳入扫描的 code inputs
- **WHEN** current revision 修改了任一纳入扫描的 code input
- **THEN** baseline commit 是 current revision 之前最近一个影响 code inputs 的代码提交
- **WHEN** current revision 没有修改任何纳入扫描的 code input
- **THEN** baseline commit 是当前历史中最近一个影响 code inputs 的代码提交

#### Scenario: 生成 baseline snapshot
- **WHEN** 观测命令已经定位 baseline commit
- **THEN** 命令在临时隔离目录中检出、导出或等价读取该 commit 的文件内容
- **THEN** 命令使用当前配置、当前 wrapper 和当前工具版本扫描该 commit，或加载 cache key 证明等价的规范化 tool/code-area 指标，生成 baseline snapshot
- **THEN** baseline raw outputs 保存在临时 artifact 目录的 raw 或 baseline 子目录中

#### Scenario: 使用 PMD CPD 分区级扫描缓存
- **WHEN** 观测命令扫描 current 或 opt-in baseline snapshot 的重复代码指标
- **THEN** 命令 MAY 在 `.cache/docnav/quality/<scan_cache_version>/` 下按 PMD CPD、code area 和 scan kind 读写规范化 duplicate-code 指标缓存
- **THEN** cache key MUST 至少包含 scan cache version、scan kind、tool name、tool version、normalized tool args、config version、code area、commit sha 和该 code area 的 input fingerprint
- **THEN** cache payload MUST 校验 cache key、scan cache version、tool/version/config/fingerprint metadata 和 payload shape
- **THEN** cache payload 损坏、shape 不匹配或 metadata 不匹配时 MUST 当作 cache miss，且 MUST NOT 使质量观测失败
- **THEN** 只有成功完成并通过规范化的 PMD CPD code-area duplicate-code 指标 MAY 写入 cache；工具不可用、运行失败或 skipped task MUST NOT 写入表示成功的空 metrics cache
- **THEN** cache payload MUST 只保存该 PMD CPD code-area 的规范化 duplicate-code raw metrics，不得固化 changed scope、warning records、aggregates、trends 或 final report
- **THEN** 命中 PMD CPD cache 后，命令 MUST 按本次 changed files 重新标注 duplicate fragments 的 `hitsChangedScope`，并 MUST 重新生成 aggregates、trends、warnings 和 report

#### Scenario: 有界并行扫描 CPD cache miss task
- **WHEN** PMD CPD cache miss 涉及多个 code area
- **THEN** 命令 MAY 为每个 cache miss code area 启动一个 bounded parallel PMD CPD scan task
- **THEN** 每个 task MUST 只扫描该 code area 的文件，并使用该 code area 配置的 `minimum tokens`
- **THEN** 成功 task 的结果写入 cache 时 MUST 使用对应 code area 的 cache key 和 payload，不得写入跨 code area 或 language-level final report
- **THEN** 并发数 MUST 受配置上限约束，避免无界启动 PMD/Java 进程

#### Scenario: 默认不生成 baseline delta
- **WHEN** 维护者运行观测命令且未显式启用 baseline comparison
- **THEN** 当前快照仍然生成
- **THEN** 命令不得定位 baseline commit、导出 baseline worktree 或扫描 baseline snapshot
- **THEN** `baseline.status` 标记为 `baseline-skipped`
- **THEN** `comparison.status` 标记为 `baseline-unavailable`
- **THEN** `warnings.changed[]` 和 `warnings.regressions[]` 为空数组

#### Scenario: baseline commit 缺失或不可生成
- **WHEN** 观测命令无法从 git history 定位上一次影响扫描输入的代码提交
- **OR** CI shallow clone 没有足够历史
- **OR** baseline checkout、导出或扫描失败
- **THEN** 当前快照仍然可以生成
- **THEN** `baseline.status` 标记为 `history-unavailable`、`no-baseline-commit`、`baseline-materialization-failed` 或 `baseline-scan-failed`
- **THEN** `comparison.status` 标记为 `baseline-unavailable`
- **THEN** 任一 baseline tool/code-area 指标不可用、运行失败或 skipped 时，命令不得把该分区解释为成功的空指标，也不得用部分 baseline 指标生成 `compared` delta
- **THEN** report 可以展示当前 top N、watchlist 和聚合指标，但不得展示伪造 delta

#### Scenario: baseline scan 被跳过
- **WHEN** 维护者运行默认观测命令或显式跳过 baseline scan
- **THEN** 当前快照仍然可以生成
- **THEN** `baseline.status` 标记为 `baseline-skipped`
- **THEN** `comparison.status` 标记为 `baseline-unavailable`
- **THEN** report 明确说明 baseline scan was skipped，不得把它描述为 Git history 不足

#### Scenario: opt-in baseline scan 成功
- **WHEN** 观测命令在显式启用 baseline comparison 后成功生成 current snapshot 和 baseline snapshot
- **AND** 至少一个纳入扫描的 code area 相对 baseline 发生 code input 变化
- **THEN** `baseline.status` 标记为 `generated`
- **THEN** `comparison.status` 标记为 `compared`
- **THEN** 快照记录整体和按 code area 的 current value、baseline value 和 delta value
- **THEN** delta 至少覆盖仓库体量、语言占比、文件数量、行数、文件级复杂度、函数级行数、函数参数数量、函数圈复杂度和重复代码片段数量中可生成 delta 的字段
- **THEN** warning records 可以引用该 delta 作为 comparison basis

#### Scenario: baseline 使用当前配置和工具
- **WHEN** 当前提交修改了质量观测配置、wrapper 或工具安装版本
- **THEN** baseline snapshot 仍使用当前提交中的配置和当前运行环境中的工具扫描生成，或仅在 cache key 证明这些配置和工具输入完全等价时复用缓存
- **THEN** report 明确说明 delta 表示当前规则和当前工具下 current snapshot 相对 previous-code baseline snapshot 的差异
