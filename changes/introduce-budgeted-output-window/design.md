# Design

设计以 `BudgetedOutput` 字段投影连接任意语义结果和统一 OutputWindow，不要求所有 operation 改成相同结构，也不把 calculator mechanics 放进生成宏。

## Context

- Current `OperationResult` 包含不同的 outline、read、find 和 info 结构；structured outline/find 还可以嵌套 auto-read。
- Current Markdown 和 JSON adapter 在 operation 内分别分页并形成 cost，shared output 随后分别投影 raw/readable 结果。
- Authority boundary — `.change-plan.json` 拥有本 Change 的 lifecycle；本 design 只拥有 change-local runtime Target。稳定 protocol、output 和 adapter owners 继续定义 Current。
- Long-term direction — [在标记的语义字段上集中执行输出预算](../../docs/decisions/product-direction/centralize-output-budgeting-over-marked-semantic-fields.md)保存统一预算数据流的未来方向。
- Token direction — [保留当前 reference tokenizer，直到可靠替代已具备](../../docs/decisions/product-direction/retain-current-reference-tokenizer-until-qualified-replacement.md)确认项目已经拥有统一 token calculator；本 Change 基于 current `o200k_base` 语义补足 bounded path。
- This Change owns — `BudgetedOutput` traversal、OutputWindow mutation、CostCalculator dispatch 和 internal OutputReport；[replace-pagination-with-unit-output-limits](../replace-pagination-with-unit-output-limits/design.md)拥有 public budget/result contract。
- Calculator boundary — 本 Change 拥有满足公共预算不变量的 calculator path，包括 current backend 上的 correctness-first token prefix wrapper。
- Separate integration — [integrate-fast-read-budget-probing](../integrate-fast-read-budget-probing/design.md)消费 probe capability，不由本 Change 修改 fast-read navigation behavior。
- Primary proving surface — read text 是首个性能与裁剪证明面；outline、find、unstructured result 和 nested auto-read 必须在本 Change 完成前映射到同一机制。

## Goals / Non-Goals

Goals:

- 让任意受支持 result struct 或 enum 静态声明参与预算的字段。
- 在一个 window 中完成 unit-specific measurement、admission、裁剪、完整性状态和实际 output cost。
- 保持裁剪后的 typed result 对 raw protocol 和 readable renderer 都合法且一致。

Non-Goals:

- 不用最终 serialization length 定义产品 limit。
- 不在宏中选择 tokenizer、默认 limit 或 public response shape。
- 第一阶段不要求 adapters 改成 lazy iterator 或 streaming producer。
- 不在本 Change 迁移 fast-read selector；相邻 Change 只复用这里提供的 probe 能力。

## Decisions

### 1. 四个责任名称保持稳定

- `BudgetedOutput`：结果类型暴露预算字段的 typed traversal contract。
- `OutputWindow`：持有本次 budget state，并执行 admission、裁剪和完整性判断。
- `CostCalculator`：只负责指定 unit 的 measurement 与合法 prefix boundary。
- `OutputReport`：独立于 operation payload，保存实际接纳成本和 complete 状态。

### 2. Trait 是契约，宏只是生成方式

核心接口是可测试的 `BudgetedOutput` traversal 与 OutputWindow。字段 attribute 或 declarative macro 只生成静态访问代码；如果 crate dependency 或实现规模不支持 proc macro，可以先使用手写 trait implementation，而不改变 runtime contract。

### 3. 预算发生在 semantic result 与 presentation 之间

Adapter 先返回 typed semantic result，OutputWindow 随后原地约束被标记字段并返回 sidecar report，raw/readable renderer 最后消费同一结果。不得先转成 generic JSON 再截断。

### 4. 保持少量字段策略并强制分类增长字段

- `text` 使用 calculator 返回的合法 prefix boundary 裁剪。
- `sequence` 按元素顺序接纳并在 item boundary 停止。
- `nested` 递归使用当前 window。
- 必须完整保留的 scalar 或 identifier 使用 atomic accounting，不做字符串截断。

所有可能随 document、query、adapter metadata 或其它输入规模增长的输出字段都必须显式参与预算；只有另有独立上限的字段才能显式跳过。新增长度不受约束的字段时，缺少分类必须成为 compile-time failure 或等价的结构检查失败。

### 5. CostCalculator 只按请求单位工作

OutputWindow 持有一个 calculator session 和剩余预算。Unit backend 可以是 bytes、lines 或 tokens，但只调用当前 limit 和显式报告需要的 measurement，不先计算所有单位再过滤。

Token calculator path 必须返回 UTF-8 合法、重新计数一致且不超过 budget 的 prefix。第一版可以完整扫描或重新计数 current `o200k_base` backend，只要真实 workload 的资源验证通过；CostCalculator contract 不承诺 backend identity、算法复杂度或 native early-stop。

### 6. Report 与 payload 分离

内部返回 `Budgeted<T> { value, output }`；OutputReport 至少保存实际 cost 和 complete 状态。Public protocol 如何投影由 budget-contract Change 决定，不要求每个 operation 自己存放 controller state。

## Risks / Trade-offs

- Post-result budgeting 限制 output 和 measurement work，但不会避免 adapter 先构造完整 entries；只有 profiling 证明该部分成为瓶颈时才扩大到 producer-time API。
- Text 可以安全裁剪，entry、ref 和嵌套 auto-read 存在原子性选择；策略过多会把宏变成第二套业务规则 owner。
- 只计标记字段意味着 cost 不是最终序列化大小；需要确认固定结构和逐 item 包装不会让输出安全目标失真。
- Correctness-first token prefix 可能完整扫描或重复计数；它可以关闭公共能力门，但必须用真实 workload 证明不会让 Limited token 请求超出本 Change 的资源预算。
- Proc macro 可能增加 crate 和编译复杂度；必须先证明它比少量手写实现更容易维护。

## Open Questions

以下问题仍属于本 draft 的 runtime design 缺口；关闭后才能选择生成机制并派生 Plan tasks：

1. 当前结果类型数量是否足以证明 proc-macro derive 优于 declarative mapping 或手写 trait？
2. Entry 的 ref、label、summary、excerpt、cost 和 metadata 哪些作为 atomic item 参与预算？
3. Sequence item 超过剩余预算时是否允许裁剪 item 内的 optional text，还是整项拒绝？
4. Info 与 error payload 是否进入普通产品预算，还是只受 transport ceiling？
5. 如何为新增未标记的可增长字段提供 compile-time failure 或结构审计？
6. 是否需要为同一已测文本保存 sidecar measurement，以便 fast-read 与最终 output 复用而不重复 tokenize？
