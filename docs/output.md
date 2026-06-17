# 输出模式

本文是 `docnav` 文档操作输出模式、readable rendering、warning 承载、阅读文案配置和输出通道的主规范。CLI 命令面和 argv 兼容规则见 [CLI](cli.md)；原始协议 envelope 见 [原始协议](protocol.md)；MCP tool handoff 见 [MCP Handoff](mcp.md)。

## 输出层边界

`--output` 只选择输出层的序列化、错误包装和通道承载方式，不改变 `docnav` 的 adapter 选择、配置合并、参数显式化、probe、invoke 或业务结果判断。Document operation 当前只接受 `readable-view`、`readable-json` 和 `protocol-json`。实现应先产出统一 outcome，再按输出模式渲染为 readable-view、readable JSON 或 protocol envelope。

机器可读输出必须优先保持稳定和可解析。若调用方选择 `protocol-json` 或 `readable-json`，stdout 必须只输出一个符合该模式 documented shape 的 JSON 值；错误发生在 CLI 参数解析、adapter 选择、adapter invoke 或输出转换阶段时，只要输出模式可以从 argv 或请求中确定，也必须使用对应 JSON 错误形态。无法确定 operation 时，协议错误 envelope 使用 `operation: null`。

统一执行管线按 [架构](architecture.md#adapter-选择) 累积可恢复候选失败；本文件定义这些候选证据在各输出模式中如何承载为 warning。

## `protocol-json`

用途：完整接口、脚本、调试和兼容性校验；不以可读性为目标。正常阅读不使用该模式。

```text
docnav outline docs/guide.md --output protocol-json
docnav read docs/guide.md --ref "<ref-from-outline>" --output protocol-json
adapter invoke
adapter outline docs/guide.md --output protocol-json
```

文档操作输出完整原始协议 envelope。`manifest` 和 `probe` 输出其专属协议 schema。

`docnav --output protocol-json` 由核心 CLI 生成非空 request id，按当前协议 schema 和字段 shape 解析最终有限参数，再调用 adapter `invoke`。

`protocol-json` stdout 不承载直接 CLI 兼容性 warning 或 adapter 选择候选 warning。若直接 CLI argv 中存在被兼容忽略的 token，或 adapter 选择过程中跳过了不可用、契约不匹配、probe 不支持的候选，warning 写入 stderr，stdout 仍只输出一个符合 protocol response schema 的 JSON envelope。若参数解析失败但 argv 已能确定 `--output protocol-json`，stdout 仍输出 protocol failure envelope，而不是退回文本错误。

## `readable-view`

用途：文档操作的默认输出模式。人类和 AI 直接阅读，信息密度高，开箱即可定位内容。输出由一个 pretty JSON header 和零个或多个 length-delimited block section 组成。调用方和测试通过字段名和值、block pointer 和 UTF-8 byte length 判断语义；JSON header object key 顺序和多个 block section 的输出顺序不作为稳定契约。

header 始终包含操作语义字段（ref、display、content_type、cost、page、capabilities 等）和可选 `warnings` 数组。renderer config 声明为 block 的字符串字段（例如 read 的 `/content`、readable error 的 `/error`）在 header 中以 `{"$block": "<pointer>", "bytes": <utf8-byte-length>}` 引用替代；实际字符串内容写入 `[block <pointer> bytes=<n>]` ... `[endblock <pointer>]` section。

renderer config 是仓库内提交的代码契约，不通过用户配置、项目配置、环境变量或 CLI flag 控制。当前声明：

| View Kind | Block Pointers |
| --- | --- |
| `outline` | 无 block |
| `read` | `/content` |
| `find` | 无 block |
| `info` | 无 block |
| `error` | `/error` |
| `warning` | 无 block |

`readable-view` framing 在所有平台使用 LF byte `0x0A`；header 以 LF 结束，存在 block 时 header 结束 LF 后有一个空 separator LF。block marker 行以 LF 结束；正文不含尾部换行时，renderer 在 block marker 前插入不属于 payload 的 framing LF。正文中的 marker 字样（`[block ...]` 等）不改变以 byte length 定界的 block 边界。

renderer 在写 stdout 前完成内存渲染。block pointer 缺失、目标值非字符串、pointer 重复或 identity 冲突时，renderer 返回 `readable_view_render_failed` 错误 id，stdout 为空，stderr 包含诊断，CLI 使用内部错误 exit code。

Markdown adapter read 的 `readable-view` 示例（省略了 entry style 格式化）：

```text
{
  "ref": "H:L4:H2:I2",
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

`readable-json` 仍属于阅读输出层中的结构化机器友好形态。它必须保持 documented shape，便于 AI、工具和轻量自动化解析阅读结果；但它不包含完整协议 envelope，也不替代 `protocol-json` 或 `adapter invoke` 的完整机器兼容接口。脚本若需要跨版本稳定错误 envelope、request id 或协议兼容校验，应使用 `protocol-json` 或 `adapter invoke`。

阅读输出 schema 按 operation 独立定义，见 [JSON Schema 索引](schemas/json-schema.md)。

成功结果存在直接 CLI 兼容性 warning 时，`readable-json` 必须在顶层输出 `warnings` 数组；没有 warning 时省略该字段。每个 warning item 必须使用稳定 warning envelope：`id`、非空 `reason`、稳定 `effect` 和 `details` 对象。CLI argv warning 使用 `id: "cli_argv_ignored"`，相关 argv token 只能作为 `details.tokens` 等 family-specific detail 表达。CLI argv warning 的 exact token 分组、`reason` 文案和 token 消费顺序不作为稳定契约。

成功结果存在 adapter 选择候选 warning 时，`readable-json` 同样必须在顶层 `warnings` 数组中保留。adapter candidate warning 使用 `id: "adapter_candidate_failure"`，`effect: "candidate_skipped"`，并在 `details` 中保留 `adapter_id`、`stage`、`code` 和可选 `preselected`。没有 warning 时省略该字段。

readable read 保留 adapter 返回的 `content_type`。如果调用方提供 `--adapter <adapter-id>` 或 MCP adapter 参数，`docnav` 先校验该 adapter；失败后再进入 registry 遍历。预选 adapter 失败不直接中断阅读链路，而是作为候选 warning 保留。

阅读错误保留 `code` 和必要 `details` 以便保持阅读语义清晰，同时使用精简、可配置的 error 与 guidance 文本。需要机器可靠错误契约时使用完整协议输出。

## 阅读文案配置

当前已实现配置不包含阅读文本模板、`readable-view` header 模板、MCP TextContent 包装模板或任意可改写 readable 字段 shape 的模板。`readable-view` 的 renderer config（block 字段声明和 framing 规则）是仓库内代码契约，不受用户配置、项目配置、环境变量或 CLI flag 控制。

后续 owner change 如需增加阅读文案配置，必须把可配置项限制在提示文案、usage、guidance 或包装文案，不得改变 protocol-json 的稳定字段、字段类型和错误 code，也不得改写 readable-json 或 MCP structuredContent 的 documented shape。

## 通道

- `readable-view` 和 `readable-json` 写 stdout。
- `protocol-json` 写 stdout，且只输出一个 JSON 值。
- 诊断写 stderr。
- adapter 选择候选 warning 在 `readable-view`、`readable-json` 和 MCP 中跟随最终阅读结果输出；在 `protocol-json` 中写 stderr，不能污染 stdout envelope。
- 直接 CLI argv 的兼容和 warning 归属见 [直接 CLI 兼容参数规则](cli.md#直接-cli-兼容参数规则)；通道承载必须与该规则一致。
