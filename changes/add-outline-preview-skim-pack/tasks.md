# Tasks

先关闭显式 surface 与预算门禁，再依次建立 contract、失败证据、composition 实现和两种输出验证。

## Readiness

- [x] 0.1 Proposal、design 和 tasks 只承接 outline skim preview，不夹带其它 operation-composition 候选。
- [x] 0.2 CLI、protocol/output、core/navigation 与 adapter/ref owner 边界已经列明。
- [x] 0.3 Selection、read reuse、typed result 和局部失败是稳定 change-local 设计；剩余 surface/budget 选择具有首个关闭任务。
- [x] 0.4 实施与验证顺序明确，未把计划目标误报为 Current 或实施授权。

## Implementation

- [ ] 1.1 重核当前 outline、auto-read、invocation-private document state、protocol/output 和 strict CLI 基线，确认没有先落地变化需要 rebase。
- [ ] 1.2 由用户或指定 CLI/protocol product owner 明确批准 CLI spelling、适用 output modes、preview count、总预算单位/默认值/覆盖面/耗尽规则、nested-read `limit` 和 closed status/continuation shape；若答案形成新的跨 change 长期方向，只有用户明确授权维护决策时才按 `decision-records` 写入，未获授权且该方向阻塞实施时保持对应门禁未关闭并停止 1.3 及之后任务。
- [ ] 1.3 把全部批准答案同步到 design Decisions，清空 Open Questions，并使 proposal、tasks 和后续测试预期引用该 contract；问题未全部关闭时不得开始 2.1 及之后任务。
- [ ] 2.1 保持 design Decisions 为唯一 change-local Target，逐项审计未来需要同步到 CLI、protocol 和 output 稳定 owner 的 Current delta；将目标位置、closed typed preview result、局部 status、diagnostic、continuation 和成立所需证据登记回本 design，不在证据闭合前修改稳定 owner。
- [ ] 2.2 在修改测试前依次读取 `docs/testing.md`、行为 owner、`docs/testing/case-maintenance.md` 和 `test-evidence-review` skill，并运行 `bun run test-evidence -- check --root .` 证明当前 static/runtime/Case 映射闭合。
- [ ] 2.3 更新 schema/examples 与独立 expected readable vectors，证明 raw/readable 消费同一 facts，expected readable text 不由待实现 renderer helper 生成。
- [ ] 2.4 先增加 selection、预算、无 ref、局部 read diagnostic、invocation-private state reuse、pagination 和 continuation 的失败测试。
- [ ] 3.1 在 core/navigation 实现按 result order、非空 ref、count 和总预算的确定性 candidate selection。
- [ ] 3.2 复用 selected invocation 的现有 read pipeline，构造 preview success/skipped/pending/diagnostic facts。
- [ ] 3.3 扩展 protocol serialization 和内置 renderer，不修改 adapter `OutlineResult` / `ReadResult` 或解析 ref。
- [ ] 3.4 更新真实 CLI smoke、release package smoke 和相关 Semantic Cases。

## Verification

- [ ] 4.1 运行范围匹配的 format、clippy 和 focused core/navigation/protocol/output tests，证明 selection、预算、同一 adapter document reuse、局部 failure 与 continuation 稳定。
- [ ] 4.2 运行 schema/example/docs validators 和 raw/readable parity checks。
- [ ] 4.3 运行代表性 Markdown/JSON outline preview，确认普通 outline/read 和 opaque ref round trip 不回归。
- [ ] 4.4 运行真实开发 CLI、canonical release-package smoke 和 `bun run smoke:docnav`，覆盖显式 surface、两种 output mode、非法/operation-inapplicable 参数和局部 diagnostic。
- [ ] 4.5 更新 Semantic Case 映射并运行 `bun run test-evidence -- check --root .`，证明完整当前树闭合。
- [ ] 4.6 只有 4.1–4.5 的实现、测试、真实 CLI 和 package 证据全部通过，才按 2.1 登记的 delta 将稳定 owner 同步为 Current；重新运行受影响文档校验和 `bun run verify:docnav-workspace`。
- [ ] 4.7 在 design 追加 `## Implementation Observations`，记录预算/延迟观察、实际组合 seam 和未形成 contract 的实现细节，再审查 scoped diff 与 whitespace。
