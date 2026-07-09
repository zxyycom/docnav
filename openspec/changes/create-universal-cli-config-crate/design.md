本 design 是 `create-universal-cli-config-crate` 的未审核技术方案：说明如何从 Docnav 当前实现抽象出可子仓库化复用的 Rust CLI/config resolution crate；当前文档只存在于 `openspec/changes/create-universal-cli-config-crate/`，不影响现有其它文档或主规范。

## Context

Docnav 当前已经有两个可抽象基础：`typed-fields` 拥有字段 identity、路径投影、类型、约束、默认值和校验事实；`parameter-resolution` 拥有 direct/project/user/default 来源优先级、默认值 fallback、diagnostic handoff 和 passthrough。

当前限制是这些能力还带有 Docnav 的角色命名和固定来源集合，CLI flag、env、config file、dynamic default、custom source 不能作为同一套 source/projection contract 被外部 Rust CLI 复用。Docnav 的 adapter、operation、protocol、readable output 和 diagnostic code identity 必须继续留在 Docnav owner 内，不能进入通用库核心。

## Goals / Non-Goals

**Goals:**

- 从现有 `typed-fields` 与 `parameter-resolution` 中抽象通用字段契约、来源投影、来源抽取、优先级合并、merge strategy、来源追踪和最终 typed materialization。
- 形成可在当前 workspace 中验证、后续可迁移/镜像为独立子仓库的 crate 边界。
- 保持 Docnav 当前 navigation input resolution 可通过兼容 adapter 或 wrapper 迁移到新库，并保持现有 public protocol、CLI 行为、adapter contract 和 readable/protocol output 不变。
- 支持 CLI flag、env var、config document、static/dynamic default 和 custom source 的统一来源模型。
- 为 explain/debug 输出保留 deterministic provenance trace，而不是只返回最终值。

**Non-Goals:**

- 不实现新的 Docnav 文档导航能力。
- 不改变 `outline -> ref -> read` 协议、request/response schema、adapter native option semantics 或 output modes。
- 不把 `clap`、TOML、JSON、Figment 等具体框架绑定进核心 crate；这些只能通过 feature 或 adapter 层进入。
- 不在未经审计前发布外部 crate 或改写现有主规范。

## Decisions

### Decision 1: Workspace-first 的通用 crate 边界

本 change 新增 `cli-config-resolution` capability，拥有通用 Rust CLI/config resolution 的长期契约。实现阶段先在当前 workspace 中建立独立 crate 边界，确保 tests、docs 和 Docnav compatibility 能复用当前验证链路；通过阻塞级审计后，再选择子仓库化方式，例如独立 repo、submodule 或 subtree。

替代方案是直接在外部新仓库从零开始实现。该方案会绕开 Docnav 已有测试和行为样本，容易遗漏 `typed-fields` 与 `parameter-resolution` 已经证明过的边界，因此不作为第一步。

### Decision 2: 字段事实与 source policy 分层

字段定义继续围绕 stable identity、value kind、constraint、default、projection path 和 validation failure facts。通用库不让字段核心拥有 CLI/config/public diagnostic 语义，而是在上层 resolution crate 中声明 source projection 和 merge behavior。

替代方案是把 CLI flag、env name、config key 全部塞进 field core。该方案短期简化 derive，但会破坏现有 `typed-fields` 的 owner 边界，使非 CLI 消费者也被迫依赖 CLI/config 概念。

### Decision 3: 将 fixed source slots 改为 ordered source collection

现有 `DirectInput / ProjectConfig / UserConfig / Default` 模型应泛化为 ordered `SourceSpec` / `SourceId` 集合。每个 source 保留 kind、priority、locator、explicitness 和 load/parse 状态；resolver 只根据 source policy 选择候选值，不硬编码 Docnav 的四层优先级。

替代方案是保留固定四槽并继续扩展 enum。该方案无法自然支持 env、workspace config、profile、runtime defaults 或外部项目自定义来源。

### Decision 4: resolution 返回最终值和 provenance trace

Resolver 必须返回 materialized typed values，同时保留 selected candidate、被覆盖 candidates、validation diagnostics、merge path 和 source locator。`explain()` 或等价 API 从 trace 派生，不从最终 struct 反推。

替代方案是只返回最终 struct。该方案会丢失用户最需要的 debug 信息，也无法稳定解释“为什么这个值生效”。

### Decision 5: merge strategy 属于 field-level resolution policy

每个字段声明可选择 merge strategy。MVP 至少支持 scalar replace、list append/replace、map merge/replace 和 deny-conflict；默认策略必须 deterministic，并可被测试直接断言。

替代方案是全局 source priority 覆盖所有字段。该方案无法表达大型 CLI 常见的 include list、plugin map、profile overlay 和 conflict-sensitive option。

### Decision 6: framework integrations 保持 adapter/feature 层

核心 crate 只定义 field contract、source value、resolver、trace 和 diagnostics。`clap` arg 生成/读取、env source、serde_json/TOML source、derive macro convenience 均作为 feature 或 companion crate 提供，并且不能反向污染核心 API。

替代方案是以 `clap` 为核心抽象。该方案会限制其它 CLI 框架或非 argv 调用入口复用，也不适合作为真正的底层库。

### Decision 7: Docnav 集成通过兼容 wrapper 渐进迁移

实现阶段应先让 Docnav 现有 navigation input resolution 通过 adapter/wrapper 使用新 resolver，同时保持当前 CLI、config、protocol 和 adapter behavior 不变。只有当 compatibility tests 证明等价后，才移除旧的固定 source resolver。

替代方案是一次性替换 Docnav 参数解析链路。该方案影响范围过大，容易把通用库抽象问题和 Docnav 行为回归混在一起。

## Risks / Trade-offs

- [Risk] 抽象过宽导致第一版 API 难以稳定 → Mitigation: MVP 只覆盖已由 Docnav 需求证明的 source/value/trace/merge 行为，外部扩展通过 custom source 与 feature adapter 补充。
- [Risk] 子仓库化过早导致验证链路丢失 → Mitigation: 先在 workspace 内完成 crate 边界和 Docnav compatibility，再执行 sub-repo/submodule/subtree 发布步骤。
- [Risk] provenance trace 设计复杂，拖慢 resolver 实现 → Mitigation: trace 先记录 selected/overridden/diagnostic/source locator 的最小闭包，human-readable explain 可以后置。
- [Risk] merge semantics 与 source priority 组合产生歧义 → Mitigation: 每个 merge strategy 必须有 deterministic ordering tests，并在 diagnostics 中保留参与 merge 的 source ids。
- [Risk] 现有 `typed-fields` macro 与新 projection 模型不匹配 → Mitigation: 先提供 builder API，再扩展 derive macro；macro 只生成显式可审查的 field/projection metadata。

## Migration Plan

1. 在当前 workspace 中新增通用 crate 边界和最小 public API，不先删除现有 Docnav resolver。
2. 把 `parameter-resolution` 的固定 source slots 迁移为动态 source collection 的兼容层，保持 Docnav direct/project/user/default 行为。
3. 增加 CLI/env/config/default/custom source adapter 的最小实现和测试。
4. 增加 merge strategy 与 provenance trace 测试，覆盖 scalar、list、map、conflict、missing required 和 invalid source value。
5. 让 Docnav navigation input resolution 通过兼容 wrapper 使用新 resolver，并运行现有 Docnav 参数解析、配置、adapter native option 和 workspace 验证。
6. 完成阻塞级审计后，确定子仓库化方式和外部 crate 发布边界。
7. 若迁移出现回归，保留旧 resolver 路径作为临时 fallback，直到 compatibility tests 证明新路径等价；fallback 必须标注移除条件。

## Open Questions

已收敛：最终 crate/package 名、子仓库化方式和发布节奏作为实现前审计项处理，不阻塞本 change artifact 生成。无未回答开放问题，可以进入实现前审计。
