本 design 记录标准参数来源解析核心的设计取向。

## Context

标准参数和 typed-field 的边界需要拆开。Typed fields 描述字段、extraction strategy 与基础校验；standard parameter resolution 描述 source construction、配置 source loading、合并顺序、operation binding、typed runtime values 和 passthrough handoff。

`docs/standard-parameters.md` 是长期行为 owner：标准参数层在返回 typed values、source info、diagnostics 和 passthrough 后结束。Core CLI、adapter SDK、protocol request construction 和 readable output 只消费结果，不接管标准参数来源规则。

代码侧当前状态：

- `docnav-typed-fields` 已表达字段 identity、extraction strategy、schema metadata、默认值和 typed value validation。
- `docnav-standard-parameters` 已覆盖手工 sources 合并。
- 本 change 仍需补齐按 strategy/registration 生成 sources、配置 source loading 和 skipped-source diagnostic data。

## Goals / Non-Goals

**Goals:**

- 建立标准参数来源模型：direct input、project config、user config 和 default。
- 建立 source construction API：按 registration 和 typed-field extraction strategy 把 direct input、project config、user config 和 default 映射为标准参数 sources。
- 建立 config source loading API：由调用方提供配置路径和入口上下文，标准参数层完成 JSON 读取、顶层 object 校验、source skipped diagnostic data 和 source object 构造。
- 按 `direct input > project config > user config > default` 形成最终 typed values，并保留每个最终值的 source info。
- 让解析结果包含 typed values、source info、diagnostics 和 passthrough handoff。
- 让 operation argument binding 成为标准参数 identity 到 protocol arguments path 的映射。
- 为后续 core/SDK 迁移提供小而稳定的实现目标。

**Non-Goals:**

- Consumer migration：core CLI、adapter SDK direct CLI、adapter `invoke` 和现有 config command behavior 留给后续 change。
- CLI frontend：本 change 不选择或替换 CLI parser。
- Non-standard-parameter JSON：manifest、probe、protocol response 等 JSON contract 留给各自 owner。
- Observable contract：public schema、examples、readable/raw output、warning 文案、stable error code 和 protocol envelope 保持当前 owner。
- Entry-specific policy：unknown argv tokenization、ignored-argv warning 承载、native option semantic validation、exit code 和 stderr/stdout placement 留给入口 owner。

## Decisions

1. Standard parameter resolution 只消费 typed-field facts，不重新定义字段事实。
   - Rationale: 字段约束由 typed-field 拥有，来源合并由标准参数拥有。
   - Alternative: 在标准参数里重复定义字段约束。暂不采用，因为会形成两套事实源。

2. typed-field builder 拥有 extraction strategy declaration。
   - Decision: `FieldDefBuilder` 只通过 `extract(strategy_id, strategy)` 注册 extraction strategy；不保留旧 `.path(...)` 兼容入口。同一 `FieldDefSet` 内相同 strategy id 必须对应同一种 input kind。JSON path 是一种 strategy，Rust field projection 作为独立 input kind 建模。
   - Rationale: config path、invoke arguments path 和 direct-input projection 都属于同一个字段事实源，不能散落到标准参数 consumer 或入口 glue。
   - Alternative: 由标准参数或各入口在调用点 bind source projection。暂不采用，因为会让配置路径和输入映射重复声明。

3. Resolver 的最小输入/输出边界以 source construction 和 merge 为中心。
   - Decision: 输入是已注册的标准参数 metadata、source construction inputs、配置 source descriptors、default source provider 和 entry passthrough policy；中间产物是 source objects；输出是 typed values、source info、diagnostics 和 passthrough handoff。
   - Rationale: 这样后续 consumer 可以分批接入同一个 resolver，同时避免 consumer 继续手写配置字段投影和来源合并。
   - Alternative: 直接把 resolver 做成 core CLI 参数解析器。暂不采用，因为会把 consumer migration 和解析核心绑死。

4. 来源优先级固定为 direct input、project config、user config、default。
   - Rationale: 该顺序已由主规范声明，resolver core 需要把它变成单一实现规则。
   - Alternative: 让每个入口自定义优先级。暂不采用，因为会让同一标准参数在不同入口中语义漂移。

5. 未映射输入按入口策略保留、丢弃或交给 owner validation。
   - Rationale: 标准参数层只校验已映射字段，避免把 adapter native option 或未来扩展字段提前判死。
   - Alternative: 统一拒绝所有未知字段。暂不采用，因为会破坏现有 loose CLI 兼容策略。

6. operation argument binding 只描述 identity 到 protocol arguments path 的映射。
   - Rationale: 跨 protocol 序列化发生在入口完成解析之后，配置值和默认值不能仅因为 request construction 被重新分类为 adapter direct input。
   - Alternative: 把所有最终标准参数值都写入 request arguments。暂不采用，因为会改变 adapter invoke 的来源语义，并扩大 observable behavior。

7. Config source loading 返回 structured diagnostic data，不直接写 warning。
   - Rationale: 标准参数层拥有“source 是否可参与合并”的机械判断；warning envelope、输出通道和 exit behavior 仍由 `docnav-diagnostics`、core 或 SDK owner 映射。
   - Alternative: 在标准参数 crate 直接依赖输出/warning 层。暂不采用，因为会反向扩大共享层职责。

## Risks / Trade-offs

- [Risk] implementation scope 扩张到 consumer migration → Mitigation: tasks 先决定 resolver crate/module 和最小可见性，request construction、CLI frontend 和 output behavior 保持在后续 owner。
- [Risk] 只实现 source merge，consumer 仍手写 source construction → Mitigation: 本 change 把 source construction、配置读取和 unmapped passthrough collection 纳入未完成 tasks。
- [Risk] passthrough policy 与 owner validation 边界不清 → Mitigation: spec 只规定交接，不规定各 owner 的 native semantics。
- [Risk] source info 未来可能进入 debug 或 context output → Mitigation: 本 change 只保留内部 attribution；任何可观察输出必须由后续独立 change 同步 docs、examples 和 tests。

## Open Questions

- source info 是否进入后续 context/debug surface；当前 change 不产生可观察输出。
- `FieldDefSet` 是否需要暴露 strategy-specific metadata projection，还是由 `docnav-standard-parameters` 组合现有 schema metadata 与 extraction strategy data。
