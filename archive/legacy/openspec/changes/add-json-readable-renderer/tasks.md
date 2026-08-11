本 tasks 清单把 JSON 专用 `readable-view` 保持为独立且 `implementation-blocked` 的 handoff；产品状态与门禁以 [proposal](proposal.md) 为准。任务 0.1–0.4 全部完成前，不得执行 1.1 及之后的 owner、测试或实现工作。

## 0. Presentation contract 阻塞门禁

- [ ] 0.1 核实实施时的 Current 基线：读取 `docs/output.md`、`docs/adapters/json.md`、当前 output/JSON 实现证据与已完成 `add-json-adapter` handoff，确认 generic `readable-view` 和 invocation-private document state 仍是已验收现状；记录 `redesign-token-cost-estimation`、`redesign-find-result-model` 和 `audit-runtime-performance-boundaries` 均非前置，不采用其未落地语义。
- [ ] 0.2 逐项回答 design Open Questions 1–5，明确每个适用 operation/branch 的 presentation scope、稳定字段与信息密度、标点与 escaping、完整 opaque ref 的路径定位信号、preview 来源/边界和 page/continuation 表达；不得解析 ref 或合成 hierarchy、depth、parent、indentation，也不得要求当前 response 不存在的事实。
- [ ] 0.3 回答 design Open Question 6，明确 linked output composition 的 renderer selection mechanics，以及未选 adapter、提前 failure、非 structured branch 和 renderer failure 的 presentation；保持现有 immutable `ProtocolResponse`、no-fallback、public output values 和 raw isolation。
- [ ] 0.4 **门禁关闭条件：** 将 0.2–0.3 的批准答案写入连续编号的 design Decisions，补全 delta spec 的 exact requirements/scenarios，并同步 proposal 与后续 tasks；确认四类 artifacts 都围绕开头核心句、capability 仍精确为 `output-contract`、没有把 planning 状态误报为已批准/Current/已实现，且 planning 修改只在本 change 目录。重新运行全部 Markdown DNM、scoped diff/whitespace 和 strict OpenSpec validation。只有 `## Open Questions` 确实无未回答项且 artifacts 一致时才能勾选本任务并开始 1.1。

## 1. Owner contract 与测试起点

- [ ] 1.1 按批准的 delta 同步 `docs/output.md` 与 `docs/adapters/json.md`：output owner 完整承接 JSON presentation/selection contract，JSON owner 只摘要 raw/readable 边界；在实现证据完成前把新行为标为 Target 而非 Current。
- [ ] 1.2 在任何测试变更前按 `docs/testing.md`、对应行为 owner、`docs/testing/case-maintenance.md` 和项目 `test-evidence-review` skill 恢复测试义务，并运行 `bun run test-evidence -- check --root .` 证明完整当前树的 static/runtime/Case closure；已有阻断先按 owner 处理。

## 2. 先建立可失败的 contract evidence

- [ ] 2.1 按批准 spec 维护最小语义 Case、独立 expected text/conformance vectors 和 raw/readable parity fixtures；expected output 不得由待实现 renderer helper 生成。
- [ ] 2.2 增加 output contract tests，逐项覆盖批准的 operation/branch、presentation、escaping/framing、opaque ref、preview、page/continuation、missing-fact 和 render-failure 行为。
- [ ] 2.3 增加 core composition tests，覆盖批准的 JSON renderer selection、未选 adapter/提前 failure/其它格式行为和 `protocol-json` bypass；保持 writer failure、diagnostic mapping、no-fallback 与 invocation logging 不变。

## 3. Output-owned implementation

- [ ] 3.1 在批准的 output owner surface 实现 JSON presentation，只消费 immutable `ProtocolResponse` 中已有 raw facts，并满足 2.1–2.2 的失败证据；不调用 adapter、不重新读取文档、不解析 ref 或把 presentation fact 写回 protocol。
- [ ] 3.2 在批准的 linked composition surface 按任务 0.3 的决定接入 JSON renderer selection，并满足 2.3；不新增 public output value、serialized renderer id、adapter-owned presentation 或未经批准的 selection abstraction。

## 4. CLI、package 与 Case 闭合

- [ ] 4.1 扩展真实 core `docnav` CLI smoke，对批准范围内的 JSON operation/branch 运行省略或显式 `readable-view` 与对应 `protocol-json`；先验证 raw schema，再用独立 expected facts 证明 presentation 与 raw parity。
- [ ] 4.2 扩展 canonical release-package smoke，使用 validated manifest 选出的 packaged core executable而非 workspace target binary，证明批准的 JSON selection/presentation和对应 schema-valid raw facts。
- [ ] 4.3 运行最窄目标 runner，更新当前测试实体与语义 Case 映射，再运行 `bun run test-evidence -- check --root .`，证明完整当前树重新闭合。

## 5. Contract 与 workspace 验证

- [ ] 5.1 运行范围匹配的 `cargo fmt --check`、clippy、output/core tests和 `bun run smoke:docnav`，核对 exact readable text以及真实 stdout/stderr、render failure和writer failure边界。
- [ ] 5.2 运行 docs/schema/example validators，确认 protocol response schema/example、JSON adapter raw facts、ref、ordering、cost和page没有未经批准的 shape或语义变化。
- [ ] 5.3 运行 `bun run verify:docnav-workspace` 与 `openspec validate add-json-readable-renderer --type change --strict --no-interactive`；最后用 scoped diff 和 whitespace 检查确认只存在批准的 owner、presentation、selection 与验证材料，没有相邻 change 依赖或未经批准的 abstraction 扩张。
- [ ] 5.4 在本 change 的 `design.md` 追加 `## Implementation Observations`，记录实际 owner 接入、格式特例、shared friction和 minimality结论；只有 owner、代码、测试、真实 CLI和package证据完整时才把目标行为标为 Current，并分别核对相关 active decision 的 alignment，未满足时保持 `unaligned` 并记录精确差距。
