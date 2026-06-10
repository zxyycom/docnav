---
name: code-simplification
description: 简化代码以提升 clarity/readability。Use when refactoring code for clarity without changing behavior, reviewing accumulated complexity, or reducing unnecessary complexity in working code.
---

# Code Simplification

> Inspired by the [Claude Code Simplifier plugin](https://github.com/anthropics/claude-plugins-official/blob/main/plugins/code-simplifier/agents/code-simplifier.md). 本技能面向任意 AI coding agent，强调在不改变行为的前提下让代码更易读、易改、易调试。

## 何时使用

使用本技能：

- 功能已经可用、测试通过，但实现比需要的更重。
- review 中指出 readability、复杂度、重复或命名问题。
- 遇到深层嵌套、长函数、含糊命名、重复条件或分散逻辑。
- 重构赶工写出的代码，或合并后出现不一致和重复。
- 用户明确要求 simplify、refactor for clarity、reduce complexity、cleanup。

不要使用：

- 代码已经清晰，只是想按个人偏好改写。
- 尚未理解代码责任、调用方、边界条件和测试。
- “更简单”的写法会让性能关键路径变慢，且没有度量支撑。
- 模块马上会被整体替换，当前整理没有实际收益。

## 核心原则

- **行为完全不变**：输入、输出、副作用、错误行为、顺序和边界条件必须一致；现有测试不应因简化而改。
- **跟随项目约定**：先读 `AGENTS.md` 和相邻代码，再匹配 import、命名、错误处理、类型深度和模块风格。
- **清晰优先于聪明**：显式控制流、命名良好的中间变量和小函数，通常胜过压缩到一行的链式表达式。
- **避免过度简化**：不要为了少几行删除有概念名称的 helper、合并无关逻辑或移除仍有测试性和扩展性价值的抽象。
- **范围贴近任务**：优先整理最近修改或用户指定的代码；不要做顺手的大范围 churn。

详细原则见 [simplification-principles.md](references/simplification-principles.md)。

## Docnav 约束

在本仓库简化代码时，额外保持这些边界：

- 保持 CLI-first 导航契约：`outline -> ref -> read`。
- `ref` 由 adapter 生成和解析，core、MCP 和其他接入层只原样传递。
- raw protocol output 与 readable output 分层清晰，不为了复用而混合传输包装。
- 修改 Rust、Node、MCP、CLI、schema、adapter 或输出模式前，先确认对应主规范和 `docs/CODING_STYLE.md`。
- 跨 adapter 边界、协议 shape、`ref` 或 output mode 的“简化”默认高风险，必须有测试证明行为未变。

## 工作流

1. **先理解**：确认代码责任、调用链、边界条件、错误路径、测试覆盖和历史原因；必要时用 CodeGraph、`rg`、`git blame`。
2. **找具体信号**：深层嵌套、长函数、重复逻辑、泛化命名、误导性命名、无价值 wrapper、死代码、重复类型断言。
3. **一次改一类问题**：小步应用，避免把重命名、结构调整和行为修复混在一起；超过约 500 行的机械重构应考虑 codemod、脚本或 AST 工具。
4. **马上验证**：运行最小相关测试、build、formatter、linter；失败时先定位是哪一步改变了行为。
5. **回看 diff**：确认 diff 可 review、没有无关改动、没有削弱错误处理、没有留下 unused import 或 unreachable branch。

完整流程和检查表见 [process-checklist.md](references/process-checklist.md)。

## 语言提示

- TypeScript / JavaScript：删除无意义 `await` 包装、用清楚的 `filter` / `map` 代替手写循环，但不要把复杂规则塞进链式调用。
- Python：可用 comprehension 简化直接映射；复杂校验优先 guard clause，保持异常类型和顺序。
- React / JSX：合并重复 render 分支时保持 props、variant 和可访问性语义；prop drilling 是否改为 context 属于设计判断，不自动重构。

示例见 [language-guidance.md](references/language-guidance.md)。

## 快速自检

- 我能说明这段代码为什么存在，以及为什么现在可以简化。
- 每个改动都只改变表达方式，不改变行为。
- 新名字比旧名字更贴近领域含义，而不是个人偏好。
- 删除的抽象、分支或注释已经确认没有仍在表达必要意图。
- 简化后的代码符合相邻代码风格。
- diff 聚焦，reviewer 能快速确认收益和风险。

## 验证

完成一次简化后：

- 现有测试无需修改即可通过。
- build 无新增 warning。
- formatter / linter 通过。
- diff 只包含目标范围。
- 错误处理、边界条件、副作用顺序未被削弱。
- 对 Docnav 改动，按范围运行相关 CLI、adapter、schema、MCP 或 workspace 验证；跨层改动优先 `pnpm run verify:docnav-workspace`。
