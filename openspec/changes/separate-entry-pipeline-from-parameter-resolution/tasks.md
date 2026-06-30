本 tasks 是 `separate-entry-pipeline-from-parameter-resolution` 的未审核临时文档，目标是按审计门禁推进标准入口管线和入口参数来源解析迁移；当前 change 只在 `openspec/changes/separate-entry-pipeline-from-parameter-resolution/` 下形成临时文档，不影响现有其它文档或主规范。

## 1. Blocking Audit

- [ ] 1.1 阻塞级审计：确认 proposal、design、specs 和 tasks 都围绕“标准入口管线 + 入口参数来源解析 + 原始输入不可变”核心句；确认 capability ID 复用现有 specs 且未创建同义新能力；确认 `design.md` 的 Open Questions 无未回答问题。审计未完成前不得执行 2.x 及后续任何实现任务。
- [ ] 1.2 审计范围检查：确认本 change 在审计前只包含 `openspec/changes/separate-entry-pipeline-from-parameter-resolution/` 下的未审核临时 artifacts，且未修改现有其它文档、主规范、代码或验证材料。

## 2. Documentation and Contract Wording

- [ ] 2.1 更新 `docs/architecture.md`，明确标准入口管线、入口参数来源解析、配置来源合并通道、标准参数身份和 typed-field definitions 的 owner 边界。
- [ ] 2.2 更新 `docs/cli.md`，把 core document operations、non-document commands、help 和 `config list --path` 的入口分类与配置读取边界写清楚。
- [ ] 2.3 更新 `docs/standard-parameters.md` 或其重命名后继文档，将旧“标准参数流程”迁移为“入口参数来源解析”，并把“配置来源合并通道”限定为 project/user config source 子流程，同时记录 explicit adapter native option sources 和 unmapped public input handoff。
- [ ] 2.4 更新 `docs/adapter-contract.md` 和 `docs/protocol.md`，明确 adapter direct CLI、manifest/probe/help 和 `invoke` 的标准入口管线边界、explicit adapter native option source 声明，以及 `invoke` raw stdin request 不可变。
- [ ] 2.5 更新 `docs/testing.md`、`docs/testing/cases.md` 和相关 coverage/case maintenance 文案，使测试目标使用新术语并覆盖不可变原始输入规则。

## 3. Implementation Naming and Entry Boundaries

- [ ] 3.1 在 core CLI 中调整内部命名，使 document operation 入口体现 standard entry pipeline 和 entry parameter source resolution 的调用边界。
- [ ] 3.2 在 adapter SDK direct CLI 中调整命名和分支结构，使 manifest/probe/help/invoke/document operation 的分类发生在 document config loading 和参数来源解析前。
- [ ] 3.3 为现有 standard parameter resolver 增加新命名 facade、module alias 或 wrapper；必要时保留旧名兼容层，但新代码路径使用 entry parameter source resolution 术语。
- [ ] 3.4 明确配置来源合并通道实现边界：只负责 project/user config source loading、skip diagnostics、registered config path projection 和 source contribution。
- [ ] 3.5 审查 request construction、output dispatch 和 diagnostic mapping，确保它们消费 derived typed runtime values，而不是让 resolver 决定 handler、输出或 exit code。

## 4. Immutable Raw Input Behavior

- [ ] 4.1 为 core CLI 增加或调整测试，证明 config/default 补足不会把缺失 argv 改写为 direct input，也不会修改 raw argv token 记录。
- [ ] 4.2 为 adapter `invoke` 增加或调整测试，证明 config/default 补足只产出 derived operation values，不回写 raw decoded stdin request JSON。
- [ ] 4.3 为 adapter direct CLI 增加或调整测试，证明 manifest/probe/help 不读取 document operation config，不进入 document parameter source resolution，也不使用 document output mode。
- [ ] 4.4 为 passthrough 和 unmapped input 增加或调整测试，证明 resolver 不删除、不重组、不重写 caller-owned raw input，并把未映射 public input 回交入口 owner 形成 blocking input diagnostic。

## 5. Validation and Cleanup

- [ ] 5.1 运行相关 Rust tests，覆盖 core CLI parser/config、adapter SDK direct CLI、invoke 和 parameter source resolution。
- [ ] 5.2 运行 schema/example/docs 相关验证，确保术语迁移没有破坏 protocol/readable/manifest/probe validation artifacts。
- [ ] 5.3 在改动跨 core、SDK、docs 和 specs 后运行 `bun run verify:docnav-workspace`。
- [ ] 5.4 审查局部 diff，确认只修改标准入口管线、参数来源解析命名/边界、不可变输入测试和对应文档验证材料。
