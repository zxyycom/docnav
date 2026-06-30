本 design 说明 `adopt-strict-input-boundaries` 如何确立 Docnav 严格公共输入边界，并通过高质量诊断保持 AI 可修复性。

## Context

Docnav 需要把公共输入问题集中到 owner 边界处理：direct CLI 识别 unknown argv、extra positional 和 operation-inapplicable known flag；adapter routing 区分显式 adapter intent 与内部 discovery；配置来源区分 absence 与 invalid state；readable 输出和 schema 按成功 payload 或 failure diagnostic 分别投影。

这些策略原本服务 AI 调用体验：尽量让一次命令在核心语义足够时成功。新的目标是把 AI 友好性放到可修复失败上，让 parser、standard parameter resolution、adapter SDK、diagnostics、output、schema 和测试围绕同一条 strict contract 收敛。

本 change 将 AI 友好性落实为错误质量：invalid caller input 在 owner 边界失败，失败面携带一个 primary `DiagnosticRecord`，包含稳定字段名、owner、位置、收到值、期望范围、修复建议和必要 details。自动 discovery 全部失败时返回候选失败列表。协议、readable 输出和 adapter process boundary 保持分层；变化集中在 public input boundary、diagnostic model 和 output projection。

实现按 owner 并行推进。Change 文档已经统一 strict contract 语言；owner 文档、active OpenSpec 协调、schema/examples、代码切片和测试切片可以在明确依赖下并行推进，并在最终集成阶段合并验证。并行 worker 的输出必须足够合并：改动文件、验证命令、阻塞点和跨轨道协调项都要显式报告。

## Goals / Non-Goals

**Goals:**

- 将 core CLI、adapter direct CLI、adapter `invoke`、配置文件、显式 adapter/ref/path/operation 参数等公共输入边界改为 strict-by-default。
- 让成功路径只表达成功业务结果；caller input diagnostics、config source failures 和 adapter discovery failure list 都通过 failure diagnostic 或内部流程处理。
- 允许内部自动 discovery 在没有 caller 显式声明候选时继续尝试；成功输出只表达被选中 adapter 的成功结果。
- 把 AI 修复能力落到一个 primary `DiagnosticRecord`：稳定字段名、明确 owner、输入位置、收到值、期望形状、修复建议和从属失败列表。
- 保留 `clap` 作为 strict direct CLI parser/help 的默认依赖；strict parser/mapper 负责把 invalid argv 转成 input diagnostic。
- 将 adapter-owned `options` 和 native options 建模为明确 owner 的输入源；未归属输入在 owner 边界失败。
- 同步 owner docs、schema/example/fixture/testing，使 strict failure 成为可验证契约。

**Scope Boundaries:**

- Document operation 集合由 operation owner change 管理。
- Ref 生成和解释继续由 adapter owner 管理。
- 格式私有导航语义继续由格式 adapter owner 管理。
- Quality scan、verifier status 和 tooling report 语义继续由 tooling/testing owner 管理。
- Markdown `doc:full` 等格式导航 fallback 继续由 Markdown adapter owner 管理。
- Proposal、design、spec deltas 和 tasks 已作为实现输入；实现代码按轨道推进。
- 当前二进制能力仍按 `docs/navigation.md` 的状态语义判断。

## Decisions

### Decision 1: Public input boundaries are strict by default

调用者显式传入的 argv、protocol/request fields、config declarations、adapter id、path、ref、query、page、limit、output 和 native options 必须按 owner contract 校验；invalid caller input 通过 failure-diagnostic proof 验证。

### Decision 2: Failures use one primary DiagnosticRecord

AI 友好性通过诊断质量实现。公共失败面使用一个 primary `DiagnosticRecord`，由 diagnostics owner 统一字段名和投影规则。最小结构为：

```json
{
  "code": "<stable machine code>",
  "message": "<short human-readable summary>",
  "owner": "<owning boundary or stage>",
  "location": "<input location object when the owner can identify it>",
  "received": "<received value or token when safe to expose>",
  "expected": "<expected shape or accepted values when available>",
  "guidance": ["<actionable repair step>"],
  "details": {
    "field_issues": [],
    "config_issues": [],
    "typed_validation_failures": [],
    "candidate_failures": []
  }
}
```

`code`、`message` 和 `owner` 是公共失败的必备字段。invalid caller input 诊断必须在可定位时提供 `location`，在可枚举或可描述时提供 `expected`，并提供至少一个可执行 `guidance`。`details` 只包含当前失败需要的从属列表键；字段问题列表、config 问题列表、typed validation failure list 和候选失败列表只能作为 `details` 下的从属结构化数据。

### Decision 3: Automatic discovery can continue internally

没有 caller-declared adapter id 时，adapter discovery 可以继续尝试多个候选；成功时输出被选中 adapter 的成功结果；全部失败时返回候选失败列表。显式 `--adapter` 或 config-provided adapter 失败时返回 adapter selection diagnostic。

### Decision 4: Config strictness distinguishes absence from invalid state

默认配置路径不存在表示 absence，可以静默跳过；显式 override 缺失、不可读或非法返回 config diagnostic；默认配置文件一旦存在但无效也返回 config diagnostic。未知 config 字段按 config input failure 处理。`options` 是 adapter-owned native options 输入源；只有 owner 已声明 option source/key 且拥有该 key 校验时才能进入后续处理。

### Decision 5: Existing in-progress changes must be reconciled before implementation

`replace-clap-with-bpaf-frontend`、`separate-entry-pipeline-from-parameter-resolution`、`implement-docnav-mcp-bridge`、`outline-unstructured-full-read`、`enable-local-core-adapter-service-mode` 和 `markdown-document-head-outline-mode` 需要先与 strict input、primary `DiagnosticRecord`、owner-scoped native options、internal-event ownership 和成功 payload 投影对齐。Track A 负责扫描其它 active changes；凡是触及 diagnostic、protocol/readable output、config、native options、adapter selection 或 CLI parser/help 的 change，都要进入 Track A 协调清单。实现轨道进入对应代码前，先通过 Track A 更新 active changes，使 OpenSpec 验收标准一致。

### Decision 6: Strict CLI keeps clap as parser/help owner

strict 模式下 direct CLI 保留 `clap` 作为 parser/help owner 的实现依赖。unknown flags、extra positional、missing value 和 invalid value 由 strict parser 或入口校验转换为统一 input diagnostic。`docnav-cli-args` 的后续职责收缩为 strict parser/mapper 需要的共享能力。

### Decision 7: Successful document output uses success payload only

document success output 只包含成功业务 payload 和该 output mode 拥有的结构。未来如果某个 operation 需要非致命说明，由对应 operation/output owner 明确建模为业务字段、hint 或 failure guidance。

## Implementation Handoff Model

实现入口使用 `tasks.md` 的轨道拆分。各轨道的 owner 负责完整更新自己的长期 owner 文档、spec delta、验证材料或代码切片；其他文件保留摘要、触发条件或引用，核心契约由 owner 位置解释。

每个并行 worker 交付时必须报告：

```json
{
  "track": "<track id>",
  "changed": ["<file paths>"],
  "validated": ["<commands or checks>"],
  "blocked": ["<blockers or empty>"],
  "coordination_items": ["<cross-track merge notes or empty>"]
}
```

主集成只接受能满足该输出格式的轨道结果。涉及 schema、examples、fixtures、tests 或 public output shape 的轨道，必须说明其字段决策来自哪个 owner doc 或 spec delta。

## Risks / Trade-offs

- 风险：smoke/unit tests 会出现大量契约变更。处理方式：先完成测试契约更新，把相关 case 改为 strict failure proof，再改实现。
- 风险：AI 调用会进入“一次失败再修正”的交互。处理方式：诊断保持短、结构化、稳定，并包含修复建议或可接受参数列表。
- 风险：protocol-json/readable-json/readable-view 的失败投影不一致。处理方式：output owner docs、schema、examples 和 fixtures 在同一实现任务中更新。
- 风险：active changes 的验收标准和本 change 不一致。处理方式：Track A 在相关实现切片前完成 active change 协调。
- 风险：adapter-owned options 的边界过窄。处理方式：`options` 作为 adapter-owned native options source；owner 负责声明、验证或拒绝。
- 风险：内部调试需要多条事件线索。处理方式：public failure contract 使用 single primary `DiagnosticRecord`；内部日志或 tracing 由实现 owner 按需设计。

## Migration Plan

1. 实现启动：使用 proposal、design、spec deltas 和 tasks 作为实现输入，按 owner 轨道分派 docs、schema、代码和测试执行。
2. 并行轨道 A：协调 active OpenSpec changes，使相邻 change 的验收标准与 strict input、primary `DiagnosticRecord`、owner-scoped native options 和 clap parser/help 决策一致。
3. 并行轨道 B：更新 owner docs，按 `docs/navigation.md` 的 owner 表拆成 architecture/CLI、standard parameters/adapter/protocol、diagnostics/output、testing/navigation/Markdown。
4. 并行轨道 C：更新 schema/examples/fixtures，表达成功 payload shape 和 failure primary `DiagnosticRecord` projection。
5. 并行轨道 D：实现 crate 切片，按 core CLI/adapter selection、standard parameters/config/native options、diagnostics/output、adapter SDK direct CLI/invoke 分工。
6. 并行轨道 E：迁移测试切片，按 unit/smoke/schema-example/case ledger 分工。
7. 集成验证：合并轨道结果，确认 valid input 成功、invalid public input 失败、success output shape 和 primary `DiagnosticRecord` 投影一致，再运行与范围匹配的 workspace 验证，优先 `bun run verify:docnav-workspace`。

## Open Questions

无未回答开放问题。Change 已准备进入实现阶段；active changes 的协调工作由 Track A 在相关实现切片前完成。
