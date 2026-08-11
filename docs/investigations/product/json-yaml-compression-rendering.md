# JSON/YAML 压缩渲染实现调查

## 调查信息

- 核心问题: 现有 token-reduction 工具在源码中怎样压缩、重排或摘要 JSON 与 YAML，它们如何处理重复结构、输出语法、信息丢失、回退和恢复，以及其中哪些机制适合作为 Docnav 未来格式专用 readable renderer 的设计输入？
- 状态: 已结束
- 最新报告时间: 2026-08-06T09:33:47+00:00

## 调查报告

### JSON/YAML 压缩算法与 Docnav renderer 可迁移性核查

- 形成时间: 2026-08-06T09:33:47+00:00

#### 形成时背景

Docnav 正在规划 JSON 专用 readable renderer，并可能在后续扩展 YAML 等文档格式。实现前需要确认外部工具究竟对 JSON/YAML 做了什么变换：哪些只改变表示，哪些会丢弃数据，哪些依赖全文、查询或外部恢复存储。

本报告是面向后续 proposal、design 和实现工作的源码调查依据，用于提供算法选项、输入要求与失败边界。报告形成于 Docnav 仓库提交 `b57252a54f7ac7de72b499c1377f73f8ac623e0a`；外部源码固定到下文列出的提交，观察日期为 2026-08-06。

Docnav 当前有一个必须保留的架构边界：专用 readable renderer 消费已经形成的同一个 `ProtocolResponse`，不能重新调用 adapter、重读文档、依赖 adapter 私有状态，或改变 raw protocol、ref、顺序、分页和 schema。因此，“某个外部项目能压缩完整文件”并不自动等于“当前 renderer 能从已有响应事实完成同样压缩”。

#### 调查目的

1. 按源码拆解 JSON 与 YAML 压缩的变换阶段、阈值、输出语法、回退和恢复机制。
2. 区分源文本可逆、解析后数据模型可逆、覆盖有损，以及依赖旁路存储的可恢复压缩。
3. 判断各算法需要哪些输入事实，并据此划分当前 renderer 可用机制、需要 contract 决策的机制，以及需要未来系统能力的机制。
4. 给出 JSON renderer 的候选实现顺序、YAML 设计前置问题和验证样本；不在本报告中批准 public contract 或实施代码。

#### 调查范围与依据

**Docnav 当前基线**

- [输出主规范](../../output.md)：generic readable view 当前以 pretty JSON 和定长 framing 展示；格式专用 renderer 仍需决定字段密度、标点、转义、ref、preview、分页和选择规则。renderer 必须把 ref 当作 opaque value，不能从 ref 猜测层级、父子关系或缩进。
- [JSON adapter 主规范](../../adapters/json.md)：JSON/JSONC 的解析、ref、顺序、分页与 read 内容属于 adapter 行为；readable renderer 不得反向改变这些语义。
- [JSON readable presentation 活动决策](../../decisions/product-direction/advance-json-readable-presentation-after-contract-approval.md)：专用展示必须复用同一不可变响应，并保持 protocol JSON 为权威机器接口。
- [`add-json-readable-renderer` OpenSpec change](../../../archive/legacy/openspec/changes/add-json-readable-renderer/design.md)：形成时仍有六组 contract 问题未批准，包括适用 operation/branch、稳定字段、opaque ref、preview 来源、page/continuation，以及 renderer 选择与 fallback。本报告只提供这些问题的调查输入。

**外部一手源码**

| 项目与固定版本 | 核查重点 |
| --- | --- |
| [RTK `3044911`](https://github.com/rtk-ai/rtk/tree/3044911b50bc59777d0dedbcd17eb513305c8de5)，`rtk 0.42.4` | JSON value/schema 投影和收益守卫 |
| [ContextZip `2c3af90`](https://github.com/jee599/contextzip/tree/2c3af901f9fe2da9535911f954e9d799aa45c731) | RTK 派生的 schema-only JSON 路径 |
| [LeanCTX `c560f8b`](https://github.com/yvgude/lean-ctx/tree/c560f8bba31f0b4cf7b1639d522a6d017bc1b3ec) | JSON 词法压缩、结构 factoring 和 YAML 归一化 |
| [Headroom `7940c05`](https://github.com/headroomlabs-ai/headroom/tree/7940c05ebf4486c6b9d00984067ae33cedf4dddb) | SmartCrusher、YAML 文本压缩和 CCR |
| [TOON `a9e6d97`](https://github.com/toon-format/toon/tree/a9e6d97eca931379824f3b6a1ba8fbfbda7d3c53) | JSON 数据模型可逆的紧凑 notation |
| [Squeez `b0aa8b0`](https://github.com/claudioemmanuel/squeez/tree/b0aa8b046e411f6f9f5ac5d3686f7a9902710b2f) | TOON 候选及其有损 fallback |
| [jDocMunch `a6a3e68`](https://github.com/jgravelle/jdocmunch-mcp/tree/a6a3e6839f422cf2ebfdfb99402612564bc20bd7) | JSON-to-Markdown 与 OpenAPI/YAML 投影 |
| [Vajra `2b67617`](https://github.com/copyleftdev/vajra/tree/2b67617d3b16d92df5d0f1b72fc7b8a3142b6986) | JSON/YAML 归一化与预算化 essence |

核查方法是固定提交、追踪入口到实际 formatter/selector/fallback，并交叉查看同仓库测试；项目 README 只用于定位，不作为实现结论的唯一依据。没有对外部项目做许可证兼容性审计，也没有把其宣传百分比当作本报告证据。算法效果仍需在 Docnav 自有响应 shape 和 tokenizer 上重新测量。

#### 调查结果与边界

##### 使用方式与权威性

本报告同时包含外部实现事实、Docnav 当前约束和调查建议。后续工作应按下表解释其权威性：

| 内容类型 | 权威性与用途 |
| --- | --- |
| Docnav 当前约束 | 以“调查范围与依据”列出的 owner 规范、活动决策和 OpenSpec 状态为准；它们决定 renderer 当前不能改变什么 |
| 外部实现事实 | 只描述固定提交中的源码行为，用于比较机制和失败边界；不构成 Docnav contract |
| 调查建议 | 用于形成 proposal、design、原型和验证计划；在 owner 规范或 OpenSpec 批准前不是当前行为 |
| 未验证效果 | tokenizer 收益、模型理解度和端到端任务效果仍需 Docnav 自有 benchmark 证明 |

若只需要形成 Docnav 方案，先读“结论摘要”“信息边界”“对 Docnav 自定义 renderer 的直接含义”和“对 OpenSpec 六组问题的调查输入”；需要核查理由时，再按项目读取 JSON/YAML 实现细节。

##### 结论摘要

| 问题 | 源码结论 | 对 Docnav 的含义 |
| --- | --- | --- |
| JSON 的主要压缩来源 | 高收益实现集中在统一对象数组的重复字段、dominant defaults 和稳定嵌套路径；删除空白只是较低风险的基础路径 | 第一版优先评估数据模型可逆的 table/default factoring |
| 通用 YAML 是否已有实现 | LeanCTX 做 YAML → JSON value 的语义归一化；Headroom 做保守文本折叠；Vajra 做预算化信息选择 | “YAML 压缩”必须先声明是源文档视图、数据视图还是摘要视图 |
| RTK/ContextZip 的可借鉴部分 | RTK/ContextZip 主要做限深、首项取样和 schema 投影；RTK 不提供通用 YAML 结构压缩 | 借鉴显式省略和 `never_worse`，不把 first-item/schema-only 作为默认 renderer |
| 更激进方案的能力来源 | Headroom/Vajra 的统计选择依赖全文、query、budget 或 CCR | 当前 output-only renderer 不能在缺少这些事实和 owner 时复制该路径 |
| 第一版推荐边界 | TOON、LeanCTX、Headroom 和 Squeez 都实现了“可逆候选 → 收益守卫 → 原展示 fallback”的路径 | 先做响应内、确定性、数据模型可逆的 shape compaction；暂缓额外信息选择 |

##### 判断压缩结果的信息边界

同一个项目可能同时使用多种压缩。为避免把“能恢复”“无损”和“没丢关键信息”混为一谈，本报告统一使用以下术语；外部源码中的 `lossless` 只在描述其原始命名时保留：

| 术语 | 可恢复的内容 | 典型例子 |
| --- | --- | --- |
| 源文本可逆 | 可逐字节恢复空白、注释、转义拼写、键顺序和 YAML 表达风格 | Headroom 的重复块折叠在 inverse self-check 通过时可恢复原文本 |
| 数据模型可逆 | 可恢复解析后的键、值、类型和数组顺序；源文本表示可能改变 | TOON；LeanCTX 默认 JSON/YAML crush；Headroom `csv-schema` compaction |
| 覆盖有损 | 当前输出省略行、字段、字符串尾部或数组元素，无法仅凭输出重建 | RTK value/schema view；jDocMunch item cap；Squeez keys/ids sketch |
| 旁路可恢复 | 当前输出覆盖有损，但可用 hash/handle 从独立存储取回原文 | Headroom CCR；LeanCTX 有损列删除与周边恢复层 |

对 JSON，数据模型可逆通常是可接受的 readable-view 候选，因为 raw protocol 仍是机器权威。对 YAML，数据模型可逆不等于源文本可逆：注释、anchor/alias、merge key、tag、多文档边界、block scalar style、键的原始拼写和排版都可能在归一化时消失。未来 YAML 设计必须先明确产品要展示“YAML 源文档”还是“解析后的配置数据”。

##### 实现可以分为五层

| 层级 | 实现机制 | 代表实现 | 需要的输入 | 主要风险 |
| --- | --- | --- | --- | --- |
| L0 词法压缩 | 删除 JSON 字符串外空白；折叠完全重复的 YAML 文本 | LeanCTX structured compact；Headroom lossless compaction | 原始文本 | 收益通常有限；输出可能不再适合人读 |
| L1 表示替换 | 用缩进、count、header、CSV row 代替 JSON 括号、逗号和重复键 | TOON；Squeez TOON | 完整且可解析的当前值 | 新语法的 escaping、歧义和模型熟悉度 |
| L2 结构 factoring | 把共用列、dominant defaults、嵌套稳定路径提升一次声明 | LeanCTX `json_crush`；Headroom SmartCrusher lossless-first | 至少一个完整数组/对象子树 | 稀疏或异构数据可能不省；必须可逆验证 |
| L3 有界投影 | 限深、限键、截字符串、数组取样、schema-only | RTK；ContextZip；jDocMunch | 当前值或文档 AST | 尾部异常、稀有字段和具体值被静默删除 |
| L4 统计/相关性选择 | 保留首尾、异常、error、top-N、query-relevant，其他卸载 | Headroom lossy planner；Vajra essence | 全量候选、统计、query/budget，通常还要恢复存储 | 输出层越权、选择错误、不可复现或无法继续读取 |

这五层不是互斥的。对 Docnav，推荐顺序是先评估 L0/L1/L2 的源文本可逆或数据模型可逆候选；只有 contract 明确允许丢失，并且省略范围与恢复路径可观察时，才考虑 L3/L4。

##### 项目能力矩阵

| 实现 | JSON 核心 | YAML 核心 | 默认信息边界 | fallback/恢复 | 对当前 Docnav renderer 的可迁移性 |
| --- | --- | --- | --- | --- | --- |
| RTK | 限深、限长、数组首项、schema sketch | JSON 命令拒绝 | 覆盖有损 | 不更短则原文；无精确元素恢复 | 仅适合借鉴显式省略和 `never_worse` |
| ContextZip | first-item schema-only | 无 | 覆盖有损，且比 RTK value view 更早丢值 | 该路径无 `never_worse` | 不作为默认展示候选 |
| LeanCTX | 空白删除；dominant defaults factoring | YAML → JSON value → reuse crush | 默认数据模型可逆；可选删高 entropy 列 | 低收益/失败回原文；有损需 CCR | factoring 可借鉴，但解析 read content 需 contract 允许 |
| Headroom | table/flatten/bucket；再做统计 row selection | 重复文本折叠；CCR comment elision | 数据模型可逆候选优先；第二阶段旁路可恢复 | 最小收益门槛、`lossless_only`、CCR | 可借鉴 compaction；统计与 CCR 暂不可移植 |
| TOON | shape-adaptive notation | 无 | JSON 数据模型可逆 | 不适用 shape 使用通用 list | 最直接的紧凑展示参考，仍需自有 grammar 决策 |
| Squeez | 统一数组 TOON；失败后 keys/ids | 行过滤与 head truncation | 第一阶段数据模型可逆，第二阶段覆盖有损 | TOON 不更短则拒绝；sketch 无恢复 | 借鉴 candidate/fallback 顺序，不借鉴有损 fallback |
| jDocMunch | JSON → synthetic Markdown | 仅 OpenAPI domain projection | 深度、item、leaf 有 cap，属于覆盖有损 | 依赖其文档索引工作流 | 适合 adapter/navigation 参考，不适合当前 output-only renderer |
| Vajra | path stats + budget essence | 归一化后同一 essence | 覆盖有损 | 预算与 drill path，不等于原文恢复 | 属于未来分析层，不是第一版 renderer |

##### JSON：各项目实际怎样做

###### RTK：廉价、确定性的有损投影

RTK 的 [`json_cmd.rs`](https://github.com/rtk-ai/rtk/blob/3044911b50bc59777d0dedbcd17eb513305c8de5/src/cmds/system/json_cmd.rs) 先用 `serde_json` 把全文解析为 `Value`，再进入两个展示方向：

- 默认 value view 递归输出自定义 pseudo-JSON。长字符串在约 80 字符处截断；较长数组只展示首项并标记还剩多少项；对象和深度也有硬上限；对象键会排序。
- `--keys-only` 把 scalar 映射为 `null`、`bool`、`int`、`float`、`string` 等类型描述，并对 URL、日期和长字符串做启发式标记；数组 schema 只从第一项推断，同时附总数。
- 结果不是保证可反向解析的 JSON：键和字符串走展示逻辑，而不是完整 JSON serializer。它应被视为阅读投影，而不是紧凑序列化。
- `never_worse` 会在转换文本更长时返回原文。这个守卫只证明字节长度没有变差，不证明信息保留或真实 tokenizer 成本更低。

实现复杂度低，特别适合 shell 输出“先看个大概”；但第一项不能代表尾部，排序也会丢失源顺序线索。对 Docnav 来说，`+N more` 这种显式省略信号值得保留，首项 schema sampling 本身不应成为默认 renderer。

###### ContextZip：比当前 RTK 更激进，但不是更完整

ContextZip 的 [`json_cmd.rs`](https://github.com/jee599/contextzip/blob/2c3af901f9fe2da9535911f954e9d799aa45c731/src/json_cmd.rs) 保留了明显的 RTK 结构和 heuristics，却只留下近似 keys/schema 的路径；当前 RTK 已有的 value-preserving compact view 和 `never_worse` 路径在这里不存在。数组仍由第一项决定 schema，值本身基本不展示。

这说明“更激进”有时只是更早丢值，并不代表更强的结构算法。若目标是未来自定义渲染，ContextZip 的主要价值是反例：把 schema sketch 当作唯一输出会让用户无法判断具体值和稀有分支。

###### LeanCTX：先保留词法，再做数据模型可逆的 structural factoring

LeanCTX 提供两个不同强度的 JSON 路径：

1. [`structured_compact`](https://github.com/yvgude/lean-ctx/blob/c560f8bba31f0b4cf7b1639d522a6d017bc1b3ec/rust/src/core/structured_compact.rs) 用字符串状态机删除 JSON string 之外的空白，输入先校验，大小上限为 4 MiB。它不经 `Value` 重写，因此保持键顺序、number 原始拼写和 string bytes；只有结果更小时才采用。JSONL 则逐行校验和压缩。
2. [`json_crush`](https://github.com/yvgude/lean-ctx/blob/c560f8bba31f0b4cf7b1639d522a6d017bc1b3ec/rust/src/core/json_crush.rs) 对对象数组做结构 factoring。候选键必须出现在每个 item；每个键按 canonical serialized value 统计 dominant value，覆盖至少一半记录时提升到 `_defaults`，各 item 只保留偏离默认的值。输出带 `_lc_crush: "arr"`、`_defaults` 和 `_items` marker，默认模式可以重建解析后的 JSON 值。
3. 该路径有明确收益门槛：压缩结果需要达到约一半输入大小才采用；marker guard 防止重复 crush 或与用户数据碰撞。
4. 可选有损模式会按 distinct/item ratio 删除高 entropy 列，并在 `_dropped` 记录字段名；精确恢复依赖周边 CCR，而不是 marker 自身。

与 RTK 相比，LeanCTX 的核心收益不是少看几项，而是把多数记录重复的事实只写一次。它尤其适合“很多记录共享 status/region/type，但少数行有例外”的数组；例外仍留在对应 row 中，尾部不会因为位置而天然丢失。

###### Headroom SmartCrusher：shape compaction 与 row selection 分层

Headroom 当前 Rust [SmartCrusher](https://github.com/headroomlabs-ai/headroom/tree/7940c05ebf4486c6b9d00984067ae33cedf4dddb/crates/headroom-core/src/transforms/smart_crusher) 是本轮最完整的 JSON 数组实现。默认配置先尝试源码所称的 `lossless compaction`，再考虑有损选择：

- 对对象数组建立 union fields，按出现频率和稳定次序排列列；统一数组变成 table，缺失 cell 显式表示。
- 稳定的嵌套对象可以摊平成 dotted column；字符串形式的 JSON 会尝试解析；异构数组可寻找 discriminator 并分桶。
- formatter 支持 `csv-schema`、JSON 和 Markdown key/value 等形式。推荐的 `csv-schema` 会先声明行数和 columns/schema，再发 CSV rows。
- lossless candidate 只有在节省达到配置门槛时采用，默认最小收益约 15%；否则继续使用原表示。

只有 lossless compaction 不够时，planner 才分析字段类型、unique ratio、numeric 分布、change point、message cluster 和 query relevance，并按数据形态选择策略：

- time series 保留首尾、change point 邻域、error、outlier 和 query match；
- logs 按 message-like field 聚类，每簇保留少量代表，同时保留 error/query；
- search results 按 score 取 top-N，并预留 error/query/high relevance；
- generic smart sample 保留 anchor、结构异常、数值异常和 query-relevant rows。

被删除的完整数组用 canonical JSON 哈希并存入 CCR，输出 `<<ccr:...>>` marker；`lossless_only` 可以完全禁止这条路径。这个设计最重要的启示是：**格式压紧与信息选择应是两个可独立关闭、独立验证的阶段**。但异常分析、query relevance 和 CCR 都超出 Docnav 当前 renderer 的既有事实与责任，不应在第一版复制。

###### TOON：通用、数据模型可逆的 shape-adaptive notation

TOON 的 [`encoders.ts`](https://github.com/toon-format/toon/blob/a9e6d97eca931379824f3b6a1ba8fbfbda7d3c53/packages/toon/src/encode/encoders.ts) 与 [`tabular.ts`](https://github.com/toon-format/toon/blob/a9e6d97eca931379824f3b6a1ba8fbfbda7d3c53/packages/toon/src/encode/tabular.ts) 不对记录评分，也不删字段；它们通过改变表示减少重复语法：

- 对象用 indentation 和 `key: value`，安全时省略 JSON braces、commas 和 quotes。
- primitive array 把 count 写在键后并内联元素。
- key set 一致的对象数组只输出一次 header，再按 row 输出值；嵌套且一致的对象可以递归 flatten 到 header group。
- 不统一的数组回退为 list 形式，而不是为了表格化删除字段。
- encoder 根据 delimiter、保留字、空白和控制字符决定 quote/escape；decoder 校验声明 count、row width 和 duplicate 等约束。

TOON 给 Docnav 的直接输入是“按 shape 选择表示”，不是“采用 TOON 品牌语法”。字段 header 一次声明、显式 `[N]`、异构时安全回退都适合 renderer；完整采用新 grammar 则要另外证明 escaping、模型理解、可读性、版本稳定性和维护成本。

###### Squeez：数据模型可逆的 TOON 候选失败后进入有损 sketch

Squeez 的 [`toon.rs`](https://github.com/claudioemmanuel/squeez/blob/b0aa8b046e411f6f9f5ac5d3686f7a9902710b2f/src/strategies/toon.rs) 用零依赖手写 parser 识别 top-level object array，要求各 row 的 key 数量与次序一致，生成 `items[N]{...}:` 加 CSV-like rows；结果不比输入短时返回 `None`。当前代码和测试还会把 nested object/array 保留为 minified、quoted cell，虽然模块顶部旧注释仍声称 nesting 会被拒绝，这是一处源码注释与实际行为的偏差。

当大型 JSON 不是规则数组、TOON 拒绝时，[cloud handler](https://github.com/claudioemmanuel/squeez/blob/b0aa8b046e411f6f9f5ac5d3686f7a9902710b2f/src/commands/cloud.rs) 会退到有损 sketch：非验证式 scanner 取 encounter order 中最多 24 个 distinct keys，再由 factsheet 提取 ids。它明确只保留 shape 与精确 id，不保留完整结构。[`jq/yq` 通用路径](https://github.com/claudioemmanuel/squeez/blob/b0aa8b046e411f6f9f5ac5d3686f7a9902710b2f/src/commands/data_tool.rs)则只是删空行、空对象/数组和 `null` 行后做 head truncation，不是 YAML 结构压缩。

这套顺序说明“先尝试数据模型可逆的专用格式，再 fallback”在小实现中也可行；但 Docnav 的 fallback 应回到 generic readable view，而不是自动进入有损 keys sketch。

###### jDocMunch：先转成 Markdown 导航文档，而非紧凑序列化

jDocMunch 的 [`json_parser.py`](https://github.com/jgravelle/jdocmunch-mcp/blob/a6a3e6839f422cf2ebfdfb99402612564bc20bd7/src/jdocmunch_mcp/parser/json_parser.py) 解析 JSON/JSONC 后构造 synthetic Markdown：top-level key 和 nested object 变为受深度限制的 heading，scalar 变成 Markdown 内容，数组最多处理前 50 个 item；对象 item 的标题优先取 `name/title/id/key/label/type` 等字段，长 leaf 也会截断。

这种做法把 JSON 复用到成熟的 Markdown indexing 流程，适合“先导航再读取”，但它同时改变结构表示并设有 item/leaf cap。对 Docnav 的启示是复用导航语义，而不是让 readable renderer 再造一份 Markdown AST；当前 Docnav 已有 adapter 结果和稳定 ref，不需要通过中间 Markdown 重建地址。

###### Vajra：预算化 semantic essence，不是 faithful renderer

Vajra 先在 [`formats.rs`](https://github.com/copyleftdev/vajra/blob/2b67617d3b16d92df5d0f1b72fc7b8a3142b6986/vajra-core/src/formats.rs) 把 JSON 与 YAML 归一化为共同 JSON value，再由 [`builder.rs`](https://github.com/copyleftdev/vajra/blob/2b67617d3b16d92df5d0f1b72fc7b8a3142b6986/vajra-essence/src/builder.rs) 建立 path trie 和统计；数组 index 会 wildcard 化以聚合同类结构。候选 observation 来自结构 motif、异常、notable value、fingerprint 等，再按 profile score 和 score/token ratio 在预算内选择，最后由 [`compact_ai.rs`](https://github.com/copyleftdev/vajra/blob/2b67617d3b16d92df5d0f1b72fc7b8a3142b6986/vajra-essence/src/compact_ai.rs) 输出使用短键的 compact-AI essence。

它展示了未来“任务相关摘要层”可以怎样建立在结构分析之上，但其输出故意只保留高价值 observation。它需要全局统计和 budget selector，也没有保持 Docnav 当前结果项逐一可追踪的要求，因此不适合作为第一版 JSON/YAML readable renderer。

##### YAML：真正实现了什么

###### LeanCTX：YAML 语义归一化

LeanCTX 的 [`yaml_crush`](https://github.com/yvgude/lean-ctx/blob/c560f8bba31f0b4cf7b1639d522a6d017bc1b3ec/rust/src/core/yaml_crush.rs) 用 YAML parser 得到 `serde_json::Value`，对非 string key、custom tag 等不能进入共同模型的输入失败并回退；structured root 会先序列化为 compact JSON，再复用 `json_crush`，外层用 `_lc_yaml_crush` marker 标识。输出需要相对原 YAML 至少约 25% 的字节收益才采用；scalar root、小输入和低收益输入保持原样。

这个实现是真正的通用 YAML 数据压缩，但“lossless”只针对解析后的共同数据模型。YAML 注释、排版、scalar style、anchor/alias 的表达方式和其他 source-level semantics 不能由 JSON value 重建。它适合配置数据视图，不等于 YAML 文档源视图。

###### Headroom：YAML 文本压缩

Headroom 的 [`ConfigCompressor`](https://github.com/headroomlabs-ai/headroom/blob/7940c05ebf4486c6b9d00984067ae33cedf4dddb/headroom/transforms/config_compressor.py) 刻意不引入 YAML semantic parser，而是分层尝试：

1. [`lossless_compaction.py`](https://github.com/headroomlabs-ai/headroom/blob/7940c05ebf4486c6b9d00984067ae33cedf4dddb/headroom/transforms/lossless_compaction.py) 把连续相同行折为一行加重复次数；完全相同的 3–64 行 block 可写成“重复此前某段”的 back-reference。变换后立即执行 inverse self-check，只有逐字恢复成功且输出更短时使用。
2. 更激进的候选删除 whole-line `#` comments 和 blank lines，但先把原文存入 CCR，并在尾部给出省略数与 recovery hash。
3. 若检测到 YAML block scalar header（`|` 或 `>`），禁用 comment elision，因为 scalar 内以 `#` 开头的行可能是数据。
4. TOML array-of-tables 可以经 stdlib parser 转 JSON 后进入 SmartCrusher；YAML 被明确排除在这条 semantic path 之外。

它牺牲的 token 通常比 LeanCTX 少，却清楚区分了源文本可逆压缩和依赖 CCR 的省略。对未来 YAML readable renderer，这种保守边界比“统一转 JSON 后仍称源文档无损”更可信。

###### 其他项目：YAML 专用能力有限或目标不同

RTK/ContextZip 的 JSON 命令不接 YAML；Squeez 的 `yq` 路径是通用行过滤和 head truncation；jDocMunch 的 [`openapi_parser.py`](https://github.com/jgravelle/jdocmunch-mcp/blob/a6a3e6839f422cf2ebfdfb99402612564bc20bd7/src/jdocmunch_mcp/parser/openapi_parser.py) 只在 sniff 到 OpenAPI/Swagger 后用 `yaml.safe_load`，随后按 API domain 投影 title/version、tag、operation、parameters、responses 和 schema；Vajra 支持通用 YAML，但目标是 essence selection 而非完整显示。

因此，现有实现给出的不是一个“最佳 YAML 压缩算法”，而是一个必须先做的产品选择：

- 如果目标是源文档导航，优先保留 comments、document boundaries、anchors/tags 与 block scalar，并只做可证明的文本展示压缩；
- 如果目标是配置数据查询，可以归一化到共同 value model，再复用 JSON 的 table/default factoring；
- 如果两者都要，应该是两个明确命名的 view，而不是一个含糊的“lossless YAML”开关。

##### 对 Docnav 自定义 renderer 的直接含义

###### 第一版 JSON renderer：响应内、确定性、数据模型可逆

建议把候选按风险排序，而不是一开始追求最大压缩率：

1. generic readable view 始终作为可靠 fallback。
2. 对 contract 明确允许解析的 JSON read content，先尝试 strict compact JSON；若 adapter 已经返回紧凑 serialization，这一步自然没有收益。
3. 对完整的统一对象数组，尝试 header-once/table 表示；保持 row count、column order、missing cell 和 escaping 可验证。
4. 对 dominant value 很强的数组，再评估 `_defaults` 式 factoring；只有 round-trip 到同一 JSON value 且达到收益门槛才采用。
5. 任何 parse failure、shape mismatch、escaping 不可证明、输出不更短或超出预算，都回到 generic view。

这里的“输出不更短”第一版可以先用 exact bytes/characters 作低成本守卫，但正式 token 效果仍需使用项目批准的估算器或 provider tokenizer 验证。守卫不应让 stable protocol shape 变化；它只在 readable presentation 内选择表示。

###### 各 operation 只能渲染自身响应事实

- `outline`/`find` 当前提供的是 entry、ref、preview、page 等响应事实。renderer 可以压紧字段排列、减少重复 labels、显式显示 count/continuation，但不能统计未返回 row、重建完整数组或从 ref 猜层级。
- `read` 可能带有一个被选中节点的 serialized content。是否允许 renderer 再解析这段内容、它是否保证完整节点、怎样标记 content type、解析失败如何 fallback，仍是 OpenSpec contract 问题。
- renderer 不能为了生成更好的表格重新读原文件或调用 adapter；若所需事实不在响应中，应修改候选方案或先批准新的 owner contract，而不是在输出层隐式获取。

###### 有损策略必须提供可观察的省略与恢复语义

RTK 的 `+N more` 至少告诉读者发生了省略，但没有给出逐项恢复地址。Docnav 已经有 ref 和 pagination，因此未来若允许 readable view 省略已返回事实，应同时回答：

- 省略的是 protocol 已分页掉的内容，还是 renderer 又删了一次？
- count 是已返回数、总命中数还是推断值？
- 用户怎样用现有 ref/continuation 精确读到省略部分？
- raw protocol 是否仍包含 readable view 省略的全部响应事实？

在这些问题批准前，renderer 不应采用首项 schema、top-N、异常抽样或 keys-only 作为默认成功输出。

###### YAML 必须先确定 source view 与 normalized data view 的责任

JSON/YAML 的共同 denominator 是解析后的 map/sequence/scalar，但源文档义务不同。未来 YAML adapter/renderer 至少需要先决定：

1. ref 指向 source construct 还是 normalized value path；
2. comments、anchors/aliases、merge keys、tags、duplicate keys、多文档和 block scalars 是否可见、可寻址；
3. semantic table view 是否只是附加 readable representation，raw/source view 怎样保留；
4. 非 string key 和无法归一化的 tag 是 fallback、错误，还是专用节点；
5. 所谓 lossless 是 byte/source round-trip，还是 parsed-value round-trip。

这也是为什么 LeanCTX 与 Headroom 的 YAML 路径都值得研究，却不能直接选一个照搬：它们分别回答了不同的 owner 问题。

##### 推荐的原型顺序

以下是调查建议，不是已批准设计：

1. **P0—建立 renderer fixture 与测量 harness。** 同时记录原始响应 bytes、readable bytes、批准 tokenizer 的 token estimate、是否 round-trip、是否触发 fallback，以及显示事实能否追溯到 `ProtocolResponse`。
2. **P1—优化现有响应字段的排列与 framing。** 不解析 adapter 私有结构，不新增有损选择；解决 operation/branch、opaque ref、preview、page/continuation 的稳定展示。
3. **P2—在 contract 允许后增加 JSON 数据模型可逆候选。** 对完整 JSON read content 优先验证 TOON-like uniform table，再评估 dominant-default factoring；两者都必须 deterministic、UTF-8/escape safe、round-trip checked、never-worse，并以 generic view fallback。
4. **P3—单独验证 YAML 双轨方案。** 一条测试源文本 folding 与 source fidelity；一条测试 YAML-to-value 后复用 JSON table/factoring。使用同一批 YAML edge fixtures 明确两者丢失什么，再决定是否需要两个 view。

统计抽样、query relevance、CCR 和 semantic essence 暂不进入上述顺序。它们会引入新的事实来源、存储、恢复、安全和责任边界，应作为独立 change 评估，而不是 JSON readable renderer 的常规格式化细节。

##### 最小验证样本集

未来实现不能只用“大而规则的对象数组”证明收益；至少应覆盖：

| 类别 | 必要样本 | 要证明的性质 |
| --- | --- | --- |
| JSON 基础 | small object、scalar root、empty object/array、already-minified、pretty JSON | 小输入不膨胀，fallback 稳定 |
| 数组 shape | uniform、same keys different order、sparse、heterogeneous、nested uniform、mixed primitive/object | table applicability 不误判，missing/order 语义清楚 |
| 重复与例外 | dominant defaults、每行唯一值、尾项才出现的字段、尾项 anomaly | factoring 保留 deviation；禁止 first-item 代表全体 |
| 字符串/数字 | comma、quote、newline、Unicode、control char、very long string、`1.0`、`1e3`、large integer | escape 与数值语义不变 |
| JSONC | comments、trailing comma、comment-like string | 明确读取的是 normalized value 还是 source text |
| YAML 结构 | comments、blank lines、anchors/aliases、merge key、custom tag、non-string key、duplicate key、multi-doc | 归一化边界显式，unsupported 时可靠 fallback |
| YAML scalar | literal/folded block、chomping indicator、`#` data line、quoted/unquoted scalar | comment elision 不删数据，source/value loss 可见 |
| 协议分支 | outline/read/find success、empty success、failure、truncated、paginated、continuation | renderer 不改变错误、分页、ref 与 raw protocol |

对数据模型可逆候选，测试必须比较解码后的结构等价，而不能只做 snapshot；对 readable presentation，还需独立 snapshot/approval 证明人和模型能稳定识别 header、row、count、省略与 ref。

##### 对 OpenSpec 六组问题的调查输入

| 未决问题 | 本轮提供的约束 |
| --- | --- |
| operation/branch | shape compaction 只对携带完整可解析 content 的 branch 有意义；entry 列表只能压紧已有字段 |
| 稳定字段、标点、转义、framing | TOON/CSV-like 表示的收益来自 header-once，但必须自己定义 delimiter、quote、newline、missing cell 和 count contract |
| opaque ref | 外部工具多用 JSON path 或自造 path；Docnav renderer 不能照搬其路径推断，只能原样显示 adapter ref |
| preview | 不能从未返回全文重新采样；只能显示响应内 preview 或经批准从当前 content 推导的 bounded preview |
| page/continuation | renderer 不应在 protocol pagination 之上静默做第二次 row selection；任何额外省略都要有 count 和继续读取解释 |
| renderer selection/fallback | 最稳妥模式是 explicit applicability → data-model-reversible candidate → round-trip/size guard → generic fallback；不要默认落入 keys-only sketch |

##### 边界与未决事项

- 本报告证明了固定版本源码中的算法，不证明这些项目的营销数字、端到端任务成功率或最新版本在未来仍相同。
- 没有运行跨项目 tokenizer benchmark；不同 notation 是否真正节省目标模型 token，必须在 Docnav 自有 corpus、真实 output framing 和批准 tokenizer 上测量。
- 数据模型可逆不保证用户更容易读，也不保证语言模型不会误解新 grammar；可逆性、token 数和可理解性需要分别验证。
- 外部实现包含不同许可证与依赖，本轮没有做复制代码或许可证兼容性评估；建议只迁移经过独立设计的算法思想。
- 本报告没有批准解析 read content、引入 TOON syntax、增加 YAML adapter、增加 CCR，或改变任何 public contract。上述动作仍应按 owner 规范与活动 OpenSpec change 决策。
