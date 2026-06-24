本 design 只记录 lexopt frontend 的高层设计取向；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## Context

用户已明确 `lexopt` 是目标。Docnav 需要一个薄的 argv frontend，让标准参数层拥有参数语义，并让 core CLI 与 adapter direct CLI 共享 warning 和 help metadata 策略。

## Goals / Non-Goals

**Goals:**

- 使用 `lexopt` 作为 direct CLI argv frontend。
- frontend 只输出 token classification、positionals、raw flag values 和 frontend diagnostics。
- 参数语义、required/default/range/enum、operation applicability 和 source merge 由标准参数流程负责。
- help 由 command context、standard parameter metadata 和 adapter native option metadata 组合生成。

**Non-Goals:**

- 不改变 adapter invoke stdin JSON。
- 不改变 standard parameter resolver 的内部模型。
- 不把 `lexopt` 写成长期 public contract；长期 contract 是 CLI 行为。
- 不在本 change 处理 service mode。

## Decisions

1. `lexopt` 是目标实现依赖，但不是 public contract。
   - Rationale: 用户希望切换到 lexopt；文档应记录实现方向，同时避免把 crate 名变成长期兼容承诺。

2. help generation 不由 frontend 独占。
   - Rationale: help 需要展示标准参数 defaults、possible values 和 adapter native options，单靠 argv parser 不能拥有这些语义。

3. unused known flag 不 eager validate。
   - Rationale: 当前 operation 没消费的 known flag 只产生 warning，实际消费的字段才严格校验。

## Risks / Trade-offs

- [Risk] 切换 frontend 改变 help 排序或文案 → Mitigation: 审计要求先列出现有 help contract，再用 golden/smoke 对照。
- [Risk] frontend 泄漏业务语义 → Mitigation: 任务只允许 tokenization/frontend mapping。
- [Risk] adapter native options help 缺失 → Mitigation: help generation 必须合并 owner metadata。

## Open Questions

- `docnav-cli-args` 是保留并改用 lexopt，还是被新的 thin frontend crate 替代。
- help output 的排序是否需要先单独固化为 contract。
