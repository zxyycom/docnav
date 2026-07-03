本 design 记录 strict core CLI 下 retained `clap` argv frontend 的实现边界、取舍和验证重点。原 `bpaf` 替换方向已由 `adopt-strict-input-boundaries` Track A 协调为 inactive。

## Context

原提案倾向 `bpaf`，目标优先级是减少自研工作量、维持目标行为，并尽可能选择热度和活跃度更高的库。strict input boundary 协调后，active decision 改为保留 `clap` 的 parser/help owner 职责。

Docnav 需要的是薄 argv frontend：它接收 argv、识别入口形态、收集 raw values 和 frontend diagnostics；navigation input resolution 继续拥有参数身份、默认值、required、range、enum、operation applicability、source merge 和 strict validation。

## Goals / Non-Goals

**Goals:**

- 保留 `clap` 作为 core CLI strict parser/help owner。
- frontend 只输出 command/subcommand、positionals、raw flag values、help request 和 frontend diagnostics。
- help 通过 `clap` surface 暴露，但仍从 command context、navigation parameter metadata 和 adapter native option metadata 生成。
- core CLI 使用 strict input diagnostic 和 metadata-driven help 策略。

**Non-Goals:**

- 不改变 protocol request handling。
- 不改变 navigation input resolution 的内部模型。
- 不把 parser crate 名写成长期 public contract；长期 contract 是 CLI 行为。
- 不让 parser 的 typed parsing、fallback/default 或 required 语义成为 Docnav 参数语义 owner。
- 不在本 change 处理 service mode。

## Approach

1. Dependency boundary
   - `clap` 是 strict core CLI parser/help owner 的实现依赖，不进入 protocol schema 或 adapter contract。
   - 本 change 不引入 `bpaf` 替换 active parser/help surface；对外可观察行为用 CLI input diagnostic、help 和 validation contract 描述。

2. Frontend output
   - frontend 输出 argv classification 结果，而不是最终 operation request。
   - 输出包含 command path、positionals、raw flag values、unknown/extra/unused diagnostics 和 help request。

3. Parameter ownership
   - Navigation input resolution 负责把 raw argv input 映射到 parameter identity，并执行 defaults、merge、required、range、enum 和 strict validation。
   - owning native option handler 负责 adapter native option 的业务语义。
   - frontend 可使用 operation metadata 判断 flag 是否被当前 operation 消费，但不校验该 flag 的 typed value。

4. Help ownership
   - help 通过 retained `clap` surface 提供 usage/rendering。
   - help 内容的 owner 是 command context、navigation parameter metadata 和 adapter native option metadata。
   - help generation 不读取 config，不执行 adapter operation。

## Decisions

1. `clap` 是 strict core CLI 的 active parser/help owner，但不是 protocol 或 adapter contract。
   - Rationale: `adopt-strict-input-boundaries` 要求 core CLI 保留 `clap` parser/help 决策，并把 unknown argv、extra positional、missing value 和 invalid value 映射为统一 input diagnostic。

2. help generation 可以复用 retained `clap` surface，但不由 frontend 独占。
   - Rationale: help 需要展示 navigation parameter defaults、possible values 和 adapter native options；parser/help surface 可以降低渲染和 usage 维护成本，但不能成为这些语义的 owner。

3. operation-inapplicable input 不进入 adapter execution。
   - Rationale: strict direct CLI contract 要求 unknown argv、extra positional 和当前 operation 不适用的 flag 在入口边界投影为 primary input diagnostic；实际消费的字段继续严格校验。

## Risks / Trade-offs

- [Risk] retained `clap` surface 与 navigation parameter metadata 之间出现职责重复 → Mitigation: 审计要求列出现有 help/default/possible-values owner，并只把 parser/help surface 作为呈现入口。
- [Risk] parser 的 typed parsing、fallback/default 或 required 语义泄漏到业务层 → Mitigation: integration 只允许使用 argv classification/frontend mapping 和 help surface；navigation input resolution 仍拥有默认值、required、range、enum 和 strict validation。
- [Risk] adapter native options help 缺失 → Mitigation: help generation 必须合并 owner metadata。
- [Risk] unknown/extra/operation-inapplicable argv 绕过 primary diagnostic projection → Mitigation: 审计和测试必须覆盖 strict input diagnostic 行为。

## Open Questions

无未回答开放问题。Track A 协调结论：`docnav-cli-args` 保留为 `clap`-backed strict parser/mapper 边界；help ordering 先保持现有 `clap` surface 与 smoke/golden 稳定性，不在本 change 单独扩大为新 public contract。
