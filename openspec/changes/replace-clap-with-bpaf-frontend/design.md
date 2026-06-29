本 design 记录用 `bpaf` 替换 clap argv frontend 的实现边界、取舍和验证重点。

## Context

用户已明确倾向 `bpaf`：目标优先级是减少自研工作量、维持目标行为，并尽可能选择热度和活跃度更高的库。

Docnav 需要的是薄 argv frontend：它接收 argv、识别入口形态、收集 raw values 和 frontend diagnostics；标准参数流程继续拥有参数身份、默认值、required、range、enum、operation applicability、source merge 和 strict validation。

## Goals / Non-Goals

**Goals:**

- 使用 `bpaf` 替换 direct CLI 路径上的 clap argv parsing 和 help surface。
- frontend 只输出 command/subcommand、positionals、raw flag values、help request 和 frontend diagnostics。
- help 复用 `bpaf` 能力时，仍从 command context、standard parameter metadata 和 adapter native option metadata 生成。
- core CLI 与 adapter direct CLI 共享同一套 loose warning 和 metadata-driven help 策略。

**Non-Goals:**

- 不改变 adapter invoke stdin JSON。
- 不改变 standard parameter resolver 的内部模型。
- 不把 `bpaf` 写成长期 public contract；长期 contract 是 CLI 行为。
- 不让 `bpaf` 的 typed parsing、fallback/default 或 required 语义成为 Docnav 参数语义 owner。
- 不在本 change 处理 service mode。

## Approach

1. Dependency boundary
   - `bpaf` 是 direct CLI frontend 的实现依赖，不进入 public contract、protocol schema 或 adapter contract。
   - 对外可观察行为用 CLI warning/help/validation contract 描述。

2. Frontend output
   - frontend 输出 argv classification 结果，而不是最终 operation request。
   - 输出包含 command path、positionals、raw flag values、unknown/extra/unused diagnostics 和 help request。

3. Parameter ownership
   - 标准参数流程负责把 raw argv input 映射到 parameter identity，并执行 defaults、merge、required、range、enum 和 strict validation。
   - owning native option handler 负责 adapter native option 的业务语义。
   - frontend 可使用 operation metadata 判断 flag 是否被当前 operation 消费，但不校验该 flag 的 typed value。

4. Help ownership
   - help 可以通过 `bpaf` 降低 usage/rendering 工作量。
   - help 内容的 owner 是 command context、standard parameter metadata 和 adapter native option metadata。
   - help generation 不读取 config，不执行 adapter operation。

## Decisions

1. `bpaf` 是当前目标实现依赖，但不是 public contract。
   - Rationale: 用户倾向选择热度和活跃度较高、能直接支持 help 的库，以减少自研工作量；长期兼容承诺仍是 CLI 行为。

2. help generation 可以复用 `bpaf` 能力，但不由 frontend 独占。
   - Rationale: help 需要展示标准参数 defaults、possible values 和 adapter native options；`bpaf` 可以降低渲染和 usage 维护成本，但不能成为这些语义的 owner。

3. unused known flag 不 eager validate。
   - Rationale: 当前 operation 没消费的 known flag 只产生 warning，实际消费的字段才严格校验。

## Risks / Trade-offs

- [Risk] 切换 frontend 改变 help 排序或文案 → Mitigation: 审计要求先列出现有 help contract，再用 golden/smoke 对照。
- [Risk] `bpaf` 的 typed parsing、fallback/default 或 required 语义泄漏到业务层 → Mitigation: integration 只允许使用 argv classification/frontend mapping 和 help surface；标准参数仍拥有默认值、required、range、enum 和 strict validation。
- [Risk] adapter native options help 缺失 → Mitigation: help generation 必须合并 owner metadata。
- [Risk] unknown/extra/unused argv 被 `bpaf` 转成 hard error → Mitigation: 审计和测试必须覆盖 loose warning 行为。

## Open Questions

- `docnav-cli-args` 是保留并改用 `bpaf`，还是被新的 thin frontend crate 替代。
- help output 的排序是否只做 smoke 稳定，还是先作为 contract 单独固化。
