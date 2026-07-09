本 design 是 `create-universal-cli-config-crate` 的 change-local 技术方案：说明如何从 Docnav 当前实现抽象出可子仓库化复用的 Rust CLI/config resolution crate；当前文档只存在于 `openspec/changes/create-universal-cli-config-crate/`，不影响主规范及现有其它文档。

## Context

Docnav 当前已经有两个可抽象基础：`typed-fields` 拥有字段 identity、路径投影、类型、约束、默认值和校验事实；`parameter-resolution` 拥有 direct/project/user/default 来源优先级、默认值 fallback、diagnostic handoff 和 passthrough。

当前限制是这些能力还带有 Docnav 的角色命名和固定来源集合，CLI flag、env、config file、dynamic default、custom source 不能作为同一套 source/projection contract 被外部 Rust CLI 复用。Docnav 的 adapter、operation、protocol、readable output 和 diagnostic code identity 必须继续留在 Docnav owner 内，不能进入通用库核心。

## Goals / Non-Goals

**Goals:**

- 从现有 `typed-fields` 与 `parameter-resolution` 中抽象通用字段契约、来源投影、来源抽取、优先级合并、merge strategy、来源追踪和最终 typed materialization。
- 形成可在当前 workspace 中验证、后续迁移为独立 repository 的 crate 边界。
- 让 Docnav 当前 navigation input resolution 在本 change 内 hard cutover 到新库，并保持现有 public protocol、CLI 行为、adapter contract、readable output 和 protocol output 不变。
- 支持 CLI flag、env var、config document、static/dynamic default 和 custom source 的统一来源模型。
- 为 explain/debug 输出保留 deterministic provenance trace，而不是只返回最终值。

**Non-Goals:**

- 不实现新的 Docnav 文档导航能力。
- 不改变 `outline -> ref -> read` 协议、request/response schema、adapter native option semantics 和 output modes。
- 不把 `clap`、TOML、JSON、Figment 等具体框架绑定进核心 crate；这些只能通过 companion crate 进入。
- 不在未经审计前发布外部 crate，也不改写现有主规范。

## Decisions

### Decision 1: Workspace-first 的通用 crate 边界

本 change 新增 `cli-config-resolution` capability，拥有通用 Rust CLI/config resolution 的长期契约。实现阶段先在当前 workspace 中建立独立 crate 边界，确保 tests、docs 和 Docnav hard cutover validation 能复用当前验证链路；release-readiness 通过后迁移到独立 repository。

### Decision 2: 字段事实与 source policy 分层

字段定义继续围绕 stable identity、value kind、constraint、default、projection path 和 validation failure facts。通用库不让字段核心拥有 CLI/config/public diagnostic 语义，而是在上层 resolution crate 中声明 source projection 和 merge behavior。

### Decision 3: 将 fixed source slots 改为 ordered source collection

现有 `DirectInput / ProjectConfig / UserConfig / Default` 模型应泛化为 ordered `SourceSpec` / `SourceId` 集合。每个 source 保留 kind、priority、locator、explicitness 和 load/parse 状态；resolver 只根据 source policy 处理 source candidates，不硬编码 Docnav 的四层优先级。

### Decision 4: resolution 返回最终值和 provenance trace

Resolver 必须返回 materialized typed values，同时保留 selected candidate、被覆盖 candidates、validation diagnostics、merge path 和 source locator。`explain()` API 从 trace 派生，不从最终 struct 反推。

### Decision 5: merge strategy 属于 field-level resolution policy

每个字段声明一个 merge strategy。MVP 至少支持 scalar replace、list append、list replace、map merge、map replace 和 deny-conflict；默认策略必须 deterministic，并可被测试直接断言。

### Decision 6: framework integrations 保持 companion crate 层

核心 crate 只定义 field contract、source value、resolver、trace 和 diagnostics。`clap` arg 生成/读取进入 `cli-config-resolution-clap` companion crate；serde-compatible structured config source 进入 `cli-config-resolution-serde` companion crate；derive macro convenience 不进入首批实现。companion crates 不能反向污染核心 API。

### Decision 7: Docnav 集成通过 hard cutover 一步切换

Docnav 集成采用 hard cutover。实现阶段先用 tests 和 fixture 对比证明新 resolver 覆盖现有 navigation input resolution 行为，然后在同一 change 内把 Docnav 调用链切到新 resolver，并删除旧 fixed source resolver 的运行时路径。实现完成状态不得保留新旧 resolver 双路径、runtime feature flag、fallback 开关或兼容 wrapper。回滚方式是代码级 revert，不是运行时切换。

### Decision 8: 审计出口与 release-readiness 决策门

Artifact 审计出口是 proposal、design、spec delta 和 tasks 围绕同一核心句展开：从 Docnav typed-fields 和 parameter-resolution 抽象出可子仓库化复用的 Rust CLI/config resolution crate。工作区实现使用 `cli-config-resolution` 作为 capability 与 crate/package 工作名；实现审计发现命名冲突、owner 冲突、public contract 风险任一问题时，必须先更新本 design 的决策再执行 crate 创建，已创建时同步重命名。

已定默认路径：工作区 crate 名为 `cli-config-resolution`，Docnav 集成使用 hard cutover，framework integrations 使用 companion crates，derive macro 延后到核心 API 稳定后的独立 change，外部发布目标为独立 repository。8.x release-readiness 前只确认外部 package 名可用性、发布节奏和 repository metadata；发现外部包名冲突、仓库策略冲突、发布渠道风险任一问题时，执行者必须主动向用户提问。任何会改变 capability ID、Docnav public behavior、外部 package 名和发布渠道的变更，都必须同步更新 proposal、design、相关 spec delta 和 tasks 后再继续。

## Risks / Trade-offs

- [Risk] 抽象过宽导致第一版 API 难以稳定 → Mitigation: MVP 只覆盖已由 Docnav 需求证明的 source/value/trace/merge 行为，外部扩展通过 custom source 与 companion crates 补充。
- [Risk] 子仓库化过早导致验证链路丢失 → Mitigation: 先在 workspace 内完成 crate 边界和 Docnav hard cutover validation，再执行独立 repository 迁移步骤。
- [Risk] provenance trace 设计复杂，拖慢 resolver 实现 → Mitigation: trace 先记录 selected/overridden/diagnostic/source locator 的最小闭包，human-readable explain 可以后置。
- [Risk] merge semantics 与 source priority 组合产生歧义 → Mitigation: 每个 merge strategy 必须有 deterministic ordering tests，并在 diagnostics 中保留参与 merge 的 source ids。
- [Risk] 现有 `typed-fields` macro 与新 projection 模型不匹配 → Mitigation: 首批实现只提供 builder API；derive macro 由后续独立 change 基于稳定 metadata 增量设计。
- [Risk] hard cutover 放大 Docnav 参数解析回归影响 → Mitigation: 切换前用等价测试覆盖 common navigation fields、outline mode config、adapter native options、unknown config keys 和 invalid typed values；验证未通过时不得删除旧 resolver，也不得将双路径作为完成状态。

## Implementation Plan

1. 在当前 workspace 中新增通用 crate 边界和最小 public API，不先删除现有 Docnav resolver。
2. 把 `parameter-resolution` 的 fixed source slots 替换为动态 source collection，保持 Docnav direct/project/user/default 行为。
3. 增加 CLI/env/config/default/custom source extraction 的最小实现和测试。
4. 增加 merge strategy 与 provenance trace 测试，覆盖 scalar、list、map、conflict、missing required 和 invalid source value。
5. 将 Docnav navigation input resolution 调用链 hard cutover 到新 resolver，并运行现有 Docnav 参数解析、配置、adapter native option 和 workspace 验证。
6. 完成 Docnav hard cutover validation 与工作区验证后，在 8.x release-readiness 决策门中确认独立 repository 迁移和外部 crate 发布边界。
7. 删除旧 fixed source resolver 的运行时路径；验证失败时停止并修复新 resolver，不保留运行时 fallback 作为完成状态。

## Decision Triggers

无需要用户立即决策的开放问题。

已收敛：

- `cli-config-resolution` 是 capability ID，也是工作区 crate/package 工作名；发现命名、owner 和 public contract 风险时按 Decision 8 更新。
- Docnav 集成采用 hard cutover；实现完成状态不得保留旧 resolver 运行路径、runtime feature flag、fallback 开关和兼容 wrapper。
- 外部 package 名默认沿用 `cli-config-resolution`，子仓库化默认迁移到独立 repository；包名不可用、仓库策略冲突和发布渠道风险需要用户决策。
- derive macro 不进入首批实现；framework integrations 使用 companion crates。
