# Proposal

本 Change 把 `outline`、`read` 和 `find` 的公开分页契约一次性替换为使用统一 `CostUnit` 的输出上限，以及显式绕过该上限的 `ignore-limit` 意图。

## Why

Current 请求同时携带 adapter-owned 正整数 `limit` 和数字 `page`；Markdown 与 JSON 的主要分页路径把 `limit` 解释为 Unicode 字符预算，而协议成本事实使用共享的 `lines`、`bytes` 和 `tokens`。关闭 pagination 时，navigation 又把 effective limit 规范化为最大正整数。调用方因此无法只从公共请求判断 limit 的成本单位、实际输出成本或完整性，adapter、protocol、readable output 和 continuation 还要共同维护 page 语义。

[用带单位的输出上限替代分页](../../docs/decisions/product-direction/replace-pagination-with-unit-output-limits.md)已经确认长期方向：默认调用保持有界，public `page` / continuation 被移除，显式 `{ unit, value }` limit 与 `ignore-limit` 互斥。本 Change 将该方向落实为一个 hard-cutover public migration。

## Outcome

`outline`、`read` 和 `find` 的有界调用使用 `Limit { unit, value }`，其中 unit 是统一 `CostUnit` 的 `lines | bytes | tokens` 之一，一次调用只选择一个 unit。该 limit 约束 operation 新返回的内容 payload，不约束协议包装、请求 ref 回显和其它固定必需 metadata；因此它是内容输出上限，不是最终序列化 response 的总大小上限。省略显式输入时使用 core-authored built-in preset；第一版 preset 为 `tokens:6000`，它是可调整的产品默认值，不改变三种合法 unit 或架构边界。

显式 `ignore-limit` 在 input resolution 后形成 `Unbounded`，绕过普通 limit enforcement 和 unit-specific output measurement，返回完整选定结果。Bounded 与 unbounded success 使用同一 common output metadata 区分模式和完整性；请求、结果和 adapter contract 不再包含 page、next-page 或其它 continuation。

## Scope

本 Change 负责：

- `outline`、`read`、`find` 及其 structured、unstructured full-read 和 nested auto-read success 路径的 limit、完整性与 bounded output cost 公共语义。
- CLI、project/user config、core parameter catalog、input resolution、protocol request construction 和 invocation metadata 中的 page、pagination、limit 与 `ignore-limit` surface。
- Protocol `0.2` success envelope / operation result、generic `readable-view`、adapter closed input 与 built-in Markdown / JSON adapter 的一次性迁移。
- 对应稳定 owner、JSON Schema、contract examples、fixtures、Semantic Case 和实现测试的同步。
- 对旧 `0.1` request/result shape、`--page`、`--pagination`、numeric-only `--limit` 与 `defaults.pagination.*` 的明确拒绝和迁移 guidance；不提供 runtime 兼容读取或双重执行语义。

普通 output limit 不覆盖 `info` success、failure envelope、protocol 固定包装、readable framing 或 invocation log。Tokenizer implementation、Budgeted Output Window 的 runtime traversal / calculator mechanics、fast-read threshold probing、serializer 后精确字节 budgeting，以及 streaming / lazy producer 优化继续由相邻 owner 负责。

## Success Criteria

- `CostUnit` 在 limit、calculator 和 bounded output cost 中使用同一个 closed enum：`lines | bytes | tokens`；一次 `Limited` 调用只执行所选 unit 的 calculator。
- CLI、config 和 machine request 都能确定性映射为 `Limited { unit, value }` 或 `Unbounded`；两者并存、非法 unit/value 和旧 pagination input 在 adapter dispatch 前失败。
- `outline`、`read`、`find`、unstructured full-read 与 nested auto-read 的 bounded success 不超过所选 unit 的内容字段预算，并通过 common output metadata 可靠区分 complete 与 incomplete；固定 envelope / root identity metadata 不计入该预算。
- 任意正数 limit 都能产生合法 success：放不下首个 sequence item 时返回 empty incomplete，text 可以返回 empty prefix incomplete，optional nested payload 可以整体省略；不引入只为极小 limit 服务的专用 failure。
- `Unbounded` 不建立普通 OutputWindow，不引入隐藏 limit 或 emergency ceiling；成功时返回完整选定结果，运行时资源或 I/O 故障继续走既有 failure 边界，不伪装为 incomplete success。
- Raw 与 readable output 消费同一个 typed result；两种输出的模式、完整性与 bounded cost 事实一致，presentation wrapper 不成为第二个预算 owner。
- Protocol `0.2`、CLI、config、adapter input 和稳定 docs 中不再存在 page、next-page、continuation 或 pagination-enabled normalization；runtime 不接受 `0.1` pagination shape。
- Protocol/config schemas、contract examples、release fixtures、当前 Semantic Case 映射和范围匹配的 Rust / workspace verification 与最终 owner 文本一致。

## Affected Owners

稳定规范与指令面：

- [`AGENTS.md`](../../AGENTS.md) 与 [文档导航](../../docs/navigation.md)：CLI-first 不变量、“有限、可继续”表述和规则 owner 路由。
- [架构](../../docs/architecture.md)、[CLI](../../docs/cli.md)、[Navigation Input Resolution](../../docs/navigation-input-resolution.md)、[原始协议](../../docs/protocol.md)、[适配器契约](../../docs/adapter-contract.md)与[输出模式](../../docs/output.md)：公共输入、责任分层、预算位置、response 和 presentation。
- [Markdown Adapter](../../docs/adapters/markdown.md)与 [JSON Adapter](../../docs/adapters/json.md)：格式 selection、原子 item、文本截止和 adapter-private 分页移除。

验证材料与实现证据面：

- [`docs/schemas/`](../../docs/schemas/) 和 [`docs/examples/`](../../docs/examples/) 中的 protocol request/response、config 与 contract examples。
- [测试策略](../../docs/testing.md)、[覆盖矩阵](../../docs/testing/coverage.md)、[`docs/testing/cases/`](../../docs/testing/cases/) 和对应测试实体。
- `crates/docnav`、`crates/shared/{navigation,protocol,adapter-contracts,output,text-cost}` 与 `crates/adapters/{markdown,json}` 的对应 contract 和 composition boundaries。
