# Tasks

先关闭 JSON presentation 与 renderer-selection 门禁，再按 contract evidence、output implementation、真实入口和完整验证的顺序交付。

## Readiness

- [x] 0.1 Proposal、design 和 tasks 指向同一 JSON 专用 `readable-view` 目标，generic/raw Current 与 Planned presentation 已区分。
- [x] 0.2 `docs/output.md` 是 presentation/selection owner，`docs/adapters/json.md` 只拥有 raw JSON 行为；protocol/schema/example 只证明 raw facts 与映射。
- [x] 0.3 已确认长期方向通过活动决策引用，六组 change-local 门禁具有明确关闭任务，且门禁前禁止 owner、测试和代码修改。
- [x] 0.4 相邻 token-cost、find、state 和 performance changes 均非统一前置；没有未声明的实施依赖。

## Implementation

- [ ] 1.1 从当前 output、JSON owner、实现和 release evidence 重核 generic `readable-view`、immutable `ProtocolResponse`、renderer failure 与 linked selection 基线。
- [ ] 1.2 形成 operation/branch × presentation matrix，由用户或指定 output/product owner 逐项批准稳定字段、信息密度、顺序、标点、escaping、block framing、完整 ref 定位信号、preview 和 page/continuation。
- [ ] 1.3 由同一批准 owner 明确 renderer selection 及未选 adapter、提前 failure、非适用 branch、renderer failure 行为；若答案形成新的跨 change 长期方向，只有用户明确授权维护决策时才按 `decision-records` 写入，未获授权且该方向阻塞实施时保持对应门禁未关闭并停止 1.4 及之后任务。
- [ ] 1.4 将批准答案写入 design Decisions，清空对应 Open Questions，并使后续任务和测试预期只引用这一份 change-local design；六组问题未全部关闭时不得开始 2.1 及之后任务。
- [ ] 2.1 保持 design Decisions 为唯一 change-local Target，逐项审计未来需要同步到 `docs/output.md` 的 Current renderer/selection delta，以及 `docs/adapters/json.md` 需要同步的 raw/readable owner 摘要；将 delta、目标位置和成立所需证据登记回本 design，不在证据闭合前修改稳定 owner。
- [ ] 2.2 在修改测试前依次读取 `docs/testing.md`、行为 owner、`docs/testing/case-maintenance.md` 和 `test-evidence-review` skill，并运行 `bun run test-evidence -- check --root .` 证明当前 static/runtime/Case 映射闭合；已有阻断先按 owner 修复。
- [ ] 2.3 先建立最小 Semantic Cases、独立 expected text 和 raw/readable parity fixtures；expected text 不得由待实现 renderer helper 生成。
- [ ] 2.4 增加 output contract tests，覆盖全部批准 branch、escaping/framing、ref、preview、page/continuation、missing fact、render failure 与 writer failure 边界。
- [ ] 2.5 增加 core composition tests，覆盖 renderer selection、未选 adapter、提前 failure、其它格式、`protocol-json` bypass、no-fallback 和 invocation logging 隔离。
- [ ] 3.1 在 output-owned surface 实现 JSON renderer，只消费 immutable response 中已有 facts。
- [ ] 3.2 在 linked composition surface 接入批准的 selection，不增加 public output value、serialized id 或 fallback。
- [ ] 3.3 更新真实 CLI smoke、canonical package smoke 和对应 Semantic Cases，证明 presentation 与 schema-valid raw facts 同源。

## Verification

- [ ] 4.1 运行范围匹配的 format、clippy、output/core/JSON focused tests 和 `bun run smoke:docnav`，逐项核对批准的 presentation matrix、stdout/stderr、render failure 与 writer failure。
- [ ] 4.2 运行 schema/example/docs validators，确认 protocol、raw JSON、ref、ordering、cost 和 page 没有 shape 或语义漂移。
- [ ] 4.3 运行真实开发 CLI 与 canonical release-package smoke，并人工比较代表性 outline/read/find/info 的 raw/readable parity。
- [ ] 4.4 运行最窄目标 runner 后更新 Semantic Case 映射，再运行 `bun run test-evidence -- check --root .`，证明完整当前树重新闭合。
- [ ] 4.5 只有 4.1–4.4 的实现、测试、真实 CLI 和 package 证据全部通过，才按 2.1 登记的 delta 将稳定 owner 同步为 Current；默认只读核对相关活动决策 alignment，只有用户明确授权维护决策时才写入 alignment 或其它 decision-record 变化，并重新运行受影响文档/决策校验。
- [ ] 4.6 在 design 追加 `## Implementation Observations`，记录实际 owner 接入、格式特例、shared friction 和 minimality 结论；运行 `bun run verify:docnav-workspace`，最后用 scoped diff 与 whitespace 检查确认没有相邻 change 依赖或未经批准的抽象扩张。
