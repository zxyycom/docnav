本 design 为 `implement-code-quality-observability` 定义非阻断代码质量观测的实现边界；实现范围限于质量扫描脚本、配置、临时产物、CI 汇报和验证，不改变 `docnav` CLI、adapter、MCP、schema 或 examples 的业务契约。

## Context

Docnav 是 Rust workspace，同时包含 Node.js 脚本、OpenSpec 文档、schema/example 验证和若干仓库内 agent skill。当前质量信号主要来自 `cargo fmt`、`cargo clippy`、单元测试、smoke 测试和 schema/docs 验证，这些命令证明行为和契约是否正确，但不会直接展示代码体量、语言占比、文件复杂度、函数复杂度或重复代码的分布。

本 change 面向可执行的“先观测”。指标输出应帮助维护者发现值得阅读、拆分或去重的热点文件、函数和重复片段，但不自动判定代码是否可合并。Clippy 继续作为 Rust 阻断式质量门禁；Lizard、scc 和 PMD CPD 作为非阻断观测输入，由仓库脚本统一输出稳定的 JSON、Markdown 和 warning records。首期趋势使用 `previous-code baseline`：同一次运行用当前配置和当前工具扫描 current checkout，再扫描上一次影响扫描输入的代码提交，并比较两份新生成的快照。未来如果要把观测指标变成门禁，需要另行设计阻断阈值、豁免和 PR 差异失败规则。

## Goals / Non-Goals

**Goals:**

- 实现一个非阻断的代码质量指标快照入口，覆盖 Rust 和仓库内 JavaScript 脚本。
- 明确 Clippy、Lizard、scc 和 PMD CPD 的职责分层：Clippy 负责 Rust 阻断式 lint gate，Lizard 负责函数级复杂度快照和 warning 来源，scc 负责仓库体量、语言占比、文件级复杂度、趋势和报告输入，PMD CPD 负责重复代码检测信号。
- 使用配置文件定义扫描范围、排除规则、默认 6 类 code areas、generated files、warning 规则和工具参数。
- 首期报告仓库体量、语言占比、文件行数、文件级复杂度、函数级行数、函数参数数量、函数圈复杂度和重复代码片段。
- 同时保留机器可读 JSON、人类可读 Markdown summary、CI/PR warning records 和必要的第三方原始输出归档，方便人工查看和后续自动报告。
- 让 warning 和 PMD CPD `minimum tokens` 按 code area 拆分，并以 changed scope、previous-code baseline delta 和绝对下限组合触发，避免把当前观测误写成硬阈值。
- 记录 current/baseline code input fingerprint、baseline commit、baseline status、comparison status 和 tool metadata；`input-unchanged` 只进入 summary，不生成动态 warning。
- 将生成产物放在可忽略的临时本地/CI artifact 目录，不污染源码和主规范。

**Non-Goals:**

- 不在本 change 中新增阻断式质量门禁。
- 不用 Lizard、scc 或 PMD CPD 替代 Clippy、测试、schema 验证、smoke 验证或人工 code review。
- 不把指标结果解释为必须重构的结论。
- 不把质量观测产物提交为源码或主规范材料。

## Decisions

1. 代码质量观测作为独立脚本或命令存在，不并入阻断验证链路。

   理由：当前目标是获得可见性，而不是执行治理。独立命令可以低成本生成报告，也能避免首版指标噪音影响现有交付节奏。

   备选方案：直接加入 `verify:docnav-workspace`。该方案曝光度更高，但会把尚未校准的观测误解为门禁。

2. 输出同时包含 JSON 和 Markdown。

   理由：JSON 适合后续自动报告、趋势比较和 CI artifact 消费；Markdown 适合本地查看、PR 评论和人工 triage。两者应来自同一指标模型，避免报告口径漂移。

   备选方案：只输出文本 summary。该方案实现更快，但后续自动化会重新解析文本，维护成本更高。

3. Clippy、Lizard、scc 和 PMD CPD 分层使用，仓库脚本拥有长期输出契约。

   理由：Clippy 是 Rust 语义 lint 和阻断式质量门禁；Lizard 提供跨语言函数级 NLOC、参数数量和圈复杂度；scc 提供仓库体量、语言占比、文件级复杂度和趋势报告输入；PMD CPD 提供跨文件 copy-paste 重复代码检测信号。仓库脚本负责统一字段、路径、排序、warning 规则和错误处理，避免把第三方原始输出当作长期契约。

   备选方案：只使用单一工具。该方案更简单，但会牺牲函数级定位或仓库级趋势中的一侧能力。

4. 首期指标使用排名、聚合、changed scope 和 previous-code baseline delta，不定义全仓库硬阈值。

   理由：仓库需要先知道当前分布和噪音来源，再决定是否采用阻断式 baseline、差异检查或阈值。首期报告可以展示 top files/functions、按 crate/script 分组的聚合值、changed files 指标，以及 current snapshot 相对同次运行生成的 previous-code baseline snapshot 的 delta。

   备选方案：立即设置默认阈值。该方案简单，但容易让历史代码一次性产生大量非行动项。

5. 指标采集必须由配置文件定义扫描边界和默认 6 类代码区域。

   理由：质量观测应反映维护者实际负责的源码、脚本和少量仓库文档辅助代码。依赖、构建产物、缓存和生成物会放大噪音，并导致结果不可复现。配置文件应定义默认排除目录、generated files 和 6 个首期 code areas：`rust-production`、`rust-tests`、`node-production-scripts`、`node-validation-smoke`、`fixtures-examples` 和 `generated`。这样报告可以按维护性质使用不同 warning 策略，并保持第一版分区足够稳定。

   初始 code area 语义：

   - `rust-production`: `crates/*/src/**` 中非 tests、fixtures、generated 的 Rust production code，使用最严格 warning 策略。
   - `rust-tests`: `crates/*/tests/**`、`crates/*/src/tests/**` 和 `tests.rs`，阈值比 production 放宽。
   - `node-production-scripts`: `scripts/*.mjs` 和非 smoke、fixture、generated 的脚本模块，使用中等严格策略。
   - `node-validation-smoke`: `scripts/validators/**`、`scripts/*smoke*/**`、`scripts/cli-smoke/**` 和类似 validation/smoke case modules，阈值比 production scripts 放宽。
   - `fixtures-examples`: fixtures、cases、测试数据和示例输入输出，默认进入 summary/watchlist，不默认发 PR annotation。
   - `generated`: generated files 和配置显式标记的生成物，默认排除 warning，只保留体量统计或完全排除。

6. 质量观测产物默认写入临时目录，CI 正常产出报告。

   理由：本地运行时维护者应能在临时目录查看完整中间过程、第三方原始输出和最终报告；CI 运行时应上传 artifact、写入 step summary，并按 warning records 发出非阻断 annotation。临时目录默认使用 `artifacts/docnav-quality/`，避免与 Cargo 编译产物所在的 `target/` 混在一起；后续可以通过配置或环境变量覆盖。

7. Dynamic warning 和 CPD minimum tokens 只针对有行动价值的变化发出。

   理由：warning 应优先指向 changed files 或 changed functions，并结合 previous-code baseline delta 与绝对下限，例如复杂度或函数长度显著上升且超过当前分布的高位区间。CPD `minimum tokens` 应按 code area 拆分：production 区域更敏感，tests 和 validation/smoke 区域更宽，fixtures/examples 默认只进 summary，generated 默认不报。历史热点保留在 watchlist 或 summary，不默认刷 PR annotation。

8. 趋势比较由当前运行自动生成 previous-code baseline snapshot。

   规则：`previous-code baseline` 是当前 revision 之前最近一个影响扫描输入的代码提交。若 current revision 修改了扫描输入，baseline 是它之前的最近代码提交；若 current revision 只修改文档或其它非扫描输入，baseline 是最近一次代码提交，current 与 baseline 的扫描输入指纹相同。

   执行方式：脚本先用当前 checkout 生成 current snapshot，再根据当前配置解析 scan inputs，通过 git history 定位 baseline commit，在临时 worktree、archive copy 或等价隔离目录中用当前配置、当前 wrapper 和当前工具版本生成 baseline snapshot，最后比较 current 与 baseline。输出使用两个独立状态：`baseline.status` 表示基线快照是否生成、是否被显式跳过或失败；`comparison.status` 表示 delta 是否可用，或是否为 `input-unchanged`。

## Risks / Trade-offs

- [Risk] 指标被误读为重构要求或合并阻断 → Mitigation：命令命名、报告标题和 spec 明确标注 non-blocking snapshot。
- [Risk] 第三方工具跨平台安装不稳定 → Mitigation：实现先检查 Lizard、scc 和 PMD CPD 可用性，缺失时输出清晰诊断；CI 引入前固定安装方式。
- [Risk] 多语言指标口径不完全一致 → Mitigation：报告按语言/路径分组展示，并只把跨语言可比的字段用于整体排序。
- [Risk] 报告过长导致不可读 → Mitigation：默认 summary 只展示 top N 和聚合，完整明细保留在 JSON。
- [Risk] CI 浅克隆或本地历史不足导致找不到 baseline commit → Mitigation：`baseline.status` 必须显式记录 `history-unavailable`、`no-baseline-commit`、`baseline-materialization-failed` 或 `baseline-scan-failed`；基线无法生成时仍生成 current snapshot，`comparison.status` 记录 `baseline-unavailable`，且不伪造 delta。
- [Risk] scc 文件级复杂度与 Lizard 函数级复杂度口径不同 → Mitigation：JSON 记录 metric source 和 scope，summary 分开展示文件级和函数级排名。
- [Risk] CPD 重复检测容易把测试 fixture 或生成代码报成噪音 → Mitigation：配置文件按 6 类默认 code area 和 generated files 过滤，并按 code area 设置不同 minimum tokens 或 warning policy。

## Migration Plan

1. 定义统一 metrics JSON schema、Markdown summary 结构、warning records 结构、基线扫描字段、趋势比较字段和临时产物目录结构。
2. 新增质量观测配置文件，定义扫描范围、排除规则、generated files、默认 6 类 code areas、tool options、按 code area 拆分的 CPD `minimum tokens` 和 warning policy。
3. 封装仓库脚本采集 Clippy gate 状态、Lizard 函数级指标、scc 仓库/文件级指标和 PMD CPD 重复代码指标。
4. 为每个 code area 计算扫描输入指纹，定位 previous-code baseline commit，并用当前配置和工具在隔离目录中生成 baseline snapshot。
5. 将中间过程和最终产物写入临时 artifact 目录，例如 `artifacts/docnav-quality/metrics.json`、`artifacts/docnav-quality/report.md`、`artifacts/docnav-quality/warnings.ndjson` 和 `artifacts/docnav-quality/raw/`。
6. 新增本地脚本入口，例如 `pnpm quality:scan`，并确保 Lizard、scc 和 PMD CPD 指标值不导致命令失败。
7. 在 CI 中正常产出 artifact、step summary 和非阻断 warning annotation。
8. 为配置解析、路径过滤、分区、baseline commit 定位、baseline materialization、baseline scan、报告生成、warning 规则、`baseline.status`、`comparison.status` 和工具输出 normalization 添加测试或 fixture 验证。
9. 积累样本后，再单独评估历史曲线、PR 评论或门禁策略。

## Open Questions

- 后续是否需要在同次运行的 previous-code baseline 之外，引入长期历史曲线或 dashboard。
