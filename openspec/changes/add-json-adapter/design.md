**目标：用 adapter-private JSON document model 实现既有 fixed strategy，并通过 static registry 和同一 core binary 交付。**

**状态：本文描述 candidate design。Decision Map 中除 A5a 的 E1 source-order 实验外，其余当前 change 方向与交付顺序已确认。当前 change 使用既有 generic `readable-view`，格式专用自定义渲染由相连的后续 change 完成。实施从 `tasks.md` 0.5 关闭后开始；Current 能力仍以 `docs/`、代码和测试为准。**

## Context

当前 core registry 通过静态 factory slice 注册 `markdown_adapter_definition`，navigation 选择 `AdapterDefinition` 后把 closed `StandardOperationInput` 分派到固定的 outline/read/find/info strategy。Core catalog 与 compile-time binding 共同拥有 caller-configurable input inventory；adapter definition 组合 manifest、strategies 和 optional capability。

JSON adapter 将成为第二个真实实现。它沿现有 crate、factory、registry、probe、closed input、protocol result 和 release package 路径交付，并使用 workspace-pinned `serde_json` 作为 parser/serializer。实际接入摩擦用于判断现有边界是否足以承载第二种格式。

长期理由由活动决策记录拥有，目标行为由 capability delta 拥有。本 design 只负责把二者映射到 JSON change，并定义 implementation mechanics、E1 gate、风险、验证分层以及自定义渲染后续阶段的交接。

## Decision Map

这张表是本 change 的决策入口。长期记录完整保存目的、理由和可独立修订的边界；“JSON 应用”只说明本 change 如何消费该方向。`CONFIRMED` 表示方向已可用于设计，`EXPERIMENT` 表示当前实现 gate，`SEQUENCED` 表示由相连的后续 change 继续交付。

| ID | 长期 owner | JSON 应用 | 状态与剩余动作 |
| --- | --- | --- | --- |
| A0 | [检验共享抽象](../../../docs/decisions/adapter-evolution/validate-boundaries-with-real-adapters.md)；[检验完整 adapter 行为](../../../docs/decisions/adapter-evolution/validate-boundaries-with-complete-adapter-behavior.md)；[选择 JSON](../../../docs/decisions/adapter-evolution/select-json-as-second-adapter.md) | 第二个真实 adapter 使用 JSON，并依次走通 fixed operations、full-read、generic output 与后续格式专用 readable presentation；依据类型明确为架构验证 | **CONFIRMED** |
| A1 | [JSON tree-path ref](../../../docs/decisions/json-navigation/use-canonical-json-pointer-refs.md) | 公开 ref 使用 canonical、ASCII-safe 的 `json:#<RFC 6901 URI fragment>`；core 保持 opaque pass-through | **CONFIRMED** |
| A2 | [保留 number token](../../../docs/decisions/json-navigation/preserve-number-tokens.md) | Raw token 是 number 的可观察文本身份；arithmetic equivalence 位于导航能力之外 | **CONFIRMED** |
| A3 | [唯一 object member](../../../docs/decisions/json-navigation/reject-duplicate-object-members.md) | Probe 只接受 decoded member name 唯一的 object，使 parser 结果和 ref identity 都保持唯一 | **CONFIRMED** |
| A4 | [Adapter-private depth](../../../docs/decisions/json-navigation/keep-depth-limit-adapter-private.md) | Root depth 为 `0`、最大 depth 为 `127`，由单一私有硬编码配置拥有；公共 input inventory 保持当前契约 | **CONFIRMED** |
| A5a | [源码顺序成本策略](../../../docs/decisions/navigation-output/treat-source-order-as-costed-format-policy.md) | 优先在既有 adapter-private model 中低成本保留 object source order，否则采用确定性 model order | **EXPERIMENT** — 完成 E1 并把实际顺序同步到 delta、tasks 和测试目标 |
| A5b | [JSON structured output](../../../docs/decisions/json-navigation/normalize-structured-json-output.md)；[structured/full-read 分层](../../../docs/decisions/navigation-output/separate-structured-and-source-reads.md) | Raw structured read 使用 pinned serializer 的自然 spelling 和两空格布局；number、duplicate member 与 E1 order 是明确例外，full-read 返回原文 | **CONFIRMED** |
| A5c | [自定义渲染边界](../../../docs/decisions/navigation-output/keep-custom-rendering-in-readable-view.md) | 当前 change 用 generic `readable-view` 走通每个 operation 并记录格式假设；后续 change 基于稳定 raw facts 确定 JSON 信息密度、层级、标点、preview、分页显示与 renderer mechanics | **SEQUENCED** — 当前 raw adapter 在 E1 后实施，完成后交接到必需的 presentation 验证阶段 |
| A7 | [原文 find](../../../docs/decisions/json-navigation/search-original-json-source.md) | JSON `find` 对 BOM-stripped 原文执行 literal search，并把源码命中确定性映射为可继续读取的 JSON ref | **CONFIRMED** |

A6 是范围边界：本 change 拥有 JSON definition 的追加和兼容性证据；registry 整体治理继续由既有 owner contract 承接。

### E1 Object source-order experiment

E1 只回答“保留 object source order 的边际成本是否值得”。实验基于 A2/A3 所需的同一 adapter-private decode model，并比较：

1. A7 所需 node/member source regions 是否已让 member pairs 自然保持 source order，或只需局部 `Vec`/等价表示。
2. 现有 parser 和私有表示能否直接承载顺序，以及额外全量副本的资源成本。
3. 对 traversal、ref resolution、structured read 和 pagination 的分支/复制成本。
4. 在 source-find region baseline 上，代表性 mixed tree 的内存与实现复杂度是否出现明显回归。

接受条件：现有 parser 和 adapter-private tree 能以局部表示自然承载 source order，且 memory、branching 和 maintenance 成本保持在当前模型量级。满足时把 source order 固化为 JSON contract；否则选择 parser/model 的确定性顺序。E1 输出只修改 JSON 私有表示，并在实施前同步 design、delta、tasks 和测试目标。

### A5c `readable-view` 自定义渲染（后续验证阶段）

Raw protocol `read.content` 已由 A5b 决定。当前 change 先让 probe、outline、read、find、info 和 full-read 经过现有 generic `readable-view`，并记录它对 JSON 暴露的格式假设。

格式专用自定义渲染需要同时确定信息密度、层级、标点、preview、分页显示和 renderer 选择 mechanics，因此由相连的后续 change 单独承接。该阶段不是可选美化；完成它之后，JSON adapter 才形成覆盖完整行为的边界验证证据。

## Target Scope

### Delivered Behavior

- 导航常见 UTF-8 JSON 文件，并支持自动/显式选择及完整 operation surface。
- 用确定性树遍历和 adapter-owned、CLI-safe JSON Pointer refs 支持 `outline/find -> ref -> read`。
- `find` 直接搜索 BOM-stripped 原文，并按源码位置把命中映射为 JSON-owned ref。
- Structured read 使用两空格 pretty layout 并接受 workspace-pinned parser/serializer 的自然 string/lexeme 输出；object member 顺序由 E1 决定，JSON number token 仍必须原样保留。
- Core parameter catalog、closed operation input、protocol 和 raw output shape 保持当前契约。
- 在 package 中通过同一个 `docnav` binary 验证 Markdown 与 JSON。

### Contract Boundaries

- Delivery shape 是包含 static JSON factory 的单一 core executable。
- Public input 是既有 closed standard operation input；JSON safety limit 由 adapter-private 配置拥有。
- 支持范围是单个 UTF-8 JSON value；其它 JSON-like syntax 和 schema-aware semantics 由独立能力决定。
- Structured read 表达 JSON value，full-read 表达原文；当前 generic `readable-view` 证明既有输出链路，后续格式专用 presentation change 完成 JSON 自定义渲染。
- 初始实现使用完整内存 model 和既有 pagination；性能扩展由测量结果触发。
- Markdown、shared protocol 和 generic output renderer 继续遵循各自现有 owner contract。

## Change-local Design

本节拥有 change-local implementation mechanics；长期方向由 Decision Map 链接的记录拥有，observable target 由 capability delta 拥有。A5a 的 E1 gate 关闭后按 C1–C5 开始实施。

### C1. 复制现有静态 adapter 形状

- 新增 `crates/adapters/json`，导出唯一 registry-facing `json_adapter_definition()`。
- JSON crate 只依赖既有 adapter contracts、protocol、text-cost、`serde` 和 `serde_json`；core 只增加 workspace dependency、registry import 和一个 static factory entry。
- `AdapterDefinition`、`NavigationAdapterRegistry`、`StandardOperationInput`、`StandardInputBinding` 和 parameter catalog 是本 change 的输入契约。基础 JSON operation 若超出该契约，实施暂停并回到边界评审。
- Registry 修改追加 JSON，并按 registry owner 的当前 membership 与 ordering contract 保留其它 definitions。

### C2. 每次 operation 建立一个 adapter-private document model

- 每次 strategy 调用读取原始 bytes，记录包含可选 BOM 的文件 byte size，去除一个可选 UTF-8 BOM 后解码，并由 `serde_json::Deserializer` 消费完整输入。
- 同一次调用只 parse 一次。Document model 保存 BOM-stripped source、parsed tree、raw number token、node/member source region，以及 traversal、source find、ref resolution、serialization、node count 和 depth 所需的私有事实。
- Duplicate-member 检测和 depth check 在 decode 层完成；number 使用 `serde_json` raw-value 能力或等价私有 mechanics；workspace-pinned `serde_json` 是唯一 parser package。
- Array 始终保持 index 顺序；object 的内部表示采用 E1 确认的顺序模型。
- Probe 成功后的 operation 会重新加载文档；重新加载失败使用 C5 的既有 error boundary。

### C3. 分离树遍历与原文命中序列

- Adapter-private tree index 保存 depth-first preorder node sequence、value kind 与 ref；outline entry set 和 root 行为以 capability delta 为准。
- `JsonRef` 私有 helper 按 A1 和 capability delta 完成 encode、parse、canonical validation 与 resolution；JSON adapter 是 grammar owner。
- Outline 先形成确定性 entries，再复用现有 entry pagination；超长 item 仍保留完整 ref、最小非空 label，并保证 page 前进。
- Adapter-private source index 保存与 tree ref 相连的嵌套 node/member byte regions。Find 扫描 BOM-stripped source，并用该 index 将 source occurrence 映射为 match。
- Tree preorder 与 source occurrence order 分别保持确定性，并各自复用既有 entry pagination；source-region 归属、重复命中和 entry facts 以 capability delta 为准。
- Capability delta 是 read、full-read、info、cost 和 presentation observable shape 的唯一 change-local owner。

### C4. 验证与实施观察按 owner 分层

- JSON crate tests 证明 decode、traversal、ref、operation、pagination 和 JSON-owned errors。
- Core tests 证明 static registry、selection 和 closed-input 交接；CLI smoke 证明真实进程；release smoke 证明 package 中同一 binary 可以执行 Markdown 与 JSON。
- Registry/package 证据验证 JSON membership，并按 registry owner 的当前契约验证其它 definitions 与代表性行为。
- 实现结束后追加 `## Implementation Observations`，记录实际接入点、shared contract/catalog 变化、跨 adapter 重复、职责绕行，以及 generic `readable-view` 暴露的格式假设。格式专用渲染进入已要求的后续 presentation change；其它有证据的后续抽象进入独立 change。

### C5. 保持既有 protocol 与 process boundary

- JSON 通过既有 document commands、output plans、protocol envelopes、diagnostic projection 和 process exits 暴露；Markdown 输出与参数解析构成兼容性基线。
- Probe 阶段的 malformed JSON、重复 member、depth 超限或 trailing input 作为 unsupported candidate 处理，并沿用 automatic/declared selection failure。
- Probe 成功后文档变为 syntactically invalid、出现重复 member 或 depth 超限时，operation 返回 `INTERNAL_ERROR` 和 `json-document-changed-after-probe`；文件消失与编码变化继续使用既有 document diagnostics。

## Risks / Trade-offs

| 风险 | 当前处理 | 后续触发条件 |
| --- | --- | --- |
| 完整 parse 与 entry materialization 占用大文件内存 | `max_depth` 负责递归边界，output limit 负责返回预算；初始 input model 仍是完整内存表示 | 测量出现真实 CPU、memory 或 latency 瓶颈后，另行设计 streaming 或 input budget |
| Object source order 需要额外 parser、全量副本或广泛分支 | E1 只接受当前 model 量级的局部成本；其它结果使用确定性 model order | E1 结果同步到全部目标契约后关闭 0.5 |
| Pinned parser/serializer 升级改变 structured output | Owner tests 固定可观察结果，依赖升级时复核对应长期决策与 JSON contract | Workspace dependency 版本或 feature 变化 |
| 原文命中需要稳定映射到 JSON node | Adapter-private source region 明确定义 occurrence 归属；find tests 覆盖 key、scalar、结构文本、空白、跨节点和重复 ref | Parser mechanics 无法以当前 model 量级提供可靠 region 时回到 JSON-private model 设计 |
| JSON ref 表达当前结构路径 | 文档变化后同一路径可以指向新值或返回 `REF_NOT_FOUND` | 跨版本持久身份需求进入独立 contract 设计 |
| 第二个 adapter 暴露 closed contract 无法表达的职责 | 记录真实阻塞并回到 adapter boundary 评审 | 基础 JSON operation 无法沿 C1–C5 落地 |

## Delivery and Reversal

1. **Entry:** 关闭 E1，更新 Decision Map、capability delta 和测试目标。
2. **Delivery:** 建立 JSON owner 文档与证据，再按 adapter-private model、fixed strategies、static registry、generic readable output、CLI/package smoke 的顺序交付；精确任务以 `tasks.md` 为准。
3. **Audit:** 追加 Implementation Observations，把格式专用渲染交接给相连的后续 change，并对活动决策执行 owner、代码、测试和 release evidence 对齐检查。
4. **Reversal:** 该能力是 additive；发布前可以移除 JSON registry entry 和 crate，现有 adapter 与 shared protocol 继续使用当前基线。

## Open Questions

1. **E1:** Object source order 能否由 adapter-private model 以已定义的低成本条件保留？

A0–A4、A5b、A5c 的交付顺序、A6 和 A7 已在 Decision Map 中定案；当前 change 的 Open Questions 只保留 E1。JSON 格式专用 presentation contract 由已排序的后续 change 决定。
