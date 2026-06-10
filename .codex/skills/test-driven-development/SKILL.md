---
name: test-driven-development
description: "用 TDD 为行为变更和 bug fix 提供可执行证明。用于先写失败 proof、选择合适测试层级、证明 public contract、output shape、schema/example、pagination/continuation 或 integration behavior。"
---

# 测试驱动开发

把测试作为行为的可执行证明。行为变更先写会失败的 test、fixture 或 smoke command，再实现；bug fix 先复现报告中的失败，再尝试修复。

## TDD 循环

1. **命名 contract。** 判断行为属于 parser/domain logic、CLI/API surface、machine/readable output、schema/example fixtures、bridge mapping、pagination/continuation，还是普通内部实现。
2. **RED:** 写出因预期原因失败的最小 proof。对于 bug，失败必须匹配报告。
3. **GREEN:** 只做让 proof 变绿所需的最小 production change。不要为了满足测试而扩大行为、隐藏 warnings 或削弱 assertions。
4. **REFACTOR:** 只有在 focused proof 保持绿色时才清理代码；有意义的编辑后重跑受影响命令。

如果 RED proof 在修复前已经通过，就收紧 setup 或 assertion，直到它能证明缺失行为；或者说明该行为已被覆盖。

## 证明目标

选择拥有该 contract 的最窄 proof。不要因为仓库有更深的验证矩阵，就把普通文档、注释、局部 helper 或无行为 refactor 拉进完整 contract 验证。

- **Pure logic:** 用 unit test 覆盖输入、输出、边界条件和错误分支。
- **Parsing/navigation/domain behavior:** 用最小 fixture 证明 stable identifier、selected region、pagination 或 matching behavior。
- **CLI/API behavior:** 用 integration/smoke proof 覆盖 arguments、defaults、exit behavior、stdout/stderr 和 user-visible errors。
- **Machine/readable output:** 当 field shape、warning/error envelope 或 output mode 变化时，验证 schema、example 或 golden fixture。
- **Bridge behavior:** 证明 tool/API call 只映射到 owning implementation，并验证 argument/result wrapping；不要在 bridge 层复制 parser 或 router expectations。
- **Cross-boundary behavior:** 只在变更实际跨越多个 owner boundary 时扩展到 workspace-level verification。

## 选择验证范围

从窄范围开始，只在 blast radius 变大时扩展：

- 可隔离的 parser、identifier、pagination math、data transformation 或 error mapping 用 unit tests。
- CLI/API、configuration、output mode、warnings 或 integration behavior 改动用对应 smoke/integration tests。
- Machine JSON、readable JSON、fixtures 或 documentation examples 变化时，用 schema/example validation。
- 只有改 bridge mapping、tool output 或 subprocess behavior 时，才用 bridge tests。
- 跨语言/runtime、schemas、examples、docs 或 output contracts 的跨边界变更，最终交付前运行仓库约定的 workspace verification；大范围 refactor 或窄检查无法界定风险时也运行它。

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
- 项目 validation ownership 从 [docs/navigation.md](../../../docs/navigation.md) 进入；只有触碰对应 contract boundary 时才读取相关主规范。

## 完成检查

交付前：

- [ ] 已先观察到 RED proof，或说明为什么无法做到。
- [ ] 新增或变更行为已在 owning boundary 覆盖。
- [ ] Navigation/domain changes 已证明完整 user path 或 equivalent observable path。
- [ ] Pagination/continuation changes 已用返回 metadata 证明前进和终止行为。
- [ ] Machine output、readable output、schema、example 和 bridge expectations 只在 contract 变化时同步更新。
- [ ] 最小相关验证已通过。
- [ ] 已运行 workspace verification，或给出 narrow-scope 跳过理由。
- [ ] 没有为了让 suite 通过而跳过、禁用或削弱 tests。
