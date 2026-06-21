本 tasks 记录 Markdown heading canonical ref 使用 line + level 结构坐标的实施、验证和归档前审计工作。

## 1. 阻塞级审计

- [x] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“Markdown canonical heading ref 使用 `H:L{line}:H{level}`”这一核心目标。
- [x] 1.2 审计 capability ID 是否正确复用 `markdown-navigation`，且没有创建一次性、同义或过宽 capability。
- [x] 1.3 实现前审计 proposal、design、specs 和 tasks 的初始范围是否限定在本 change artifact。
- [x] 1.4 审计 proposal、design 和 specs 是否明确共享 ref opaque pass-through 契约保持稳定，Markdown grammar 变化由 Markdown adapter 拥有。
- [x] 1.5 确认 1.1-1.4 全部完成后，再执行实现任务、主规范更新、示例更新或代码改动。

## 2. 规范与验证材料同步

- [x] 2.1 按 `docs/navigation.md` 读取 `docs/CODING_STYLE.md`、`docs/adapters/markdown.md`、`docs/ref-contract.md`、`docs/protocol.md`、`docs/testing.md` 和 `docs/testing/case-maintenance.md` 中与本 change 相关的 owner 规则。
- [x] 2.2 更新 `docs/adapters/markdown.md` 的 Heading Ref Grammar、Read、错误分类、Display 职责、结构快照语义和验证入口，把 canonical heading ref 改为 `H:L{line}:H{level}`。
- [x] 2.3 确认 `docs/ref-contract.md` 和共享协议文档保持 opaque pass-through 描述；如需要调整示例或交叉引用，按 owner 边界做最小更新。
- [x] 2.4 更新 `docs/examples/json/**`、MCP 示例和 readable/protocol 示例中出现的 Markdown heading ref 字符串。
- [x] 2.5 如测试函数、case 归属或公开验证目标发生变化，按 `docs/testing/case-maintenance.md` 更新测试用例账本和源码 `@case` 标记。

## 3. Markdown Adapter 实现

- [x] 3.1 更新 `crates/docnav-markdown` 的 heading ref 生成逻辑，使 outline 和 find 输出 `H:L{line}:H{level}`。
- [x] 3.2 更新 heading ref parser，使合法 Markdown ref 集合包含 `H:L{line}:H{level}` 和现有其它 Markdown ref，并保持 canonical 十进制正整数和 level `1`-`6` 约束。
- [x] 3.3 更新 read ref 匹配逻辑，使当前解析结果中的 line 和 level 成为 heading 身份输入。
- [x] 3.4 更新 `REF_INVALID` reason 文案和相关错误映射，确保当前合法 Markdown ref grammar 之外的非空 ref 映射为 `REF_INVALID`。
- [x] 3.5 清理当前 grammar 用不到的 helper、注释或断言，并保持改动范围聚焦本 change。

## 4. 测试与示例覆盖

- [x] 4.1 更新 Markdown ref 单元测试，覆盖 canonical grammar、canonical decimal 约束、level 范围和 grammar 外输入的 `REF_INVALID`。
- [x] 4.2 更新 adapter roundtrip 测试，证明 outline/find 返回的 canonical ref 可被 read 读取对应 Markdown section。
- [x] 4.3 更新错误边界测试，证明合法 `H:L{line}:H{level}` 缺少匹配项时返回 `REF_NOT_FOUND`，grammar 外输入返回 `REF_INVALID`。
- [x] 4.4 更新 CLI smoke、readable-json、protocol-json 和 MCP 示例断言，确保所有公开输出中的 Markdown heading ref 使用当前 canonical grammar。
- [x] 4.5 更新分页或字符预算相关断言时，确认 ref 完整保留和 display 截断行为仍有覆盖。

## 5. 验证与收尾

- [x] 5.1 运行 Rust 格式化和 Markdown adapter 范围测试，至少覆盖 `crates/docnav-markdown` 的单元、adapter 和 CLI smoke 路径。
- [x] 5.2 运行 schema/example 验证，确认 readable/protocol/MCP 示例与当前 schema 和 canonical ref 字符串一致。
- [x] 5.3 对涉及 CLI、adapter、schema/example 和 docs 边界的最终改动运行 `pnpm run verify:docnav-workspace`。
- [x] 5.4 使用局部 diff 确认实现范围聚焦 Markdown ref grammar 相关代码、文档、示例、测试和本 change artifacts。
- [x] 5.5 在所有实现任务和验证任务完成后，再运行 `openspec validate remove-markdown-heading-ref-index --type change --strict --no-interactive` 并准备归档评估。
