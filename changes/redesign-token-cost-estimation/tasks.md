# Tasks

按“Current baseline 与候选证据 → 人工 Q1–Q7 门禁 → Current delta 审计与失败证据 → bounded 实现 → 行为验证 → Current owner 同步”推进；Target 由 design Decisions 承接，任何失败 evidence 都沿同一路径重新打开受影响批准。

## Readiness

- [x] 0.1 Proposal、design 和 tasks 共享 bounded approximate token cost 的单一目标，长期方向与 change-local选择分开。
- [x] 0.2 Protocol/output、adapter/navigation、shared helper、schema/example、测试和 release owners 已完整列出。
- [x] 0.3 Q1–Q7、dependency review、批准 owner、同步动作和 gate-reopen 行为明确；门禁前不执行 contract/production 修改。
- [x] 0.4 相邻 find、renderer、state、service 和 performance changes 均非统一前置，重叠只按 Current owner rebase。

## Implementation

Wave A 为 1.1–1.4：1.1 固定共同 baseline 后，candidate comparison 与 dependency review 可以分工，但必须由一个 evidence owner 合并同口径结果。Wave B 的 1.5 是不可委托给执行子代理的人工硬门禁，1.6 负责固化和审计 Target。Wave C 的 2.1–2.4 只审计 Current delta 并建立失败证据，不把 Target 写入稳定 owner；Wave D 的 3.1–3.4 才能修改 production；Wave E 先证明行为，再同步 Current owner 并做最终一致性验证。1.6 未通过时，2.1 之后全部保持阻塞。

- [ ] 1.1 审计当前 owner、source、schema/examples 和 release behavior，重建 exact/estimate、selection/returned scope、full-read threshold 与 structured admission 的 Current baseline。
- [ ] 1.2 在同一代表性 Markdown、JSON、code、English、CJK、mixed、emoji/combining、escaping、whitespace、long-piece/scalar 和大型 state/config corpus 比较候选 encoding/calculator。
- [ ] 1.3 记录 error distribution、under/over-estimation、worst-case CPU、peak RSS、cold start、platform/target、package、per-entry/page 和 measurement noise。
- [ ] 1.4 对每个新或替换 dependency 完成生态、维护、安全、license、MSRV/targets、transitives、native/build、package、worst cases 和 alternatives 审核。
- [ ] 1.5 取得 Q1–Q4/Q7 的明确人工批准，并单独取得 Q5–Q6 的 threshold/consumer migration 批准；benchmark 或 agent recommendation 不得代替。
- [ ] 1.6 把批准答案的 change-local 实施含义写入连续编号的 design Decisions，清除 Open Questions，并同步 proposal/tasks 的 exact contract；跨 change 长期方向先整理为 decision handoff，只有用户明确要求维护决策时才按 `decision-records` 写入。随后执行阻断审计，确认 Q1–Q7、owner delta 和依赖边界均已闭合；否则不得开始 2.1。
- [ ] 2.1 依据 design Decisions 审计 protocol/output 与必要 adapter/navigation owner 的 Current clauses，逐项登记 approximation representation、per-surface scope、admission、migration 的待实现 delta 和验收映射；本任务不修改稳定 owner，也不把 estimator mechanics 写入选择 owner。
- [ ] 2.2 审计 schema/examples、runtime validation 和 readable mapping 的 Current 基线，固定旧 consumer 的兼容或版本失败证据与待实现 delta；本任务不把 Target 写入 schema/examples 或 production validation。
- [ ] 2.3 在修改测试前，按 `docs/testing.md`、对应行为 owner、`docs/testing/case-maintenance.md` 和 `test-evidence-review` skill 恢复测试契约，并运行项目 wrapper 证明当前树的静态实体、runner 实体和 Case 映射闭合。
- [ ] 2.4 先建立 corpus、boundary、page-membership、hidden-work prohibition 和 gate-reopen failure tests/benchmarks；失败证据必须对应获批 Target 和 budgets。
- [ ] 3.1 实现获批 shared estimator，只接受 caller 已选定的 bounded content/facts，不读取文档或决定 membership。
- [ ] 3.2 让 ordinary/nested read 和 unstructured full-read 只传 returned content，structured outline 在 page membership 确定后才估算 visible entries。
- [ ] 3.3 按 design Decisions 接入 runtime machine/readable projection、必要 JSON runtime handoff、CLI/package smoke 和 Semantic Cases；稳定文档 owner、schema 和 examples 仍保持 Current，等待行为证据成立。
- [ ] 3.4 若 calculator/dependency 在 final evidence 中违反 Q2–Q4，停止依赖任务，把所有依赖失效批准的已完成 checkbox 重新置为未完成，记录失败证据，作废受影响批准并返回 1.2–1.6；替换 evidence 和新的人类批准完成前不恢复下游任务，也不放宽 acceptance。

## Verification

Wave E 在 3.1–3.4 完成后执行：4.1–4.4 先以 design Decisions 证明 accuracy、资源边界、runtime contract 和真实分发；证据成立后，4.5 才同步稳定 owner 为 Current，4.6 最后证明 owner、schema/examples、实现和分发一致。4.1 或 4.2 失败时执行 3.4 的 gate-reopen 路径。

- [ ] 4.1 运行获批 corpus accuracy 与 underestimation/worst-case checks，分别覆盖普通与 adversarial input。
- [ ] 4.2 运行 CPU/RSS/cold-start/platform/package/per-entry/page budgets，证明没有隐藏全文物化、序列化或 tokenize。
- [ ] 4.3 运行 runtime protocol validation、adapter/navigation、raw/readable、pagination/continuation 和 compatibility tests，证明实现符合 design Decisions，而不是从尚未同步的稳定 owner 反推 Target。
- [ ] 4.4 运行真实 CLI、canonical package 和 Semantic Cases，确认获批 Target 已由当前实现和分发行为证明。
- [ ] 4.5 仅在 4.1–4.4 通过后，把实际成立的 approximation representation、per-surface scope、admission 和 migration 同步到 protocol/output 与必要 adapter/navigation owner，并更新对应 schema/examples 和 release materials 为 Current；不得写入未由实现证据支持的 Target clause。
- [ ] 4.6 运行 schema/example、docs、owner link、真实 CLI/package 和 `bun run verify:docnav-workspace`，再审计长期 decision、Current owner 与实现证据的实际关系。
