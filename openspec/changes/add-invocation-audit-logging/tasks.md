本 tasks 只在 `openspec/changes/add-invocation-audit-logging/` 下形成未审核临时文档，执行前必须先完成 invocation logging 方案审计门禁。

## 1. 阻塞级审计门禁

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“为 Docnav 核心调用链引入默认元数据级调用日志和可选协议追踪”这一核心目标；审计未完成前不得执行任何实现任务。
- [ ] 1.2 审计 capability ID 是否只新增 `invocation-logging`，且没有把 change name、`code-quality-observability` 或过宽 runtime umbrella 当作错误 owner。
- [ ] 1.3 审计当前 change 是否只包含 `openspec/changes/add-invocation-audit-logging/` 下的未审核临时 artifacts，且没有修改现有 specs、docs、schemas、examples、测试或实现代码。
- [ ] 1.4 审计 `design.md` 的 Open Questions 是否没有未回答问题，并确认日志路径、开关名称和日志库选型会在实现前由主规范与测试固化。
- [ ] 1.5 审计安全边界：metadata-only 默认、raw trace 显式 opt-in、payload 截断/脱敏、stdout purity、adapter stdout/stderr 边界和日志失败降级都已进入 specs。
- [ ] 1.6 审计依赖边界：首期是否使用内部 JSONL writer；若要引入 `tracing` 或其它日志库，必须先完成依赖、feature、初始化和输出通道审计。

## 2. 规范同步

- [ ] 2.1 在审计通过后，更新对应主规范，声明 `invocation-logging` 的 owner、运行时日志用途、JSONL 格式、事件字段和状态语义。
- [ ] 2.2 明确日志开关、默认模式、日志文件位置、路径显示策略、query/ref 摘要策略和 trace 模式启用方式。
- [ ] 2.3 明确 runtime invocation log 与 verify/smoke `.log`、code-quality observability artifacts 的边界，避免复用测试日志格式作为运行时 contract。
- [ ] 2.4 如日志字段成为可验证机器输出，补充 schema、example 或 fixture 验证材料；若不建立正式 JSON Schema，记录原因和替代验证方式。

## 3. Core 实现

- [ ] 3.1 在 core CLI 文档操作链路中确定最小插桩点，覆盖 adapter selection 后的 protocol request 构造、adapter `invoke` 调用、响应校验和错误映射结果。
- [ ] 3.2 实现 metadata-only JSONL event writer，支持 schema version、timestamp、event、request id、operation、adapter id、duration、exit/status metadata、response size 和 bounded diagnostic summary。
- [ ] 3.3 实现日志配置解析和 sink 初始化，保证未启用日志时不产生运行时开销或可观察输出变化。
- [ ] 3.4 实现日志写入失败降级，确保日志目录不可写、序列化失败或 append 失败不改变原本文档操作结果。
- [ ] 3.5 实现 raw protocol trace 的显式启用、字段脱敏、大小限制和截断标记；未开启 trace 时不得写完整 request/response envelope。
- [ ] 3.6 如审计决定引入 `tracing` 或其它日志库，先完成依赖更新与初始化隔离，再接入 writer；否则保留仓库内 JSONL writer。

## 4. 测试与验证

- [ ] 4.1 增加 core 层单元或集成测试，覆盖成功调用写入可解析 JSONL event，并用 `request_id` 关联 request/response。
- [ ] 4.2 增加失败路径测试，覆盖 adapter 启动失败、stdout 非 JSON、protocol response validation 失败和稳定错误映射摘要。
- [ ] 4.3 增加 stdout purity 测试，证明启用日志后 `protocol-json` stdout、readable stdout 和 adapter protocol stdout 不被日志污染。
- [ ] 4.4 增加安全测试，证明 metadata-only 默认不记录 full read content、完整 request/response payload、完整 stderr 或无界 query/ref。
- [ ] 4.5 增加 trace opt-in 测试，证明完整 payload 只在显式 trace 模式下出现，且超限字段带截断标记。
- [ ] 4.6 增加日志写入失败降级测试，证明不可写日志路径不会改变原本文档操作的成功/失败语义。
- [ ] 4.7 运行受影响 Rust tests、OpenSpec validation；若同步修改主规范、schema、examples 或跨 crate 行为，运行 `bun run verify:docnav-workspace`。

## 5. 交付审计

- [ ] 5.1 用局部 diff 审计实现是否只触及 invocation logging 相关 docs、specs、tests 和代码。
- [ ] 5.2 抽查启用 metadata-only 日志后的实际 JSONL，确认每行可独立解析、字段稳定、诊断有界且没有 document content。
- [ ] 5.3 抽查 trace 模式，确认它只能通过显式配置开启，并且截断/脱敏策略实际生效。
- [ ] 5.4 记录最终验证命令、结果和任何未覆盖风险，再进入归档或后续审计改进 change。
