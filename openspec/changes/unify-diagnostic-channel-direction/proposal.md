本 change 定义强制迁移到统一内部 `DiagnosticStack` 的目标方向：Docnav runtime 和 public surface 上的错误与诊断都先把 warning、跳过原因和 fatal context 压入 `docnav-diagnostics` 提供的可按 id 查找的 LIFO 诊断栈，再由调用方或 surface owner 自行决定继续、失败和输出。当前内容只是 `openspec/changes/unify-diagnostic-channel-direction/` 下的未审核临时文档；实施本 change 时必须同步更新主规范、schema、示例、测试和实现。

## Why

当前项目已经有 `StableError`、稳定 warning envelope、standard parameter diagnostics 和若干直接 stderr 诊断，但它们不是同一个可收集、可传递、可延迟输出的通道。结果是可恢复点可以继续执行，但错误信息可能被拆到 readable payload、protocol stderr 或纯文本 stderr，执行链中也不能在任意时刻统一取出完整诊断集合。

这个 change 用于记录方向：统一内部诊断栈和跨层 handoff；调用点默认记录诊断并在输入仍有效时继续，通道本身不判断“错不错”。最终由输出层、adapter machine command、protocol surface owner 或其它调用方按新的 DiagnosticStack contract 组织输出。既有 `protocol-json`、manifest、probe、readable output、`StableError`、warning envelope 和直接 stderr 行为需要强制切换，而不是作为兼容边界保留。

## What Changes

- 引入目标性要求：文档操作、adapter direct CLI、adapter invoke、标准参数解析和输出编排必须迁移到统一 `DiagnosticStack` 模型，warning、skipped diagnostic 和 fatal context 都先作为结构化事件压入同一内部栈。
- 明确内部栈语义：push 时由 stack 生成并返回可保存的 `DiagnosticId`，调用方可用 id 查找事件；调用方也可创建 `DiagnosticMark`，之后按 mark 或 event id 批量弹出后续事件。
- 明确强制迁移路径：不保留第一阶段外部兼容承诺；涉及的主规范、protocol/readable/manifest/probe schema、示例、fixtures 和 consumer tests 必须随实现一起切换到新的 diagnostic 输出策略。
- 明确 surface policy：输出层仍按调用者选择的 surface 决定事件呈现位置，但事件事实源必须来自 `DiagnosticStack`；分组、反转、过滤和格式化由使用者自行处理，不由通道负责。
- 收口直接 stderr 旁路：SDK direct CLI、adapter boundary、invoke decode、JSON/schema/write failure 等路径迁移时必须先把 diagnostic event 压入 stack，再由 surface owner flush 到 stderr 或投影为目标 surface error。
- 反转错误归属：`DiagnosticEvent.code` 是错误和 warning 的机械 id 事实源；`StableError` 如继续存在，应成为 fatal diagnostic code 的 protocol/readable 投影，而不是被 stack 持有的内部对象。
- 延后 `diagnostic_only`：当前迁移不补齐该 effect；只有出现明确行为需求时再新增 enum/schema/tests。

## Capabilities

### New Capabilities

本 change 不新增长期 capability。

### Modified Capabilities

- `docnav-contracts`: 记录统一 `DiagnosticStack` 的目标边界、breaking migration 和跨 surface 输出策略。

## Impact

- 影响面包括 `docnav-diagnostics`、`docnav-output`、`docnav` core output/runtime、`docnav-adapter-sdk` direct CLI/invoke/output、`docnav-standard-parameters` diagnostics handoff、非 document command 错误路径，以及相关 smoke/assertion。
- 本 change 是 breaking contract migration；实现时必须同步更新 docs、schema、examples、fixtures 和 consumer tests，不再把既有 stdout/stderr shape 作为兼容目标。
- 本 change 只保存方向和任务入口；实现前必须完成阻塞级审计，确认所有现有错误、warning、diagnostic 和直接 stderr 路径的强制切换范围。
