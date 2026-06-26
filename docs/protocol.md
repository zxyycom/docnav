# 原始协议

本文是 adapter invoke 原始协议的主规范。该协议服务于 `docnav`、脚本、调试和兼容性校验，是 Docnav v0 的机器稳定接口层；它不是 CLI 阅读输出 schema。

## 协议字段与生命周期

v0 协议字段按 `0.1` schema 校验。`protocol_version` 是 envelope 的固定 schema 识别字段，不参与 adapter 路由、安装、更新或 invoke 的版本区间协商；`docnav` 通过该 schema、必需字段、字段类型、operation/result 绑定和语义校验判断 adapter 输出是否可用。正常响应的 `protocol_version`、operation 和 result shape 必须与请求及 schema 对齐；无法解析请求或无法提取版本字段时，错误响应使用协议常量 `protocol_version` 和 `operation: null`，请求 operation 可确定时在错误响应中保留该 operation。

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

`arguments` 是 adapter `invoke` 的显式 operation 输入。缺失的已注册标准参数可以由 adapter `invoke` 入口的配置或默认值补足。Protocol schema 只校验 envelope、operation、document path、raw `arguments` object 和已出现已注册字段的基础 JSON 类型；未映射 `arguments` 字段不由标准参数层解释，adapter 入口可按 [标准参数](standard-parameters.md#合并透传与校验) 中的透传处理结果和入口策略保留、丢弃或交给 adapter-owned 语义校验。

v0 operation 参数：

| operation | 请求必需参数 | 可选参数 |
| --- | --- | --- |
| `outline` | 无 | `limit_chars`、`page`、`options` |
| `read` | `ref` | `limit_chars`、`page`、`options` |
| `find` | `query` | `limit_chars`、`page`、`options` |
| `info` | 无 | `options` |

- 最终 `limit_chars` 和 `page` 必须是正整数，第一页固定为 `1`。
- `limit_chars` 是语义结果的字符预算，用于分页；它按 UTF-8 解码后的 Unicode 字符计数，不按行数，也不按 protocol envelope 字节数。
- 字符预算只约束阅读负载字段：outline/find 按 `ref + display` 计入预算，read 按 `content` 计入预算；`protocol_version`、`request_id`、`operation`、`ok`、JSON 字段名和固定包装不计入预算。
- outline/find 遇到下一条 entry 或 match 会超过预算时，应在当前页停止并返回下一页 page。若单条记录本身超过预算，适配器必须保留完整 ref，并压缩或截断 display，使该页仍能前进；若完整 ref 本身已超过 `limit_chars`，该单条记录可超出预算，但 display 仍应压缩到最小可读文本。
- read 按字符预算切分 content，不能切断 Unicode 字符；若当前位置后仍有内容，返回下一页 page。
- `options` 是 adapter-owned 格式原生参数对象。只有在对应 registration 声明时，`options` 或其中 native option 才参与标准参数解析；`docnav` 和接入层不从 manifest、core 配置或隐式默认值合成格式专属 options。
- 继续读取时，调用方保持 path、ref、query 和其它显式参数稳定，只使用响应返回的 page。
- page 是调用位置，不是配置默认参数；入口省略 page 时固定从 `1` 开始。

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

envelope 仅存在于原始协议层。CLI `readable-view` header 和 `readable-json` 不得包含 `protocol_version`、`request_id`、`operation` 或 `ok`，也不替代完整协议接口。

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

阅读输出可以将 ref 和 display 作为紧凑可读记录展示；`readable-view` header 仍保留结构化 `ref` 和 `display` 字段。

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

ref 规则由 [ref-contract.md](ref-contract.md) 定义。原始协议、`docnav` 和接入层只把 ref 当作非空字符串；适配器负责生成和解析。

## 编码

所有格式适配器的 v0 契约只支持 UTF-8，可接受 UTF-8 BOM。无法解码时返回 `DOCUMENT_ENCODING_UNSUPPORTED`。

## 稳定错误

| 错误码 | 稳定 details |
| --- | --- |
| `INVALID_REQUEST` | `field`、`reason` |
| `DOCUMENT_NOT_FOUND` | `path` |
| `DOCUMENT_PATH_INVALID` | `path`、`reason` |
| `DOCUMENT_ENCODING_UNSUPPORTED` | `path`、`encoding` |
| `FORMAT_UNKNOWN` | `path`、`reason`、`candidates` |
| `FORMAT_AMBIGUOUS` | `path`、`candidates` |
| `CAPABILITY_UNSUPPORTED` | `capability`、`adapter_id` |
| `REF_NOT_FOUND` | `ref` |
| `REF_AMBIGUOUS` | `ref`、`candidate_count` |
| `REF_INVALID` | `ref`、`reason` |
| `ADAPTER_UNAVAILABLE` | `adapter_id`、`reason` |
| `ADAPTER_INVOKE_FAILED` | `adapter_id`、`reason`、`exit_code` 可选 |
| `INTERNAL_ERROR` | `error_id` |

本地可执行文件 adapter 的 hash 校验失败时，`docnav` 使用 `ADAPTER_UNAVAILABLE`，并将 `details.reason` 设置为可机器识别的 `hash_mismatch`。

错误 message 和 guidance 是可定制文案；调用方只解析 code 和 details。

本节是稳定错误语义的 owner；[error-rules.json](protocol/error-rules.json) 是 required details 的机器维护入口，用于统一 Rust 校验、文档验证常量和 `protocol-response.schema.json` 的错误 details 校验块。修改错误码或 required details 后运行 `bun run generate:error-rules`。

## Schema 所有权

[protocol-request.schema.json](schemas/protocol-request.schema.json) 和 [protocol-response.schema.json](schemas/protocol-response.schema.json) 只校验原始协议。响应 schema 使用 `operation` 绑定成功 result 类型。阅读输出使用独立 schema 做示例和工具输出校验，不作为机器兼容协议。

CLI argv 兼容 warning 和 adapter candidate warning 只属于阅读输出层或 stderr 诊断。protocol response、manifest 和 probe schema 不增加 `warnings` 字段；`docnav --output protocol-json` 和 adapter direct `protocol-json` stdout 仍只输出对应 protocol-shaped payload。

`docnav-protocol` decode pipeline 的可复用范围是 `serde_json::Value -> schema validate -> typed deserialize -> semantic validate`。pipeline 先按 owning schema 校验，再反序列化为 typed contract data，最后执行语义校验；调用方继续拥有 stable error category、field path、diagnostic text、request id fallback、stdout/stderr placement 和 exit behavior。readable wrapper、warning envelope、manifest/probe policy 由各自 owner 文档定义；直接 CLI argv 映射与兼容边界见 [标准参数](standard-parameters.md#输入与配置映射)。
