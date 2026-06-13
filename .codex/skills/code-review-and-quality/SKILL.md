---
name: code-review-and-quality
description: "中文优先的 Docnav findings-first code review 指南，用于 PR/local diff/agent handoff 审查，覆盖 CLI、adapter、ref、pagination、raw/readable output、schema、MCP、OpenSpec、tests、security/performance。"
---

# 代码审查与质量（Code Review and Quality）

## 用途

使用本 skill 审查已经完成的 diff，适用于 merge、handoff 或 approval 前的最后质量关。输出必须 findings-first，按 severity 排序，并把每个需要行动的问题绑定到具体 file/line。

只有在改动保持 Docnav contract、包含合适的测试或验证，并且没有未解决的 blocking issue 时才 approve。不要因为个人风格阻塞改动；如果代码符合项目惯例且风险可接受，把风格意见降级为非阻塞说明。

## 何时使用

- 审查 PR、local diff 或 agent 生成的改动。
- 检查 Docnav CLI 行为、adapter、adapter-owned refs、pagination/continuation、raw/readable output、schema、examples、MCP bridge mapping、docs、OpenSpec artifacts 或 tests 的改动。
- 在 implementation、debugging、security hardening 或 performance tuning 后做最终质量审查。

## 先使用其他 skill 的情况

- 设计行为、写 regression coverage 或修改逻辑时，先用 `test-driven-development`；完成 diff 后再回到这里 review。
- 行为失败且 root cause 未知时，先用 `debugging-and-error-recovery`；定位并修复后再回到这里。
- 主要工作是 threat modeling、secrets、untrusted input、sandboxing、adapter process、external command 或 trust boundary hardening 时，先用 `security-and-hardening`；本 review 中碰到这些面也要升级检查。
- 主要工作是性能调优或度量时，先走 performance-oriented workflow；本 review 中只有在 hot path、unbounded work 或 regression 风险真实存在时才加载 `references/performance-checklist.md`。

## 工作流

1. 先确认 intent、changed surfaces、预期行为和 governing spec。需要 Docnav 规范上下文时，从 `docs/navigation.md` 进入，只读当前任务相关主规范。
2. 先看 tests。若回归合理可能发生或代价较高，缺失或薄弱 coverage 就是 finding。
3. 沿改动跨过的 ownership boundary 追踪实现：CLI、adapter、protocol、schema/example、MCP bridge、docs 或 OpenSpec。
4. 在纠结 style 前先检查下面的 Docnav cues。
5. 检查 verification。优先要求能证明 touched surface 的最小命令；跨边界行为需要更宽的验证。
6. 交付 findings-first verdict。如果 diff 过宽导致无法可靠 review，要求拆分。

只有在需要正式 worksheet 或 independent second pass 时，才加载 `references/review-checklist.md`。

## 严重级别（Severity）

按以下顺序报告 findings：

| Label | 含义 | Merge 影响 |
| --- | --- | --- |
| Critical | Security vulnerability、data loss、核心行为 broken，或 public contract violation | Blocks |
| High | 很可能 user-visible regression、缺失必要测试，或严重 maintainability risk | Blocks |
| Medium | 真实问题但影响有界，或未来成本明确 | 通常 blocks，除非明确延期 |
| Low | 小缺陷，或有 correctness/clarity 价值的局部清理 | Reviewer judgment |
| Nit / Optional / FYI | 偏好、替代方案或背景信息 | Non-blocking |

不要把 blocking bug 软化成 suggestion。也不要把 optional preference 写得像 mandatory request。

## Docnav 审查线索（Cues）

- 保持 CLI-first navigation flow：`outline -> ref -> read`。
- 把 ref 视为 adapter-generated 和 adapter-owned。核心 `docnav`、MCP 与其他入口只原样传递 ref，不解析、不重写、不发明 ref。
- 分清 raw protocol output 与 readable output。它们可以复用业务语义，但不能复用 transport wrapper、schema、pagination envelope 或 stability promise。
- 检查 pagination 与 continuation：有限读取、稳定 page metadata、强制 limits，且没有意外 full-document load。
- adapter 职责留在 adapter 内：format detection、parsing、navigation strategy、ref generation/parsing，以及 adapter direct CLI behavior。
- MCP bridge 必须保持 thin。它只把 MCP tool call 映射到 `docnav` 行为，不复制 adapter parsing、routing 或 protocol logic。
- 检查 Windows path 下的 CLI 行为：drive letters、backslashes、spaces、quoting、stdin/stdout/stderr 与 readable error output。
- protocol、schema、examples、CLI output、adapter behavior 或 MCP mapping 改动时，确认对应主规范、`docs/schemas/` 与 `docs/examples/` 同步。
- 若本 work item 已有相关 OpenSpec artifacts，则 review artifacts、implementation、tests 与 verification 的一致性。
- 在正确层级检查 tests：adapter unit/smoke tests、core CLI tests、MCP mapping tests、schema/example validation，以及 bug fix 的 regression tests。

只有在 security-sensitive changes 中使用 `references/security-checklist.md`。只有 performance risk 真实存在时使用 `references/performance-checklist.md`。

## 验证（Verification）

记录 review 了什么、运行了什么、没有运行什么。缺失必要 verification 时，把它作为 finding 或 residual risk。

```text
Skill-only 或 Markdown review changes:
  target/debug/docnav-markdown.exe info <changed-md>
  target/debug/docnav-markdown.exe outline <changed-md>

Markdown adapter 或 direct CLI changes:
  pnpm run smoke:docnav-markdown

Core docnav CLI、routing、config 或 adapter registry changes:
  pnpm run smoke:docnav-core

Protocol、schema、examples、docs、MCP mapping 或跨边界 changes:
  pnpm run verify:docnav-workspace
```

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

如果没有 findings，要明确说明，并仍然提到 residual risk 或 test gaps。summary 保持简短，且放在 review result 之后。

## 参考资料（See Also）

- [Review checklist](references/review-checklist.md)：可选 worksheet 与 independent-review prompt。
- [Security checklist](references/security-checklist.md)：更深入的 Docnav security checks。
- [Performance checklist](references/performance-checklist.md)：更深入的 Docnav performance checks。
