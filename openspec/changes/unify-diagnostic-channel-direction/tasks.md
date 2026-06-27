本 tasks 记录强制迁移到统一 `DiagnosticStack` 的推进路径；当前内容只是 `openspec/changes/unify-diagnostic-channel-direction/` 下的未审核临时文档，本文件本身不立即修改主规范、schema、示例或实现行为。

## 1. 审计门禁

- [ ] 1.1 阻塞级审计：确认 proposal、design、specs 和 tasks 都围绕“强制迁移到 `docnav-diagnostics::DiagnosticStack`，既有错误和诊断 surface 同步切换”这一核心句展开；审计未完成前不得执行任何实现任务。
- [ ] 1.2 确认本 change 只修改 `openspec/changes/unify-diagnostic-channel-direction/` 下的未审核临时 artifacts，没有修改现有主规范、schema、示例或代码。
- [ ] 1.3 确认 `docnav-contracts` 是本 change 的正确 capability ID，且没有创建与 `core-cli`、`adapter-protocol`、`readable-view-output` 重复的总括 capability。
- [ ] 1.4 审计当前行为证据：列出 `StableError`、`Warning`、`StandardParameterDiagnostic`、adapter candidate/config source warning 和直接 stderr 诊断的现有 owner、输出通道和测试覆盖。
- [ ] 1.5 审计并记录 breaking contract surface：`protocol-json`、manifest、probe、readable output、stderr、exit behavior、schema、examples、fixtures 和 consumer tests 的切换范围。
- [ ] 1.6 决策 `diagnostic_only` 延后策略：当前迁移不补齐该 effect；从当前 schema/example 目标中移除或避免使用，后续有明确行为需求时再新增。

## 2. 诊断模型设计

- [ ] 2.1 在 `docnav-diagnostics` 定义最小 `DiagnosticCode` enum 和 `DiagnosticEvent` 字段：severity、code、reason/message、effect、details、source/surface hint 和 fatal/recoverable 语义。
- [ ] 2.2 定义 `DiagnosticId` 和 `DiagnosticMark`：id 只由 stack 分配，mark 标识 stack checkpoint；二者都是内部 scoped control point，不进入 public output contract。
- [ ] 2.3 定义 `DiagnosticStack` API，支持 push、get by id、mark、pop、drain_after(mark)、drain_after_event(id, include_anchor)、snapshot 和最终 flush 后的测试断言。
- [ ] 2.4 明确 stack ordering：内部 pop/drain/snapshot 默认 LIFO；需要正序或分组时由调用方显式 reverse 或 group。
- [ ] 2.5 明确 `StableError` 与 fatal diagnostic 的关系：stack 不持有 `StableError`，`StableError` 不再作为错误 identity owner；protocol/readable surface 直接从 `DiagnosticCode` 投影。
- [ ] 2.6 明确 `Warning` 与 recoverable diagnostic 的关系：warning id 来自 diagnostic code，effect/details 来自 stack event。
- [ ] 2.7 为 standard parameter validation/source-skipped/ignored argv/adapter candidate failure 以及非 document command failures 设计 family-specific details。

## 3. 强制迁移实现

- [ ] 3.1 先在 `docnav-diagnostics` 实现 `DiagnosticStack`、event、id 和 mark，保持依赖方向不引入 core 或 adapter SDK 反向依赖。
- [ ] 3.2 迁移 core document operation，使 parse/runtime/output 能携带 result 或 failure outcome 加 accumulated diagnostic stack。
- [ ] 3.3 迁移 `docnav-output`，由 stack entries 决定 readable warning/error、protocol output 和 stderr flush 的新 surface policy。
- [ ] 3.4 迁移 adapter direct document operation，使 readable/document protocol 输出与 core 共享同一 diagnostic stack handoff。
- [ ] 3.5 分阶段收口 adapter SDK 直接 stderr 旁路：direct CLI input error、manifest/probe warning、invoke decode diagnostic 和 output write failure。
- [ ] 3.6 迁移 `docnav-standard-parameters` diagnostics handoff，使 validation failure 和 source-skipped warning 通过统一 stack 交给 caller。
- [ ] 3.7 迁移非 document 命令（config/init/doctor/version/help 等适用路径），使项目内错误和诊断统一进入 `DiagnosticStack`。
- [ ] 3.8 同步更新主规范、schema、examples、fixtures 和 consumer tests，反映 breaking output contract。

## 4. 验证

- [ ] 4.1 添加 unit tests，证明 recoverable diagnostics 在跨 parser/runtime/output 边界后仍可按 id 或 snapshot 取出且不会由 stack 自身阻断有效 operation。
- [ ] 4.2 添加 unit tests，证明 `DiagnosticMark` 和 event id 支持批量 drain，`drain_after_event` 可选择是否包含 anchor event，且 drain 不会删除边界之前的事件。
- [ ] 4.3 添加 unit tests，证明 pop、drain 和默认 snapshot 返回 LIFO 顺序，调用方显式 reverse 后可得到 insertion order。
- [ ] 4.4 添加 tests，证明 fatal diagnostic code 可投影为 protocol/readable error code/details/guidance 和 exit behavior，且 stack event 不持有也不依赖 `StableError`。
- [ ] 4.5 添加 output tests，证明 `protocol-json`、manifest、probe、readable output 和 stderr 使用新的 DiagnosticStack-based contract。
- [ ] 4.6 添加 adapter SDK tests，证明 manifest/probe/invoke 诊断从 stack flush 或 projection 产生。
- [ ] 4.7 添加 readable tests，证明 `readable-view` 和 `readable-json` 从 stack event 投影 warning/error，并覆盖 `diagnostic_only` 延后策略。
- [ ] 4.8 若实现触及 protocol、schema、examples、output contract 或多个 crate，运行 `bun run verify:docnav-workspace`；否则记录 targeted tests 和未跑全量验证的理由。
