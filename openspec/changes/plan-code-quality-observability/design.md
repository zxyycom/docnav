本 design 只为 `plan-code-quality-observability` 记录未来实现非阻断代码质量观测的方案边界；当前仍是未审核临时文档，不改变现有实现、主规范或验证链路。

## Context

Docnav 是 Rust workspace，同时包含 Node.js 脚本、OpenSpec 文档、schema/example 验证和若干仓库内 agent skill。当前质量信号主要来自 `cargo fmt`、`cargo clippy`、单元测试、smoke 测试和 schema/docs 验证，这些命令证明行为和契约是否正确，但不会直接展示代码体量、符号密度或函数复杂度的分布。

本 change 只面向“先观测”。指标输出应帮助维护者发现值得阅读或拆分的热点文件和函数，但不自动判定代码是否可合并。未来如果要把指标变成门禁，需要另行设计 baseline、阈值、豁免和 PR 差异规则。

## Goals / Non-Goals

**Goals:**

- 建立一个非阻断的代码质量指标快照计划，覆盖 Rust 和仓库内 JavaScript 脚本。
- 首期报告文件级行数、文件级符号数量、函数级行数、函数参数数量和函数圈复杂度。
- 同时保留机器可读 JSON 和人类可读 Markdown summary，方便人工查看和后续自动报告。
- 让指标以排名、分组和趋势为主，避免把当前观测误写成硬阈值。
- 将生成产物放在可忽略的本地/CI artifact 目录，不污染源码和主规范。

**Non-Goals:**

- 不在当前计划中要求新增阻断式质量门禁。
- 不要求当前立即选择唯一度量引擎；未来实现可以先比较 `lizard`、`rust-code-analysis`、CodeGraph 或仓库自定义脚本。
- 不把指标结果解释为必须重构的结论。
- 不替代 `cargo fmt`、`cargo clippy`、测试、schema 验证或人工 code review。
- 不扫描 `target`、`node_modules`、`.venv`、`dist`、`build` 和缓存目录。

## Decisions

1. 代码质量观测先作为独立脚本或命令存在，不并入阻断验证链路。

   理由：当前目标是获得可见性，而不是执行治理。独立命令可以低成本生成报告，也能避免首版指标噪音影响现有交付节奏。

   备选方案：直接加入 `verify:docnav-workspace`。该方案曝光度更高，但会把尚未校准的观测误解为门禁。

2. 输出同时包含 JSON 和 Markdown。

   理由：JSON 适合后续自动报告、趋势比较和 CI artifact 消费；Markdown 适合本地查看、PR 评论和人工 triage。两者应来自同一指标模型，避免报告口径漂移。

   备选方案：只输出文本 summary。该方案实现更快，但后续自动化会重新解析文本，维护成本更高。

3. 首期指标使用排名和聚合，不定义硬阈值。

   理由：仓库需要先知道当前分布和噪音来源，再决定是否采用 baseline、差异检查或阈值。首期报告可以展示 top files/functions、按 crate/script 分组的聚合值和 changed files 指标。

   备选方案：立即设置默认阈值。该方案简单，但容易让历史代码一次性产生大量非行动项。

4. 指标采集必须过滤生成物和依赖目录。

   理由：质量观测应反映维护者实际负责的源码、脚本和少量仓库文档辅助代码。依赖、构建产物和缓存会放大噪音，并导致结果不可复现。

5. 第三方度量工具应包在仓库脚本后面。

   理由：`lizard`、`rust-code-analysis` 或其它工具的字段和输出格式可能变化。仓库脚本应负责统一输出 schema、路径归一、排序规则和错误处理，避免将工具原始输出作为长期契约。

## Risks / Trade-offs

- [Risk] 指标被误读为重构要求或合并阻断 → Mitigation：命令命名、报告标题和 spec 明确标注 non-blocking snapshot。
- [Risk] 第三方工具跨平台安装不稳定 → Mitigation：未来实现先检查本地依赖可用性，缺失时输出清晰诊断；CI 引入前固定安装方式。
- [Risk] 多语言指标口径不完全一致 → Mitigation：报告按语言/路径分组展示，并只把跨语言可比的字段用于整体排序。
- [Risk] 报告过长导致不可读 → Mitigation：默认 summary 只展示 top N 和聚合，完整明细保留在 JSON。
- [Risk] 历史数据没有上下文 → Mitigation：首期只生成当前快照；趋势报告需要另行定义 baseline 存储和比较策略。

## Migration Plan

1. 当前只保留本 change 作为计划，不执行实现。
2. 未来实现时先定义统一 metrics JSON schema 和 Markdown summary 结构。
3. 封装一个仓库脚本采集 Rust 与 JavaScript 指标，并过滤依赖、构建产物和缓存目录。
4. 生成本地 artifact，例如 `target/docnav-quality/metrics.json` 和 `target/docnav-quality/report.md`。
5. 在 CI 中先上传或打印报告，不使 job 因指标值失败。
6. 积累样本后，再单独评估 baseline、趋势比较、PR 评论或门禁策略。

## Open Questions

- 首期实现使用 `lizard` 作为跨语言入口，还是 Rust 使用 `rust-code-analysis`、JavaScript 使用独立解析器。
- 报告是否需要区分 production code、tests、scripts 和 skill runtime。
- 后续趋势数据存储在 CI artifact、仓库 baseline 文件，还是外部报告系统。
