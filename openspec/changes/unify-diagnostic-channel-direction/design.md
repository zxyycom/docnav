本 design 记录统一 diagnostic channel 的技术方向；当前内容只是 `openspec/changes/unify-diagnostic-channel-direction/` 下的未审核临时文档，不改变现有主规范、schema、示例或实现行为。

## Context

当前实现已经有若干接近统一的基础：

- `docnav-protocol::StableError` 表达阻断执行的稳定错误，并决定 protocol/readable error 和 exit code 映射。
- `docnav-diagnostics::Warning` 表达稳定 warning envelope，并提供 readable JSON 注入和 stderr warning line formatter。
- `docnav-output` 集中文档操作的 `readable-view`、`readable-json` 和 `protocol-json` 输出分流。
- `docnav-standard-parameters` 已有 `StandardParameterDiagnostic`，但它还不是 `docnav-diagnostics::Warning` 或 `StableError`。
- `docnav-adapter-sdk` 仍存在直接 `emit_diagnostic(stderr, text)` 的旁路，例如 direct CLI input error、manifest/probe/schema 校验、invoke decode 诊断和 output write failure。

主约束是协议兼容性：`protocol-json`、adapter `invoke` stdout、manifest 和 probe stdout 必须继续只输出各自 machine payload；warning 或诊断不能污染 stdout envelope。统一 channel 首先应统一内部事件收集和交接，不应默认改变外部 schema。

## Goals / Non-Goals

**Goals:**

- 定义一个方向：warning、recoverable diagnostic 和 fatal stable error 在进入输出层前都可以被同一诊断集合携带。
- 保持“跳过错误点但不跳过错误信息”：可恢复事件不阻断执行，但必须保留到最终输出策略可见。
- 允许调用链在任意边界取出当前 accumulated diagnostics，用于测试、输出、日志或后续映射。
- 保持当前外部 surface 兼容，除非后续显式选择 breaking contract 迁移。

**Non-Goals:**

- 本 change 不要求立即修改 `protocol-response.schema.json`、manifest schema、probe schema 或 readable schema。
- 本 change 不把 warning 变成 `StableError`，也不把所有 stderr 文本写入 protocol response。
- 本 change 不改变 adapter-owned ref、format parsing、native option validation 或 document operation 业务语义。
- 本 change 不一次性迁移所有脚本、测试 harness 或非 Rust 开发工具的 stderr 输出。

## Decisions

### Decision 1: 先统一内部 diagnostic event，再保持 surface 输出策略

后续实现应在 `docnav-diagnostics` 或相邻共享 crate 中定义统一事件模型，例如 `DiagnosticEvent` 与 `DiagnosticBag`。事件至少需要表达 severity、stable id/code、message/reason、effect、details、source/surface hint，以及是否阻断执行。

选择这个方案而不是直接给 protocol response 增加 `warnings` 字段，是因为现有 protocol schema 明确封闭，stdout 纯净是 machine contract。内部统一可以先解决“可取回”和“跨层 handoff”问题，同时避免立刻打破消费者。

### Decision 2: `StableError` 继续作为 fatal protocol error 的稳定语义

统一 channel 不替代 `StableError`。阻断执行时，event bag 可以携带 fatal event，但最终 protocol/readable error 仍由 `StableError` 负责 code、message、details、guidance 和 exit category。这样可以保留现有 error-rules、schema validation 和 exit code 映射。

备选方案是把 `StableError` 降级为 diagnostic event 的一个 variant，并由输出层重新构造 protocol error。该方案长期可能更统一，但短期会扩大迁移面，并增加稳定错误 details 丢失或重映射的风险。

### Decision 3: warnings 应成为 diagnostic event 的一种兼容投影

现有 `Warning` 可以作为可恢复 event 的 readable/protocol-stderr 投影继续存在。后续可以让 `Warning` either 嵌入 `DiagnosticEvent`，或提供从 event 到 warning envelope 的确定性转换。

这比让每个 caller 自己构造 `Warning` 更好，因为 `StandardParameterDiagnostic`、adapter candidate failure、config source skipped 和 ignored argv 都可以通过同一 handoff 返回，最终由 output owner 决定是否注入 readable JSON 或写 stderr。

### Decision 4: 直接 stderr 旁路逐步替换为 event + flush

`docnav-adapter-sdk::emit_diagnostic` 不应作为长期公共诊断入口。后续迁移可以先保留 formatter，但调用方应优先把诊断写入 `DiagnosticBag`，最后由 surface-specific flush 写入 stderr。

这个迁移需要按入口分阶段完成：document operation 输出优先，其次 adapter direct machine commands，再处理 invoke decode 和 IO/write failure。每个阶段都需要证明 stdout shape 未变化。

### Decision 5: `diagnostic_only` 需要单独决策

当前 readable schema 允许 `warning.effect = diagnostic_only`，Rust `WarningEffect` 未实现该 variant。后续实现统一 channel 前必须决定：

- 如果需要表达“不影响 operation、只用于 stderr/readable metadata”的事件，则补齐 Rust enum 和 tests。
- 如果该值只是 schema 预留且没有当前行为依据，则从 schema 删除或改成未来 change 的明确目标。

## Risks / Trade-offs

- [Risk] 统一 event model 变成过宽的“万能错误对象”。→ Mitigation: 保持 `StableError`、`Warning` 投影和 surface owner 边界，event 只承载跨层 handoff 所需字段。
- [Risk] protocol stdout 兼容性被无意破坏。→ Mitigation: 第一阶段测试必须断言 protocol response、manifest 和 probe stdout 不新增 diagnostics/warnings 字段。
- [Risk] fatal error 和 recoverable warning 的语义混淆。→ Mitigation: event 必须有明确 severity/effect，并由 exit code mapping 继续只消费 fatal stable error。
- [Risk] 直接 stderr 旁路迁移过大。→ Mitigation: 先收口 document operation 和 SDK direct CLI 的代表路径，保留旧 formatter 作为最终 flush 实现。
- [Risk] standard parameter diagnostics 与 CLI warnings 的字段模型不一致。→ Mitigation: 先定义 source-skipped、validation-failed、ignored-argv 等 event family，再做 caller-specific projection。

## Migration Plan

1. 审计当前所有 warning、stable error、standard parameter diagnostic 和直接 stderr 写入点，分类为 recoverable、fatal、diagnostic-only 和 IO/write failure。
2. 在共享 diagnostics 层定义最小 event/bag API，并提供从现有 `Warning` 到 event、从 event 到 warning text/readable warning 的兼容转换。
3. 让 core document operation 和 adapter direct document operation 返回 result/outcome + diagnostic bag，输出层统一按 output mode flush。
4. 将 SDK direct CLI input error、manifest/probe warning、invoke decode diagnostic 和 output write failure 分阶段迁移到 event + flush。
5. 更新 tests，重点证明可恢复事件不丢失、fatal error 仍映射稳定错误、protocol/manifest/probe stdout shape 不变。
6. 如后续选择 breaking contract，再单独提出 protocol/readable/schema/examples 的 observable change。

## Open Questions

- 是否需要把 `DiagnosticEvent` 放在现有 `docnav-diagnostics`，还是新建更窄的 crate 避免依赖 `docnav-protocol`？
- fatal event 与 `StableError` 的关系是“event 包含 StableError”，还是“AppError/AdapterError 同时携带 StableError + DiagnosticBag”？
- `diagnostic_only` 是否成为当前 Rust warning effect，还是从 schema 中移除预留值？
- 非 document 命令（config/init/doctor/version/help）是否进入同一 event/bag，还是保留各自 plain text/stderr policy 到后续 change？
