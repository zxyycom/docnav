本 tasks 清单拆解 adapter definition/descriptor、内部 typed native option handoff 和 capability group 的实施步骤；当前文档只在 `openspec/changes/streamline-adapter-definition-contract/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级审计

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“收敛 linked adapter 扩展面为 registry-facing descriptor、高层 operation handler、内部 typed native option handoff/accessor 和 capability group”这一核心目标。
- [ ] 1.2 审计 capability ID 是否只复用现有 `adapter-contract` 和 `navigation-input-resolution`，不创建同义新 capability。
- [ ] 1.3 审计当前 change 是否只包含 `openspec/changes/streamline-adapter-definition-contract/` 下的未审核临时 artifacts，且没有修改主规范、schema、example 或实现代码。
- [ ] 1.4 审计 specs 是否保留 `protocol-json`、`readable-json`、`readable-view`、ref opacity、pagination、schema/example 和 adapter implementation source 的兼容边界。
- [ ] 1.5 审计 `design.md` 的 `## Open Questions` 是否没有未回答问题或已收敛歧义。
- [ ] 1.6 审计未完成前不得执行任何实现任务；只有 1.1-1.5 全部完成后才能开始第 2 组及之后任务。

## 2. Contract 与文档同步

- [ ] 2.1 更新 `docs/adapter-contract.md`，定义 registry-facing adapter definition/descriptor facade、operation handler boundary、内部 typed native option handoff/accessor 和 capability group。
- [ ] 2.2 更新 `docs/navigation-input-resolution.md`，说明 navigation 从 selected adapter definition 注册 native option declarations、解析 typed values，并向 request construction/dispatch boundary 交付内部 typed handoff。
- [ ] 2.3 更新 `docs/architecture.md` 的共享库 owner 边界，确保 `docnav-adapter-contracts`、`docnav-navigation`、core static registry 与 adapter private semantics 的职责表述一致。
- [ ] 2.4 更新 `docs/adapters/markdown.md`，说明 Markdown adapter 的 native option、full-read capability 和 handler 消费方式迁移到 descriptor-first 形态。
- [ ] 2.5 检查 `docs/protocol.md`、`docs/output.md`、schemas 和 examples 是否无需改变；若需要 observable shape 变化，同步更新 owner 文档、schema、example 和 fixture。

## 3. Adapter Contract 实现

- [ ] 3.1 在 `docnav-adapter-contracts` 中引入 adapter definition/descriptor 类型，集中表达 identity、manifest metadata、formats、native option declarations、operation handlers 和 optional capability groups。
- [ ] 3.2 为现有 `Adapter` trait 或 handler path 提供兼容 shim，使 descriptor-first 迁移可以小步完成。
- [ ] 3.3 定义 native option typed handoff/accessor 的内部 contract，保留 adapter-owned identity、owner、namespace、key、source 和 type metadata，并默认不改变 external protocol JSON shape。
- [ ] 3.4 将 unstructured full-read 的 content、cost measurement 和 result facts 能力收敛为 full-read capability group。
- [ ] 3.5 为 adapter definition validation 添加单元测试，覆盖 missing required handlers、invalid native option path、duplicate declarations 和 unsupported capability combinations。

## 4. Navigation 与 Registry 迁移

- [ ] 4.1 更新 core static registry，使 built-in adapters 通过 adapter definition 或等价 facade 注册，同时保持 implementation source 是 core static registry fact。
- [ ] 4.2 更新 `docnav-navigation` 的 selected adapter declaration registration，从 selected adapter definition 读取当前 operation 的 native option declarations。
- [ ] 4.3 更新 parameter resolution 到 operation binding 的 handoff，生成 selected adapter typed native option values 或 adapter-specific accessor，并保留 source attribution。
- [ ] 4.4 更新 request construction 或 dispatch 层，使 operation handler 接收 typed operation arguments 和内部 typed native option handoff，且不转发 raw config JSON。
- [ ] 4.5 更新 full-read pre-dispatch policy，使 navigation 只依据 selected adapter definition 的 full-read capability group 判断 support、measurement 和 fallback。

## 5. Markdown Adapter 迁移

- [ ] 5.1 将 Markdown adapter 的 identity、format descriptors、native option declaration、operation handlers 和 full-read capability 集中到 descriptor-first 声明。
- [ ] 5.2 将 `max_heading_level` 消费从 generic options bag 迁移到 typed handoff/accessor，并保留 owner、namespace、key 和 source attribution 的防错测试。
- [ ] 5.3 保持 Markdown `outline`、`read`、`find`、`info` 的业务语义、ref grammar、pagination 和 cost facts 不变。
- [ ] 5.4 更新 Markdown adapter tests，覆盖 descriptor metadata、native option defaults/range validation、typed option consumption 和 full-read capability declaration。

## 6. Verification

- [ ] 6.1 运行 OpenSpec 验证：`openspec validate streamline-adapter-definition-contract --type change --json --strict --no-interactive`。
- [ ] 6.2 运行 adapter contract 相关 Rust tests，证明 descriptor validation、typed native option handoff 和 capability group 行为。
- [ ] 6.3 运行 navigation input resolution tests，证明 selected adapter declaration registration、source precedence、strict unknown native option 和 typed dispatch handoff。
- [ ] 6.4 运行 Markdown adapter tests 和 core CLI smoke，证明 `protocol-json`、`readable-json`、`readable-view`、ref、pagination、schema/example 和 invocation behavior 保持兼容。
- [ ] 6.5 跨 `docnav-adapter-contracts`、`docnav-navigation`、core registry 和 Markdown adapter 后运行 `bun run verify:docnav-workspace`。
- [ ] 6.6 用局部 diff 确认实现阶段只改动本 change 涉及的 contract、navigation、registry、Markdown adapter、docs 和测试材料。
