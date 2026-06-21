本 tasks 定义 Markdown frontmatter 作为可选 readable metadata block 随普通 read 输出的实现入口；它只在 `openspec/changes/markdown-frontmatter-readable-block/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级审计

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“Markdown frontmatter 可按配置作为可选 metadata block 随普通 read 输出”这一核心目标。
- [ ] 1.2 审计 capability ID 是否正确复用 `adapter-protocol`、`docnav-contracts`、`markdown-navigation` 和 `readable-view-output`，且没有创建一次性、同义或过宽 capability。
- [ ] 1.3 审计当前 change 是否只包含 `openspec/changes/markdown-frontmatter-readable-block/` 下的未审核临时 artifacts，且没有修改现有 specs、docs、schemas、examples 或实现代码。
- [ ] 1.4 审计 proposal、design 和 specs 是否明确 core、MCP、shared output 层不得解析 YAML，frontmatter 由 Markdown adapter 拥有。
- [ ] 1.5 在 1.1-1.4 全部完成前，不得执行任何实现任务、主规范更新、示例更新、测试更新或代码改动。

## 2. 规范与验证材料同步

- [ ] 2.1 按 `docs/navigation.md` 读取 `docs/CODING_STYLE.md`、`docs/protocol.md`、`docs/output.md`、`docs/mcp.md`、`docs/adapters/markdown.md`、`docs/testing.md` 和 `docs/testing/case-maintenance.md` 中与本 change 相关的 owner 规则。
- [ ] 2.2 更新 `docs/protocol.md`，记录 read success result 的 optional adapter-owned metadata/frontmatter 字段。
- [ ] 2.3 更新 `docs/output.md`，记录 readable-view renderer required/optional block pointer 契约和 `/frontmatter/content` optional block。
- [ ] 2.4 更新 `docs/adapters/markdown.md`，记录 frontmatter 识别、配置、普通 read metadata 输出、outline 排除和全文读取不重复 metadata。
- [ ] 2.5 更新 `docs/mcp.md`、schema 索引、examples 和 testing 文档，覆盖有 frontmatter、无 frontmatter、配置关闭和全文读取四种路径。

## 3. 协议、Schema 与 Readable Renderer

- [ ] 3.1 更新 shared protocol/read result 类型，使 read success result 可以包含 optional adapter-owned metadata/frontmatter。
- [ ] 3.2 更新 protocol JSON schema、readable JSON schema 和 MCP outputSchema，允许 frontmatter metadata 字段省略或出现。
- [ ] 3.3 更新 readable-view renderer config model，区分 required block pointers 和 optional block pointers。
- [ ] 3.4 实现 optional block extraction：pointer 缺失时跳过，存在但非字符串、重复或 identity 冲突时仍返回 `readable_view_render_failed`。
- [ ] 3.5 增加 conformance vectors 或等价测试，覆盖 optional block 缺失、存在、类型错误和多 block 输出。

## 4. Markdown Adapter 实现

- [ ] 4.1 在 Markdown parser 层识别文档开头合法 YAML frontmatter，并保留 YAML 原文 payload。
- [ ] 4.2 确认 frontmatter 中的伪 heading 不进入 heading model、outline entries 或 heading index 分配。
- [ ] 4.3 接入 `docnav-markdown` adapter-owned 配置开关/native option，用于控制普通 read 是否默认附带 frontmatter metadata；不在本 change 固化具体配置文件、格式或合并方式。
- [ ] 4.4 更新 Markdown read result construction：普通 heading/section read 在配置启用且存在 frontmatter 时返回 metadata；配置关闭或无 frontmatter 时省略字段。
- [ ] 4.5 确认 `doc:full` 和非结构化全文读取默认不重复输出 frontmatter metadata block。

## 5. 测试、示例与 MCP 覆盖

- [ ] 5.1 更新 Markdown unit/adapter tests，覆盖合法 YAML frontmatter、无 frontmatter、frontmatter 伪 heading、非法 frontmatter 和配置开关。
- [ ] 5.2 更新 Markdown CLI smoke fixture，验证 readable-json、readable-view 和 protocol-json 下 frontmatter metadata 的存在与省略。
- [ ] 5.3 更新 readable-view tests，验证 `/frontmatter/content` optional block 的 byte length、payload 还原和 header 字段。
- [ ] 5.4 更新 protocol/readable/MCP examples，展示有 frontmatter metadata 的 read result 和没有 metadata 的兼容 shape。
- [ ] 5.5 更新 MCP mapping tests，确认 MCP 只承载 structuredContent 中的 metadata，不解析 YAML。

## 6. 验证与收尾

- [ ] 6.1 运行 Rust 格式化和相关 crate 测试，至少覆盖 `docnav-protocol`、`docnav-readable`、`docnav-output` 和 `docnav-markdown`。
- [ ] 6.2 运行 schema/example 验证，确认 read protocol、readable 和 MCP 示例都通过更新后的 schema。
- [ ] 6.3 对涉及 adapter、schema/example、readable renderer 和 docs 边界的最终改动运行 `pnpm run verify:docnav-workspace`，除非有明确、记录在最终说明中的环境阻塞。
- [ ] 6.4 使用局部 diff 确认实现只改动 frontmatter metadata、optional block 相关代码、文档、示例、测试和本 change artifacts。
- [ ] 6.5 在所有实现任务和验证任务完成后，再运行 `openspec validate markdown-frontmatter-readable-block --type change --strict --no-interactive` 并准备归档评估。
