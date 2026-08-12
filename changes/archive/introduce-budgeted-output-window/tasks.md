# Tasks

本 Plan 交付 producer-time OutputSession shared capability 和直接证据；Current CLI/protocol/adapter 行为切换继续由 downstream public Change 拥有。

## Readiness

- [x] 0.1 Proposal、Design 与 Tasks 共享一个正向目标：建立可组合的逐项输出构造链路；带单位 limit 是 Limited Gate 的能力，不是整条架构的中心。
- [x] 0.2 已核对 Plan 确认时的 adapter/navigation/output/text-cost 基线：operation contract 返回完整 result，多条 adapter path eager collect/serialize，renderer 消费完整 `ProtocolResponse`，text-cost 只接受完整 `&str`；实施后由 architecture owner 重新区分 Current shared capability 与 Target integration。
- [x] 0.3 已建立 active、unaligned 的[Gate、InputCost 与 Collector 长期决策](../../../docs/decisions/product-direction/compose-incremental-output-through-gates-policies-and-collectors.md)；本 Plan 只实施其 shared capability 部分。
- [x] 0.4 已确认组合 contract：caller 拥有 producer/input granularity，Gate 拥有 admission/flow，InputCost 组合 Projection 与 Meter，Collector 拥有保存与 typed output，navigation/presentation 只消费 finish 后的完整结果。
- [x] 0.5 已确认 push/finish contract：Limited 原子接纳或拒绝，outcome 返回 input/flow/budget/stop；Unbounded 复用调用形状且跳过 measurement；source completion 由 producer owner 确认。
- [x] 0.6 已确认 text projection contract：Projection 是显式、可替换、可逐段提供借用文本的调用策略；同一输入类型可按 operation 使用不同 policy，代码生成只可作为该契约的便利实现。
- [x] 0.7 已确认 Collector contract：accepted input exactly once 移交，首版 accept infallible，finish 形成 typed result；Session 不固定物化 `Vec<I>`，最终 renderer 不观察 partial state。
- [x] 0.8 已识别跨片段计量约束：一个输入的 projected fragments 必须按逻辑连接文本计量，不能独立相加 line/token cost；只有能够证明 exceed 时才可提前停止 projection。Token 首版允许 input-local buffering，但必须与现有 reference calculator 等价。
- [x] 0.9 修改测试前的完整 Current evidence baseline 已通过 `bun run test-evidence -- check --root .`：559 个 Current entities 全部由 159 个 Semantic Cases 覆盖；`WB-TEXT-COST-001` 位于 `shared-foundations`，新 capability 将新增独立 Case。

## Implementation

`1.1`–`1.6` 建立 shared enum、bounded TextMeter、Session 组合接口、Gate/Collector state 和最小 policies；`1.7` 完成直接行为证据；`1.8`–`1.9` 同步 owner、Case 与 downstream gate。

- [x] 1.1 在 `docnav-protocol` 定义 closed `CostUnit::{Lines, Bytes, Tokens}` 及稳定 Rust string/serde mapping，供 shared crates 和 future protocol `0.2` 复用；保持 Current request/response、schema、examples 和 protocol version 不变，并用 focused tests 证明没有 wire delta。
- [x] 1.2 扩展 `docnav-text-cost`：提供 requested-unit bounded `TextMeter`/等价 session，接受有序文本 fragments 和 threshold，只运行所选 measurement；bytes/lines 保持跨片段状态，tokens 与现有 `o200k_base` ordinary-text 在逻辑拼接文本上等价。Meter 只有在能够证明 exceed 时才提前结束 fragment consumption；既有 `line_cost`、`byte_cost`、`token_cost` 保持兼容并复用同一语义。
- [x] 1.3 新增 workspace member `crates/shared/output-session` / `docnav-output-session`，定义 generic `OutputSession`、Limited/Unbounded Gate、`InputCost<I>`、`TextProjection<I>`、`Collector<I, Output = R>`、input disposition、flow、`StopReason`、mode-specific `PushOutcome` 和 `OutputReport`；crate 只依赖 protocol/text-cost owners。
- [x] 1.4 实现 Session 协调与 Gate state：Limited 使用 Gate-owned unit/limit 对 `&I` measurement 后执行 accepted+continue、accepted+stop 或 rejected+stop；Unbounded 直接 accepted+continue。Measurement failure、违反 bounded measurement contract 的结果与 stopped-session push 使用 error channel，并在 Collector commit 前保持 state 不变；`cost <= remaining` 已证明后的累加作为局部程序不变量。
- [x] 1.5 实现 Collector exactly-once commit 与 finish：首版 `accept(I)` infallible，只有 accepted input 才移交；finish 接受 caller-owned source completion，返回 Collector typed output 与 report，finish failure 在 `ProtocolResponse` 形成前结束。提供 String builder、`Vec<I>` 和 operation-specific fake builder，证明核心不固定中间表示。
- [x] 1.6 实现最小 input-cost policy：String/text chunk identity projection；使用 caller-owned structured test projection 证明业务字段选择留在理解 operation 语义的调用方。Projection 以普通 Rust policy 向 bounded TextMeter 提供借用 fragments 与必要连接片段，并保持可调用契约为语义 owner。
- [x] 1.7 新增 focused Rust tests，证明三 unit requested-only dispatch 和跨片段等价性/threshold stop、Limited 三条 transition、Unbounded measurement bypass、structured outcome、snapshot invariants、failure non-mutation、terminal state、Collector exactly-once/order、三种 Collector output、finish completeness。用 lazy counting producer 驱动同一个 canonical loop，证明 Limited stop 后 tail 未被请求。
- [x] 1.8 在 `docs/architecture.md` 登记 future producer → Session(Gate + InputCost + Collector) → typed result → `ProtocolResponse` → presentation 边界、shared crate responsibility 和 Current 尚未接入事实；更新 `WB-TEXT-COST-001` 的 bounded fragment evidence，并在 production entities 存在后新增 `WB-OUTPUT-SESSION-001`。
- [x] 1.9 形成 capability handoff：列出 future `AdapterDocument`/Markdown/JSON/navigation integration 必须遵守的 input granularity、policy placement、push ordering、stop handling、Collector finish 和 complete obligations；标记相邻 public/fast-read Change 需要在下一实施步骤前重审旧 post-result/field-traversal 描述，但不在本 Change 偷跑其行为或 lifecycle。

## Verification

- [x] 2.1 对 `docnav-protocol`、`docnav-text-cost` 和 `docnav-output-session` 运行 Rust format、Clippy 与 focused tests，确认无 warning、rejected-input commit、measurement-error mutation、terminal reopen 或 Collector double-accept，并核对 budget 累加由 `cost <= remaining` 不变量封闭。
- [x] 2.2 运行 measurement contract evidence：对空片段、跨换行边界、Unicode、token merge-sensitive split 和 threshold-before-tail 等输入，比对 fragment session 与逻辑拼接后的既有三 unit calculator，证明 requested-only dispatch 与 bounded stop。
- [x] 2.3 运行 direct composition evidence：同一个长 lazy producer 分别驱动 Limited/Unbounded Gate 和至少三种 Collector；记录 produced/pushed/accepted/collected 数量，证明 Limited stop 后不生成 tail、Unbounded 不调用 InputCost、accepted inputs exactly once 到达 typed output，并明确证据尚未覆盖真实 adapter integration。
- [x] 2.4 按真实证明目的审阅 `WB-TEXT-COST-001` 与 `WB-OUTPUT-SESSION-001`，运行最窄 target runner 后执行 `bun run test-evidence -- check --root .`，证明完整 Current entity/runner/Case 映射重新闭合。
- [x] 2.5 用 `dnm outline/read`、局部 diff 和 decision trace 检查 proposal/design/tasks、architecture、Case 与 active successor 的读取路径；运行 Decision Records strict check、Change Plan、docs link/Markdown 和 whitespace validators。
- [x] 2.6 运行 `bun run verify:docnav-workspace`，证明 workspace members、dependency graph、Rust tests、test evidence、docs 与 release checks 在没有 public behavior delta 的前提下整体通过。
- [x] 2.7 复核 Current CLI help、config、protocol schema/examples、adapter output、raw/readable rendering 与 release behavior均无变化；确认 evidence 区分“shared Session/参考 producer 可增量构造并早停”和“真实 adapters 尚待 downstream 接入”。
