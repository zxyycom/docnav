---
name: code-review-and-quality
description: >-
  Findings-first code review for PRs、local diffs、agent handoffs
  and approval reviews. Use to verify touched surfaces have enough verification evidence
  for correctness、observable behavior、security/performance risks、maintainability, and Docnav contract issues
  involving CLI、adapter、ref、pagination、raw/readable output、schema 或 OpenSpec artifacts.
---

# 代码审查与质量（Code Review and Quality）

## 目标

审查已经完成的 diff，适用于 merge、handoff 或 approval 前的 focused review。输出必须 findings-first，按 severity 排序，并把每个需要行动的问题绑定到具体 file/line。

只有在改动符合意图、touched surface 有足够验证证据，并且没有未解决 blocking issue 时才 approve。风格意见只有在影响 correctness、maintainability 或项目约定时才升级；纯偏好保持非阻塞。

## 读取策略

默认只读本文件。按风险加载一层 reference：

1. 需要正式 worksheet、second-pass prompt 或可复用审查清单：读 [review-checklist.md](references/review-checklist.md)。
2. Docnav contract、CLI/adapter/ref/pagination/raw-readable/schema/OpenSpec 相关 diff：读 [docnav-review-cues.md](references/docnav-review-cues.md)。
3. Security-sensitive changes：读 [security-checklist.md](references/security-checklist.md)。
4. Hot path、large input、unbounded work 或已有 performance budget/report 指向的风险：读 [performance-checklist.md](references/performance-checklist.md)。

## 工作流

1. 确认 intent、changed surfaces、预期行为、governing spec 或 user request。
2. 检查 touched surface 的验证证据：tests、fixture、schema/example validation、docs sync、smoke command、reproduction note 或其他 owner-declared evidence。只有新增或改变 stable public contract、自定义 invariant、equivalence class，或当前 owner 明确承诺的 observable semantics 时，证据缺口才作为 finding。
3. 沿 ownership boundary 追踪实现：core、adapter/service、bridge/tool、schema/example、docs、tests 或 generated artifacts。
4. 优先检查 correctness、user-visible behavior、data loss/security、compatibility 和验证证据缺口，再处理 maintainability/style。
5. 检查 verification。优先要求能证明 touched surface 的最小命令；跨边界行为需要更宽验证。
6. 如果 diff 过宽导致无法可靠 review，要求拆分或提供 focused context。
7. 交付 findings-first verdict。

## 严重级别（Severity）

按以下顺序报告 findings：

| Label | 含义 | Merge 影响 |
| --- | --- | --- |
| Critical | Security vulnerability、data loss、核心行为 broken，或 public contract violation | Blocks |
| High | 很可能 user-visible break、缺失必要验证证据，或严重 maintainability risk | Blocks |
| Medium | 真实问题但影响有界，或未来成本明确 | 通常 blocks，除非明确延期 |
| Low | 小缺陷，或有 correctness/clarity 价值的局部清理 | Reviewer judgment |
| Nit / Optional / FYI | 偏好、替代方案或背景信息 | Non-blocking |

不要把 blocking bug 软化成 suggestion。也不要把 optional preference 写得像 mandatory request。

## 验证（Verification）

记录 review 了什么、运行了什么、没有运行什么。缺失必要 verification 时，把它作为 finding 或 residual risk。

Verification commands must come from the current repository scripts, nearby tests, governing docs, or the user's provided commands. Do not require hardcoded build artifact paths as reusable review rules.

## 输出形状

从 findings 开始。有问题时不要用 summary 开头。

```markdown
## Findings（问题）
- Critical: [file:line] 缺陷、影响和期望修正。
- High: [file:line] ...

## Open Questions（开放问题）
- 影响 verdict 的问题，如有。

## Verification（验证）
- Reviewed: 已审查的 files、paths 和 behavior。
- Ran: 命令和结果。
- Not run: 跳过的命令及原因。

## Verdict（结论）
Request changes / Approve / No findings.
```

如果没有 findings，要明确说明，并仍然提到 residual risk 或验证证据缺口。summary 保持简短，且放在 review result 之后。

## 参考资料（See Also）

- [Review checklist](references/review-checklist.md)：可选 worksheet 与 independent-review prompt。
- [Docnav review cues](references/docnav-review-cues.md)：Docnav contract scope 的审查线索与验证范围。
- [Security checklist](references/security-checklist.md)：更深入的 Docnav security checks。
- [Performance checklist](references/performance-checklist.md)：更深入的 Docnav performance checks。
