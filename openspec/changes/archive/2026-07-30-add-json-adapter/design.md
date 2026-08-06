**目标：用 adapter-private JSON document model 实现既有 fixed strategy，并通过 static registry 和同一 core binary 交付。**

**适用状态：本文定义 `add-json-adapter` 的 change-local mechanics，不表示 Current 能力；当前支持仍以 `docs/`、代码和测试为准。**

## Context

当前 core registry 通过静态 factory slice 注册 `markdown_adapter_definition`，navigation 选择 `AdapterDefinition` 后把 closed `StandardOperationInput` 分派到固定的 outline/read/find/info strategy。Core catalog 与 compile-time binding 共同拥有 caller-configurable input inventory；adapter definition 组合 manifest、strategies 和 optional capability。

JSON adapter 将成为第二个真实实现。它沿现有 crate、factory、registry、probe、closed input、protocol result 和 release package 路径交付，并使用 workspace-pinned `serde_json` 作为 parser/serializer。实际接入摩擦用于判断现有边界是否足以承载第二种格式。

长期理由由活动决策记录拥有，目标行为由 capability delta 拥有。本 design 只负责把二者映射到 JSON change，并定义 implementation mechanics、风险、验证分层以及自定义渲染后续阶段的交接。

## Decision Map

这张表是本 change 的决策入口。长期记录完整保存目的、理由和可独立修订的边界；“JSON 应用”只说明本 change 如何消费该方向。`CONFIRMED` 表示当前 change 已确定该方向，`SEQUENCED` 表示任务 6.4 将该方向交接给专门的后续 change。

| ID | 长期 owner | JSON 应用 | 状态 |
| --- | --- | --- | --- |
| A0 | [检验共享抽象](../../../docs/decisions/adapter-boundary-evidence/validate-shared-abstractions-with-heterogeneous-real-adapters.md)；[检验完整 adapter 行为](../../../docs/decisions/adapter-boundary-evidence/validate-boundaries-with-complete-adapter-behavior.md)；[选择 JSON](../../../docs/decisions/adapter-boundary-evidence/select-json-as-second-adapter.md) | 第二个真实 adapter 使用 JSON，并依次走通 fixed operations、full-read、generic output 与后续格式专用 readable presentation；依据类型明确为架构验证 | **CONFIRMED** |
| A1 | [JSON tree-path ref](../../../docs/decisions/json-navigation/use-canonical-json-pointer-refs.md) | 公开 ref 使用 canonical、ASCII-safe 的 `json:#<RFC 6901 URI fragment>`；core 保持 opaque pass-through | **CONFIRMED** |
| A2 | [保留 number token](../../../docs/decisions/json-navigation/preserve-number-tokens.md) | Raw token 是 number 的可观察文本身份；arithmetic equivalence 位于导航能力之外 | **CONFIRMED** |
| A3 | [唯一 object member](../../../docs/decisions/json-navigation/reject-duplicate-object-members.md) | Probe 只接受 decoded member name 唯一的 object，使 parser 结果和 ref identity 都保持唯一 | **CONFIRMED** |
| A4 | [Adapter-private depth](../../../docs/decisions/json-navigation/keep-depth-limit-adapter-private.md) | Root depth 为 `0`、最大 depth 为 `127`，由单一私有硬编码配置拥有；公共 input inventory 保持当前契约 | **CONFIRMED** |
| A5a | [源码顺序成本策略](../../../docs/decisions/structured-read-semantics/treat-source-order-as-costed-format-policy.md) | Adapter-private decode model 用唯一 member `Vec` 直接保留 object source order；outline 与 structured read 使用该顺序 | **CONFIRMED** |
| A5b | [JSON structured output](../../../docs/decisions/json-navigation/normalize-structured-json-output.md)；[structured/full-read 分层](../../../docs/decisions/structured-read-semantics/separate-structured-and-source-reads.md) | Raw structured read 使用 object source order、pinned serializer 的自然 spelling 和两空格布局；number token 是明确例外，full-read 返回原文 | **CONFIRMED** |
| A5c | [自定义渲染边界](../../../docs/decisions/readable-presentation/keep-custom-rendering-in-readable-view.md) | 当前 change 用 generic `readable-view` 走通每个 operation 并记录格式假设；后续 change 基于稳定 raw facts 确定 JSON 信息密度、完整 opaque ref 的路径定位信号、标点、preview、分页显示与 renderer mechanics，不解析 ref 或合成 hierarchy/depth/parent/indentation | **SEQUENCED** — 由任务 6.4 建立后续 change |
| A7 | [原文 find](../../../docs/decisions/json-navigation/search-original-json-source.md) | JSON `find` 对 BOM-stripped 原文执行 literal search，并把源码命中确定性映射为可继续读取的 JSON ref | **CONFIRMED** |

A6 是范围边界：本 change 拥有 JSON definition 的追加和兼容性证据；registry 整体治理继续由既有 owner contract 承接。

### A5a Object source-order 结论

JSON 私有 decode 本来就要同时拒绝 decoded duplicate member、保留 number token 并建立 source regions，因此 object 使用唯一的 `Vec<(String, Node)>` member storage。Workspace 锁定的 `serde_json 1.0.150` 有界探针证明，自定义 `serde` visitor 会按 parser visitation order 把嵌套 object member 直接追加到该 `Vec`，在同一 decode path 拒绝重复 member，并按相同顺序取得保留原始 number token 的借用 `RawValue` 源码切片。

这份 primary tree 同时服务 traversal、ref resolution、structured read 和 source indexing；保留 source order 不需要第二份全量 tree、`preserve_order` feature、替代 parser 或 shared contract 修改。Object traversal 与 structured read 因此使用源码顺序，array 继续使用 index 顺序。

### A5c `readable-view` 自定义渲染（后续验证阶段）

Raw protocol `read.content` 已由 A5b 决定。当前 change 先让 probe、outline、read、find、info 和 full-read 经过现有 generic `readable-view`，并记录它对 JSON 暴露的格式假设。

格式专用自定义渲染需要同时确定信息密度、完整 opaque ref 的路径定位信号、标点、preview、分页显示和 renderer 选择 mechanics，因此由任务 6.4 建立的专门 change 承接。Renderer 保持 ref opaque，不合成 hierarchy、depth、parent 或 indentation。该阶段不是可选美化；完成它之后，JSON adapter 才形成覆盖完整行为的边界验证证据。

## Target Scope

### Delivered Behavior

- 导航常见 UTF-8 JSON 文件，并支持自动/显式选择及完整 operation surface。
- 用确定性树遍历和 adapter-owned、CLI-safe JSON Pointer refs 支持 `outline/find -> ref -> read`。
- `find` 直接搜索 BOM-stripped 原文，并按源码位置把命中映射为 JSON-owned ref。
- Structured read 使用 object member 源码顺序、两空格 pretty layout 和 workspace-pinned parser/serializer 的自然 string/lexeme 输出；JSON number token 必须原样保留。
- Core parameter catalog、closed operation input、protocol 和 raw output shape 保持当前契约。
- 在 package 中通过同一个 `docnav` binary 验证 Markdown 与 JSON。

### Contract Boundaries

- Delivery shape 是包含 static JSON factory 的单一 core executable。
- Public input 是既有 closed standard operation input；JSON safety limit 由 adapter-private 配置拥有。
- 支持范围是单个 UTF-8 JSON value；其它 JSON-like syntax 和 schema-aware semantics 由独立能力决定。
- Structured read 表达 JSON value，full-read 表达原文；当前 generic `readable-view` 证明既有输出链路，后续格式专用 presentation change 完成 JSON 自定义渲染，并只把完整 opaque ref 用作路径定位信号，不解析 ref 或合成 hierarchy/depth/parent/indentation。
- 初始实现使用完整内存 model 和既有 pagination；性能扩展由测量结果触发。
- Markdown、shared protocol 和 generic output renderer 继续遵循各自现有 owner contract。

## Change-local Design

本节拥有 change-local implementation mechanics；长期方向由 Decision Map 链接的记录拥有，observable target 由 capability delta 拥有。实施按 C1–C5 展开。

### C1. 复制现有静态 adapter 形状

- 新增 `crates/adapters/json`，导出唯一 registry-facing `json_adapter_definition()`。
- JSON crate 只依赖既有 adapter contracts、protocol、text-cost、`serde` 和 `serde_json`；core 只增加 workspace dependency、registry import 和一个 static factory entry。
- `AdapterDefinition`、`NavigationAdapterRegistry`、`StandardOperationInput`、`StandardInputBinding` 和 parameter catalog 是本 change 的输入契约。基础 JSON operation 若超出该契约，实施暂停并回到边界评审。
- Registry 修改追加 JSON，并按 registry owner 的当前 membership 与 ordering contract 保留其它 definitions。

### C2. 每次 operation 建立一个 adapter-private document model

- 每次 strategy 调用读取原始 bytes，记录包含可选 BOM 的文件 byte size，去除一个可选 UTF-8 BOM 后解码，并由 `serde_json::Deserializer` 消费完整输入。
- 同一次调用只 parse 一次。Document model 保存 BOM-stripped source、parsed tree、raw number token、node/member source region，以及 traversal、source find、ref resolution、serialization、node count 和 depth 所需的私有事实。
- Duplicate-member 检测和 depth check 在 decode 层完成；number 使用 `serde_json` raw-value 能力或等价私有 mechanics；workspace-pinned `serde_json` 是唯一 parser package。
- Array 始终保持 index 顺序；object 使用唯一 member `Vec` 保持源码顺序。
- Probe 成功后的 operation 会重新加载文档；重新加载失败使用 C5 的既有 error boundary。

### C3. 分离树遍历与原文命中序列

- Adapter-private tree index 保存 depth-first preorder node sequence、value kind 与 ref；outline entry set 和 root 行为以 capability delta 为准。
- `JsonRef` 私有 helper 按 A1 和 capability delta 完成 encode、parse、canonical validation 与 resolution；JSON adapter 是 grammar owner。
- 非空 object key 的正常 outline label 使用 decoded member name；空 key 的 ref 仍为 `json:#/`，正常 label 使用两个双引号字符 `""`。该 spelling 可读、满足 shared entry schema 的 `label.minLength: 1`，且 generic `readable-view` 无需 JSON-specific renderer；`.` 只保留给预算截断后没有可见正常 label 内容时的最小非空 fallback，不使用 `<empty key>`。
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
| Object source-order mechanics 在实际 source-region 实现中出现额外全量副本或广泛分支 | 使用同一 member `Vec` 承载 decode、遍历、序列化和 source index；不建立第二份顺序 tree | 实现证据偏离 A5a 结论时回到 JSON-private model 评审 |
| Pinned parser/serializer 升级改变 structured output | Owner tests 固定可观察结果，依赖升级时复核对应长期决策与 JSON contract | Workspace dependency 版本或 feature 变化 |
| 原文命中需要稳定映射到 JSON node | Adapter-private source region 明确定义 occurrence 归属；find tests 覆盖 key、scalar、结构文本、空白、跨节点和重复 ref | Parser mechanics 无法以当前 model 量级提供可靠 region 时回到 JSON-private model 设计 |
| JSON ref 表达当前结构路径 | 文档变化后同一路径可以指向新值或返回 `REF_NOT_FOUND` | 跨版本持久身份需求进入独立 contract 设计 |
| 第二个 adapter 暴露 closed contract 无法表达的职责 | 记录真实阻塞并回到 adapter boundary 评审 | 基础 JSON operation 无法沿 C1–C5 落地 |

## Delivery and Reversal

1. **Entry:** 建立 JSON owner 文档、fixtures 和 owner-level 证明目标。
2. **Delivery:** 按 adapter-private model、fixed strategies、static registry、generic readable output、CLI/package smoke 的顺序交付；精确任务以 `tasks.md` 为准。
3. **Audit:** 追加 Implementation Observations，按任务 6.4 建立格式专用渲染 change，并对活动决策执行 owner、代码、测试和 release evidence 对齐检查。
4. **Reversal:** 该能力是 additive；发布前可以移除 JSON registry entry 和 crate，现有 adapter 与 shared protocol 继续使用当前基线。

## Implementation Observations

本节记录第二个真实异构 adapter 接入后的实现事实和边界，不把本 change、尚未执行的最终验证或后续 presentation change 当作 Current 证明。

### 实际接入面与 shared surface

- Production 接入点保持为一条窄静态路径：workspace/core 增加 `docnav-json` 依赖，JSON crate 导出 `json_adapter_definition()`，core registry 在 Markdown definition 后追加该 factory。Automatic/declared selection、closed operation dispatch、full-read orchestration、protocol/error projection 和 generic output 随后都复用既有路径；package 仍只交付同一个 `docnav` binary。
- Shared contract 和 public catalog shape 没有因 JSON 扩张：`Adapter`/`AdapterDefinition`、`StandardOperationInput`、`StandardInputBinding`、protocol result 与 readable renderer 的 production 定义均未修改。Catalog 的 known-adapter membership 新增 `docnav-json`，但 fields、entries、bindings 以及 CLI/env/config/protocol accepted input inventory 保持注册前形状；JSON 的 depth 等限制没有进入 caller-configurable input。
- JSON-private model 承接 duplicate decoded member rejection、raw number token、node/member source region、source-order traversal 和 JSON ref grammar。Core 只传递 normalized path、closed input 和 opaque ref，没有解析 JSON tree、region 或 ref。

### 重复、绕行与抽象证据

- Markdown 与 JSON 都在 adapter 内保留少量文件读取/BOM/UTF-8、I/O diagnostic mapping、text cost、entry construction、line/excerpt 和 pagination mechanics，但格式语义及数据形状不同。JSON 的 raw token、region ownership、canonical ref 和 duplicate-member mechanics 没有对应的跨 adapter 共同责任。
- 没有为了接入 JSON 绕过 production owner：selection/dispatch、protocol、readable output 和 release binary 仍走原 owner。确定性 TOCTOU 证明没有向 probe/reload production 路径加入测试 hook，而是由 test-only supervisor 通过 smoke harness 启动真实 binary；harness 额外记录实际 executable，使该辅助进程仍可审计。
- 第二个 adapter 暴露的重复尚不足以证明 shared helper 或新 extension point 的稳定共同契约。当前实现因此保留局部重复和 JSON-private mechanics，不从两个样本抽取 production abstraction。

### 实施中出现的非预期修改点

- 空 object key 使 decoded-name label 与 shared `Entry.label` 非空约束发生冲突。占位语义会把 presentation 文案写入 raw facts，最终使用 JSON literal `""` 作为正常 label；`json:#/` ref 不变，`.` 只用于预算截断后的最小非空 fallback。
- Cargo feature unification 使 `serde_json/arbitrary_precision` 会影响 workspace 中其它 `serde_json` consumers。最终实现没有启用该 feature，而由 `docnav-json` 使用 `raw_value` 取得原始 number token，并把相关处理留在私有 decode/serialization path。
- 原文 find 的 region-to-ref 映射最初显露 sibling scan 成本；实际 source regions 已按源码有序且 sibling 不重叠，因此实现用 `partition_point` 定位唯一候选 sibling，再向最深覆盖节点下降。该优化没有要求 shared source-index contract。
- Canonical package smoke 原先按 target 推导 binary 名称，这不足以证明实际 manifest entry。最终 smoke 从已验证 manifest 解析唯一 core executable，要求它直接位于 package 目录；package validation 改用 `lstat` 拒绝 symlink，避免 hash/smoke 跟随 package 外目标。
- Probe 后替换文件的确定性证明需要 Linux x86_64 `ptrace` barrier。常规 JSON/Markdown roundtrip 保持在 core 与 package smoke；TOCTOU case 只进入 Linux x86_64 core direct profile，`release-package` profile 不运行仓库内 supervisor。这个分层是验证环境边界，不是 JSON production capability 或跨平台行为分叉。

### Generic `readable-view` 暴露的格式假设

| Operation | 实际暴露的 generic 假设 |
| --- | --- |
| `outline` | Renderer 只专门识别 `heading`、`match` 和 `document` kind；JSON 的 object/array/scalar entries 落入 generic label display。完整 opaque ref 可以继续读取，但 generic display 未把它与 value kind、JSON punctuation 组合成格式专用路径定位信号；raw entry 没有 depth/parent facts，后续 renderer 也不能解析 ref 来合成 hierarchy 或 indentation。空 key 因而必须在 raw label 中已有可见 spelling。 |
| `read` | `content` 被当作 opaque text block，header 只保留 ref、content type、common cost 和 page。Structured JSON 与原始 number token 可以无损通过，完整 opaque ref 仍只是原样保留的路径定位信号，没有 syntax-aware preview 或专用 continuation presentation。 |
| `find` | `match` display 固定为行号加 bounded line label；JSON ref 仍作为独立字段存在，但 source-region ownership、path context 和 matched value kind 不参与 display。 |
| `info` | Display 只组合 format、content type、通用 size，并只认识既有 `heading_count` 特例；JSON 的 `root_kind`、`node_count` 和 `max_depth` metadata 不进入 readable text，format label 也没有 JSON-specific spelling。 |

这些结果证明 generic `readable-view` 可以承载 JSON raw facts 和继续导航所需 ref，但也确认信息密度、完整 opaque ref 的路径定位信号、标点、preview、分页显示和 renderer selection 必须由任务 6.4 的格式专用 change 完整决定；该 change 不解析 ref 或合成 hierarchy/depth/parent/indentation。本次接入本身没有提供其它 shared extraction change 的充分证据。

### Minimal-implementation 审计结论

任务 6.2 以当前实现 diff、真实消费者、owner 边界和完整 quality scan 为范围。审计发现 `JsonAdapter` 的 operation 方法曾只把相同输入直接转发给同文件私有函数；该层没有独立契约、调用方或失败 owner。实现已把 behavior 收回 `Adapter` trait implementation，移除 direct-forwarding 层并净删 37 行，不改变 adapter public surface。重构前 `adapter.rs` 的 file-size observation 已从当前 full scan 消失，因此没有建立 accepted-warning 记录。

其余 10 条 current observation 是审计信号，不是新的拆分 owner；当前接受结论如下。Acceptance 以 rule、tool、path、code area、metric、value 和 message 片段精确绑定，后续数值或 warning shape 变化时必须重新审计，不能由现有记录泛化放行。表中同时记录本轮性能修复已消除的 `compact_label` observation，避免把历史接受理由误当成当前事实。

| Observation | 审计结论 |
| --- | --- |
| `adapter/tests.rs` file code lines `937` | 该文件是 JSON adapter contract suite；probe、全部 operation、diagnostic projection 与 full-read hooks 共用 `TempDocument` 和 operation-result helpers。拆分只会复制 test fixture plumbing，不会形成新的行为 owner。 |
| `document.rs` file code lines `483`; `load` function code density `54` | 文件是 adapter-private document boundary；同一 tree representation、source cursor 和 serde visitor 共同拥有 duplicate-key/depth enforcement、raw number 与 source regions。`load` 是从 BOM/UTF-8 到 parser-state failure、trailing input、root region/metrics 的单个有序 transaction，拆分会暴露内部协调状态。 |
| `smoke-harness.ts` file code lines `310` | 该文件统一拥有 smoke command preparation、process execution、assertion/audit recording 与 executable identity；test-helper 路径复用同一记录链，拆出文件会分散 stateful harness owner。 |
| `find.rs` file code lines `341`；原 `compact_label` function code density `79` / cyclomatic complexity `20` 已消失 | Production `find` 直接消费 source-order occurrence iterator，分页只保留请求页和一个 continuation lookahead，不再先物化全部 refs/labels。每个 label 只扫描 occurrence 以及其每侧最多 97 个 raw Unicode scalar，并以四个最多 96 chars 的私有 buffer 保留 edge/center window；label 输入因此限制为 query span 加固定预算派生的 context，不会为寻找非空白字符穿越任意长度的空白，也不再为完整 source line 建立逐字符 text、region 和 boundary vectors。为隔离 find 工作集，5 MB 单行、单命中的同类受控 debug CLI 复现显式使用 `--auto-read disabled`；当前口径下峰值 RSS 约 `24.5 MiB`，与 missing-query 基线约 `25 MiB` 相当。旧的逐字符全行索引结构已删除，并由 `find_label_working_set_is_bounded_by_the_label_budget` 和 `find_label_context_scan_is_bounded_by_raw_unicode_scalars` 分别对固定 buffered working-set 与 raw context scan 上限提供结构性回归证据；Unicode、whitespace collapse、match marker、location、ref、page/limit 与 error shape 保持不变。默认 `auto-read: unique-ref` 会在 find 后进入独立 read 编排。`find.rs` 的 occurrence/ref、line 与 bounded excerpt state 共属 JSON-private find pipeline；拆分只会把同一 source-region invariant 跨文件传递，因此接受当前 file-size observation，不增加公共或跨 adapter 抽象。 |
| `paging.rs::{paginate_entry_slice,entries_page,fit_entry}` parameter count `7/7/6` | 三个函数实际 top-level parameter count 是 `5/5/4`；Lizard 把 function-pointer type 中的两个嵌套逗号计为额外参数。Callbacks 只在私有 paging mechanics 中适配 outline/find entry shape；为扫描器计数引入 parameter object 或 public entry abstraction 会增加维护面。 |
| `real-json.ts` file code lines `432`; `runProtocolFailure` parameter count `6` | 文件保存围绕同一 `SmokeProject` 和真实 executable 的 registry、selection、outline/read、find、readable-view 与 failure 证据。`runProtocolFailure` 是必要的薄 audit helper，六个参数分别是 label、arguments、project、operation、protocol code 与 exit code；一次性 parameter object 不会形成领域边界。 |

上述接受项没有改变 threshold，也没有证明 JSON 与 Markdown 应共享 production helper。两者目前只重复文件读取、成本、entry 和 pagination 等表面 mechanics；格式 tree、ref、source region、error 和 presentation 语义仍由不同 owner 承担。审计没有发现能同时减少总维护面且保持这些语义的 shared extraction，因而以当前局部实现结束任务 6.2。

### 其它后续结构 change 判定

任务 6.5 的 production diff 只在 workspace/core 增加 JSON 依赖与 static factory registration，`crates/shared/**` 没有新增实现分支或 contract 修改；selection、dispatch、catalog、protocol 和 release 继续由既有 owner 承接。跨 adapter 相似处仍是 format-owned mechanics：Markdown 按 `Entry` 的 label/summary/excerpt 计分页预算，JSON 在 protocol projection 前分别分页 `JsonEntry`/`FindEntry` 并随内容保留完整 cost；共享抽取需要新增 callback/trait owner，却没有第三个消费者、重复修改链或未解除 blocker。空 key、Cargo feature、find 映射、package executable 与 TOCTOU 证据摩擦均已在各自 owner 内收敛。因此除任务 6.4 已明确的 presentation change 外，不建立其它后续结构 change，以现有 static boundary 结束本轮 raw adapter 交付。

### Decision alignment 事实核对

任务 6.6 以 JSON、adapter、ref、output、testing 和 release owner 为稳定规则，以当前代码、Case 映射、JSON/core tests、真实 CLI smoke 和 canonical package smoke 为实现证据。完整核对后，第二个异构真实 adapter、JSON 选择、canonical ref、number token、decoded member 唯一性、adapter-private depth、format-costed source order、structured/source read 分层、JSON structured output 和原文 find 已全部成为 Current 基线；对应活动记录已分别执行 `mark-aligned`。

以下两条记录保持 `active + unaligned`，没有因 raw adapter 或 generic renderer 已可用而提前建立基线：

- `adapter-boundary-evidence/validate-boundaries-with-complete-adapter-behavior.md`：probe、固定 operations、full-read 和 generic `readable-view` 已交付，但格式专用 readable presentation 及其完成后的整体验证观察尚未交付。
- `readable-presentation/keep-custom-rendering-in-readable-view.md`：shared output 已保持 raw/readable 分层，JSON raw facts 也已稳定；仍缺经批准的 JSON presentation contract、format-aware renderer/selection mechanics 以及对应 tests、CLI 和 package evidence。

上述差距由任务 6.4 建立的 `add-json-readable-renderer` change 承接；在该 presentation change 完成前，两条记录继续作为已生效但未对齐的长期方向。
