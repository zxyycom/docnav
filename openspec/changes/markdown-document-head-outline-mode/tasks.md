本 tasks 只在 `openspec/changes/markdown-document-head-outline-mode/` 下形成未审核临时文档，执行前必须先完成 document head 方案审计门禁。

## 1. 阻塞级审计门禁

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“Markdown structured outline 暴露 document head 可读区域，而不是新增 outline metadata 字段或整篇非结构化读取”这一核心目标。
- [ ] 1.2 审计 capability ID 是否正确只复用 `markdown-navigation`，且没有创建一次性、同义或过宽 capability。
- [ ] 1.3 审计当前 change 是否只包含 `openspec/changes/markdown-document-head-outline-mode/` 下的未审核临时 artifacts，且没有修改现有 specs、docs、schemas、examples、测试或实现代码。
- [ ] 1.4 审计本 change 与 `markdown-frontmatter-outline-mode` 的重叠关系，并在进入实现前明确后者是被 supersede、合并还是继续保留非重叠范围。
- [ ] 1.5 在 1.1-1.4 全部完成前，不得执行任何实现任务、主规范更新、示例更新、测试更新或代码改动。

## 2. 主规范与验证材料同步

- [ ] 2.1 更新 `docs/adapters/markdown.md`，记录 document head 范围、`document_head_outline_mode`、`frontmatter_block_policy`、`HEAD:leading`、`FM:L{line}`、`P:preamble`、`doc:full` fallback 和 find 映射。
- [ ] 2.2 更新 `docs/schemas/docnav-markdown-config.schema.json` 和 `docs/examples/json/docnav-markdown-config.json`，加入 document head 和 frontmatter block policy options，并确保只声明 adapter owner 已接受的 native option values。
- [ ] 2.3 更新 Markdown adapter 测试边界说明，明确 combined/split/hidden、opening-only、多 metadata block、未闭合 delimiter、horizontal rule 和 Unicode 分页证明目标。

## 3. Markdown Adapter 实现

- [ ] 3.1 在 Markdown parser/document model 中计算 document head 原文范围，并保持 heading model、heading line/level ref 和 section 范围不变。
- [ ] 3.2 实现 frontmatter block classification，默认 `opening_only`，并为显式多 metadata block policy 留出清晰、可测试的识别边界。
- [ ] 3.3 接入 adapter-owned native option sources `document_head_outline_mode` 和 `frontmatter_block_policy`，并让 direct CLI/config 合并后传入 outline/find/read 语义；未声明 public input 必须被拒绝。
- [ ] 3.4 更新 outline construction：combined 返回 `HEAD:leading`，split 返回 `FM:L{line}` 和 `P:preamble`，hidden 不返回 document head entry，且无可见 heading 时保留 `doc:full` fallback。
- [ ] 3.5 更新 read ref handling：`HEAD:leading` 返回 document head 原文，`FM:L{line}` 返回 delimiter 内部 YAML payload，`P:preamble` 返回 frontmatter 后的前导 Markdown 原文。
- [ ] 3.6 更新 find ref selection：document head 命中在 combined/split 模式下返回可 read 的 head region ref，hidden 或 fallback 场景保持可读行为。

## 4. 测试覆盖

- [ ] 4.1 增加 Markdown unit/adapter tests，覆盖 combined、split、hidden、空 document head、只有 frontmatter、只有 preamble、frontmatter+preamble 和 no visible heading fallback。
- [ ] 4.2 增加 frontmatter policy tests，覆盖 opening-only、多 metadata block 显式模式、未闭合 delimiter、普通 horizontal rule 和 frontmatter 伪 heading 排除。
- [ ] 4.3 增加 read/find roundtrip tests，验证 `HEAD:leading`、`FM:L{line}`、`P:preamble` 的 content、content_type、page 和 Unicode 分页边界。
- [ ] 4.4 更新 Markdown CLI smoke fixture，验证 readable-json、readable-view 和 protocol-json 下 document head entries 与 read content block 行为。

## 5. 验证与收尾

- [ ] 5.1 运行 Rust 格式化和相关 crate 测试，至少覆盖 `docnav-markdown` 以及受配置/schema 影响的 crate。
- [ ] 5.2 运行 schema/example 校验，证明 Markdown config 示例包含的新 options 符合 schema。
- [ ] 5.3 对涉及 adapter、schema/example、readable output 和 docs 边界的最终改动运行 `bun run verify:docnav-workspace`，除非有明确、记录在最终说明中的环境阻塞。
- [ ] 5.4 使用局部 diff 确认实现只改动 document head outline mode 相关代码、文档、示例、测试和本 change artifacts。
- [ ] 5.5 在所有实现任务和验证任务完成后，运行 `openspec validate markdown-document-head-outline-mode --type change --strict --no-interactive` 并准备归档评估。
