本 tasks 已按 `adopt-core-linked-adapter-libraries` 更新，范围是 core entry pipeline、navigation input resolution 和 static registry implementation source boundary。

## 1. Blocking Audit

- [ ] 1.1 确认 proposal、design、specs 和 tasks 都围绕“core entry pipeline + navigation input resolution + static registry implementation source boundary + raw input immutable”。
- [ ] 1.2 确认本 change 不恢复 adapter direct CLI、adapter `invoke` 或 dynamic adapter management。

## 2. Documentation and Contract Wording

- [ ] 2.1 更新 `docs/architecture.md`，明确 entry pipeline、navigation input resolution、static registry 和 selected adapter dispatch owner 边界。
- [ ] 2.2 更新 `docs/cli.md`，把 core document operations、non-document commands、help、`config list --path` 和 `adapter list` 的入口分类与配置读取边界写清楚。
- [ ] 2.3 更新 `docs/navigation-input-resolution.md`，将 protocol arguments、CLI argv 和 config sources 的映射写成 derived values，不暗示 external adapter process direct input。
- [ ] 2.4 更新 `docs/adapter-contract.md` 和 `docs/protocol.md`，明确 adapter library handle/protocol request execution boundary 和 raw protocol request 不可变。
- [ ] 2.5 更新 `docs/testing.md`、`docs/testing/cases.md` 和 coverage 文案，覆盖 raw input immutable、static registry source boundary 和 dynamic command removal。

## 3. Implementation Naming and Entry Boundaries

- [ ] 3.1 在 core CLI 中调整内部命名，使 document operation 入口体现 entry pipeline 和 navigation input resolution 调用边界。
- [ ] 3.2 在 `docnav-navigation` 中保持 request construction/adapter dispatch 与 navigation input source resolution 分离。
- [ ] 3.3 明确配置来源合并通道实现边界：由 `docnav-navigation` 负责 project/user config source loading、skip diagnostics、registered config path projection 和 source contribution。
- [ ] 3.4 审查 adapter lookup、request construction、output dispatch 和 diagnostic mapping，确保它们消费 derived typed runtime values，而不是让 resolver 决定 handler、输出或 exit code。

## 4. Immutable Raw Input Behavior

- [ ] 4.1 为 core CLI 增加或调整测试，证明 config/default 补足不会把缺失 argv 改写为 direct input。
- [ ] 4.2 为 protocol request construction 增加或调整测试，证明 derived values 写入新 request，不回写 caller-owned raw input。
- [ ] 4.3 为 `adapter list` 和 help 增加或调整测试，证明它们不进入 document parameter source resolution。
- [ ] 4.4 为 passthrough 和 unmapped input 增加或调整测试，证明 resolver 不删除、不重组、不重写 caller-owned raw input。

## 5. Validation and Cleanup

- [ ] 5.1 运行相关 Rust tests，覆盖 core CLI parser/config handoff、navigation request construction 和 navigation input resolution。
- [ ] 5.2 运行 schema/example/docs 验证，确保术语迁移没有破坏 protocol/readable/manifest/probe validation artifacts。
- [ ] 5.3 在改动跨 core、docs 和 specs 后运行 `bun run verify:docnav-workspace`。
