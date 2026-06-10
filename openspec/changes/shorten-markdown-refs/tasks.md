本 tasks 仅为 `shorten-markdown-refs` change 的未审核临时文档，核心目标是把 Markdown adapter 的 heading ref 完全迁移为短标识，并用阻塞级审计防止范围漂移。

本 change 只在 `openspec/changes/shorten-markdown-refs/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级审计门禁

- [ ] 1.1 阻塞级审计：在执行任何实现任务前，审计 proposal、design、tasks 和 delta spec 是否都围绕“Markdown adapter ref 完全迁移为短标识”这一核心句，没有引入 `--like`、fuzzy resolve、legacy alias 或 core/MCP 侧 ref 解析。
- [ ] 1.2 阻塞级审计：确认当前 change 只包含 `openspec/changes/shorten-markdown-refs/` 下的未审核临时 artifacts，没有修改或影响现有主规范、docs、schema、examples、代码或其它 change；审计未完成前不得执行 2.x 及后续任何实现任务。

## 2. Markdown ref 生成与解析

- [ ] 2.1 实现 Markdown heading canonical ref 生成格式 `H{line}:{token}`，其中 token 由 canonical heading breadcrumb 和 occurrence ordinal 派生，并在当前文档内冲突时扩展到唯一。
- [ ] 2.2 将全文 fallback ref 完全迁移为 `D`，并移除 `doc:full` 的生成和解析。
- [ ] 2.3 更新 Markdown read 解析逻辑，只接受 `H{line}:{token}` 和 `D`，并对旧 `L{line}:{path}`、`L{line}#{ordinal}:{path}`、`L{line}#1:{path}` 和 `doc:full` 返回稳定 ref 错误。
- [ ] 2.4 确认 `outline` 和 `find` 统一使用新短 ref，且 `find` 返回的 match ref 可被 `read` 原样消费。

## 3. Contract Materials

- [ ] 3.1 更新 `docs/refs.md`、`docs/cli.md`、`docs/protocol.md` 或 `docs/adapter-contract.md` 中涉及 Markdown 示例或 ref 迁移边界的内容，保持 core/MCP 仍只原样传递 ref。
- [ ] 3.2 更新 docs schema/example/golden output 中所有旧 Markdown heading ref 和旧全文 ref 示例，明确该 change 是 breaking migration。
- [ ] 3.3 全仓搜索旧 ref 示例模式，确认 `L{line}:{path}`、`L{line}#{ordinal}:{path}`、显式 `#1` 旧 ref 和 `doc:full` 不再作为有效 Markdown ref 出现在验收材料中。

## 4. Tests

- [ ] 4.1 更新 Markdown adapter 单元测试，覆盖短 heading ref、重复 heading 唯一 token、全文 `D`、旧 heading ref 拒绝和旧全文 ref 拒绝。
- [ ] 4.2 更新 Markdown CLI smoke 测试和 fixture/golden output，覆盖 `outline -> ref -> read`、`find -> ref -> read` 和无 heading fallback `D`。
- [ ] 4.3 更新负向 CLI 矩阵，断言旧 `L...` heading ref、显式旧 `#1` ref 和 `doc:full` 返回稳定 ref 错误。

## 5. Verification

- [ ] 5.1 运行 Markdown adapter 相关 Rust 测试和 CLI smoke，确认短 ref 行为通过。
- [ ] 5.2 运行 docs/schema/example 相关验证，确认旧 ref 示例没有残留导致校验失败。
- [ ] 5.3 运行 `pnpm run verify:docnav-workspace`，并在无法运行时记录原因、失败范围和后续补救。
- [ ] 5.4 用局部 diff 审查确认实现只覆盖本 change 目标范围，未引入 `--like` 或 legacy compatibility。
