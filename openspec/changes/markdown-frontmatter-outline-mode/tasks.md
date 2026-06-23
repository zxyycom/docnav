本 tasks 定义 Markdown frontmatter 按 `frontmatter_outline_mode` enum 在 outline 阶段暴露的实现入口；它只在 `openspec/changes/markdown-frontmatter-outline-mode/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级审计

- [x] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“Markdown frontmatter 由 `frontmatter_outline_mode` enum 控制，在 outline 中以 inline/ref/hidden 三种模式暴露”这一核心目标。
- [x] 1.2 审计 capability ID 是否正确复用 `adapter-protocol`、`docnav-contracts`、`markdown-navigation` 和 `readable-view-output`，且没有创建一次性、同义或过宽 capability。
- [x] 1.3 审计当前 change 是否只包含 `openspec/changes/markdown-frontmatter-outline-mode/` 下的未审核临时 artifacts，且没有修改现有 specs、docs、schemas、examples 或实现代码。
- [x] 1.4 审计 proposal、design 和 specs 是否明确 core、MCP、shared output 层不得解析 YAML，frontmatter 由 Markdown adapter 拥有。
- [x] 1.5 在 1.1-1.4 全部完成前，不得执行任何实现任务、主规范更新、示例更新、测试更新或代码改动。

## 2. 规范与验证材料同步

- [ ] 2.1 按 `docs/navigation.md` 读取 `docs/CODING_STYLE.md`、`docs/protocol.md`、`docs/output.md`、`docs/mcp.md`、`docs/adapters/markdown.md`、`docs/testing.md` 和 `docs/testing/case-maintenance.md` 中与本 change 相关的 owner 规则。
- [ ] 2.2 更新 `docs/protocol.md`，记录 outline success result 的 optional adapter-owned `frontmatter` 字段，并明确 Markdown-owned payload shape、inline pagination 和 `page` continuation 语义。
- [ ] 2.3 更新 `docs/output.md`，记录 readable-view renderer required/optional block pointer 契约和 outline `/frontmatter/content` optional block。
- [ ] 2.4 更新 `docs/adapters/markdown.md`，记录 frontmatter delimiter 识别、`frontmatter_outline_mode` enum、默认 `inline`、`ref` 模式的 `FM:frontmatter` read、`hidden` 行为和 frontmatter 伪 heading 排除。
- [ ] 2.5 更新 `docs/mcp.md`、schema 索引、examples 和 testing 文档，覆盖 inline、ref、hidden、无 frontmatter、frontmatter 超预算分页和 `FM:frontmatter` read 六种路径。

## 3. 协议、Schema 与 Readable Renderer

- [ ] 3.1 更新 shared protocol/outline result 类型，使 outline success result 可以包含 optional adapter-owned `frontmatter` metadata。
- [ ] 3.2 更新 protocol JSON schema、readable JSON schema 和 MCP outputSchema，允许 outline `frontmatter` 字段省略或以 `{ content_type: "application/yaml", content: string }` 出现。
- [ ] 3.3 更新 `docs/schemas/docnav-markdown-config.schema.json` 和 `docs/examples/json/docnav-markdown-config.json`，加入 `options.frontmatter_outline_mode` enum 示例与合法值。
- [ ] 3.4 更新 readable-view renderer config model，区分 required block pointers 和 optional block pointers。
- [ ] 3.5 实现 optional block extraction：pointer 缺失时跳过，存在但非字符串、重复或 identity 冲突时仍返回 `readable_view_render_failed`。
- [ ] 3.6 增加 conformance vectors 或等价测试，覆盖 optional block 缺失、存在、类型错误和多 block 输出。

## 4. Markdown Adapter 实现

- [ ] 4.1 在 Markdown parser 层识别文档开头 YAML frontmatter delimiter block，并保留不含起止 delimiter 的 YAML 原文 payload。
- [ ] 4.2 确认 frontmatter 中的伪 heading 不进入 heading model、heading entries 或 heading index 分配。
- [ ] 4.3 接入 `docnav-markdown` adapter-owned enum option `frontmatter_outline_mode`，合法值为 `inline`、`ref` 和 `hidden`，默认值为 `inline`。
- [ ] 4.4 更新 Markdown outline result construction：`inline` 返回顶层 `frontmatter` 字段，`ref` 返回 `FM:frontmatter` entry，`hidden` 不暴露 frontmatter。
- [ ] 4.5 为 `inline` 模式实现 read content 同款字符预算与 page 续读规则，并确认 heading entries 在 frontmatter 当前 page slice 之后消耗剩余预算。
- [ ] 4.6 更新 Markdown read ref handling：读取 `FM:frontmatter` 时返回 `application/yaml` primary content，并使用普通 read content 分页规则。
- [ ] 4.7 确认普通 heading/section read 不新增 frontmatter metadata 字段，既有 heading read content 行为保持不变。

## 5. 测试、示例与 MCP 覆盖

- [ ] 5.1 更新 Markdown unit/adapter tests，覆盖可识别 frontmatter delimiter block、无 frontmatter、frontmatter 伪 heading、未闭合/非文档开头 delimiter 和三种 `frontmatter_outline_mode`。
- [ ] 5.2 更新 Markdown CLI smoke fixture，验证 readable-json、readable-view 和 protocol-json 下 inline/ref/hidden 的 outline 行为。
- [ ] 5.3 更新 Markdown pagination tests，验证 inline frontmatter 超过 `limit_chars` 时按 read content 规则续读且不切断 Unicode 字符。
- [ ] 5.4 更新 Markdown read tests，验证 `FM:frontmatter` ref 返回 YAML primary content、content_type、page 和 invalid/missing frontmatter 边界。
- [ ] 5.5 更新 readable-view tests，验证 outline `/frontmatter/content` optional block 的 byte length、payload 还原和 header 字段。
- [ ] 5.6 更新 protocol/readable/MCP examples，展示 inline outline frontmatter、ref mode entry、hidden mode omission 和无 frontmatter 兼容 shape。
- [ ] 5.7 更新 MCP mapping tests，确认 MCP 只承载 `document_outline` structuredContent 中的 frontmatter metadata，不解析 YAML。

## 6. 验证与收尾

- [ ] 6.1 运行 Rust 格式化和相关 crate 测试，至少覆盖 `docnav-protocol`、`docnav-readable`、`docnav-output` 和 `docnav-markdown`。
- [ ] 6.2 运行 schema/example 验证，确认 outline protocol、readable、MCP 示例和 Markdown config 示例都通过更新后的 schema。
- [ ] 6.3 对涉及 adapter、schema/example、readable renderer 和 docs 边界的最终改动运行 `bun run verify:docnav-workspace`，除非有明确、记录在最终说明中的环境阻塞。
- [ ] 6.4 使用局部 diff 确认实现只改动 frontmatter outline mode、optional block 相关代码、文档、示例、测试和本 change artifacts。
- [ ] 6.5 在所有实现任务和验证任务完成后，再运行 `openspec validate markdown-frontmatter-outline-mode --type change --strict --no-interactive` 并准备归档评估。
