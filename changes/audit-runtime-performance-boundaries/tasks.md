# Tasks

按“证据准备 → 人工采用门禁 → 获准 handoff → 整体验证”推进：先固定审计契约和 workload，再把测量与归因写入独立 investigation report，最后请求人类分别决定 baseline、budget、gate 与 owner handoff。

## Readiness

- [x] 0.1 Proposal、design 和 tasks 都以可复现 runtime audit 与 owner handoff 为目标，不预选优化机制。
- [x] 0.2 Investigation report、长期 decision、未来稳定 owner、Change、tooling/quality 和行为 owner 的职责已分开。
- [x] 0.3 Workload selection、measurement record、attribution 和 human decision 顺序明确，未知项有合法状态。
- [x] 0.4 所有人工决定都有独立任务和被阻塞出口；没有未声明的跨 change 实施依赖。

## Implementation

Wave A 为 1.1–2.5。1.1–1.3 建立所有测量共享的契约后，各 required cell 可以由不同执行者采集，但 2.4–2.5 必须由一个报告 owner 合并并核对可比性。Wave B 为 3.1–3.5，全部由用户或其指定 owner 接管；Wave C 的 4.1–4.2 只能消费已批准结果。任一硬门禁未关闭时，执行者必须停在该任务并汇报，不能跳到下一个 wave。

- [ ] 1.1 在测量前固定本轮调查的核心问题、形成时背景、目的和边界，并确认当前实施请求已明确授权沉淀调查报告；未授权时停止并请求用户决定，不在 Change 目录创建 report 占位文件。
- [ ] 1.2 固定有限 initial workload packet、required cells、fixture selection、unavailable/not-applicable 条件和停止规则。
- [ ] 1.3 记录被测 binary/build/process boundary，分别包含 startup、package、primary outline/find/ref-derived read、later page、有限 secondary format 与分层 stress cells。
- [ ] 2.1 按自足 measurement contract 记录 wall/CPU、I/O/准备次数、peak/retained memory、stdout/stderr/page、package size 和伸缩样本。
- [ ] 2.2 尝试复现既有 JSON、超长 key/output 和 tokenizer seed observations；条件不足或不可复现时保持 seed/unknown。
- [ ] 2.3 对 material observations 做最小充分归因，不能证明的部分标为 `unattributed`。
- [ ] 2.4 分别总结 representative 与 stress 结果、噪声、未知项、不可比条件和未测矩阵。
- [ ] 2.5 在 1.1 已确认调查报告写入授权的前提下，按 `investigation-report` skill 在 `docs/investigations/runtime/` 写入一份完整、可独立阅读的报告；只在必要时引用最小随附资源，并同步、检查调查索引。
- [ ] 3.1 由人类逐项批准或拒绝哪些 records 成为 reproducible baselines；其余保持 observation。
- [ ] 3.2 由人类单独决定是否需要数字 budget，并为每个 budget 固定 workload、指标、范围、统计、build、host/cache、噪声和复核条件。
- [ ] 3.3 由人类单独决定是否需要 blocking gate；只有 enforcement owner、入口、失败语义和更新/移除流程均获批准时才创建 gate 工作。
- [ ] 3.4 把经确认且跨 change 持续有效的 baseline/budget/gate 方向整理为决策 handoff；只有用户明确要求维护决策时才按 `decision-records` 写入，否则保持 handoff pending 并停止受该决定阻塞的稳定 owner 或 gate 工作，不把采用决定回写成调查报告的权威结论。
- [ ] 3.5 由人类审阅 owner handoffs，明确哪些 finding 只需 owner 修正、哪些值得建议 owner-specific optimization Change。
- [ ] 4.1 只对获准 finding 提出带 before/after workload 的后续候选；只有用户明确要求时才创建或维护持久 Change，本计划不实现优化。
- [ ] 4.2 只有获准长期规则需要稳定 owner 时，创建或更新 runtime-performance owner 文档和 navigation 入口；对应决定只由 decision record 拥有。

## Verification

Wave D 只能在已批准 handoff 全部完成或明确记为不采用后开始；5.1–5.3 先证明内容和采用边界，5.4 最后运行统一验证。

- [ ] 5.1 用 `dnm outline/read` 验证 proposal、design、tasks 和 investigation report 可独立恢复，并运行 Change、调查、决策、Markdown link 和 whitespace 检查。
- [ ] 5.2 复核每个 required workload cell 都有 measured、unavailable 或 not-applicable 状态，未选组合明确为 `unmeasured/future`。
- [ ] 5.3 复核 baseline、budget、gate 和 handoff 均有显式人类结论，未批准 observation 没有进入稳定 owner 或实现任务。
- [ ] 5.4 运行 `bun run validate:change-plans`、`bun run validate:docs` 和 `bun run verify:docnav-workspace`。
