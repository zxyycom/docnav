本 design 只记录标准参数 consumer 迁移的高层设计取向；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## Context

core 和 adapter SDK 都会消费标准参数，但它们拥有不同入口边界。core 拥有 routing、adapter management、shared CLI defaults 和 output mapping；adapter SDK 拥有 adapter direct CLI、invoke I/O 和 adapter-side operation dispatch。

## Goals / Non-Goals

**Goals:**

- 让 core CLI 与 adapter SDK 使用共享 registration 和 typed runtime values。
- 保留 direct CLI loose warning 策略和 consumed field strict validation。
- 保留 adapter invoke stdin JSON 的严格 protocol boundary。
- 让 help/default 文案从 metadata 获取，避免手写漂移。

**Non-Goals:**

- 不在本 change 替换 `clap`。
- 不改变 protocol envelope、readable output wrapper 或 adapter-owned ref。
- 不把 adapter native options 提升为 core-owned 标准参数。
- 不实现 typed JSON contract validation。

## Decisions

1. core 和 adapter SDK 分阶段迁移，但放在同一 adoption change 里审计。
   - Rationale: 两者共享标准参数语义，需要一起验证跨入口一致性。
   - Alternative: 拆成两个 change。审计若认为范围仍过大，可在实现前拆分。

2. adapter invoke 作为独立入口重新解析 request arguments。
   - Rationale: core 解析出的配置值或默认值不应被重新标记为 adapter invoke direct input。

3. warning 与 strict validation 的边界保持现状。
   - Rationale: 这是用户明确要求保留的 CLI 行为，也是迁移是否安全的关键验收点。

## Risks / Trade-offs

- [Risk] 同时迁移 core 和 SDK 过大 → Mitigation: 审计允许实现前拆成 core-only 与 SDK-only。
- [Risk] help 文案变化造成 golden drift → Mitigation: 迁移任务必须包含 help/default 行为对照。
- [Risk] adapter native options 被标准参数层误校验 → Mitigation: passthrough 和 owner validation 必须有测试覆盖。

## Open Questions

- 首轮迁移是否只覆盖 shared defaults，再扩展到全部 document operation 参数。
- context/debug 输出是否应暴露 source info。
