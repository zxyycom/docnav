---
name: doubt-driven-development
description: "对高风险 Docnav 决策进行 bounded adversarial check。用于 protocol、schema、ref、adapter contract、CLI/MCP output、irreversible migration、security-sensitive logic，或 unprovable correctness claims。"
---

# 怀疑驱动开发（Doubt-Driven Development）

## 用途

当一个 Docnav decision 足够关键，而 confidence 不能当作 evidence 时，使用本技能。它是一个 bounded challenge：把 risky claim 写成可观察 contract，再主动寻找 artifact 违反 contract 的方式，最后处理 findings 并在明确条件下停止。

本技能不替代 `api-and-interface-design`、OpenSpec skills、`test-driven-development` 或 `code-review-and-quality`。它只在这些工作中出现高风险 claim 时提供 adversarial pass。

## 触发条件

只对高风险 Docnav 决策使用：

- protocol、schema、serialized output 或 example meaning 变化。
- ref generation、parsing、stability、compatibility、pagination 或 continuation 变化。
- adapter contract 变化，包括 outline/read/find behavior、ordering、limits 和 paging semantics。
- CLI/MCP output behavior、error mapping 或 user-visible compatibility。
- irreversible migration、persisted data 影响或 downgrade/rollback 风险。
- security-sensitive input、path handling、external command execution 或 untrusted document behavior。
- compiler/tests 只能部分证明的 claims，例如 compatibility、idempotence、ordering 或 "不会破坏现有 consumers"。

Mechanical edits、formatting、直接的 documentation cleanup、明确的 OpenSpec task execution 和普通 code review 使用对应技能；只有其中的 risky decision 需要 bounded challenge 时才触发本技能。

## 最小流程

1. **Claim**: 用一到两句话写出必须为真的 decision，以及它为什么重要。
2. **Contract**: 列出可观察 requirements、compatibility promises、security/migration constraints、edge cases 和 out-of-scope。
3. **Artifact**: 锁定最小 diff、design note、schema fragment、CLI/MCP output sample 或 behavior description。
4. **Evidence gate**: 从 `docs/navigation.md` role path、protocol/schema/ref/adapter spec、CodeGraph、tests 或 command output 取得 contract evidence。
5. **Bounded challenge**: 用 [doubt-cycle.md](references/doubt-cycle.md) 的 checklist 尝试证明 artifact 不满足 contract。
6. **Reconcile**: 将每个 finding 分类为 contract gap、valid issue、accepted trade-off 或 noise，并更新 artifact、contract 或 validation。
7. **Stop**: 满足 stop condition 后结束；若仍有 substantive unresolved risk，直接暴露风险和下一步。

## Docnav 风险面

优先挑战这些 contract surfaces：

- raw protocol shape、schema meaning、example compatibility。
- readable CLI output 与 MCP tool mapping 的用户可见行为。
- adapter-owned refs、routing boundaries、pagination 和 continuation semantics。
- path handling、external commands、untrusted document input。
- persisted refs/configs、old examples、old MCP callers 和 downstream consumers。

按 [risk-map.md](references/risk-map.md) 选择要检查的 surface；不要把所有风险面默认加载进上下文。

## Fresh-Context Reviewer

独立 reviewer 是可选的，不是默认路径。只有用户授权对应 tool 或 worker 时才使用。

给 reviewer 的 packet 只包含 `ARTIFACT` 和 `CONTRACT`，必要时附最小 source refs。避免传递 claim、个人 reasoning 或无关 session history。Reviewer output 是 evidence，不是 verdict；按 reconcile taxonomy 逐项处理。

可复用 prompt 和 output contract 见 [reviewer-prompts.md](references/reviewer-prompts.md)。

## References

- [doubt-cycle.md](references/doubt-cycle.md): claim/contract/artifact 模板、adversarial checklist、finding taxonomy、stop conditions。
- [risk-map.md](references/risk-map.md): Docnav protocol、schema、ref、adapter contract、CLI/MCP output、security 风险面。
- [reviewer-prompts.md](references/reviewer-prompts.md): fresh-context reviewer prompt、最小 packet 和结果格式。

## 输出形状

当本技能影响用户需要理解的 high-risk decision 时，使用短记录：

```text
Claim: ...
Contract checked: ...
Findings: valid issue / contract gap / trade-off / noise / none
Action taken: ...
Stop condition: ...
```

Routine implementation updates 不需要输出 process log。

## 验证

完成前确认：

- Claim 可被 artifact 和 contract 检查。
- Contract evidence 来自 role path、spec、CodeGraph、tests 或 command output。
- Findings 已逐项分类，并对应 action 或 accepted trade-off。
- Stop condition 明确；未解决风险已暴露。
