本 design 记录 JSON 专用 `readable-view` handoff 的已确认边界和实现门禁。

## 文档状态

本 design 处于 `implementation-blocked` 状态，不表示 Current 或可直接实施。Current/Target、产品方向与门禁以 [proposal](proposal.md) 为准；必须按 [tasks](tasks.md) 先关闭 presentation contract 和 renderer-selection 开放问题。

## Context

`add-json-adapter` 已交付 JSON 当前选择链路、outline、read、find、info、full-read、generic `readable-view` 和 `protocol-json`。当前实现事实由代码、测试和 release artifact 证明；其 OpenSpec artifacts 只记录交接背景。`docs/output.md` 也明确把 JSON 专用 renderer 标为 Planned 后续能力，并把信息密度、完整 opaque ref 的路径定位信号、标点、preview、分页显示和 renderer selection mechanics 留给本 change 决定。

本 handoff 的 owner 关系如下：

| 内容 | Owner / 证据 |
| --- | --- |
| public output modes、`ProtocolResponse` 到 readable presentation 的边界、renderer contract、framing、failure 与输出通道 | `docs/output.md` |
| JSON raw result facts、ref grammar、ordering、pagination/cost 和 adapter-owned error | `docs/adapters/json.md` |
| 当前 generic JSON readable 行为 | 当前代码、测试和 release artifact |
| JSON 专用 presentation 的 change-local 选择、实施顺序和验收依据 | 本 change；完成后同步回长期 owner |

Generic renderer 已满足 raw JSON adapter 的当前验收，但尚未提供 JSON 专用 presentation contract。两者是顺序相连而验收独立的阶段：本 change 不追溯阻塞或重开 `add-json-adapter`。

## Goals / Non-Goals

**Goals:**

- 把 Current generic presentation、Target JSON 专用 presentation 和 implementation-blocked planning 状态明确分开。
- 在 output owner 边界内决定并交付 JSON 专用 `readable-view`，且只从同一个 immutable `ProtocolResponse` 的已有 raw facts 形成阅读输出。
- 在实施前把所有会改变 observable presentation 或 selection behavior 的选择写成明确 Decisions、可证伪 requirements 和 scenarios。
- 用同源 raw/readable、真实 CLI 和 package 证据证明最终实现没有改变 machine contract。

**Non-Goals:**

- 不修改或重新验收 `add-json-adapter`，不把专用 renderer 设为 raw adapter 交付前置。
- 不修改 `ProtocolResponse`、protocol/schema/example、JSON adapter result/ref/order/pagination/cost、public output values 或 diagnostic mapping。
- 不解析 opaque ref，不从 ref 合成 hierarchy、depth、parent 或 indentation，不重新读取文档或取得 adapter-private state。
- 不预选信息密度、字段 shape、标点、escaping、preview、分页文案或 renderer selection mechanics。
- Invocation-private document state 只作为 Current 输入基线；本 change 不依赖、合并或替代 `redesign-token-cost-estimation`、`redesign-find-result-model` 或 `audit-runtime-performance-boundaries`，也不采用这些 change 尚未落地的语义。
- 不把 presentation 所有权移入 adapter，不新增用户可配置 renderer 或第二套 readable schema；内部 selection mechanics 由 Open Question 6 决定，当前 design 不预选具体 mechanics，也不以候选方案锚定解空间。

## Decisions

### Decision 1: 把 JSON 专用 presentation 作为独立后续 handoff

`add-json-adapter` 的 Current 验收以 generic `readable-view` 为准。本 change 独立承接格式专用 presentation；它完成后补足长期决策所需的完整 presentation evidence，但它不是前序 raw adapter change 的前置或返工条件。

### Decision 2: 由 output 层消费 immutable `ProtocolResponse`

JSON 专用 presentation 属于既有 `readable-view`，长期 owner 是 `docs/output.md`。Presentation 只能使用传入的同一个 immutable `ProtocolResponse` 和其中已有的 adapter raw facts，不调用 adapter、不重新读取文档，也不把 presentation fact 写回 protocol 或 JSON adapter。

`ProtocolResponse` 中的 ref 对 output 层保持 opaque。最终 contract 可以决定是否以及怎样把完整 ref 用作路径定位信号，但 renderer 不拆解 ref token，也不由 ref 推导 hierarchy、depth、parent 或 indentation。

### Decision 3: 相邻未完成 workstream 不构成本 change 的依赖

本 change 以实施门禁关闭时的 Current output、JSON raw contract 和 invocation-private document state 为输入基线，不等待 token-cost、find-result 或 runtime-performance workstream 完成。相邻 workstream 尚未确定的 cost、find 或 performance 语义不得进入本 design；其中任一先落地并改变 Current 输入时，只触发 scoped contract re-audit，不建立反向依赖。

### Decision 4: 开放问题关闭前保持 implementation-blocked

OpenSpec artifact 存在或 strict validation 通过，只证明 planning 文件结构有效，不证明 presentation contract 已批准。任务 0.1–0.4 是阻塞门禁：所有开放问题必须先得到明确答案，答案进入连续编号的 Decisions，delta spec 随之给出 exact、testable requirements，tasks 再按已批准方案收敛。门禁完成前不得修改长期 owner、测试或实现。

## Risks / Trade-offs

- [Risk] 把候选 display grammar 写成既定 contract，会让未来代理误把未决偏好当作授权。→ 开放语义只列在 `## Open Questions`，不提供默认答案或候选实现细节。
- [Risk] 把 Planned renderer 误报为 Current，会削弱实现与验收判断。→ proposal、design、spec 和 tasks 都显式区分 Current、Target 与 blocked state；只有代码、测试和 release evidence 齐备后才更新 Current。
- [Risk] 期望的 presentation 可能需要 response 中不存在的 fact。→ 门禁只允许基于现有 raw facts设计；若确需新增 machine fact，停止本 change 并单独评估 owner、protocol 和 adapter 影响，不在 renderer 中合成。
- [Risk] 相邻 active change 可能在本 change 实施前改变输入事实。→ 在门禁审计中重读当时的 Current owner；这是一项 rebase 检查，不是等待相邻 change 的前置条件。
- [Trade-off] 在开放问题关闭前，delta spec 只固定边界而不固定逐 operation 文案。→ Change 保持 implementation-blocked，避免用看似完整但未经批准的 contract 换取表面 apply-ready。

## Migration Plan

1. 完成任务 0.1–0.4：核实 Current 基线，回答全部开放问题，把答案同步为 Decisions、完整 delta requirements 和按依赖排序的 tasks。
2. 门禁关闭后，先把批准的 Target contract 同步到 output/JSON owner 并恢复测试证据起点，再按 tests → output-owned implementation → core selection → CLI/package evidence 的顺序实施。
3. 通过范围验证、Case closure、schema/raw parity 和 workspace verification 后，才把 owner 状态更新为 Current，并记录 implementation observations。
4. 本 change 不迁移用户数据或 raw protocol。需要回退时移除专用 presentation 与其 selection 接入，恢复当前 generic `readable-view`；JSON adapter 和 `protocol-json` 不需要迁移。

## Open Questions

以下问题都会改变 observable contract 或 owner 接入，当前没有批准答案：

1. JSON 专用 presentation 精确覆盖哪些 operation 和 branch：structured outline、read、find、info、nested auto-read、unstructured full-read 与 failure 各自使用专用还是现有 generic presentation？
2. 各适用 operation 的稳定 header/display 字段、信息密度、字段顺序约束、JSON 标点、escaping 和 block framing contract 是什么？
3. 完整 opaque ref 是否以及怎样作为路径定位信号呈现，同时保持 ref 原样传递且不合成 hierarchy、depth、parent 或 indentation？
4. Preview 对各 operation 从哪个现有 raw fact 取得，是否需要截断，以及截断上限和可观察 spelling 是什么？
5. Page/continuation 在各适用 operation 中怎样显示；哪些 raw page facts原样保留，哪些阅读文案属于稳定 contract？
6. Linked output composition 用哪个既有事实选择 JSON 专用 renderer；未选 adapter、提前 failure、非 structured branch 和 renderer failure 分别选择哪个已批准 presentation，同时怎样保持现有 no-fallback 与 raw isolation？

只有六项全部得到回答、写入 Decisions 并落实为完整 delta scenarios 后，任务 0.4 才可完成；在此之前不得开始任务 1.1 及之后工作。
