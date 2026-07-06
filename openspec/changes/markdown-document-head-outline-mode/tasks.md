本 tasks 从强制暴露的合并 document head 方案开始执行；任务边界限定为 Markdown adapter 的 structured outline/read/find、文档和验证材料。

## 1. 主规范与验证材料同步

- [ ] 1.1 更新 `docs/adapters/markdown.md`，记录 document head 范围、非空 entry 条件、`HEAD:leading`、`doc:full` fallback、read 语义和 find 映射。
- [ ] 1.2 在 Markdown adapter 文档中固定 raw entry facts：document head entry 使用非空 `label`、非 heading `kind`、`location.line_start` 和必要 metadata；不得把 readable-only `display` 写成 raw protocol contract。
- [ ] 1.3 更新 `docs/testing.md`、`docs/testing/cases.md` 或对应 Markdown adapter 测试边界说明，明确 always exposed when eligible、空或纯空白 document head、frontmatter 原文保留、find-to-read roundtrip 和 Unicode 分页证明目标。

## 2. Markdown Adapter 实现

- [ ] 2.1 在 Markdown parser/document model 中计算 document head 原文范围，并保持 heading model、heading line/level ref 和 section 范围不变。
- [ ] 2.2 实现 document head entry eligibility：document head 为空或纯空白时不返回 entry；当前 structured outline 无可见 heading entry 时保留 `doc:full` fallback。
- [ ] 2.3 更新 outline construction：满足 eligibility 时返回 `HEAD:leading`，且无可见 heading时保留 `doc:full` fallback。
- [ ] 2.4 更新 read ref handling：`HEAD:leading` 返回 document head 原文，`content_type` 为 `text/markdown`。
- [ ] 2.5 更新 find ref selection：document head 命中且当前 structured outline 至少有一个可见 heading entry 时返回 `HEAD:leading`，fallback 场景保持可读 fallback 行为。

## 3. 测试覆盖

- [ ] 3.1 增加 Markdown unit/adapter tests，覆盖默认暴露、空或纯空白 document head、只有 frontmatter、只有普通前导正文、frontmatter+普通前导正文和 no visible heading fallback。
- [ ] 3.2 增加 read/find roundtrip tests，验证 `HEAD:leading` 的 content、content_type、page 和 Unicode 分页边界。
- [ ] 3.3 增加 frontmatter 原文边界 tests，覆盖 YAML delimiter 保留、frontmatter 伪 heading 排除、代码围栏伪 heading 排除和普通 horizontal rule 原文保留。
- [ ] 3.4 更新 Markdown CLI smoke fixture，验证 readable-json、readable-view 和 protocol-json 下 document head entry、raw item facts、derived display 与 read content block 行为。

## 4. 验证与收尾

- [ ] 4.1 运行 Rust 格式化和相关 crate 测试，至少覆盖 `docnav-markdown`。
- [ ] 4.2 对涉及 adapter、readable output 和 docs 边界的最终改动运行 `bun run verify:docnav-workspace`，除非有明确、记录在最终说明中的环境阻塞。
- [ ] 4.3 使用局部 diff 确认实现只改动 document head outline mode 相关代码、文档、测试和本 change artifacts。
- [ ] 4.4 在所有实现任务和验证任务完成后，运行 `openspec validate markdown-document-head-outline-mode --type change --strict --no-interactive` 并准备归档评估。
