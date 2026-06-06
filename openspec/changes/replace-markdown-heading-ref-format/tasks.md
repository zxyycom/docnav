## 1. 审查闸门

- [x] 1.1 【审查通过】已审查并确认本 change 的 proposal、design、spec delta 和 tasks；后续从文档对齐开始执行。

## 2. 先行批次 A：文档契约对齐

- [ ] 2.1 【可执行】文件 `docs/refs.md`：定义 Markdown heading ref 为 `L{line}:{path}` 和 `L{line}#{ordinal}:{path}`，并说明 heading breadcrumb、重复 path、显式 `#1` 和 `doc:full` 边界。
- [ ] 2.2 【可执行】路径 `docs/examples/`、`docs/schemas/`、`scripts/validators/`：先扫描是否存在旧 ref 示例或断言；若存在，改为 canonical heading ref 或 `doc:full`。
- [ ] 2.3 【可执行】路径 `openspec/specs/`、`openspec/changes/`：先扫描 active OpenSpec spec/task 是否存在旧 ref 文案；若存在，更新为与本 change 一致的 canonical 表述。
- [ ] 2.4 【可执行】完成文档契约自检：实现者后续查阅 README 角色路径、`docs/refs.md` 和 active OpenSpec 时，看到的 ref 规则与本 change 目标一致。

## 3. 并行批次 B：Markdown Adapter 实现

- [ ] 3.1 【等待 2.x 文档对齐完成】文件 `crates/docnav-markdown/src/markdown.rs`：更新 `heading_ref`、`ParsedRef` 和同文件模块测试；实现 `L{line}:{path}` / `L{line}#{ordinal}:{path}` 生成与解析，接受显式 `#1`，删除旧方括号 ordinal 后缀解析路径，并保留 `doc:full` fallback。

## 4. 并行批次 C：Rust Adapter 测试

- [ ] 4.1 【等待 2.x 文档对齐完成】文件 `crates/docnav-markdown/tests/adapter.rs`：更新重复 heading path 断言、missing ref 断言和分页/read 用例；新增或调整 canonical ref、显式 `#1`、旧方括号 ordinal 后缀拒绝用例。
- [ ] 4.2 【等待 2.x 文档对齐完成】文件 `crates/docnav-markdown/tests/cli.rs`：更新 text、readable JSON、protocol JSON 和 error detail 断言，使用新 ref 写法并保留 invalid ref 稳定错误校验。

## 5. 并行批次 D：CLI Smoke Cases

- [ ] 5.1 【等待 2.x 文档对齐完成】文件 `scripts/docnav-markdown-cli-smoke/cases/corpus.mjs`：更新重复 heading、BOM、CRLF fixture 的固定 ref 断言，期望包含 `L1:Repeat`、`L9#2:Repeat`、`L5:Repeat > Child` 和 `L13#2:Repeat > Child`。
- [ ] 5.2 【等待 2.x 文档对齐完成】文件 `scripts/docnav-markdown-cli-smoke/cases/operation-errors.mjs`：更新 invalid ref 输入和 error detail 断言，覆盖旧方括号 ordinal 后缀输入不会被接受。
- [ ] 5.3 【等待 2.x 文档对齐完成】文件 `scripts/docnav-markdown-cli-smoke/cases/text.mjs`：更新 text 输出断言，改为检查新 canonical ref 或 outline 提取出的 ref，不再依赖旧 suffix marker。
- [ ] 5.4 【等待 2.x 文档对齐完成】文件 `scripts/docnav-markdown-cli-smoke/cases/readable.mjs`、`protocol.mjs`、`fixtures.mjs`：确认这些 case 继续从 outline 提取 ref 并原样传给 read；只有发现硬编码旧 ref 时才修改。

## 6. 合并后扫描与清理

- [ ] 6.1 【等待 3.x、4.x、5.x 完成】执行旧 suffix marker 仓库扫描，确认 `crates/`、`scripts/`、`docs/` 和 `openspec/` 中不再残留旧格式示例、断言或解析逻辑。
- [ ] 6.2 【等待 6.1 完成】按扫描结果清理剩余命中；若命中是必须保留的负向测试输入，使用组合字符串或局部构造方式表达，避免把旧 marker 固化为文档示例或正向断言。

## 7. 合并后验证

- [ ] 7.1 【等待 6.x 完成】运行 `cargo test -p docnav-markdown`。
- [ ] 7.2 【等待 6.x 完成】运行 `pnpm run smoke:docnav-markdown`。
- [ ] 7.3 【等待 6.x 完成】运行 `pnpm run validate:openspec`。
- [ ] 7.4 【等待 6.x 完成】运行 `pnpm run verify:docnav-workspace`。
- [ ] 7.5 【等待 7.1-7.4 完成】做局部 diff review，确认只修改 Markdown ref 格式、相关测试/fixtures、文档和本 change 范围。
