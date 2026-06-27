本 tasks 清单记录 token-informed document cost 的后续实施步骤；当前只在 `openspec/changes/use-token-based-document-cost/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级审计

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“Markdown cost 从文件大小为主改为 token-informed 估算”这一核心目标；审计未完成前不得执行任何实现任务。
- [ ] 1.2 审计 capability ID 是否只使用现有 `docnav-contracts` 和 `markdown-navigation`，且没有把 change name 当作新 capability。
- [ ] 1.3 审计当前 change 是否只包含 `openspec/changes/use-token-based-document-cost/` 下的未审核临时 artifacts，且没有修改现有其它文档或主规范。
- [ ] 1.4 审计 `design.md` 的 Open Questions 是否没有未回答问题，并确认 crate、encoding 和验证口径需要在实现前固化为可验证结论。
- [ ] 1.5 审计 Rust `tiktoken` crate 的 crate 名称、许可、维护状态、离线构建行为、encoding 初始化方式、性能成本和 release 影响。

## 2. 规范和验证材料同步

- [ ] 2.1 更新 `docs/protocol.md`，说明 `cost` 仍是 adapter-owned readable string，token-informed cost 不改变 protocol shape。
- [ ] 2.2 更新 `docs/adapters/markdown.md`，定义 Markdown read cost、outline section cost 和 `doc:full` cost 的 token-informed 行为。
- [ ] 2.3 更新 `docs/examples/` 中包含 `lines | KB` 或 read/outline cost 的示例，使其反映 token-informed cost 文案。
- [ ] 2.4 检查 `docs/schemas/` 是否只需要示例/说明同步；若字段 shape 不变，不新增机器必需 token 字段。

## 3. Markdown adapter 实现

- [ ] 3.1 在依赖审计通过后，将选定的 Rust tokenizer crate 加入合适的 Rust package，并记录固定 encoding 选择。
- [ ] 3.2 为 `docnav-markdown` 增加 token cost helper，计算 read target 或 outline section 的 token count。
- [ ] 3.3 更新 Markdown read result `cost` 生成逻辑，使其对未分页的 selected read target 输出 token-informed cost。
- [ ] 3.4 更新 Markdown outline display 组装逻辑，使 heading section 和 `doc:full` entry 包含 token-informed section cost。
- [ ] 3.5 确认 `limit_chars`、page 计算、display 截断和 ref 生成仍沿用现有字符预算和 adapter-owned ref 行为。

## 4. 测试和验证

- [ ] 4.1 更新 Markdown adapter 单元测试和 CLI fixture，覆盖 read cost、heading outline cost 和 `doc:full` token-informed cost。
- [ ] 4.2 更新 protocol/readable 示例验证，确认 `cost` 字段仍是 string 且示例通过 schema。
- [ ] 4.3 运行相关 Rust 测试，至少覆盖 `docnav-markdown`、adapter SDK 分页边界和输出透传路径。
- [ ] 4.4 运行 `bun run verify:docnav-workspace` 或记录无法运行的原因及影响。
- [ ] 4.5 用局部 diff 确认实现只触及 token-informed cost、依赖、测试和对应规范/示例材料。
