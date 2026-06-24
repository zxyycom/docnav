# 审查清单（Review Checklist）

仅在需要正式 checklist、second-pass review 或可复用 review prompt 时使用此参考。主 `SKILL.md` 仍是 review 行为与输出形状的权威来源。

## 紧凑工作表（Compact Worksheet）

```markdown
## Context（上下文）
- [ ] 我理解 intended behavior 与 touched surfaces。
- [ ] 对 Docnav，我知道 governing role spec 或 OpenSpec change。

## Correctness（正确性）
- [ ] 改动符合 task/spec。
- [ ] Edge cases 与 error paths 已处理。
- [ ] Touched surface 有验证证据：tests、fixture、schema/example validation、docs sync、smoke command 或 reproduction note，足以证明声明的 contract 或 observable semantics。
- [ ] 适用时，`outline -> ref -> read` 仍完整。

## Architecture（架构）
- [ ] 现有 patterns 与 ownership boundaries 保持不变。
- [ ] Adapter-owned refs 没有在 adapter 外被解析或重写。
- [ ] Raw protocol 与 readable output contracts 保持分离。

## Security and Performance（安全与性能）
- [ ] 没有 secret 泄露、injection path、unchecked untrusted data 或 trust boundary 混淆。
- [ ] 没有 N+1、unbounded operation、意外 full-document load 或可避免的 hot-path cost。
- [ ] 新 dependency 有必要性说明，并经过 supply-chain 检查。

## Verification（验证）
- [ ] 验证 touched surface 的最小 tests/build/smoke/schema/docs checks 已通过。
- [ ] 适用时，schema、examples、specs 与 OpenSpec artifacts 已同步。
- [ ] 已按改动范围运行必要 Docnav verification command。

## Verdict（结论）
- [ ] Approve
- [ ] Request changes
```

## 可选独立复核（Optional Independent Pass）

只有在可用且被授权时，才使用第二 reviewer 或 model。它补充常规 review，不能替代自己的 findings-first pass。

```text
请审查这个 change 的 correctness、security、performance、maintainability
以及 Docnav contract adherence。检查 `outline -> ref -> read` flow、
mapping、schema/example/spec sync、适用时的 OpenSpec consistency，
以及 required verification。

请 findings first 返回结果，按 severity 排序，并带 file/line references。
```

## 行动线索（Action Cues）

- Review 结论带上已审查 scope、证据和 verdict。
- Bug fix 新增或改变 stable observable semantics 时，验证证据证明修正后的行为。
- Security-sensitive change 包含 security-focused review。
- Diff scope 足够聚焦，reviewer 可以可靠追踪 touched surfaces。
- Review comment 带 severity 和明确 author action。
- Deferred cleanup 有 owner、reason 和 follow-up 条件。
