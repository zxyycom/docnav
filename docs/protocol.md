# 原始协议

本文是 Docnav 原始协议的主规范。该协议服务于 `docnav --output protocol-json`、脚本、调试和兼容性校验，是 Docnav v0 的机器稳定接口。CLI 阅读输出由 [输出模式](output.md) 拥有；adapter execution 由 [适配器契约](adapter-contract.md) 拥有。

## 协议字段与生命周期

v0 协议字段由 `0.1` schema 记录 documented shape。`protocol_version` 是 envelope 的固定 schema 识别字段，不参与 runtime routing 或 implementation selection；runtime 在 core 参数补全和 adapter execution 后产出 protocol envelope，public schema 用于示例、fixture 和 drift check。正常响应的 `protocol_version`、operation 和 result shape 必须与请求及 schema 对齐；无法确定请求 operation 时，错误响应使用 `operation: null`。

`docnav --output protocol-json`：

1. 每次 CLI 调用只处理一个 document operation。
2. stdout 只输出一个原始协议 JSON 响应或 failure envelope。
3. 诊断可投影到 stderr。
4. 成功退出码为 `0`；失败尽可能输出结构化错误并返回非零退出码。

## 请求包装

所有字段必需：

```text
protocol_version
request_id
operation          outline | read | find | info
document.path
arguments
```

`arguments` 是 adapter layer 的显式 operation 输入。当前 `docnav` CLI 先通过 [导航配置](navigation-config.md) 和 native option enrichment 得到完成后的 operation input，再由 `docnav-navigation` 构造内部 protocol request。Protocol envelope 不作为配置合并入口；它记录 adapter 调用时实际传入的 operation、document path 和 arguments。

v0 operation 参数：

| operation | 请求必需参数 | 可选参数 |
| --- | --- | --- |
| `outline` | 无 | `limit`、`page`、`options` |
| `read` | `ref` | `limit`、`page`、`options` |
| `find` | `query` | `limit`、`page`、`options` |
| `info` | 无 | `options` |

- 最终 `limit` 和 `page` 必须是正整数，第一页固定为 `1`。
- `limit` 是 adapter-owned 的结果预算，用于分页和单页结果压缩；标准协议只要求它是正整数，不规定预算单位。具体 adapter 必须在自身规范中声明预算如何计数。
- 预算只约束 adapter-owned 结果负载：outline/find 约束每页 entry facts 的可继续输出，read 约束 `content` 切分；`protocol_version`、`request_id`、`operation`、`ok`、JSON 字段名和固定包装不计入预算。
- outline/find 遇到下一条 entry 或 match 会超过预算时，应在当前页停止并返回下一页 page。若单条记录本身超过预算，适配器必须保留完整 ref，并压缩 adapter-owned `label`、`summary`、`excerpt`、`cost` 或 `metadata` 等补充事实，使该页仍能前进；若完整 ref 本身已超过 `limit`，该单条记录可超出预算，但 `label` 仍应保留最小非空定位语义。
- read 按 adapter 声明的预算切分 content；文本 adapter 不能切断 Unicode 字符；若当前位置后仍有内容，返回下一页 page。
- `options` 是 adapter-owned 格式原生参数对象。原始协议只承载该对象，不解释格式语义；当前 core native option enrichment 和 selected-adapter 投影见 [导航配置](navigation-config.md#native-options) 与 [适配器契约](adapter-contract.md#文档操作执行边界)。Type mismatch、range invalid 和格式语义由 consuming adapter 返回 adapter-owned structured diagnostic。
- 继续读取时，调用方保持 path、ref、query 和其它显式参数稳定，只使用响应返回的 page。
- page 是调用位置，不是配置默认参数；入口省略 page 时固定从 `1` 开始。

## 响应包装

成功：

```text
protocol_version
request_id
operation
ok: true
result
```

成功响应的 `operation` 必须与请求一致，并决定 `result` 的具体类型。

失败：

```text
protocol_version
request_id
operation          outline | read | find | info | null
ok: false
error.code
error.message
error.owner
error.location?
error.received?
error.expected?
error.guidance?
error.details
```

失败响应的 `operation` 在请求 operation 可确定时必须与请求一致；请求无法解析到 operation 时使用 `null`。`error` 是本次 failed request 的 primary `DiagnosticRecord` protocol projection；`code`、`message` 和 `owner` 必需，`location`、`received`、`expected`、`guidance` 和 `details` 按 diagnostics owner 的记录内容投影。`details` 只包含当前失败需要的 subordinate list 或 stable detail object，不得用多条 sibling errors 替代 primary record。

envelope 仅存在于原始协议输出。CLI `readable-view` header 和 `readable-json` 不得包含 `protocol_version`、`request_id`、`operation` 或 `ok`，也不替代完整协议接口。

## 紧凑语义结果

outline 和 find 的 item 共享同一组基础字段：

```text
ref       string, required
label     string, required
kind      string, optional
location:
  line_start positive integer, required
  line_end   positive integer, optional
summary   string, optional
excerpt   string, optional
rank      number, optional
cost:
  measurements[]:
    unit   string, required
    value  non-negative integer, required
    scope  string, optional
metadata  object, optional
```

`ref` 是 adapter-owned opaque identifier；`label` 是该 item 的最小非空名称或片段。其它字段是可选结构化事实，adapter 只在能稳定表达时返回。原始协议不返回 `display` 字段；阅读输出按这些事实派生 `display`。

### OutlineResult

```text
entries[]:
  entry item, required
page:
  positive integer | null, required
```

outline 永远返回扁平 entries。层级、位置、摘要和成本等信息只能作为每条 entry 的结构化事实返回，不改变扁平列表模型。

### ReadResult

```text
ref          string, required
content      string, required
content_type string, required
cost:
  measurements[]:
    unit   string, required
    value  non-negative integer, required
    scope  string | null, optional
page:
  positive integer | null, required
```

`cost.measurements[]` 是机器稳定的结构化成本事实。常见单位包括 `lines` 和 `bytes`，但具体单位集合由 adapter 拥有；阅读输出负责把它压缩为人类可读摘要。

### FindResult

```text
matches[]:
  entry item, required
page:
  positive integer | null, required
```

### InfoResult

```text
document:
  content_type string, optional
  encoding     string, optional
  size:
    unit   string, required
    value  non-negative integer, required
    scope  string, optional
adapter:
  id      string, optional
  format  string, optional
metadata  object, optional
```

`document`、`adapter` 和 `metadata` 是可选事实容器，用于表达文档类型、编码、大小、adapter 身份和 adapter-owned 统计信息。标准文档操作集合由 adapter contract、`Operation` enum 和 required adapter trait methods 定义，不由 info result 自报。原始协议不返回 info `display`；阅读输出从这些事实派生可读摘要。

## 分页模型

`outline`、`read` 和 `find` 使用同一分页模型：

```json
{"page": 2}
```

- 响应中的 page 是下一页页码；非 null 时必须等于请求 page 加 1，并可直接作为下一次请求的 page。
- `page: null` 表示当前操作已经没有更多信息。
- page 只表达下一页编号，不携带命令、其他参数或不透明游标。
- 请求超过结果末尾的 page 时返回空结果和 `page: null`，不作为错误。
- 文档变化后，调用方应从第一页重新读取。

## ref 规则

ref 规则由 [ref-contract.md](ref-contract.md) 定义。原始协议、`docnav` 和调用入口只把 ref 当作非空字符串；适配器负责生成和解析。

## 编码

所有格式适配器的 v0 契约只支持 UTF-8，可接受 UTF-8 BOM。无法解码时返回 `DOCUMENT_ENCODING_UNSUPPORTED`。

## 错误投影

本节定义 primary `DiagnosticRecord` 投影到 protocol surface 后的 `code`、`owner`、`location`、`received`、`expected`、`details`、`message` 和 `guidance`。错误机械身份、primary record 字段规则和 details list shape 由 [错误通道](diagnostics.md) 提供；本节拥有这些规则在原始协议中的可观察投影形状。protocol 调用方只依赖本节列出的 code、owner 和 stable details；schema、examples、错误通道实现和消费方测试跟随本节同步。

| 协议 `code` | 必需 `details` |
| --- | --- |
| `INVALID_REQUEST` | `field`、`reason`；可包含 `field_issues`、`typed_validation_failures`、`config_issues` 或 `option_issues` |
| `DOCUMENT_NOT_FOUND` | `path` |
| `DOCUMENT_PATH_INVALID` | `path`、`reason` |
| `DOCUMENT_ENCODING_UNSUPPORTED` | `path`、`encoding` |
| `FORMAT_UNKNOWN` | `path`、`reason`、`candidates`；primary record 可使用 `candidate_failures` 列表表达同一候选摘要 |
| `FORMAT_AMBIGUOUS` | `path`、`candidates` |
| `REF_NOT_FOUND` | `ref` |
| `REF_AMBIGUOUS` | `ref`、`candidate_count` |
| `REF_INVALID` | `ref`、`reason` |
| `ADAPTER_UNAVAILABLE` | `adapter_id`、`reason`；`selection_source`、`stage` 可选 |
| `INTERNAL_ERROR` | `error_id` |

`selection_source` 和 `stage` 只在声明式 adapter 选择失败需要区分来源和失败阶段时出现；自动 discovery 的候选 probe 失败列表使用 `FORMAT_UNKNOWN`/`FORMAT_AMBIGUOUS` candidate summary 表达。

`FORMAT_UNKNOWN.details.reason` 当前稳定值为 `NO_SUPPORTED_ADAPTER`。`FORMAT_UNKNOWN` 和 `FORMAT_AMBIGUOUS` 的 `details.candidates` 是候选摘要数组；primary `DiagnosticRecord.details.candidate_failures` 使用同一元素 shape。每个元素包含 `adapter_id`、`stage` 和 `reason`。`stage` 取值为 `resolve` 或 `probe`；`reason` 是候选层稳定原因码，当前取值包括 `ADAPTER_NOT_FOUND`、`ADAPTER_UNAVAILABLE`、`PROBE_INVALID`、`PROBE_UNSUPPORTED` 和 `CONTENT_MATCH`。Protocol error details 的稳定契约到候选摘要为止；adapter probe payload 和人类说明文案由内部错误通道按各自契约承载。Manifest metadata invalid 和 missing linked handler 属于 adapter layer invariant failure，不进入默认 automatic discovery candidate reason set。

错误 message 和 guidance 是可定制文案；调用方只解析 code、owner 和 stable details。`INVALID_REQUEST` 可以在 top-level projection 中附带 `location`、`received` 或 `expected`，也可以在 details 中附带 `field_issues`、`typed_validation_failures`、`config_issues` 或 `option_issues`。Core key/source/shape failures 使用 `field_issues` 或 `config_issues`；adapter-owned native option validation 使用 `option_issues` 表达 option owner、namespace/key、source、reason_code，以及可用的 type_variant、received 和 expected。range/type failure 必须在 top-level projection 或对应 option issue 中提供可比较的 received/expected 信息。显式 adapter 不存在时仍返回 adapter selection diagnostic，不投影为 option validation error。这些补充字段不得替代必需的 `field` 和 `reason`。

Protocol response schema 是本节的验证材料，用于校验 protocol-visible code、details 字段集合、字段类型和 required details。修改 protocol error code 或 details 规则时，先更新本节和对应 schema/examples，再同步错误通道实现和消费方测试。

## Schema 所有权

[protocol-request.schema.json](schemas/protocol-request.schema.json) 和 [protocol-response.schema.json](schemas/protocol-response.schema.json) 只校验原始协议。响应 schema 使用 `operation` 绑定成功 result 类型。阅读输出使用独立 schema 做示例和工具输出校验，不作为机器兼容协议。

Protocol response stdout 只输出 schema payload；`docnav --output protocol-json` stdout 只输出对应 protocol-shaped payload。Strict public input failure 和 automatic discovery 全部失败时的 candidate list 都使用 protocol failure projection。

`docnav-protocol` 当前只承接 protocol request/response 数据结构、schema 对齐和 protocol error projection。导航配置合并发生在 core 构造 operation input 之前；`docnav-navigation` 消费完成后的 operation input 并生成 protocol request。Native options 在 protocol schema 中保持 opaque object；adapter-owned semantics 由 consuming adapter 诊断。

Operation-specific typed request 由 core 参数补全和 `docnav-navigation` request construction 共同产出。response、manifest 和 probe 等已归一化 payload 可以继续使用 typed deserialize + semantic validate。

调用方继续拥有错误归属、field path、request id fallback、stdout/stderr placement 和 exit behavior。public JSON Schema 文件保留为 contract material、examples/fixtures 校验和工具链 drift check，不作为 production decode path 的 generic schema validator。[错误通道](diagnostics.md) 拥有内部 code、category、primary record 字段和 details rule；这些规则投影到 protocol surface 时必须符合本文件定义的 code、owner 和 details。readable wrapper、manifest/probe policy 由各自 owner 文档定义。
