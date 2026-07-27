本 change 准备把 Docnav 测试证据从人工逐 case 文档升级为“机器保证入口完整性、AI 审查长期语义”的分层模型；本 proposal 只拥有变更动机、范围和影响，不证明 change 已审计、已实施，也不改变当前测试、证据目录或验证行为。

## Why

建 change 时的 v7 目录只能严格检查已经存在的 case、topic 和派生索引，不能发现 runner 原生测试入口、验证 `Entry` locator 或阻止新增入口漏登；当时的迁移基线还明确留下 81 个 Rust test 和 3 个 Bun supporting test 未纳入目录。与此同时，431 个独立 case 中大多数 `Contract` 与 `Proves` 只是模板化复述，增加维护面却不能提高证据判断质量；实施前必须按 tasks 的阻塞审计重新测量这些数量。

## What Changes

- **BREAKING**：把“每个原生入口一篇手写 case Markdown”替换为“每个受支持 runner 原生入口一条机器生成 case inventory 记录”；完整当前树中的入口、运行时报告和 inventory 必须一一对应。
- **BREAKING**：把 case 内必填的 `Contract` / `Proves` 迁移为独立 Evidence Claim。Claim 以稳定 owner requirement 和可观察判断为长期语义，必须关联一个或多个 machine case；普通内部测试允许只有 inventory 记录，不生成无信息叙述。
- 接入项目级 `ast-grep` skill，并建立带正反例和规则测试的 Rust、Bun 与 smoke 静态入口发现规则；静态结果必须与 Rust test list、Bun runner report 和 smoke leaf task list 交叉核对。
- 增加完整性与变更诊断：漏登、悬空、重复、仅静态可见、仅运行时可见、不支持的动态入口、实现 fingerprint 变化和 owner claim 变化必须使用稳定机器结果报告。
- 保留仓库内、离线、确定性的 required 验证；任何分支、合并或其它 change 带来的测试入口变化都按最终当前树检查，不依赖源码 marker 或本次 Git diff 才能发现。
- 迁移阻塞审计确认的全部 v7 case、topic 和派生索引（建 change 时基线为 431 个 case、11 个 topic）：机器可恢复的入口事实进入 inventory；非模板证据只作为 Claim 审计候选；旧 case ID 的历史去向和回滚方式必须可追踪。
- 同步测试策略、维护指南、AGENTS、验证脚本、工作区检查和仍依赖 v7 目录语义的 active change；已归档 change 保持历史原文。
- 不修改 Docnav CLI、adapter、protocol、ref、输出或 release 产品行为；不把开发期 ast-grep executable 带入 canonical release，也不声称结构发现能够自动证明产品测试充分性或 Evidence Claim 语义。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `test-evidence-management`: 将入口覆盖从人工流程升级为静态与运行时交叉核对的全树不变量，并把 machine case inventory 与长期 Evidence Claim 分层。

## Impact

- Skill 与依赖：按固定上游分发接入 `.codex/skills/ast-grep/`；以固定的上游 `test-evidence-review` v7 为基线，在项目内演进并拥有 `.codex/skills/test-evidence-review/` v8；固定开发期 ast-grep CLI 来源和 lockfile。
- 发现与验证：项目级 ast-grep rules/rule tests、Rust/Bun/smoke inventory adapters、case/claim 查询与索引工具、`validate:docs -- cases` 和 workspace verifier。
- 测试证据：`docs/test-evidence/` 中经阻塞审计确认的全部 v7 case Markdown、topic 表、派生索引和新的 Claim/机器 inventory 表示。
- 测试实现：不支持稳定发现的动态注册、wrapper、参数化或 smoke 聚合形态可能需要收敛为显式项目入口；产品实现不因本 change 改变。
- 文档与规划：`docs/navigation.md`、`docs/testing.md`、`docs/testing/case-maintenance.md`、覆盖材料、AGENTS 和相关 active changes。
