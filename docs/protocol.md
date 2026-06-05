# 原始协议

本文是 adapter invoke 原始协议的主规范。该协议服务于 `docnav`、脚本、调试和兼容性校验，是 Docnav v0 的机器稳定接口层；它不是 CLI 或 MCP 的阅读输出 schema。

## 版本与生命周期

v0 协议版本为 `0.1`。manifest 的 `protocol.min` 和 `protocol.max` 是闭区间；major/minor 按无符号整数比较。`docnav` 选择自身范围与 adapter 范围交集中的最高兼容版本；没有交集时返回 `PROTOCOL_INCOMPATIBLE`。正常响应版本和 operation 必须与请求一致；无法解析请求或请求版本不受支持时，错误响应使用 adapter 支持的最高版本，并在 details 中保留请求版本或解析原因。

`invoke`：

1. 从 stdin 读取一个 JSON 请求直到 EOF。
2. 只处理一个请求。
3. stdout 只输出一个原始协议 JSON 响应。
4. stderr 只输出诊断。
5. 成功退出码为 `0`；失败尽可能输出结构化错误并返回非零退出码。

## 请求 Envelope

所有字段必需：

```text
protocol_version
request_id
operation          outline | read | find | info
document.path
arguments
```

`arguments` 必须是调用方完成配置解析后的最终显式参数。invoke 不根据调用来源选择隐式默认值。

v0 operation 参数：

| operation | 必需参数 | 可选参数 |
| --- | --- | --- |
| `outline` | `limit_chars`、`page` | `options` |
| `read` | `ref`、`limit_chars`、`page` | `options` |
| `find` | `query`、`limit_chars`、`page` | `options` |
| `info` | 无 | `options` |

- `limit_chars` 和 `page` 必须是正整数，第一页固定为 `1`。
- `limit_chars` 是语义结果的字符预算，用于分页；它按 UTF-8 解码后的 Unicode 字符计数，不按行数，也不按 protocol envelope 字节数。
- 字符预算只约束阅读负载字段：outline/find 按 `ref + display` 计入预算，read 按 `content` 计入预算；`protocol_version`、`request_id`、`operation`、`ok`、JSON 字段名和固定包装不计入预算。
- outline/find 遇到下一条 entry 或 match 会超过预算时，应在当前页停止并返回下一页 page。若单条记录本身超过预算，适配器必须保留完整 ref，并压缩或截断 display，使该页仍能前进；若完整 ref 本身已超过 `limit_chars`，该单条记录可超出预算，但 display 仍应压缩到最小可读文本。
- read 按字符预算切分 content，不能切断 Unicode 字符；若当前位置后仍有内容，返回下一页 page。
- `options` 是调用方从显式参数、配置和 manifest 推荐值解析出的格式原生参数对象；`docnav` 和接入层原样传递其内容。
- 继续读取时，调用方保持 path、ref、query、limit_chars 和 options 不变，只使用响应返回的 page。
- page 是调用位置，不是配置默认参数；CLI 或 MCP 省略 page 时必须显式转换为 `page: 1` 后再启动 invoke。

## 响应 Envelope

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
error.details
error.guidance?
```

失败响应的 `operation` 在请求 operation 可确定时必须与请求一致；请求无法解析到 operation 时使用 `null`。

envelope 仅存在于原始协议层。CLI readable JSON 和 MCP structuredContent 不得包含 `protocol_version`、`request_id`、`operation` 或 `ok`，也不替代完整协议接口。

## 紧凑语义结果

### OutlineResult

```text
entries[]:
  ref       string, required
  display   string, required
page:
  positive integer | null, required
```

outline 永远返回扁平 entries。每条 entry 只承载 ref 与 ref 之外的紧凑 display。例如：

```text
6 lines | 0.1 KB
```

默认文本输出可以将 ref 和 display 拼接为一行。

### ReadResult

```text
ref          string, required
content      string, required
content_type string, required
cost         string, required
page:
  positive integer | null, required
```

### FindResult

```text
matches[]:
  ref       string, required
  display   string, required
page:
  positive integer | null, required
```

### InfoResult

```text
display       string, required
capabilities  array<string>, required
```

## Page

`outline`、`read` 和 `find` 使用同一分页模型：

```json
{"page": 2}
```

- 响应中的 page 是下一页页码；非 null 时必须等于请求 page 加 1，并可直接作为下一次请求的 page。
- `page: null` 表示当前操作已经没有更多信息。
- page 只表达字符分页位置，不携带命令、其他参数或不透明游标。
- 请求超过结果末尾的 page 时返回空结果和 `page: null`，不作为错误。
- 文档变化后，调用方应从第一页重新读取。

## Ref

ref 规则由 [refs.md](refs.md) 定义。原始协议、`docnav` 和接入层只把 ref 当作非空字符串；适配器负责生成和解析。

## 编码

所有格式适配器的 v0 契约只支持 UTF-8，可接受 UTF-8 BOM。无法解码时返回 `DOCUMENT_ENCODING_UNSUPPORTED`。

## 稳定错误

| 错误码 | 稳定 details |
| --- | --- |
| `INVALID_REQUEST` | `field`、`reason` |
| `PROTOCOL_INCOMPATIBLE` | `requested`、`supported_min`、`supported_max` |
| `DOCUMENT_NOT_FOUND` | `path` |
| `DOCUMENT_PATH_INVALID` | `path`、`reason` |
| `DOCUMENT_ENCODING_UNSUPPORTED` | `path`、`encoding` |
| `FORMAT_UNKNOWN` | `path`、`reason`、`candidates` |
| `FORMAT_AMBIGUOUS` | `path`、`candidates` |
| `CAPABILITY_UNSUPPORTED` | `capability`、`adapter_id` |
| `REF_NOT_FOUND` | `ref` |
| `REF_AMBIGUOUS` | `ref`、`candidate_count` |
| `ADAPTER_UNAVAILABLE` | `adapter_id`、`reason` |
| `ADAPTER_INVOKE_FAILED` | `adapter_id`、`reason`、`exit_code` 可选 |
| `INTERNAL_ERROR` | `error_id` |

本地可执行文件 adapter 的 hash 校验失败时，`docnav` 使用 `ADAPTER_UNAVAILABLE`，并将 `details.reason` 设置为可机器识别的 `hash_mismatch`。

错误 message 和 guidance 是可定制文案；调用方只解析 code 和 details。

## Schema 所有权

[protocol-request.schema.json](schemas/protocol-request.schema.json) 和 [protocol-response.schema.json](schemas/protocol-response.schema.json) 只校验原始协议。响应 schema 使用 `operation` 绑定成功 result 类型。阅读输出使用独立 schema 做示例和工具输出校验，不作为机器兼容协议。
