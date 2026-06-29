本 design 记录 Docnav raw protocol 字段结构化方案，以及 readable 输出的组织规则。

## 范围

本 change 覆盖以下 public contract surface：

- Protocol request arguments：分页预算字段。
- Protocol success results：read cost、outline/find navigation items、info metadata、page。
- Protocol failure results：`error.code`、`error.details`、`error.message`、`error.guidance`。
- Readable output：readable-json/readable-view 的 display、成本摘要、warning、error 和 continuation。
- Validation material：protocol/readable schema、examples、fixtures 和 Rust contract tests。

## 字段基线

| 区域 | 原字段形态 | 目标归属与字段形状 |
| --- | --- | --- |
| 预算 | `limit_chars` positive integer。 | Protocol 硬切换为 `limit` positive integer。Core/SDK 负责来源合并；adapter 负责预算单位解释。 |
| read 成本 | `cost` 展示字符串。 | Protocol 使用 `cost.measurements[]`；readable 拥有成本摘要文本。 |
| outline 条目 | `ref` + `display`。 | Protocol 使用 `ref` + 条目事实字段；readable 拥有 display row。 |
| find match | `ref` + `display`。 | Protocol 使用 `ref` + match 事实字段；readable 拥有 match row。 |
| info result | `display` + `capabilities`。 | Protocol 使用结构化 document/adapter metadata；readable 拥有紧凑摘要。 |
| page | 下一页整数或 null。 | 保持 protocol-owned continuation 不变。 |
| error | code/message/details/guidance。 | Diagnostics 拥有 identity/effect/details payload；protocol/readable 拥有投影。 |
| warning | id/reason/effect/details。 | Diagnostics 拥有 warning identity/effect/details；readable 拥有 warning 投影和文本渲染。 |

## 协议字段形状

### 请求预算

Document operation arguments 使用 `limit`：

```json
{
  "limit": 80,
  "page": 1
}
```

新 schema、examples、typed arguments、operation handling 和 renderer input 都使用 `limit`。

### 成本

Protocol cost 使用结构化 measurement object：

```json
{
  "measurements": [
    { "unit": "lines", "value": 7, "scope": "selection" },
    { "unit": "bytes", "value": 100, "scope": "selection" }
  ]
}
```

必需字段：`unit`、`value`。

可选字段：`scope`。

Adapter-owned policy 决定 measurement unit、计算方式、排序和 tokenizer 行为。SDK helper 可以提供机制，但不决定 adapter 报告策略。

### 导航条目

Outline entries 和 find matches 使用共享 item base：

```json
{
  "ref": "H:L4:H2",
  "label": "Install",
  "kind": "heading",
  "location": { "line_start": 4, "line_end": 10 },
  "summary": "Primary installation instructions.",
  "cost": {
    "measurements": [
      { "unit": "lines", "value": 7, "scope": "entry" }
    ]
  },
  "metadata": { "heading_level": 2 }
}
```

必需字段：`ref`、`label`。

可选字段：`kind`、`location`、`summary`、`excerpt`、`rank`、`cost`、`metadata`。

`ref` 保持 adapter-owned opaque string。共享层只校验 shape 并原样传递，不从 ref 解析 heading grammar、唯一性或 region boundary。

### Info Result

Info 把 metadata 与 readable summary 分开：

```json
{
  "document": {
    "content_type": "text/markdown",
    "encoding": "utf-8",
    "size": { "unit": "bytes", "value": 188 }
  },
  "adapter": {
    "id": "docnav-markdown",
    "format": "markdown"
  },
  "capabilities": ["outline", "read", "find", "info"],
  "metadata": {}
}
```

必需字段：`capabilities`。

可选字段：`document`、`adapter`、`metadata`。

Adapter 只暴露能够稳定维护为契约的 metadata。

### 错误 details

Protocol failure output 保持 protocol envelope，并使用 diagnostics-derived error projection：

```json
{
  "ok": false,
  "error": {
    "code": "REF_INVALID",
    "message": "ref 不符合当前 adapter 的 ref 格式要求。",
    "details": {
      "ref": "bad:ref",
      "reason": "unrecognized ref grammar"
    },
    "guidance": ["重新调用 outline 获取有效 ref。"]
  }
}
```

`error.code` 和 `error.details` 是机器可读字段。`error.message` 和 `error.guidance` 是展示字段。

按 code 定义的 details：

| Code | Details shape |
| --- | --- |
| `INVALID_REQUEST` | 必需 `field`、`reason`；可选 `path`、`received`、`accepted[]`。 |
| `DOCUMENT_NOT_FOUND` | 必需 `path`。 |
| `DOCUMENT_PATH_INVALID` | 必需 `path`、`reason`。 |
| `DOCUMENT_ENCODING_UNSUPPORTED` | 必需 `path`、`encoding`。 |
| `FORMAT_UNKNOWN` | 必需 `path`、`reason`、`candidates[]`。 |
| `FORMAT_AMBIGUOUS` | 必需 `path`、`candidates[]`。 |
| `CAPABILITY_UNSUPPORTED` | 必需 `capability`、`adapter_id`。 |
| `REF_NOT_FOUND` | 必需 `ref`。 |
| `REF_AMBIGUOUS` | 必需 `ref`、`candidate_count`。 |
| `REF_INVALID` | 必需 `ref`、`reason`。 |
| `ADAPTER_UNAVAILABLE` | 必需 `adapter_id`、`reason`；可选 `exit_code`、`stderr`。 |
| `ADAPTER_INVOKE_FAILED` | 必需 `adapter_id`、`reason`；可选 `exit_code`、`stderr`。 |
| `INTERNAL_ERROR` | 必需 `error_id`。 |

Format candidates 使用：

```json
{
  "adapter_id": "docnav-markdown",
  "supported": false,
  "confidence": 0.0,
  "reason_codes": ["CONTENT_CONFLICT"]
}
```

## Readable 组织

Readable outputs 是面向阅读的投影。它们保持 operation-specific shape，并省略 `protocol_version`、`request_id`、`operation` 和 `ok` 等 protocol envelope 字段。

规则：

1. `display` 归 readable 所有，可以组合 label、location、cost 和 adapter-specific summary。
2. readable `cost` 是从 `cost.measurements[]` 派生的紧凑字符串或阅读块。
3. readable `page` 原样复制 protocol continuation。
4. readable errors 保留稳定 `code` 和结构化 `details`，`error` 与 `guidance` 是展示字段。
5. readable warnings 保留稳定 `id`、`effect` 和 per-id `details`，`reason` 是展示字段。

Warning details：

| Warning id | Details shape |
| --- | --- |
| `cli_argv_ignored` | 必需 `tokens[]`。 |
| `adapter_candidate_failure` | 必需 `adapter_id`、`stage`、`code`；可选 `preselected`。 |
| `adapter_config_source_skipped` | 必需 `source_level`、`path_origin`、`path`、`reason_code`。 |

## 迁移顺序

1. 更新 owner 文档：`docs/protocol.md`、`docs/output.md`、`docs/diagnostics.md`、`docs/standard-parameters.md`、`docs/adapter-contract.md` 和 adapter 文档。
2. 更新 protocol/readable schemas 和 examples。
3. 更新 Rust protocol 与 diagnostics projection 类型和测试。
4. 更新 core 与 SDK 参数映射，使其使用 `limit`。
5. 更新 Markdown adapter 输出。
6. 更新 readable renderer。
7. 运行 schema/example validation、Rust tests、CLI smoke 和 workspace verification。
