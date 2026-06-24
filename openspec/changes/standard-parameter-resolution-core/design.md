本 design 只记录标准参数解析核心的高层设计取向；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## Context

标准参数和 typed-field 的边界需要拆开。typed-field 描述字段与基础校验，standard parameter resolution 描述入口来源、合并顺序、operation binding、typed runtime values 和 passthrough。

## Goals / Non-Goals

**Goals:**

- 建立标准参数来源模型：direct input、project config、user config 和 default。
- 让解析结果包含 typed values、source info、diagnostics 和 passthrough。
- 让 operation argument binding 成为标准参数 identity 到 protocol arguments path 的映射。
- 为后续 core/SDK 迁移提供小而稳定的实现目标。

**Non-Goals:**

- 不迁移 core CLI 或 adapter SDK。
- 不替换 `clap`。
- 不处理 manifest/probe/protocol response 这类非标准参数 JSON。
- 不改变现有 public schema 或 examples。

## Decisions

1. standard parameter resolution 只消费 typed-field metadata。
   - Rationale: 字段约束由 typed-field 拥有，来源合并由标准参数拥有。
   - Alternative: 在标准参数里重复定义字段约束。暂不采用，因为会形成两套事实源。

2. 未映射输入按入口策略保留、丢弃或交给 owner validation。
   - Rationale: 标准参数层只校验已映射字段，避免把 adapter native option 或未来扩展字段提前判死。
   - Alternative: 统一拒绝所有未知字段。暂不采用，因为会破坏现有 loose CLI 兼容策略。

3. 本 change 不迁移 consumer。
   - Rationale: 解析核心需要先被审计，再让 core/SDK 分批接入。

## Risks / Trade-offs

- [Risk] resolution core 仍然过大 → Mitigation: 审计门禁要求剔除 CLI frontend 和具体 consumer 迁移。
- [Risk] passthrough policy 与 owner validation 边界不清 → Mitigation: spec 只规定交接，不规定各 owner 的 native semantics。
- [Risk] 与旧 active change 重叠 → Mitigation: 不修改旧 change，后续通过审计决定是否替代或迁移。

## Open Questions

- 标准参数解析核心的最小 public Rust API 是否应先保持 crate-private。
- source info 的可观察输出是否进入后续 context/debug surface。
