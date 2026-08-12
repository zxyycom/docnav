# Proposal

本 Change 把 `outline`、`read` 和 `find` 的公开分页契约一次性替换为带单位的输出上限，并把已经落地的 `OutputSession` 共享能力接入真实 adapter、navigation、protocol 和 presentation 调用链。

## Why

Current 请求把 adapter-owned 正整数 `limit` 与数字 `page` 一起传入 Markdown 和 JSON adapter；主要分页路径按 Unicode 字符解释 limit，而 protocol cost 又能报告 `lines`、`bytes` 和 `tokens`。关闭 pagination 时，navigation 还会把 effective limit 规范化为最大正整数。调用方因此不能从公共输入恢复 limit 的成本单位，也必须理解 page、continuation、adapter 分页和 selection cost 之间的差异。

[用带单位的输出上限替代分页](../../docs/decisions/product-direction/replace-pagination-with-unit-output-limits.md)已经确认 public 方向：[用 Gate、计量策略与 Collector 组合增量输出](../../docs/decisions/product-direction/compose-incremental-output-through-gates-policies-and-collectors.md)已经确认 producer-time 执行架构。归档的 [introduce-budgeted-output-window](../archive/introduce-budgeted-output-window/proposal.md) 已交付 shared `CostUnit`、requested-unit `TextMeter` 和 `OutputSession`，但明确没有改变 Current CLI/protocol/adapter 行为。本 Change 负责把这两条方向原子落实为真实 public cutover。

## Outcome

`outline`、`read` 和 `find` 的有界调用使用 `Limit { unit, value }`；合法 unit 恰好是 `lines | bytes | tokens`，一次调用只选择一个 unit。省略显式输入时使用 core-authored `tokens:6000` preset；该数值是可调整的产品默认值，不是架构不变量。

Input resolution 把 CLI、config 或 machine request 规范化为 `OutputConstraint::Limited(Limit)` 或 `OutputConstraint::Unbounded`。Limited operation 由 adapter-owned producer 逐项驱动 shared `OutputSession`，通过 operation-specific `InputCost` / `TextProjection` 和 Collector 形成 typed result 与 report；Unbounded 使用同一 producer/Collector 形状但不构造 measurement policy。Navigation 在 finish 后校验 typed result、组合 nested auto-read、建立 common output facts，再把同一个完整 `ProtocolResponse` 交给 protocol-json 或 readable-view。

Protocol `0.2`、CLI、config、adapter input、schema、examples 和 release artifact 在同一 breaking migration 中移除 page、next-page、continuation 与 pagination-enabled normalization。Incomplete 结果不携带继续位置；调用方通过更具体的 ref/query、更高 limit 或显式 unbounded 请求重新调用。

## Scope

本 Change 纳入：

- `outline`、`read`、`find` 的 structured sequence、text、unstructured full-read 和 success-only nested auto-read 输出控制。
- 基于现有 `o200k_base` ordinary-text calculator 的 deterministic bounded text-prefix policy，包括 UTF-8 安全边界、重算一致性和资源证据。
- CLI、project/user config、core parameter catalog、input resolution、machine request、protocol request construction 和 invocation metadata 的 output-constraint surface。
- Object-safe adapter execution handoff、built-in Markdown/JSON producer、operation-specific projection/Collector、Limited/Unbounded Session 与 navigation report composition。
- Protocol `0.2` affected-success output sidecar、generic readable projection、JSON Schema、examples、fixtures、help、迁移诊断、Semantic Cases、测试和 release package。
- 旧 `0.1` pagination request、`--page`、`--pagination`、numeric-only `--limit` 与 `defaults.pagination.*` 的明确拒绝；不提供 runtime alias、自动转换或双版本 routing。

本 Change 不纳入 tokenizer backend 替换、fast-read threshold probing、stdout streaming、最终序列化 response 的精确 size budgeting、billing-grade token accounting，或一般性 producer framework。`info` success、failure envelope、readable framing 和 invocation log 不进入普通 output budget；host、allocation、serialization 和 I/O failure 继续使用既有 failure boundary。

## Success Criteria

- Public limit、shared calculator、Session report 和 bounded output cost 复用同一个 closed `CostUnit::{Lines, Bytes, Tokens}`；Limited 只执行所选 unit 的 calculator。
- `docnav-text-cost` 能为 text selection 返回 deterministic UTF-8 prefix；重新使用同一 unit calculator 计数时等于 reported cost 且不超过 limit，full text fits 时保持原文与 complete，任意正数 limit 都允许 empty incomplete prefix。
- Structured outline/find 把一个完整 `Entry` 作为原子 input；放不下当前 item 时在该 item 前停止，不裁剪或软突破 item。Accepted item 的 cost 使用该 item 的 canonical compact protocol JSON object，不包含 collection delimiter 或 root envelope。
- CLI `--limit <unit>:<positive-integer>` XOR `--ignore-limit`、machine `limit:{unit,value}` XOR `ignore_limit:true`、config `defaults.output_limit` closed union 和 built-in `tokens:6000` 都确定性形成一个 `OutputConstraint`；非法组合在 adapter dispatch 前失败。
- AdapterDocument 的 content operations 使用 caller-owned producer、projection 和 Collector 驱动 Limited/Unbounded `OutputSession`；Limited stop 后不访问 producer tail，Unbounded 不构造或调用 `InputCost`。
- Base operation content 先消费预算；success-only nested auto-read 只消费同一 invocation 的剩余预算。Public cost 是各 accepted phase cost 之和，complete 对 base 与成功选择的 nested content 做 invocation-level 汇总。
- Affected success 使用 common output union：Limited 报告原始 limit、同 unit actual cost 和 complete；Unbounded 不报告 unit/cost 且 `complete:true`。`info`、failure 和 invocation log 不获得该 sidecar。
- Raw protocol 与 readable presentation 消费同一个已完成、已校验的 typed response；renderer、serializer 和 stdout 不观察 partial Session state，也不成为第二个 budget/complete owner。
- Protocol `0.2`、CLI/config surface、adapter contract、built-in adapters、stable owner、schema/examples、tests 和 release artifact 不再保留 page、next-page、continuation、numeric-only public limit 或 pagination-enabled normalization。
- Focused Rust tests、真实开发 CLI smoke、canonical release-package smoke、schema/docs validators、完整 Semantic Case check 和 `bun run verify:docnav-workspace` 共同证明 hard cutover；已知 package smoke baseline failure 必须在最终 release evidence 中关闭。

## Affected Owners

稳定规范与 instruction owners：

- [`AGENTS.md`](../../AGENTS.md) 与 [文档导航](../../docs/navigation.md)：CLI-first 不变量和“有限、可继续”表述。
- [架构](../../docs/architecture.md)：Current `OutputSession` capability、adapter producer integration、navigation response boundary 和 shared crate responsibility。
- [CLI](../../docs/cli.md) 与 [Navigation Input Resolution](../../docs/navigation-input-resolution.md)：参数声明、source resolution、config shape、默认值和迁移诊断。
- [原始协议](../../docs/protocol.md)、[适配器契约](../../docs/adapter-contract.md)与[输出模式](../../docs/output.md)：protocol `0.2`、adapter execution handoff、output facts 和 presentation boundary。
- [Markdown Adapter](../../docs/adapters/markdown.md)与 [JSON Adapter](../../docs/adapters/json.md)：format-owned producer、ref、selection、Entry/text inputs 和旧分页删除。

实现与验证 owners：

- [`docnav-protocol`](../../crates/shared/protocol/)、[`docnav-text-cost`](../../crates/shared/text-cost/)、[`docnav-output-session`](../../crates/shared/output-session/)、[`docnav-adapter-contracts`](../../crates/shared/adapter-contracts/)、[`docnav-navigation`](../../crates/shared/navigation/) 和 [`docnav-output`](../../crates/shared/output/) 的 shared contract。
- [`crates/adapters/markdown`](../../crates/adapters/markdown/)、[`crates/adapters/json`](../../crates/adapters/json/) 与 [`crates/docnav`](../../crates/docnav/) 的 producer、input surface、runtime orchestration 和 invocation metadata。
- [`docs/schemas/`](../../docs/schemas/)、[`docs/examples/`](../../docs/examples/)、[测试策略](../../docs/testing.md)、[覆盖矩阵](../../docs/testing/coverage.md)、[`docs/testing/cases/`](../../docs/testing/cases/) 和 release-package/smoke artifacts。
