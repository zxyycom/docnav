---
name: test-driven-development
description: "用 TDD 为 Docnav 行为变更和 bug fix 提供可执行证明。用于证明 adapter protocol behavior、`outline -> ref -> read`、CLI raw/readable modes、schemas/examples、MCP mapping、pagination/continuation、smoke tests，并选择 narrow 或 workspace verification。"
---

# 测试驱动开发

把测试作为 Docnav 行为的可执行证明。行为变更先写会失败的 test、fixture 或 smoke command，再实现；bug fix 先复现报告中的失败，再尝试修复。

## TDD 循环

1. **命名 contract。** 判断行为属于 Markdown adapter、core `docnav` CLI、raw protocol、readable output、schema/example fixtures、MCP bridge mapping，还是 pagination/continuation。
2. **RED:** 写出因预期原因失败的最小 proof。对于 bug，失败必须匹配报告。
3. **GREEN:** 只做让 proof 变绿所需的最小 production change。不要为了满足测试而扩大行为、隐藏 warnings 或削弱 assertions。
4. **REFACTOR:** 只有在 focused proof 保持绿色时才清理代码；有意义的编辑后重跑受影响命令。

如果 RED proof 在修复前已经通过，就收紧 setup 或 assertion，直到它能证明缺失行为；或者说明该行为已被覆盖。

## Docnav 证明目标

选择拥有该 contract 的 proof：

- **Markdown adapter:** 验证 flat outline entries、稳定的 `ref` 和 `display`、通过 adapter ref 执行 `read`、被触碰时的 `find`/`info`、duplicate headings、frontmatter/code-fence exclusion，以及稳定的 no-match 或 multi-match errors。
- **Navigation behavior:** 覆盖完整 `outline -> ref -> read` path；当 `page` 可能非 null 时，覆盖 page continuation。
- **Pagination:** 断言 `page` 从 1 开始、用相同 semantic parameters 前进、结尾返回 `null`，并按 Unicode character budget 遵守 `limit_chars`。
- **Protocol and output modes:** 在被改动层覆盖 adapter `invoke`、`docnav --output protocol-json`、default readable text 和 `docnav --output readable-json`。
- **Schemas and examples:** 每当 protocol 或 readable field shape 变化时，验证匹配的 schema、example 或 fixture updates。
- **Core CLI integration:** 验证 adapter selection、config precedence、explicit invoke parameters、warning boundaries 和 unchanged ref pass-through。
- **MCP bridge:** 证明 tool calls 映射到 core `docnav` CLI，且 readable results 变为 TextContent/structuredContent；不要复制 parsing、routing 或 protocol envelopes。

## 选择验证范围

从窄范围开始，只在 blast radius 变大时扩展：

- 可隔离的 parser、ref、pagination math、data transformation 或 error mapping 用 unit tests。
- Adapter 行为用 adapter CLI smokes，尤其是直接 `outline`、捕获的 `ref`，以及对该 ref 的 `read`。
- Routing、configuration、output mode mapping、warnings 或 cross-adapter integration 改动用 core `docnav` CLI smokes。
- Raw protocol、readable JSON、fixtures 或 documentation examples 变化时，用 schema/example validation。
- 只有改 `docnav-mcp` mapping、tool output 或 bridge subprocess behavior 时，才用 MCP tests。
- 跨 Rust、Node/MCP、schemas、examples、docs 或 output contracts 的跨边界变更，最终交付前运行 `pnpm run verify:docnav-workspace`；大范围 refactor 或窄检查无法界定风险时也运行它。

不要为了安心重复运行未变化且已通过的命令。只有在编辑可能影响结果后重跑；当变更跨边界时再扩展验证。

## 修复流程（Bug Fixes）

使用 Prove-It Pattern：

1. 用最小 failing test、fixture 或 command 重建 bug。
2. 确认 failure text 或 assertion 与报告匹配。
3. 实现修复。
4. 确认原始复现现在通过。
5. 运行被触碰边界所需的下一层更宽验证。

## 可选运行时检查

Browser verification 是可选项，只与 browser-facing changes 有关。测试通过后，验证 local page path，检查 console/network/DOM 证据；只有视觉行为变化时才截图。

## 可选独立评审

对于非平凡行为，如果明确可用且已授权，可以使用单独 reviewer 或 worker。请他们按 contract review failing proof 和 final diff；不要为了等待评审阻塞正常 TDD work。

## 参考资料

- 使用 [testing-patterns.md](references/testing-patterns.md) 查看通用 test structure、assertions、mocking、component/API/E2E patterns 和 anti-patterns。
- 当前 Docnav validation ownership 从 [docs/navigation.md](../../../docs/navigation.md) 进入，再读取相关主规范。

## 完成检查

交付前：

- [ ] 已先观察到 RED proof，或说明为什么无法做到。
- [ ] 新增或变更行为已在 owning boundary 覆盖。
- [ ] Navigation changes 已证明 `outline -> ref -> read`。
- [ ] Pagination changes 已用返回的 `page` 证明 continuation。
- [ ] Raw protocol、readable output、schema、example 和 MCP expectations 在 contract 变化时已同步更新。
- [ ] 最小相关验证已通过。
- [ ] 已运行 workspace verification，或给出 narrow-scope 跳过理由。
- [ ] 没有为了让 suite 通过而跳过、禁用或削弱 tests。
