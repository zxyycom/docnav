本 design 只记录 typed field definition 的高层设计取向；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## Context

当前讨论希望把标准参数方案拆小，并让非标准参数 JSON 字段也能复用同一套 typed path/value 约束。typed field definition 是底层候选能力，不拥有 CLI argv、配置来源合并、operation binding、manifest/probe 语义或完整 schema 文件生成。

## Goals / Non-Goals

**Goals:**

- 定义可复用的 typed key、JSON path 和 value constraint metadata。
- 让上层 consumer 可以复用 decoder/validator 和 error attribution。
- 第一版输出 schema metadata，供后续 schema generator、docs 或 fixture tooling 消费。
- 保留足够少的约束，避免提前限制标准参数、manifest/probe 和 protocol 的未来形态。

**Non-Goals:**

- 不直接生成完整 JSON Schema 文件。
- 不拥有标准参数来源优先级、默认值合并、passthrough 或 warning 行为。
- 不迁移任何当前 runtime validation。
- 不替代现有 public schema、examples 或 protocol owner。

## Decisions

1. typed-field 是底层 field engine，不命名为 standard parameter。
   - Rationale: 标准参数只是 consumer 之一；manifest/probe/protocol JSON 也会复用同一原理。
   - Alternative: 继续扩展标准参数 change。暂不采用，因为会扩大 owner 并增加实现风险。

2. 第一版输出 schema metadata，不直接拥有 schema 文件生成。
   - Rationale: metadata 能先统一事实源，同时不绑定具体 schema 文件布局和 generator 策略。
   - Alternative: 直接生成 JSON Schema。暂不采用，避免在审计前锁定 schema pipeline。

3. definition fingerprint 只作为一致性检查方向记录。
   - Rationale: 需要防止同名字段在不同 consumer 中语义漂移。
   - Alternative: 首版不做一致性检查。风险是后续迁移时更难发现重复定义。

## Risks / Trade-offs

- [Risk] 抽象过早导致字段模型不适配具体 consumer → Mitigation: 审计门禁要求只保留 path/value/metadata 层，不上收来源合并或业务语义。
- [Risk] schema metadata 与现有 schema 文件漂移 → Mitigation: 后续 JSON validation change 必须补一致性测试或明确保留人工 schema owner。
- [Risk] 与旧 `unify-standard-parameter-definitions` 范围重叠 → Mitigation: 本 change 不修改旧 change，审计时再决定迁移或废弃路径。

## Open Questions

- typed-field 最终应落在哪个共享 crate，还是先作为标准参数 crate 的内部模块起步。
- schema metadata 的最小字段集合是否只覆盖当前 schema keyword，还是允许后续扩展。
