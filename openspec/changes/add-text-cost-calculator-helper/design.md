本 design 记录通用 text cost calculator helper 的设计方向，并把 Markdown adapter 作为首个接入方。当前实现尚未开始；本 change 基于 current `cost.measurements[]` shape，不重新打开 raw protocol 或 readable 输出字段设计。

## Context

Docnav 已把机器稳定协议字段和 readable 展示分层处理：raw protocol 使用 `cost.measurements[]`，readable 输出从 measurements 派生成本摘要。当前 Markdown adapter 已报告 `lines` 和 `bytes`，但计算逻辑留在 Markdown 私有实现中；token cost 也应复用同一个 helper 边界。Token calculation 依赖选择 `tiktoken-rs`，encoding 固定为 `o200k_base`，不引入独立 adapter policy API、model preset API 或用户可选 tokenizer API。

这个 change 的核心承诺是最小通用 text cost calculator：helper 由一组同签名函数组成，每个函数只接收已选纯文本并返回对应 cost measurement。调用方通过选择调用哪个函数来选择 cost 类型；格式解析、区域选择、函数集合选择、ref 定位和输出展示仍由各自 owner 负责。helper 不需要知道文本来自 Markdown、PDF、HTML、OCR 还是其它 adapter。

## Goals / Non-Goals

**Goals:**

- 在 shared library 中提供可复用的 text cost calculator helper。
- 提供一组统一接口的 helper functions：每个函数只接收纯文本并输出 protocol-compatible cost measurement。
- 首期提供 `lines`、`bytes` 和基于 `tiktoken-rs` `o200k_base` 的 token cost helper functions，且 token cost 遵守同一个 text-only 接口。
- 让任何已经拥有纯文本的 Docnav 组件都可以直接调用所需函数；Markdown adapter 是本 change 的首个接入方。
- 让 Markdown adapter 自己选择 helper function 集合、scope 和排序；首期在现有 read selection 与 outline entry cost surfaces 使用 helper 结果表达 selected text cost。
- 在实现时同步 owner 主规范、schema/example、fixture 和测试，证明 raw protocol shape 与 readable summary 分层不变。

**Non-Goals:**

- 不开放用户选择任意分页预算方案。
- 不要求所有 adapter 或所有调用方立即展示同一组 cost unit。
- 不让 helper 自动选择或返回默认 measurement 集合。
- 不引入接收 unit、adapter policy 或 strategy object 的复杂 calculator API。
- 不开放调用方选择 tokenizer dependency、encoding 或 model preset；token helper 使用 `tiktoken-rs` `o200k_base`。
- 不让 helper 解析格式源文件、选择文档区域或重建 adapter navigation 语义。
- 不让 core 从 adapter output 之外重新推断格式语义。
- 不改变 raw protocol `cost.measurements[]` shape 或 readable output payload shape。

## Decisions

### Decision 1: Current protocol shape 是输出边界

本 change 使用 current `Cost { measurements: Vec<Measurement> }` 作为 raw protocol 成本事实容器。Helper 输出的 cost measurement 必须能直接放入该容器；readable output 层继续从 measurement list 派生成本摘要。本 change 不新增 raw `cost` 字段、不把 readable-only 字段写入 raw protocol，也不改变 response envelope。

### Decision 2: Helper functions 统一为 text -> cost

Helper 必须提供一组同签名函数，概念接口为“text -> cost measurement”。每个函数只接收纯文本，函数自身定义返回 measurement 的 unit，value 由 helper 根据 text 计算。调用方不需要提供 adapter id、format id、operation、ref、path、parser state、unit 参数或策略对象。Helper 返回的 measurement 不附加 `scope`；需要在 protocol result 中表达 `scope` 时，调用方在嵌入结果时附加 scope，或使用不改变 helper function 接口的薄 wrapper。

### Decision 3: 调用方通过选择函数来选择 cost 类型

Adapter 或其它调用方继续拥有要调用哪些 helper function、以什么顺序报告、是否附加 scope、是否暴露到 public output 的策略。Core、output layer 或 helper 不替 adapter 选择 cost function 集合；adapter 需要成本 measurement 时自己直接调用所需函数。这个边界允许 Markdown、未来其它 adapter、输出工具或验证脚本复用同一个计算机制，而不把格式策略塞进共享层。

### Decision 4: Helper 不拥有文本选择

Helper 的输入是调用方已经选好的纯文本。调用方继续拥有格式解析、document region selection、content truncation、ref 定位、native option 语义和是否把 helper 结果放进 public output。

### Decision 5: Markdown 是首个消费者，不是 helper 的边界

Markdown adapter 首期把 read selection、outline entry section 或 `doc:full` 等已经选中的 Markdown 文本交给它自己选择的 helper functions。Markdown adapter 继续拥有哪些 operation/scope/function 报告 cost；helper 只替代 Markdown 私有的文本成本计算。

### Decision 6: Token helper 使用 tiktoken-rs o200k_base

Token cost helper function 使用 `tiktoken-rs` `0.12.0` 的 `o200k_base` encoding 计算 token value，并保持与其它 helper function 相同的 text-only 接口。实现使用 `o200k_base_singleton()` 避免每次调用重新初始化 tokenizer，并使用 ordinary plain-text token counting；文本中形似 special token 的字符串按普通输出文本计数，不作为 tokenizer control token 解释。这些选择不得改变 helper 的输入形状，也不得让 adapter 传入 tokenizer policy、encoding 或 model preset。

依赖审计结论：

- `tiktoken-rs` `0.12.0` 的 license 是 MIT，与 workspace license 兼容。
- `tiktoken-rs` `0.12.0` 要求 Rust 1.85+；本 change 接受该 floor，实施时需要确认 CI 和 release toolchain 不低于 1.85。
- `o200k_base` tokenizer assets 随 crate 发布并通过 crate source 嵌入；依赖下载完成后，helper 初始化不得依赖运行期网络请求或外部 tokenizer 文件。
- `o200k_base_singleton()` 内部只初始化一次 tokenizer；helper 不应在每次 `token_cost` 调用时构建新的 tokenizer。
- `tiktoken-rs` 只由 `docnav-text-cost` 依赖；`docnav-protocol`、`docnav-output`、core 和不使用 token cost 的调用方不拥有 tokenizer policy。

### Decision 7: Helper 放在 docnav-text-cost crate

新增 shared crate `docnav-text-cost` 作为 text cost calculator owner。该 crate 依赖 `docnav-protocol` 以返回 protocol-compatible `Measurement`，并依赖 `tiktoken-rs` `0.12.0` 以实现 token cost；`docnav-protocol`、`docnav-output` 和 core 不反向依赖 tokenizer。Adapters 或其它调用方需要 text cost 时依赖 `docnav-text-cost` 并直接调用所需 function。

### Decision 8: 首期 public API 和 unit 固定

`docnav-text-cost` 首期暴露三个同签名 public functions：`line_cost(text: &str) -> Measurement`、`byte_cost(text: &str) -> Measurement`、`token_cost(text: &str) -> Measurement`。返回 unit 分别固定为 `lines`、`bytes`、`tokens`，value 为非负整数。Helper 不返回 `Cost` container，不附加 `scope`，不提供自动组合默认集合的 API；调用方负责把 measurements 放入 `Cost { measurements }` 并按需要附加 scope。`token_cost` 必须是 infallible public API；tokenizer 初始化失败属于依赖或构建问题，应由 implementation tests 和 workspace validation 暴露，而不是扩散为 adapter contract。

### Decision 9: Markdown 首期接入现有 cost surfaces

Markdown adapter 首期只替换当前已有 cost surfaces：read result 使用 `selection` scope，outline full entry 和 heading section entry 使用 `entry` scope。Markdown adapter 对每个 selected text 调用 `line_cost`、`byte_cost`、`token_cost`，并按 `lines`、`bytes`、`tokens` 顺序写入 `cost.measurements[]`。readable output 继续使用现有 generic measurement summary，不新增 token-specific readable rule。

### Decision 10: 分页预算选择暂不对用户开放

Helper 可以为后续预算策略留下复用基础，但本 change 不新增用户可配置的 budget unit，也不要求 CLI/protocol 暴露预算函数选择。Markdown adapter 当前把 `limit` 解释为 Unicode 字符预算；除非后续任务明确更新 Markdown owner 文档、测试和用户可观察行为，否则不把 Markdown pagination budget 从字符切换为 token。

## Risks / Trade-offs

- Helper 抽象过度会侵占函数集合选择、文本选择或格式解析职责；实现时需要用 API shape 和 tests 证明 helper functions 只接收纯文本并返回对应 cost measurement。
- `tiktoken-rs` 依赖会提高 Rust floor 到 1.85+，并增加 tokenizer assets 的 release package 体积；implementation checklist 需要覆盖 toolchain、license、offline build 和 package impact。
- 新增 `tokens` measurement 会让 readable cost summary 变长；实现时需要用 Markdown readable/raw fixture 证明 output 层仍只做 generic measurement summary。
- 若把某个 cost measurement 误用为 pagination budget，会改变 continuation 行为；首期实现必须保持 Markdown `limit` 语义不变，除非另起 change。
