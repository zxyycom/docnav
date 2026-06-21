本 tasks 定义移除 Markdown heading ref `I{index}` 字段的实现入口；它只在 `openspec/changes/remove-markdown-heading-ref-index/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级审计

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“Markdown canonical heading ref 从 `H:L{line}:H{level}:I{index}` 改为 `H:L{line}:H{level}`”这一核心目标。
- [ ] 1.2 审计 capability ID 是否正确复用 `markdown-navigation`，且没有创建一次性、同义或过宽 capability。
- [ ] 1.3 审计当前 change 是否只包含 `openspec/changes/remove-markdown-heading-ref-index/` 下的未审核临时 artifacts，且没有修改现有 specs、docs、schemas、examples 或实现代码。
- [ ] 1.4 审计 proposal、design 和 specs 是否明确共享 ref opaque pass-through 契约不变，Markdown grammar 变化只由 Markdown adapter 拥有。
- [ ] 1.5 在 1.1-1.4 全部完成前，不得执行任何实现任务、主规范更新、示例更新、测试更新或代码改动。

## 2. 规范与验证材料同步

- [ ] 2.1 按 `docs/navigation.md` 读取 `docs/CODING_STYLE.md`、`docs/adapters/markdown.md`、`docs/ref-contract.md`、`docs/protocol.md`、`docs/testing.md` 和 `docs/testing/case-maintenance.md` 中与本 change 相关的 owner 规则。
- [ ] 2.2 更新 `docs/adapters/markdown.md` 的 Heading Ref Grammar、Read、错误分类、Display 职责、结构快照语义和验证入口，把 canonical heading ref 改为 `H:L{line}:H{level}`。
- [ ] 2.3 确认 `docs/ref-contract.md` 和共享协议文档仍只描述 opaque pass-through；如需要调整示例或交叉引用，只做 owner 边界内的最小更新。
- [ ] 2.4 更新 `docs/examples/json/**`、MCP 示例和 readable/protocol 示例中出现的 Markdown heading ref 字符串。
- [ ] 2.5 如测试函数、case 归属或公开验证目标发生变化，按 `docs/testing/case-maintenance.md` 更新测试用例账本和源码 `@case` 标记。

## 3. Markdown Adapter 实现

- [ ] 3.1 更新 `crates/docnav-markdown` 的 heading ref 生成逻辑，使 outline 和 find 输出 `H:L{line}:H{level}`。
- [ ] 3.2 更新 heading ref parser，只接受 `H:L{line}:H{level}` 和现有其它合法 Markdown ref，并保持正整数、无前导零和 level `1`-`6` 约束。
- [ ] 3.3 更新 read ref 匹配逻辑，只按当前解析结果中的 line 和 level 匹配 heading，不使用 title、breadcrumb、section 内容或全文 heading index 补充匹配。
- [ ] 3.4 更新 `REF_INVALID` reason 文案和相关错误映射，确保旧 `H:L{line}:H{level}:I{index}` 格式进入非法 grammar 路径。
- [ ] 3.5 清理因 ref 不再使用 index 而变成无意义的 helper、注释或断言，但不做与本 change 无关的重构。

## 4. 测试与示例覆盖

- [ ] 4.1 更新 Markdown ref 单元测试，覆盖新 canonical grammar、无前导零约束、level 范围和旧 `:I{index}` 格式的 `REF_INVALID`。
- [ ] 4.2 更新 adapter roundtrip 测试，证明 outline/find 返回的新 ref 可被 read 读取对应 Markdown section。
- [ ] 4.3 更新错误边界测试，证明合法 `H:L{line}:H{level}` 无匹配返回 `REF_NOT_FOUND`，旧 `H:L{line}:H{level}:I{index}` 返回 `REF_INVALID`。
- [ ] 4.4 更新 CLI smoke、readable-json、protocol-json 和 MCP 示例断言，确保所有公开输出中的 Markdown heading ref 使用新格式。
- [ ] 4.5 更新分页或字符预算相关断言时，确认 ref 完整保留且 display 截断行为不因 ref 变短而丢失覆盖。

## 5. 验证与收尾

- [ ] 5.1 运行 Rust 格式化和 Markdown adapter 范围测试，至少覆盖 `crates/docnav-markdown` 的单元、adapter 和 CLI smoke 路径。
- [ ] 5.2 运行 schema/example 验证，确认 readable/protocol/MCP 示例与当前 schema 和新 ref 字符串一致。
- [ ] 5.3 对涉及 CLI、adapter、schema/example 和 docs 边界的最终改动运行 `pnpm run verify:docnav-workspace`，除非有明确、记录在最终说明中的环境阻塞。
- [ ] 5.4 使用局部 diff 确认实现只改动 Markdown ref grammar 相关代码、文档、示例、测试和本 change artifacts。
- [ ] 5.5 在所有实现任务和验证任务完成后，再运行 `openspec validate remove-markdown-heading-ref-index --type change --strict --no-interactive` 并准备归档评估。
