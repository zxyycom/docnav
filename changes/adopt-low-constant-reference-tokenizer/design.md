# Design

设计先固定候选比较和 calculator capability，再以证据选择并接入真实 tokenizer；不把目标模型 parity 或纯统计公式当成 token contract。

## Context

- Current `docnav-text-cost` 使用 `tiktoken_rs::o200k_base_singleton().count_ordinary` 形成 token measurement。
- Markdown 与 JSON 当前都可以在一次 cost 请求中同时计算 lines、bytes 和 tokens，导致只需要 cheap unit 时仍可能承担 tokenizer 常数。
- Authority boundary — `.change-plan.json` 拥有本 Change 的 lifecycle；本 design 拥有候选证据、选定 backend 和 shared calculator replacement Target。Current token semantics 与实现仍由稳定 owner 和 source 定义。
- Long-term direction — [输出 token 成本使用低常数参考分词器](../../docs/decisions/product-direction/use-low-constant-reference-tokenizer-for-output-cost.md)保存已经确认但尚未成为 Current 的 token 语义和选择约束。
- Contract consumers — [introduce-budgeted-output-window](../introduce-budgeted-output-window/design.md)和 [integrate-fast-read-budget-probing](../integrate-fast-read-budget-probing/design.md)只依赖这里形成的 shared calculator contract，不能依赖某个库的私有类型。
- Public migration — [replace-pagination-with-unit-output-limits](../replace-pagination-with-unit-output-limits/design.md)拥有 observable token-unit 和兼容性边界；本 Change 不自行改写 public schema。

## Goals / Non-Goals

Goals:

- 选择并接入一个版本固定、真实执行 tokenization 的低常数 backend。
- 提供完整计数、预算提前停止和合法文本 prefix boundary。
- 证明 requested-unit laziness、普通语料成本和 adversarial worst cases。

Non-Goals:

- 不与每个 OpenAI 或其它模型 tokenizer 保持 exact parity。
- 不使用 bytes/character ratio 作为正式 tokens 实现。
- 不在本 Change 迁移 page、result shapes 或 fast-read navigation behavior。

## Decisions

### 1. 比较同一个 shared calculator contract

每个候选通过相同的 `measure(text)` 与 `measure_prefix(text, budget)` harness 比较。Prefix API 必须区分完整输入先结束和预算先耗尽，并在 UTF-8 合法边界返回可接纳 prefix。

### 2. 资源常数优先于模型 parity

选择重点是 CLI cold start、CPU、peak RSS、binary/package/transitive impact 和达到 threshold 后的停止成本；相对计数精度用于淘汰明显失真的候选，不追求任一外部模型完全一致。

### 3. 真实 tokenizer 是硬要求

候选必须实际执行稳定 tokenization。字符、字节和语言权重公式只能作为 benchmark baseline，不能成为 production `tokens` backend。

### 4. Unit dispatch 保持 lazy

Shared calculator 按 unit 选择 backend；bytes 只读取 byte length，lines 只执行 line counting，只有 tokens 才初始化 reference tokenizer。

## Risks / Trade-offs

- 更低常数的 tokenizer 可能与现有 o200k token values 明显不同；这是 public cost 语义迁移，需要由协议 Change 明确版本边界。
- 支持 prefix boundary 的库可能增加 offsets、allocation 或依赖成本；必须测量真实 bounded path，而不只测完整 count throughput。
- CJK、emoji、long piece、invalid assumptions 和 adversarial text 可能暴露平均基准看不到的 worst case。
- 新 dependency 需要单独核对 license、维护、MSRV、targets、native build 和供应链成本。

## Open Questions

以下问题是 tokenizer 选择和 production adoption 的证据门禁；没有可复现答案时不得只因平均 throughput 较快就替换 backend：

1. 哪些 tokenizer implementations 进入候选集合，当前实现是否保留为基线？
2. 相对精度以哪些参考模型、corpus weighting 和最大偏差门槛验收？
3. Cold start、CPU、RSS、package 和 adversarial budgets 的具体上限是什么？
4. 候选能否原生提前停止；若不能，哪种 incremental wrapper 仍保持低常数？
5. Tokenizer version 怎样进入 observable contract、info 或 release material？
