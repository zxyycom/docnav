# 输出模式

本文是 `docnav` 文档操作输出模式、readable rendering、primary diagnostic 投影、阅读文案配置和输出通道的主规范。CLI 命令面见 [CLI](cli.md)；navigation input resolution 见 [Navigation Input Resolution](navigation-input-resolution.md)；原始协议 envelope、result 和 error 字段见 [原始协议](protocol.md)。

## 输出层边界

`--output` 只选择已产生的 document outcome 或 primary diagnostic 如何序列化、包装和写入通道，不改变上游解析、dispatch 或业务结果判断。Document operation 只接受 `readable-view`、`readable-json` 和 `protocol-json`。实现应先产出成功结果或 primary diagnostic，再按 output context 渲染为 readable-view、readable JSON 或 protocol envelope。

`docnav-output` 是 document operation 输出编排和 primary failure projection owner：调用方传入 operation、request id、output mode、document outcome 和 primary `DiagnosticRecord` 或成功 payload，由该层决定 `readable-view`、`readable-json` 或 `protocol-json` 的包装、error 投影和 stdout/stderr 分流。

机器可读输出必须优先保持稳定和可解析。若调用方选择 `protocol-json` 或 `readable-json`，stdout 必须只输出一个符合该模式 documented shape 的 JSON 值；只要输出模式可以从 argv 或请求中确定，失败也必须使用对应 JSON 错误形态。无法确定 operation 时，协议错误 envelope 使用 `operation: null`。

上游失败或 renderer 失败进入输出层时，必须已经归并为一个 primary `DiagnosticRecord`。输出层只负责按当前 output mode 投影该 primary diagnostic，不重新分类失败来源、不改写 details 语义、不新增 sibling error list。

## `protocol-json`

用途：完整接口、脚本、调试和稳定接口校验；不以可读性为目标。正常阅读不使用该模式。

```text
docnav outline docs/guide.md --output protocol-json
docnav read docs/guide.md --ref "<ref-from-outline>" --output protocol-json
```

文档操作输出完整原始协议 envelope，字段语义按 [原始协议](protocol.md) 的 envelope、result 和 protocol error object。`protocol-json` mode 的输出层规则是把该 envelope 作为 stdout 的唯一 JSON 值。

`protocol-json` stdout 只承载 protocol response 或 failure envelope。只要 argv 或请求能确定 `--output protocol-json`，document operation 的 input、selection、dispatch 或 output conversion failure 都投影为 protocol failure envelope，而不是退回文本错误。无法确定 operation 时，failure envelope 使用 `operation: null`。

成功 response 的 `result` 保留 adapter 返回的结构化事实。阅读输出需要的 `display`、成本摘要和精简字段由 readable output 从这些事实派生；原始协议不反向接受 readable-only 字段。

## `readable-view`

用途：文档操作的默认输出模式。人类和 AI 直接阅读，信息密度高，开箱即可定位内容。输出由一个 pretty JSON header 和零个或多个 length-delimited block section 组成。调用方和测试通过字段名和值、block pointer 和 UTF-8 byte length 判断语义；JSON header object key 顺序和多个 block section 的输出顺序不作为稳定契约。

成功 header 始终只包含阅读层操作字段（ref、display、content_type、cost、page 等）和该 operation 拥有的 success payload 字段。outline/find 的 `display` 由 raw item facts 派生；read 的 `cost` 是由 `cost.measurements[]` 派生的人类可读摘要。renderer config 声明为 block 的字符串字段（例如 read 的 `/content`、readable error 的 `/error`）在 header 中以 `{"$block": "<pointer>", "bytes": <utf8-byte-length>}` 引用替代；实际字符串内容写入 `[block <pointer> bytes=<n>]` ... `[endblock <pointer>]` section。

renderer config 是仓库内提交的代码契约，不通过用户配置、项目配置、环境变量或 CLI flag 控制。声明：

| View Kind | Block Pointers |
| --- | --- |
| `outline` | 无 block |
| `read` | `/content` |
| `find` | 无 block |
| `info` | 无 block |
| `error` | `/error` |

`readable-view` framing 在所有平台使用 LF byte `0x0A`；header 以 LF 结束，存在 block 时 header 结束 LF 后有一个空 separator LF。block marker 行以 LF 结束；正文不含尾部换行时，renderer 在 block marker 前插入不属于 payload 的 framing LF。正文中的 marker 字样（`[block ...]` 等）不改变以 byte length 定界的 block 边界。

renderer 在写 stdout 前完成内存渲染。block pointer 缺失、目标值非字符串、pointer 重复或 identity 冲突时，renderer 产生可投影为 `readable_view_render_failed` 的 fatal diagnostic。此时没有可写入的 valid readable-view payload，stdout 为空，stderr 包含诊断；进程退出码按 [CLI](cli.md#退出码) 映射。

`read` 的 `readable-view` 示例：

```text
{
  "ref": "H:L4:H2",
  "content": {
    "$block": "/content",
    "bytes": 38
  },
  "content_type": "text/markdown",
  "cost": "6 lines | 0.1 KB",
  "page": null
}

[block /content bytes=38]
## Guide > Install

Some install text.
[endblock /content]
```

## `readable-json`

用途：需要结构化阅读结果但不需要协议 envelope 的 AI 和人类辅助流程。输出不包含 `protocol_version`、`request_id`、`operation`、`ok` 或原始进程错误字段。

`readable-json` 是阅读输出中的结构化机器友好形态。它必须保持 documented shape，便于 AI、工具和轻量自动化解析阅读结果；但它不包含完整协议 envelope，也不替代 `protocol-json` 的完整机器接口。脚本若需要稳定错误 envelope、request id、raw item facts、结构化成本或协议校验，应使用 `protocol-json`。

阅读输出 schema 按 operation 独立定义，见 [JSON Schema 索引](schemas/json-schema.md)。

readable success output 只包含成功业务 payload 与该 output mode 拥有的结构。失败输出只投影 failed request 的一个 primary `DiagnosticRecord`，不承载 sibling error list。Future non-fatal operation notes 必须由 owning operation/output contract 建模为 explicit business fields 或 guidance。

readable read 保留 adapter 返回的 `content_type`，并把 adapter 返回的 `cost.measurements[]` 压缩为 `cost` 摘要字符串。outline/find 只保留 `ref` 和派生 `display`；需要 raw item facts、结构化成本、metadata 或完整 protocol error envelope 的调用方使用 `protocol-json`。

Readable error 保留 `code`、`owner`、必要 `details`、`guidance` 和可用的 `location`、`received`、`expected`，并使用精简 error 文本。需要机器可靠错误契约时使用完整协议输出。

## 阅读文案配置

已定义配置项不包含阅读文本模板、`readable-view` header 模板或任意可改写 readable 字段 shape 的模板。`readable-view` 的 renderer config 是仓库内代码契约，不受用户配置、项目配置、环境变量或 CLI flag 控制。

阅读文案配置如需扩展，必须把可配置项限制在提示文案、usage、guidance 或包装文案，不得改变 protocol-json 的稳定字段、字段类型和错误 code，也不得改写 readable-json 的 documented shape。

## 通道

- `readable-view` 成功或可投影失败输出写 stdout。
- `readable-json` 成功或失败输出写 stdout，且只输出一个 JSON 值。
- `protocol-json` 成功或失败输出写 stdout，且只输出一个 JSON 值。
- renderer 无法生成 valid readable-view payload 时，stdout 为空，诊断写 stderr。
