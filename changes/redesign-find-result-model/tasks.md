# Tasks

按“人工选择 → Target 固化 → Current delta 审计与失败证据 → production 实现 → 行为验证 → Current owner 同步”推进：先由人类批准完整 find model/work packet，唯一 Target 由 design Decisions 承接；稳定 owner 只在实现与行为验证成立后同步。

## Readiness

- [x] 0.1 Proposal、design 和 tasks 把 logical unit、wire fields、work budget、pagination、auto-read 与迁移视为同一目标。
- [x] 0.2 Protocol、navigation、adapter、output、schema/example 和测试 owners 完整；token estimator 与 JSON presentation 明确独立。
- [x] 0.3 Current occurrence baseline 与候选 Target 已分开，人工 gate、持久化动作和被阻塞任务明确。
- [x] 0.4 十四项 packet 覆盖 identity、fields、evidence、ordering、page、proof、resources、compatibility 和 handoff，没有未归属问题。

## Implementation

Wave A 的 1.1 是不可委托给执行子代理的人工硬门禁。Wave B 的 1.2–1.4 固化和审计唯一 Target；Wave C 的 2.1–2.4 只审计 Current delta 并建立失败证据，不把 Target 写入稳定 owner；Wave D 的 3.1–3.4 才能修改 production；Wave E 先证明行为，再同步 Current owner 并做最终一致性验证。1.4 未通过时，2.1 之后的任务全部保持阻塞。

- [ ] 1.1 向用户或指定 product/architecture owner 提交完整 packet，对 Markdown、Current JSON、代码和大型 state/config 场景比较 occurrence、distinct node/ref 与 grouped 模型，并取得逐项明确批准；执行子代理只能准备证据和问题，不能选择答案。
- [ ] 1.2 把所有批准答案的 change-local 实施含义写成连续编号的 design Decisions，清除 Open Questions，并整理跨 change 长期方向的 decision handoff；只有用户明确要求维护决策时才按 `decision-records` 写入，agent review 或 benchmark 不得代替批准或决策写入授权。
- [ ] 1.3 从获批答案形成一个 exact Target，逐项固定 type/top-level field、九个字段、identity、multiplicity、ordering、page/continuation、auto-read、scan/retained budget、exhaustion、compatibility 和 JSON handoff。
- [ ] 1.4 执行阻断审计：重核 artifacts 与当前 owner/schema/types，确认不再含 provisional alternatives，九个字段和十四项 packet 都有 exact Target，必要的 decision handoff 已获授权并闭合或明确不需要，且 Current 行为未被误报为已改变；任一项失败时返回 1.1–1.3，不得开始 2.1。
- [ ] 2.1 依据 design Decisions 审计 protocol、navigation、Markdown、output 与必要 JSON owner 的 Current clauses，逐项登记实施后可能成立的 delta、owner 路径和验收映射；本任务不修改稳定 owner，也不修改历史 artifacts。
- [ ] 2.2 审计 request/response schema、examples、shared Rust types 和 decode/semantic validation 的 Current 基线，固定兼容或 breaking version 的失败证据与待实现 delta；本任务不把 Target 写入 schema/examples 或 production types。
- [ ] 2.3 在修改测试前，按 `docs/testing.md`、对应行为 owner、`docs/testing/case-maintenance.md` 和 `test-evidence-review` skill 恢复测试契约，并运行项目 wrapper 证明当前树的静态实体、runner 实体和 Case 映射闭合。
- [ ] 2.4 先写 model-independent 和 model-specific failure tests，覆盖九字段、multiplicity completeness、ordering、later page、lookahead、budget exhaustion、auto-read 和 migration；测试失败必须对应获批 Target，而不是候选分支。
- [ ] 3.1 按 design Decisions 实现 shared logical-unit/type、runtime decode/semantic validation 与 page facts，不解析 opaque ref，不把 presentation fact 写入 raw result。
- [ ] 3.2 在 Markdown 及获批需要的其它 adapter 实现单调 traversal/replay、有限 retained work 和 exact evidence semantics。
- [ ] 3.3 更新 navigation auto-read eligibility，只使用获批 completeness scope，partial/incomplete 情况按 contract 抑制或标记。
- [ ] 3.4 更新 readable projection、CLI smoke、canonical package smoke 和 Semantic Cases。

## Verification

Wave E 在 3.1–3.4 完成后执行：4.1–4.4 先以 design Decisions 证明实现、工作预算、contract parity 和迁移；证据成立后，4.5 才同步稳定 owner 为 Current，4.6 最后证明 owner、schema/examples、实现和分发一致。

- [ ] 4.1 对 representative 与 adversarial sources 验证 first/later page scan、retained state、lookahead/replay 和 exhaustion 均在获批预算内。
- [ ] 4.2 运行 runtime protocol validation、adapter conformance、raw/readable parity 和 auto-read tests，证明实现符合 design Decisions，而不是从尚未同步的稳定 owner 反推 Target。
- [ ] 4.3 验证兼容/version/rollback 路径，并确认 token calculator 与 JSON presentation 没有被本计划隐式选择。
- [ ] 4.4 运行真实 CLI、canonical package 和 Semantic Cases，确认获批 Target 已由当前实现和分发行为证明。
- [ ] 4.5 仅在 4.1–4.4 通过后，把实际成立的 contract 同步到 protocol、navigation、Markdown、output 与必要 JSON owner，并更新对应 schema/examples 和 release materials 为 Current；不得写入未由实现证据支持的 Target clause。
- [ ] 4.6 运行 schema/example、docs、owner link、真实 CLI/package 和 `bun run verify:docnav-workspace`，再做 owner/artifact/scoped diff 审计。
