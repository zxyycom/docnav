# 原始协议

本文是 Docnav 原始协议的主规范。该协议服务于 `docnav --output protocol-json`、脚本、调试和兼容性校验，是 Docnav v0 的机器稳定接口。CLI 阅读输出由 [输出模式](output.md) 拥有；adapter execution 由 [适配器契约](adapter-contract.md) 拥有。

## 协议字段与生命周期

v0 协议字段由 `0.1` schema 记录 documented shape。`protocol_version` 是 envelope 的固定 schema 识别字段，不参与 runtime routing 或 implementation selection；runtime 在 navigation input resolution 和 adapter execution 后产出 protocol envelope，public schema 用于示例、fixture 和 drift check。正常响应的 `protocol_version`、operation 和 result shape 必须与请求及 schema 对齐；无法确定请求 operation 时，错误响应使用 `operation: null`。

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

`arguments` 是已解析的 document operation 输入，只记录进入 adapter 调用的 operation-specific 参数。来源解析、adapter selection、typed-field validation/extraction 和 request construction 由 [Navigation Input Resolution](navigation-input-resolution.md#request-construction) 定义。

v0 operation 参数：

| operation | 请求必需参数 | 可选参数 |
| --- | --- | --- |
| `outline` | 无 | `limit`、`page`、`options` |
| `read` | `ref` | `limit`、`page`、`options` |
| `find` | `query` | `limit`、`page`、`options` |
| `info` | 无 | `options` |

- 最终 `limit` 和 `page` 必须是正整数；入口省略 page 时固定从 `1` 开始。
- `limit` 是 adapter-owned 结果预算。原始协议只要求它是正整数，不定义预算单位；分页执行边界由 [适配器契约](adapter-contract.md#文档操作执行边界) 定义。
- 预算只约束 adapter-owned 结果负载；`protocol_version`、`request_id`、`operation`、`ok`、JSON 字段名和固定包装不计入预算。
- `options` 是 adapter-owned 格式原生参数对象。原始协议只承载该对象；字段声明、来源解析、校验和格式语义见 [Navigation Input Resolution](navigation-input-resolution.md#selected-adapter-参数声明) 与 [适配器契约](adapter-contract.md#文档操作执行边界)。

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

失败响应的 `operation` 在请求 operation 可确定时必须与请求一致；请求无法解析到 operation 时使用 `null`。`error` 对象见 [协议错误对象](#协议错误对象)。

Protocol envelope 只属于 `protocol-json` 输出模式。阅读输出的包装和字段 shape 见 [输出模式](output.md)。

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
kind:
  "structured" | "unstructured", required

structured branch:
  entries[]:
    entry item, required
  page:
    positive integer | null, required

unstructured branch:
  reason:
    "path_rule" | "cost_threshold", required
  content:
    string, required
  content_type:
    string, required
  cost:
    measurements[]:
      unit   string, required
      value  non-negative integer, required
      scope  string, optional
```

Structured outline 返回扁平 entries。层级、位置、摘要和成本等信息只能作为每条 entry 的结构化事实返回，不改变扁平列表模型。

Navigation `outline_mode = "unstructured_full"` 触发的 outline success 使用 unstructured branch。该 branch 直接返回完整 content，不返回 entries、ref、page 或 continuation。`cost.measurements[]` 可以为空；`reason` 稳定区分 `path_rule` 和 `cost_threshold`。

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

`cost.measurements[]` 是机器稳定的结构化成本事实。常见单位包括 `lines`、`bytes` 和 `tokens`，但具体单位集合由 adapter 拥有；阅读输出负责把它压缩为人类可读摘要。

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

`document`、`adapter` 和 `metadata` 是可选事实容器，用于表达文档类型、编码、大小、adapter 身份和 adapter-owned 统计信息。InfoResult 的 protocol-visible scope 是当前文档和选中 adapter 的 facts；operation set 由 adapter contract 定义，manifest/probe 输出由对应 owner 定义。原始协议不返回 info `display`。

## 分页模型

`outline`、`read` 和 `find` result 使用同一 continuation 字段：

```json
{"page": 2}
```

- 响应中的 `page` 是下一页页码；非 null 时必须等于请求 `page + 1`。
- `page: null` 表示当前操作已经没有更多信息。
- `page` 只表达下一页编号，不携带命令、参数或不透明游标。
- 继续读取时，调用方保持 operation、document path 和其它 arguments 稳定，只替换为响应返回的 `page`。
- 请求超过结果末尾的 page 时返回空结果和 `page: null`，不作为错误。

## ref 规则

ref 共享规则由 [Ref](ref-contract.md) 定义。原始协议只承载非空 opaque string。

## 编码

所有格式适配器的 v0 契约只支持 UTF-8，可接受 UTF-8 BOM。无法解码时返回 `DOCUMENT_ENCODING_UNSUPPORTED`。

## 协议错误对象

失败响应的 `error` 来自一个 primary `DiagnosticRecord`。Protocol 调用方稳定解析 `code`、`owner` 和本节列出的 `details`；`message` 和 `guidance` 是可读文案。

| 协议 `code` | 必需 `details` | 可选 `details` |
| --- | --- | --- |
| `INVALID_REQUEST` | `field`、`reason` | `field_issues`、`typed_validation_failures`、`config_issues`、`option_issues` |
| `DOCUMENT_NOT_FOUND` | `path` | 无 |
| `DOCUMENT_PATH_INVALID` | `path`、`reason` | 无 |
| `DOCUMENT_ENCODING_UNSUPPORTED` | `path`、`encoding` | 无 |
| `FORMAT_UNKNOWN` | `path`、`reason`、`candidates` | `candidate_failures` |
| `FORMAT_AMBIGUOUS` | `path`、`candidates` | `candidate_failures` |
| `REF_NOT_FOUND` | `ref` | 无 |
| `REF_AMBIGUOUS` | `ref`、`candidate_count` | 无 |
| `REF_INVALID` | `ref`、`reason` | 无 |
| `ADAPTER_UNAVAILABLE` | `adapter_id`、`reason` | `selection_source`、`stage` |
| `INTERNAL_ERROR` | `error_id` | 无 |

`FORMAT_UNKNOWN.details.reason` 当前稳定值为 `NO_SUPPORTED_ADAPTER`。`candidates` 和 `candidate_failures` 的元素包含 `adapter_id`、`stage` 和 `reason`；`stage` 取值为 `resolve` 或 `probe`。

相关失败只能作为 `details` 的从属结构出现，不形成 sibling error list。修改 protocol error code 或 details 时，先更新本节，再同步 schema、examples、fixtures 和消费方测试。

## Schema 所有权

[protocol-request.schema.json](schemas/protocol-request.schema.json) 和 [protocol-response.schema.json](schemas/protocol-response.schema.json) 只校验原始协议。响应 schema 使用 `operation` 绑定成功 result 类型；`options` 在 protocol schema 中保持 opaque object。阅读输出、manifest 和 probe 使用各自 schema，由对应 owner 文档定义。

Schema 是本文件的验证材料，不重新定义产品语义。修改 protocol-visible envelope、operation 参数、result shape、page、error code 或 details 时，先更新本文件，再同步 schema、examples、fixtures 和消费方测试。
