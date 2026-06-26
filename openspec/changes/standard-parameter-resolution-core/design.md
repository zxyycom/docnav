本 design 只记录标准参数解析核心的高层设计取向；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## Context

标准参数和 typed-field 的边界需要拆开。typed-field 描述字段与基础校验，standard parameter resolution 描述入口来源、合并顺序、operation binding、typed runtime values 和 passthrough handoff。

本轮基础审计把 `docs/standard-parameters.md` 作为长期行为 owner：标准参数层在返回 typed values、source info、diagnostics 和 passthrough 后结束，后续 core CLI、adapter SDK、protocol request construction 或 readable output 由各自 owner 消费。代码侧现状是 `docnav-typed-fields` 已能表达字段 identity、extraction strategy、schema metadata、默认值和 typed value validation；resolver core 应围绕来源建模和合并结果补齐缺口。

## Goals / Non-Goals

**Goals:**

- 建立标准参数来源模型：direct input、project config、user config 和 default。
- 按 `direct input > project config > user config > default` 形成最终 typed values，并保留每个最终值的 source info。
- 让解析结果包含 typed values、source info、diagnostics 和 passthrough handoff。
- 让 operation argument binding 成为标准参数 identity 到 protocol arguments path 的映射。
- 为后续 core/SDK 迁移提供小而稳定的实现目标。

**Non-Goals:**

- Consumer migration：core CLI、adapter SDK、adapter direct CLI 和现有 config command behavior 留给后续 change。
- CLI frontend：本 change 不选择或替换 CLI parser。
- Non-standard-parameter JSON：manifest、probe、protocol response 等 JSON contract 留给各自 owner。
- Observable contract：public schema、examples、readable/raw output、warning 文案、stable error code 和 protocol envelope 保持当前 owner。

## Decisions

1. standard parameter resolution 只消费 typed-field metadata。
   - Rationale: 字段约束由 typed-field 拥有，来源合并由标准参数拥有。
   - Alternative: 在标准参数里重复定义字段约束。暂不采用，因为会形成两套事实源。

2. typed-field builder 拥有 extraction strategy declaration。
   - Decision: `FieldDefBuilder` 只通过 `extract(strategy_id, strategy)` 注册 extraction strategy；不保留旧 `.path(...)` 兼容入口。同一 `FieldDefSet` 内相同 strategy id 必须对应同一种 input kind。JSON path 是一种 strategy，Rust field projection 作为独立 input kind 建模。
   - Rationale: config path、invoke arguments path 和 direct-input projection 都属于同一个字段事实源，不能散落到标准参数 consumer 或入口 glue。
   - Alternative: 由标准参数或各入口在调用点 bind source projection。暂不采用，因为会让配置路径和输入映射重复声明。

3. resolver 的最小输入/输出边界以 source merge 为中心。
   - Decision: 输入是已注册的标准参数 metadata、按入口映射形成的 source objects、default source 和 entry passthrough policy；输出是 typed values、source info、diagnostics 和 passthrough handoff。
   - Rationale: 这样后续 consumer 可以分批接入同一个 resolver，当前 change 仍聚焦来源解析核心。
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

## Risks / Trade-offs

- [Risk] implementation scope 扩张到 consumer migration → Mitigation: tasks 先决定 resolver crate/module 和最小可见性，request construction、CLI frontend 和 output behavior 保持在后续 owner。
- [Risk] passthrough policy 与 owner validation 边界不清 → Mitigation: spec 只规定交接，不规定各 owner 的 native semantics。
- [Risk] source info 未来可能进入 debug 或 context output → Mitigation: 本 change 只保留内部 attribution；任何可观察输出必须由后续独立 change 同步 docs、examples 和 tests。

## Open Questions

- 标准参数解析核心的最小 Rust API 应放在现有共享 crate 内，还是新增窄边界 crate。
- source info 是否进入后续 context/debug surface；当前 change 不产生可观察输出。
