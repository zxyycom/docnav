# JSON Adapter

本文是 `docnav-json` 当前导航行为和私有契约的长期 owner。Pathname routing hints、
fixed-strategy probe deletion、`DOCUMENT_CONTENT_INVALID` migration、raw adapter
operations、core static integration 与 generic readable path 均已有 Current 证据；
格式专用 readable renderer 仍按下文边界保持 Planned。本文中的 `MUST` 按
[文档导航的状态语义](../navigation.md#规范状态与实现状态)表达稳定契约。

共享 adapter interface、protocol result shape、ref 传递和输出编排分别由
[适配器契约](../adapter-contract.md)、[原始协议](../protocol.md)、
[Ref](../ref-contract.md)和[输出模式](../output.md)拥有。本文只拥有 JSON 的
识别、解析模型、ref grammar、导航语义和 JSON-owned 错误边界。

## 交付与公共边界

当前 adapter identity 为：

| Fact | Value |
| --- | --- |
| adapter id | `docnav-json` |
| format id | `json` |
| `extensions[]` routing suffixes | `.json`、`.code-workspace`，按共享契约做 ASCII 大小写归一化 |
| `filenames[]` exact routing hints | `.prettierrc`、`.watchmanconfig`，大小写敏感 exact basename |
| content type | `application/json` |

Adapter id、format id、content type 和 pathname hint rows 均为 Current metadata。

`docnav-json` MUST 作为 core static registry 中的 linked adapter，通过 package
内同一个 `docnav` executable 交付。Current fixed strategy surface 提供 `outline`、
`read`、`find` 和 `info`，并声明既有 unstructured full-read content/cost capability；
pathname hint rows 和 probe deletion 服从上方 Current 状态。
它的 executable set 精确为 package core `docnav` 这个单元素集合。

JSON strategy MUST 只消费各 operation 的 closed standard input。注册 JSON 不得
增加 core parameter、`StandardInputBinding`、CLI、env、config 或 protocol
input，也不得把 JSON 私有上限提升为公共选项。首期支持一个 UTF-8 JSON value；
JSON-like syntax、schema-aware semantics 和 arithmetic number semantics 不在本
契约内。

## Pathname routing 与私有解析模型

**Current routing contract：** JSON manifest 的上述 suffix/exact-filename hints 只用于
navigation-private complete-basename lookup。Routing 在 target-document I/O 前选择
linked `docnav-json` definition，不读取内容、不构造 JSON model，也不把 matched
filename/suffix 或 format identity 传给 strategy。固定 JSON strategy 不保留 selection
probe、probe result/reason/version 或兼容 inspection surface。完整 lookup、explicit
override 和 no-fallback 规则由[适配器契约](../adapter-contract.md#adapter-选择)与
[Navigation Input Resolution](../navigation-input-resolution.md#adapter-selection-and-path-sequencing)
拥有。

Pathname hint 不是 JSON validity proof。每次 selected operation MUST 从 normalized
document path 获取实际 document view，并按当前 strict JSON grammar：

1. 去除至多一个开头 UTF-8 BOM，并执行 UTF-8 decode。
2. 完整解析一个 JSON value，只允许其后存在 whitespace。
3. 以 root depth `0` 计算深度，并要求最大 depth 不超过 `127`。
4. 按解码后的 member name 检查每个 object；同一 object 内的 name 必须唯一。

`.prettierrc` 可以包含 strict JSON 之外的 YAML，`.code-workspace` 可以包含 JSON
comments；在独立 JSONC grammar change 落地前，这两个 pathname alias 都只是
best-effort hint。内容超出当前 strict JSON grammar 时，selected JSON strategy 返回
正常 JSON-owned parse diagnostic，navigation 不重新 route 或尝试其它 adapter。本
pathname-routing contract 不实现 JSONC grammar。

`127` 由 adapter-private 单一硬编码配置源拥有，不形成公共 input。一次 selected
operation 只解析一次，并建立一个 primary document model。该 model MUST 同时保存：

- 去除可选 BOM 后的原文和原文件 byte size；
- JSON tree、node kind、depth 和 node count；
- object member 的 decoded name、源码顺序与 member/node source region；
- array element 的 index 顺序与 node source region；
- 每个 number 的原始 source token；
- ref resolution、tree preorder 和 source occurrence mapping 所需的索引。

Object MUST 以唯一的有序 member sequence 保存；array 保持 index 顺序。同一
primary tree 服务 traversal、ref resolution、structured read 和 source indexing，
不得为源码顺序建立第二份全量 tree。Workspace-pinned parser/serializer 是唯一
JSON parser package；升级它时必须复核本文拥有的可观察 spelling、顺序和 region
行为。

## JSON Ref Grammar

JSON ref 是非空、ASCII-safe 的 RFC 6901 URI fragment identifier
representation：

```text
json:#<fragment>
```

Root ref 固定为 `json:#`。每个 path token 先把 `~` 和 `/` canonical escape 为
`~0` 和 `~1`，再把 URI fragment 不允许的 UTF-8 bytes percent-encode；percent
escape 的十六进制字母必须大写。生成的 ref 不得包含原始 NUL 或其它控制字符。

- 空 object key 使用 `json:#/`。
- Object token 按 decoded member name 解释，包括纯数字 member name。
- Array token 只能是 `0` 或不带前导零的十进制 index；`-` 不是可读节点。
- 同一个 token（例如 `01`）在 object 上可以是 member name，在 array 上则非法。

以下情况返回 `REF_INVALID`：

- 缺少 `json:#` prefix；
- non-root fragment 缺少前导 `/`；
- percent escape 或 `~` escape 非法或不是 canonical spelling；
- 当前 path context 是 array，但 token 不是 canonical array index。

Grammar canonical、但当前 document model 中不存在的 object member 或 array
index 返回 `REF_NOT_FOUND`。有效 JSON 已拒绝重复 decoded member，因此一个
canonical ref 在同一次解析结果中至多定位一个 value。文档变化后，相同结构路径
可以指向新 value 或返回 `REF_NOT_FOUND`；ref 不承诺跨文档版本的持久身份。

## Outline

JSON outline MUST 先形成确定性 flat entries，再使用既有 entry pagination：

- Object member 和 array element 按 depth-first preorder 遍历。
- Object member 保持源码顺序；array element 按 index 升序。
- 每个 descendant entry 的 `ref` 是完整 JSON ref，`kind` 是
  `object|array|string|number|boolean|null`。
- Object child 的 `label` 是 decoded member name；空 key 的正常 label 是两个双引号字符 `""`，其 ref 仍是 `json:#/`。
- Array child 的 `label` 是 `[<index>]`。

Root object 或 array 本身不形成 entry，其 entries 只包含 descendants；因此空
`{}` 或 `[]` 返回空 entries 和 `page: null`。Root scalar 返回唯一 entry：
`ref: "json:#"`、`label: "<root>"`，kind 对应 scalar 类型。

首期 JSON outline 只使用既有 common item fields，不增加 JSON-specific raw
field；source `location` 和 JSON-specific `metadata` 均不返回。任何 entry 的
ref MUST 能原样传给 `read`。

## Read

JSON read MUST 解析 JSON-owned ref，并把目标 value 序列化为确定性的 structured
JSON：

- object member 保持源码顺序；
- container 使用两空格缩进的 pretty layout；
- number 使用原始 source token；
- string escape、其它 scalar spelling 和尾随换行使用 workspace-pinned
  parser/serializer 的自然结果。

Raw `ReadResult` MUST 保留输入 ref，使用 `application/json`，并对分页前的完整
structured text 计算 cost。`content` 按既有 Unicode-safe text pagination 返回，
不得切断多字节字符；`page` 指向下一页，结束时为 null。

## Find

JSON find 的 query 长度必须至少为 `1`；空 query 必须拒绝。Find MUST 在去除一个
可选开头 UTF-8 BOM 后的原文中执行大小写敏感、从左到右、非重叠的 literal
search。Canonical pointer 和 structured-read serialization 不属于搜索语料。

Source region 按以下规则把每个 occurrence 归属到可读取 value：

- root region 覆盖完整 BOM-stripped source；
- object child region 从 member name token 开始并覆盖其 value；
- array child region 覆盖其 value；
- occurrence 归属完整覆盖其范围的最深 region；
- child region 外的 container 结构或空白归属最近 container，root value 外围
  whitespace 归属 root；
- 跨越多个 child region 的 occurrence 归属同时覆盖它们的最近 container。

每个 source occurrence MUST 形成一个 match，并按 source offset 排序；多个
occurrence 即使映射到同一 ref 也不得合并。Raw match 使用完整 ref、
`kind: "match"`、从原文派生的非空 bounded excerpt `label` 和 source line
location。返回的 ref MUST 能原样传给 `read`。

作为 adapter-private operational invariant，每个 match 的 label construction
所保留的 character state 和 context scan work MUST 由 label budget 约束；
context scan 不得随 source line 或 occurrence 周围连续空白的长度增长。具体
buffer 数量和 scan threshold 是实现与测试证据，不属于 public contract。

## Info 与 Full-read

JSON info MUST 返回：

- document content type `application/json` 和 encoding `UTF-8`；
- 包含可选 BOM 的原文件 byte size；
- adapter id `docnav-json` 和 format id `json`；
- key set 精确为 `{root_kind, node_count, max_depth}` 的 JSON metadata。

`root_kind` 使用 `object|array|string|number|boolean|null`；`node_count` 包含
root；`max_depth` 以 root 为 `0`。

Unstructured full-read MUST 返回去除一个可选 UTF-8 BOM 后、其它内容保持原样的
JSON source text 和 `application/json`。Cost 的 `lines`、`bytes`、`tokens`
measurement 针对实际返回 text。该 capability 只补充既有 unstructured full-read
content/cost facts，不返回 entries、ref、page、continuation 或 readable-only
wrapper。

## Pagination、Cost 与截断

JSON 不定义新的 pagination 或 cost shape。Outline/find 使用既有 entry
pagination，read 使用既有 Unicode-safe text pagination；响应 `page` 是下一页
页码，结果耗尽或请求超过末页时为 null。

Entry pagination 截断 display facts 时 MUST 保持 occurrence/traversal order，
始终保留完整 ref 和最小非空 label，并保证分页前进。分页预算截断后没有可见的
正常 label 内容可保留时，使用 `.` 作为最小非空 fallback；该 fallback 不替代空
key 的正常 label `""`。Read cost 描述分页前的完整 selected value；full-read
cost 描述实际返回 source text。结构化 measurement 和 page shape
继续由[原始协议](../protocol.md)拥有。

Find pagination 的 retained entry working set MUST 随请求的 page limit 保持
有界，continuation 判定只增加有界状态；不得为分页预先保留完整 occurrence set。
具体 iterator 和 lookahead mechanics 是 adapter-private 实现证据。

## 错误边界

**Current error contract：** 下表的 selected JSON content reasons 与 post-selection
document-change 处理已随 no-probe cutover 生效。JSON internal parser mapping 不由本
routing contract 重述。

JSON-owned failure 按以下边界映射：

| 条件 | 结果 |
| --- | --- |
| Selected document 缺失、path/access 无效 | 既有 `DOCUMENT_NOT_FOUND` / `DOCUMENT_PATH_INVALID` diagnostic |
| Selected document 不是有效 UTF-8 | `DOCUMENT_ENCODING_UNSUPPORTED` |
| JSON syntax 无效 | `DOCUMENT_CONTENT_INVALID`，reason `JSON_SYNTAX_INVALID` |
| 完整 value 后存在 non-whitespace trailing input | `DOCUMENT_CONTENT_INVALID`，reason `JSON_TRAILING_INPUT` |
| 同一 object 存在重复 decoded member name | `DOCUMENT_CONTENT_INVALID`，reason `JSON_DUPLICATE_MEMBER` |
| 最大 depth 超过 `127` | `DOCUMENT_CONTENT_INVALID`，reason `JSON_MAXIMUM_DEPTH_EXCEEDED` |
| JSON ref grammar 或 context-sensitive array token 非法 | `REF_INVALID` |
| Canonical ref 在当前 document model 中不存在 | `REF_NOT_FOUND` |
| Pathname selection 后、selected operation 读取前文档发生变化 | 按 operation 实际打开的 document view 返回上述正常 document/JSON diagnostic；不使用独立 mutation stage id |

`DOCUMENT_CONTENT_INVALID.details` 只包含 normalized `path` 与上表 stable `reason`。
Parser library type/message、unstable offset、duplicate member name 和 dependency trace
保持私有。错误 envelope、共享 diagnostic code 和 details shape 由
[原始协议](../protocol.md)拥有。Selected failure 不触发 pathname routing 或第二个
adapter。

## Raw 与 Readable 输出边界

JSON strategy 只返回本文规定的 raw result facts。`protocol-json` 序列化包含这些
facts 的同一个 `ProtocolResponse`；generic `readable-view` 从它派生 outline/find
display、read header、cost summary 和 length-delimited content block。Raw
protocol 不包含 `display` 或 readable framing。

当前实现使用现有 generic renderer 走通 outline、read、find、info 和
full-read，不包含 JSON renderer、renderer 选择输入或公共输出 shape。JSON-specific
信息密度、完整 opaque ref 的路径定位信号、标点、preview 和分页 presentation
由后续独立 change 规划；该 renderer 不解析 ref，也不合成 hierarchy、depth、
parent 或 indentation。这些都不是本页的 Current 行为。

## 验证边界

实现证据 MUST 覆盖 manifest pathname hints、selected-operation parse、decoded
duplicate key、depth 上限、source order、raw number、source-region mapping、ref
roundtrip 和错误分类；operation 证据 MUST 覆盖 outline/read/find/info/full-read、
空容器、root scalar、Unicode pagination、cost 和 generic readable view。
Core/CLI/release 证据另外覆盖 automatic/explicit selection、route-before-document-I/O、
closed public input、selected failure no-fallback、static registry，以及同一个 release
binary 中的 Markdown 与 JSON 行为。

测试层级和 release 验证边界见[测试策略](../testing.md)和
[发布包验证](../testing/release.md)。格式专用 readable renderer 的实现证据不在
本页已建立的 Current 基线内。
