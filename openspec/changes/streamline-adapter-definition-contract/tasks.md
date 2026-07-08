本 tasks 清单拆解单一 adapter definition/descriptor authoring surface、内部 typed native option handoff 和 capability group 的实施步骤。实施和归档前，主规范与当前二进制状态仍以 `docs/`、代码和测试为准。

## 1. 实施前确认

- [ ] 1.1 确认 proposal、design、specs 和 tasks 都围绕“adapter 作者只在单一 registry-facing adapter definition/descriptor 中声明 adapter facts，然后由 core/navigation/CLI consumers 传递和派生使用”这一核心目标。
- [ ] 1.2 确认 capability ID 只复用现有 `adapter-contract` 和 `navigation-input-resolution`。
- [ ] 1.3 确认本 change 的范围覆盖 proposal、design、tasks、spec deltas，以及后续实现涉及的主规范、schema、example、代码和测试 owner。
- [ ] 1.4 确认 specs 保留 `protocol-json`、`readable-json`、`readable-view`、ref opacity、pagination、schema/example 和 adapter implementation source 的稳定 owner 边界。
- [ ] 1.5 确认 `design.md` 明确受控过渡适配层的 owner、移除条件和单一定义事实源。
- [ ] 1.6 确认 `design.md` 的 `## Open Questions` 没有未回答问题或已收敛歧义。

## 2. Contract 与文档同步

- [ ] 2.1 更新 `docs/adapter-contract.md`，定义 registry-facing adapter definition/descriptor 是 adapter 作者的单一 registry-facing authoring surface，并说明 identity、manifest/formats、native options、operation handlers 和 full-read capability group 的 owner。
- [ ] 2.2 更新 `docs/navigation-input-resolution.md`，说明 navigation 从 selected adapter definition 注册 native option declarations、解析 typed values，并向 request construction/dispatch boundary 交付内部 typed handoff/accessor。
- [ ] 2.3 更新 `docs/architecture.md` 的共享库 owner 边界，确保 `docnav-adapter-contracts`、`docnav-navigation`、core static registry 与 adapter private semantics 的职责表述一致。
- [ ] 2.4 更新 `docs/adapters/markdown.md`，说明 Markdown adapter 的 native option、full-read support/content/cost/facts 和 handler 消费方式迁移到 descriptor-first 形态。
- [ ] 2.5 更新或新增一个最小 authoring 示例，展示新 adapter 通过一个 definition/factory 声明 metadata、handlers、native option 和 full-read capability group。
- [ ] 2.6 检查 `docs/protocol.md`、`docs/output.md`、schemas 和 examples 是否无需改变；若需要 observable shape 变化，同步更新 owner 文档、schema、example 和 fixture。

## 3. Adapter Contract 实现

- [ ] 3.1 在 `docnav-adapter-contracts` 中引入 adapter definition/descriptor 类型，集中表达 identity、manifest metadata、formats、native option declarations、operation handlers 和 full-read capability group。
- [ ] 3.2 提供 definition builder/factory 或等价构造 API，使 adapter crate 通过一个 exported registry-facing 接口汇出完整 adapter facts；adapter 内部 helper/module 可以拆分，但不能成为额外 shared-layer 声明入口。
- [ ] 3.3 为当前 `Adapter` trait 或 handler path 提供受控过渡适配层，使 descriptor-first 迁移可以小步完成；适配层由 contract/registry 层拥有，并记录移除条件。
- [ ] 3.4 定义 native option typed handoff/accessor 的内部 contract，保留 adapter-owned identity、owner、namespace、key、source 和 type metadata，并保持 external protocol JSON shape 的 owner 边界。
- [ ] 3.5 将 unstructured full-read 的 support declaration、content hook、cost measurement hook 和 result facts hook 收敛为 full-read capability group。
- [ ] 3.6 为 adapter definition validation 添加单元测试，覆盖 missing required handlers、invalid native option path、duplicate declarations、duplicate handler/capability declaration 和 unsupported capability combinations。

## 4. Navigation 与 Registry 迁移

- [ ] 4.1 更新 core static registry，使 built-in adapters 通过 adapter definition 注册，同时保持 implementation source 是 core static registry fact。
- [ ] 4.2 更新 adapter inspection、doctor 和 CLI native option catalog，使 metadata、native CLI flags 和 validation facts 从同一个 adapter definition 派生，并替换独立 helper 重建路径。
- [ ] 4.3 更新 `docnav-navigation` 的 selected adapter declaration registration，从 selected adapter definition 读取当前 operation 的 native option declarations。
- [ ] 4.4 更新 parameter resolution 到 operation binding 的 handoff，生成 selected adapter typed native option values 或 adapter-specific accessor，并保留 source attribution。
- [ ] 4.5 更新 request construction 或 dispatch 层，使 operation handler 接收 typed operation arguments 和内部 typed native option handoff/accessor，且不转发 raw config JSON。
- [ ] 4.6 更新 full-read pre-dispatch policy，使 navigation 依据 selected adapter definition 的 full-read capability group 判断 support、content、measurement、facts 和 fallback。

## 5. Markdown Adapter 迁移

- [ ] 5.1 将 Markdown adapter 的 identity、format descriptors、native option declaration、operation handlers 和 full-read support/content/cost/facts 集中到 descriptor-first 声明，并确认 adapter crate 只维护一个 registry-facing definition。
- [ ] 5.2 将 `max_heading_level` 消费从 generic options bag 迁移到 typed handoff/accessor，并保留 owner、namespace、key 和 source attribution 的防错测试。
- [ ] 5.3 保持 Markdown `outline`、`read`、`find`、`info` 的业务语义、ref grammar、pagination 和 cost facts 不变。
- [ ] 5.4 更新 Markdown adapter tests，覆盖 descriptor metadata、native option defaults/range validation、typed option consumption、single-definition authoring path 和 full-read capability group。

## 6. Verification

- [ ] 6.1 运行 OpenSpec 验证：`openspec validate streamline-adapter-definition-contract --type change --json --strict --no-interactive`。
- [ ] 6.2 运行 adapter contract 相关 Rust tests，证明 descriptor validation、single-definition authoring path、typed native option handoff 和 capability group 行为。
- [ ] 6.3 运行 navigation input resolution tests，证明 selected adapter declaration registration、source precedence、strict unknown native option 和 typed dispatch handoff 都来自 selected definition。
- [ ] 6.4 运行 Markdown adapter tests 和 core CLI smoke，证明 `protocol-json`、`readable-json`、`readable-view`、ref、pagination、schema/example、single-definition adapter registration 和 invocation behavior 保持稳定。
- [ ] 6.5 跨 `docnav-adapter-contracts`、`docnav-navigation`、core registry 和 Markdown adapter 后运行 `bun run verify:docnav-workspace`。
- [ ] 6.6 用局部 diff 确认实现阶段只改动本 change 涉及的 contract、navigation、registry、Markdown adapter、docs 和测试材料。
