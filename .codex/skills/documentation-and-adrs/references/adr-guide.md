# ADR 指南

## 何时写 ADR

ADR 是 decision receipt 和长期 rationale 索引，不是变更管理系统。默认只记录已接受的 durable decision；不要用 ADR 承担 proposal、task breakdown、spec delta 或 acceptance 流程。

满足任一条件时才写 ADR：

1. 决定长期架构方向、系统边界、ownership boundary 或 platform/runtime 选择。
2. 选择会跨多个 OpenSpec change 持续影响 protocol、schema、adapter contract、auth strategy 或 public API architecture。
3. 决策回滚成本高、迁移成本高，或会影响多个模块。
4. tradeoff 未来很可能被反复争论，需要留下稳定的 rationale 索引。
5. 单个 change 之外也需要保留 why，供后续设计、审查或回滚判断复用。

不要为显而易见的局部实现写 ADR。change-local rationale 留在 OpenSpec design 或相关 docs；当前行为承诺写进主规范；局部 gotcha 用 inline documentation；用户操作流程用 README 或 task docs。

## 存放与命名

- 默认放在 `docs/decisions/`。
- 使用顺序编号，例如 `ADR-001-short-title.md`。
- 标题使用 `# ADR-001: Short Decision Title`。
- 新 ADR 可以引用 OpenSpec change、issue、PR、schema、examples 或 source citations，但不要复制完整材料。
- ADR 默认表示当时已经接受的 durable decision；一般不需要单独的提议阶段。

## 模板

```markdown
# ADR-001: Short Decision Title

## Date
YYYY-MM-DD

## Context
说明当时的问题、约束、版本、团队条件和已有系统边界。

## Decision
用一两段写清选择了什么，以及这个选择的适用范围。

## Rationale
记录为什么长期选择这条路，特别是关键约束和 tradeoff。

## Consequences
- 正向后果：
- 成本和风险：
- 迁移或回滚影响：

## Links
- OpenSpec change、主规范、schema、examples、PR、issue 或 source citations：

## Alternatives
可选。只在替代方案会帮助未来读者理解 tradeoff 时填写。
```

## 历史演进

- 不删除旧 ADR。旧记录解释历史上下文。
- 决策变化时写新 ADR 或在相关 docs 中链接说明；必要时在新旧 ADR 中互相链接。
- ADR 默认记录已采纳的长期决策；历史演进通过 Links 或正文说明替代关系。
- OpenSpec 负责“要改变什么行为”；主规范负责“当前承诺什么行为”；ADR 只负责“为什么长期选择这条路”。

## 质量标准

ADR 合格条件：

1. 读者能看出为什么这是长期架构决策，而不是单个 change 的实现说明。
2. Decision 和 Rationale 足够短，能作为后续讨论的稳定索引。
3. 后果包含成本、风险和迁移或回滚影响。
4. Links 能指向相关 OpenSpec、主规范、schema、examples、tests 或 source citations。
5. Alternatives 只在确实能解释重要 tradeoff 时出现，不强制写完整方案对比。
