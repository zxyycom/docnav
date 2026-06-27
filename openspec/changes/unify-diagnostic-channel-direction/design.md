本 design 记录强制迁移到统一 `DiagnosticStack` 的技术方向；当前内容只是 `openspec/changes/unify-diagnostic-channel-direction/` 下的未审核临时文档，本文件本身不立即修改主规范、schema、示例或实现行为。

## Context

当前实现已经有若干接近统一的基础：

- `docnav-protocol::StableError` 表达阻断执行的稳定错误，并决定 protocol/readable error 和 exit code 映射。
- `docnav-diagnostics::Warning` 表达稳定 warning envelope，并提供 readable JSON 注入和 stderr warning line formatter。
- `docnav-output` 集中文档操作的 `readable-view`、`readable-json` 和 `protocol-json` 输出分流。
- `docnav-standard-parameters` 已有 `StandardParameterDiagnostic`，但它还不是 `docnav-diagnostics::Warning` 或 `StableError`。
- `docnav-adapter-sdk` 仍存在直接 `emit_diagnostic(stderr, text)` 的旁路，例如 direct CLI input error、manifest/probe/schema 校验、invoke decode 诊断和 output write failure。

主约束是迁移完整性：所有项目内错误、warning、skipped condition、decode failure、validation failure 和直接 stderr 诊断都应进入 `docnav-diagnostics::DiagnosticStack`。`protocol-json`、adapter `invoke`、manifest、probe、readable output、`StableError`、warning envelope 和直接 stderr 行为需要随实现强制切换；相应主规范、schema、示例、fixtures 和 consumer tests 必须同步更新。

## Goals / Non-Goals

**Goals:**

- 定义一个方向：项目内所有 warning、recoverable diagnostic、skipped-condition diagnostic 和 fatal context 在进入输出层前都必须被同一 `DiagnosticStack` 携带。
- 保持“跳过错误点但不跳过错误信息”：可恢复事件不阻断执行，但必须保留到最终输出策略可见。
- 允许调用链在任意边界按 stack 分配的 `DiagnosticId` 查找事件、按 `DiagnosticMark` 或 event id 批量弹出后续事件，并在输出前按 LIFO 取回诊断。
- 明确本 change 是 breaking migration：实施时强制切换既有错误、warning、stderr 和 schema/output contract。

**Non-Goals:**

- 本 change 不要求在 OpenSpec 提案阶段立即修改 `protocol-response.schema.json`、manifest schema、probe schema 或 readable schema；实施阶段必须同步修改受影响验证材料。
- 本 change 不把 `DiagnosticId`、stack index、mark 或 LIFO 顺序暴露为 public protocol/readable contract。
- 本 change 不让 `DiagnosticStack` 判断 operation 是否失败；失败、继续和 exit behavior 由调用方或 surface owner 决定。
- 本 change 不改变 adapter-owned ref、format parsing、native option validation 或 document operation 业务语义。
- 本 change 不要求迁移不属于 Docnav runtime 或 public surface 的一次性开发脚本、测试 harness 和辅助工具 stderr 输出。

## Decisions

### Decision 1: 在 `docnav-diagnostics` 统一内部 `DiagnosticStack`

后续实现应在 `docnav-diagnostics` 中定义统一栈模型，例如 `DiagnosticEvent` 与 `DiagnosticStack`。事件至少需要表达 severity、diagnostic code、message/reason、effect、details 和 source/surface hint。栈负责在 push 时生成 opaque `DiagnosticId`，并按 LIFO 作为默认取回策略。

选择这个方案而不是继续维护 `StableError`、`Warning`、standard parameter diagnostic 和直接 stderr 的并行入口，是因为并行入口会让同一失败事实在多个 owner 中重复建模。`DiagnosticStack` 是新的内部事实源；既有 surface 输出必须迁移为从 stack 读取或投影。

### Decision 2: `DiagnosticId` 和 `DiagnosticMark` 是内部栈控制点

`DiagnosticId` 标识一次 push 后的具体事件，只能由 stack 分配。调用方可以保存该 id，并用它在同一 stack 生命周期内查找事件或弹出该事件之后的后续事件。`DiagnosticMark` 表示一个 stack checkpoint，用于阶段性执行：调用方在尝试某个候选、配置源或 decode 分支前创建 mark，失败后可以 `drain_after(mark)` 批量取回本阶段新增诊断。

栈生命周期不跨进程；每个 top-level command、adapter direct command 或 `invoke` request 拥有自己的 stack。栈默认按 LIFO 策略取回事件，`pop`、`drain_after(mark)` 和 `drain_after_event(id, include_anchor)` 都返回逆序结果；调用方需要正序输出时必须显式反转。分组、过滤和展示排序不由通道负责。

### Decision 3: 入栈记录事实，不直接决定失败

调用点发现问题时应优先创建 diagnostic event 并 push 到 stack。若剩余输入仍能形成有效 operation，调用点继续执行；若问题阻断执行，调用点仍先记录 fatal context，再返回或传播自己的失败结果。`DiagnosticStack` 只是存储和取回诊断数据的地方，不判断 operation 是否失败，不决定 exit code，也不负责分组或格式化。

### Decision 4: Diagnostic code 是错误机械 id，`StableError` 不再作为 owner

`DiagnosticStack` 不持有 `StableError`。`DiagnosticEvent.code` 是失败或 warning 的机械 id；实施时由 `docnav-diagnostics` 定义稳定 code enum，并作为 protocol/readable error code、warning id 和 stderr formatter 的事实源。

`StableError` 作为稳定错误 owner 应被移除或降级为迁移期适配层；错误归属于 diagnostic code。输出层只负责把 diagnostic code 和 details 投影到目标 surface，不再从 `StableError` 反向构造 diagnostic event。

### Decision 5: warnings 应迁移为 diagnostic event 的投影

现有 `Warning` 可以在迁移期作为可恢复 event 的 readable/protocol-stderr 投影继续存在，但事实源应是 `DiagnosticEvent.code`、effect 和 details。迁移完成后，调用方不应自行构造独立于 stack 的 `Warning`。

这比让每个 caller 自己构造 `Warning` 更好，因为 `StandardParameterDiagnostic`、adapter candidate failure、config source skipped 和 ignored argv 都可以通过同一 handoff 返回，最终由 surface owner 决定投影字段、输出通道和展示顺序。

### Decision 6: 直接 stderr 旁路强制替换为 stack push + flush

`docnav-adapter-sdk::emit_diagnostic` 不应作为长期公共诊断入口。迁移后调用方必须先把诊断压入 `DiagnosticStack`；现有 formatter 只能作为 surface projection 或 flush 实现复用。

这个迁移可以按入口分阶段完成：document operation 输出优先，其次 adapter direct machine commands，再处理 invoke decode 和 IO/write failure。每个阶段都需要同步更新并验证新的 surface shape。

### Decision 7: `diagnostic_only` 需要时再添加

当前迁移不补齐 `warning.effect = diagnostic_only`。如果该值只是 schema 预留且没有当前行为依据，应在强制迁移中从当前 readable schema 目标移除或避免使用；只有出现明确行为需求时，才以新的 change 补齐 enum、schema、examples 和 tests。

## Risks / Trade-offs

- [Risk] 统一 event model 变成过宽的“万能错误对象”。→ Mitigation: event 只承载诊断事实、机械 code、effect 和 details；失败判断、分组和格式化留给调用方或 surface owner。
- [Risk] breaking migration 同时触及多个 output surface。→ Mitigation: 实施任务必须同步更新 docs、schema、examples、fixtures 和 consumer tests，并用 workspace verification 覆盖跨 surface mapping。
- [Risk] 内部 LIFO 顺序不适合某些用户可见输出。→ Mitigation: stack 默认 LIFO；需要正序、分组或 surface-specific ordering 的调用方显式反转或自行组织。
- [Risk] `DiagnosticId` 被误认为跨进程或跨输出的 public ref。→ Mitigation: id 只在同一 stack 生命周期内有效；public protocol/readable output 不暴露 stack id、mark 或 index。
- [Risk] fatal error 和 recoverable warning 的语义混淆。→ Mitigation: event 必须有明确 severity/effect；是否失败由调用方或 surface owner 根据 operation outcome 和 diagnostic code 决定。
- [Risk] 直接 stderr 旁路迁移过大。→ Mitigation: 先收口 document operation 和 SDK direct CLI 的代表路径，旧 formatter 只能复用为 stack event 的 surface projection。
- [Risk] standard parameter diagnostics 与 CLI warnings 的字段模型不一致。→ Mitigation: 先定义 source-skipped、validation-failed、ignored-argv 等 event family，再做 caller-specific projection。

## Migration Plan

1. 审计当前所有 warning、stable error、standard parameter diagnostic 和直接 stderr 写入点，分类为 recoverable、fatal、diagnostic-only 和 IO/write failure。
2. 在 `docnav-diagnostics` 定义最小 `DiagnosticCode` enum、`DiagnosticEvent`、`DiagnosticId`、`DiagnosticMark` 和 `DiagnosticStack` API，并提供从 event 到 warning text/readable warning/protocol error 的投影。
3. 让 core document operation、adapter direct command、adapter invoke、standard parameters 和非 document commands 返回 result/outcome + diagnostic stack，输出层统一按 output mode snapshot 或 flush。
4. 将 SDK direct CLI input error、manifest/probe warning、invoke decode diagnostic 和 output write failure 分阶段迁移到 stack push + flush。
5. 更新 docs、schema、examples、fixtures 和 consumer tests，明确新的 protocol/readable/manifest/probe/stderr 输出策略。
6. 更新 tests，重点证明 id lookup、mark/event-id drain、LIFO 返回顺序、显式反转、可恢复事件不丢失、fatal diagnostic code 可投影为目标 surface error。

## Open Questions

- 是否为 `DiagnosticCode` 设计单一总 enum，还是按 family 拆分 enum 并由 formatter/schema 层合并为公共 code 字符串？
- 新的 `protocol-json`、manifest、probe、readable output 和 stderr surface 应分别如何投影 stack entries？
- 哪些 existing protocol/readable/schema field 名称可以直接映射到 `DiagnosticCode`，哪些需要在 breaking migration 中重命名？
