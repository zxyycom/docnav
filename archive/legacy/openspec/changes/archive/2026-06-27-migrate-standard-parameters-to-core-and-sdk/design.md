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

## Audit Results

1. 本 change 不替换 CLI frontend。
   - Core 和 adapter direct CLI 继续使用当前 `clap`/loose argv front-end；本 change 只迁移 document operation 参数的 registration、metadata、来源合并和 typed runtime value 消费。

2. 本 change 不处理 manifest、probe 或 protocol response typed validation。
   - Manifest/probe 命令保持现有 machine command 边界；adapter `invoke` 只在 request `arguments` 的标准参数消费处接入 resolver，不改变 request envelope decode 或 response envelope validation。

3. core 与 adapter SDK 仍适合放在同一 adoption change。
   - 两者消费同一标准参数 resolver，且本轮范围限制在 document operation 参数、help/default metadata 和 request construction；adapter native options 仍通过 passthrough 交给 adapter-owned validation，不提升为 core-owned 参数。

4. 兼容行为必须保留。
   - unknown argv、extra positional 和当前 operation 不使用的 known value flag 继续产生 warning 并允许 operation 继续；当前 operation 实际消费的字段通过 standard parameter result 严格校验。

## Risks / Trade-offs

- [Risk] 同时迁移 core 和 SDK 过大 → Mitigation: 审计允许实现前拆成 core-only 与 SDK-only。
- [Risk] help 文案变化造成 golden drift → Mitigation: 迁移任务必须包含 help/default 行为对照。
- [Risk] adapter native options 被标准参数层误校验 → Mitigation: passthrough 和 owner validation 必须有测试覆盖。

## Open Questions

- Resolved: 首轮覆盖当前 document operation 已消费的标准参数。Core 覆盖 path/ref/query/page/limit_chars/output/adapter；adapter SDK 覆盖 path/ref/query/page/limit_chars/output，并通过 passthrough 保留 native options 的 adapter-owned validation。
- Out of scope: context/debug 输出暂不新增 source info；source info 只作为 request construction、默认值来源和测试证明材料消费。
