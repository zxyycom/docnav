# 输出模式

本文是 `docnav` 文档操作输出模式、readable rendering、诊断投影承载、阅读文案配置和输出通道的主规范。CLI 命令面见 [CLI](cli.md)；core 配置合并和运行时补全见 [导航配置](navigation-config.md)；原始协议 envelope 见 [原始协议](protocol.md)。

## 输出层边界

`--output` 只选择输出层的序列化、错误包装和通道承载方式，不改变 `docnav` 的 adapter 选择、配置合并、参数显式化、probe、adapter library dispatch 或业务结果判断。Document operation 只接受 `readable-view`、`readable-json` 和 `protocol-json`。实现应先产出成功结果或诊断错误记录，再按输出 context 渲染为 readable-view、readable JSON、protocol envelope、命令自有 JSON 或 PlainText。

`docnav-output` 是 document operation 输出编排和 primary failure projection owner：调用方传入 operation、request id、output mode、document outcome 和 primary `DiagnosticRecord` 或成功 payload，由该层决定 `readable-view`、`readable-json` 或 `protocol-json` 的包装、error 投影和 stdout/stderr 分流。共享 crate 边界见 [架构](architecture.md#共享库)；本文件定义文档输出模式、统一错误投影和通道契约。help、version、manifest、probe 和其它非 document 成功输出不承诺 document result shape，但致命诊断不应绕过统一错误投影。

机器可读输出必须优先保持稳定和可解析。若调用方选择 `protocol-json` 或 `readable-json`，stdout 必须只输出一个符合该模式 documented shape 的 JSON 值；错误发生在 CLI 参数解析、adapter 选择、adapter layer dispatch 或输出转换阶段时，只要输出模式可以从 argv 或请求中确定，也必须使用对应 JSON 错误形态。无法确定 operation 时，协议错误 envelope 使用 `operation: null`。

统一执行管线按 [适配器契约](adapter-contract.md#adapter-选择) 累积 automatic discovery 候选失败并写入错误通道；全部候选失败时，这些记录从属于 primary `DiagnosticRecord.details.candidate_failures`。后续候选成功时，候选失败保持 internal discovery state，不进入 public document output。

## `protocol-json`

用途：完整接口、脚本、调试和兼容性校验；不以可读性为目标。正常阅读不使用该模式。

```text
docnav outline docs/guide.md --output protocol-json
docnav read docs/guide.md --ref "<ref-from-outline>" --output protocol-json
```

文档操作输出完整原始协议 envelope。`adapter list`、doctor、manifest metadata 和 probe result 由各自 owner 定义输出 shape。

`docnav --output protocol-json` 由 core 先完成导航配置合并、adapter selection 和 native option enrichment，再由 `docnav-navigation` 生成非空 request id 并构造内部 protocol request；写入 `arguments` 的字段是 adapter layer 的显式 operation input。

`protocol-json` stdout 只承载 protocol response 或 failure envelope。若直接 CLI argv 中存在 unknown token、多余 positional 或 operation-inapplicable flag，stdout 输出 protocol failure envelope。若 automatic discovery 全部失败，stdout 输出包含 primary diagnostic 和 candidate failure list 的 protocol failure envelope。若后续候选成功，stdout 只输出成功 protocol response envelope。若参数解析失败但 argv 已能确定 `--output protocol-json`，stdout 仍输出 protocol failure envelope，而不是退回文本错误。

原始协议成功结果保留 adapter 返回的结构化事实，例如 outline/find item 的 `label`、`kind`、`location`、`summary`、`excerpt`、`cost`、`metadata`，read 的 `cost.measurements[]`，以及 info 的 `document`、`adapter`、`metadata`。这些字段是 `protocol-json` 的事实来源；`display` 和人类可读成本摘要属于阅读输出。

`docnav-json-io` 拥有低层 JSON serialization、newline writing 和 write failure plumbing；protocol envelope、诊断投影、错误归属和 exit code policy 仍由 document output 编排和调用方 surface 决定。

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

renderer 在写 stdout 前完成内存渲染。block pointer 缺失、目标值非字符串、pointer 重复或 identity 冲突时，renderer 产生可投影为 `readable_view_render_failed` 的 fatal diagnostic，stdout 为空，stderr 包含诊断，CLI 使用内部错误 exit code。

Markdown adapter read 的 `readable-view` 示例（省略了 entry style 格式化）：

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

`readable-json` 是阅读输出中的结构化机器友好形态。它必须保持 documented shape，便于 AI、工具和轻量自动化解析阅读结果；但它不包含完整协议 envelope，也不替代 `protocol-json` 的完整机器兼容接口。脚本若需要跨版本稳定错误 envelope、request id、raw item facts、结构化成本或协议兼容校验，应使用 `protocol-json`。

阅读输出 schema 按 operation 独立定义，见 [JSON Schema 索引](schemas/json-schema.md)。

readable success output 只包含成功业务 payload 与该 output mode 拥有的结构。Rejected argv、invalid config sources、explicit adapter failure、explicit ref failure 和 automatic discovery all-failed list 都通过 readable error projection 表达。后续成功的 discovery attempts 保持为 internal state。Future non-fatal operation notes 必须由 owning operation/output contract 建模为 explicit business fields 或 guidance。

readable read 保留 adapter 返回的 `content_type`，并把 adapter 返回的 `cost.measurements[]` 压缩为 `cost` 摘要字符串。outline/find 只保留 `ref` 和派生 `display`；需要 raw `kind`、`location`、`summary`、`excerpt`、`rank`、`cost` 或 `metadata` 的调用方使用 `protocol-json`。

阅读错误投影 failed request 的一个 primary `DiagnosticRecord`。Readable error 保留 `code`、`owner`、必要 `details`、`guidance` 和可用的 `location`、`received`、`expected`，并使用精简、可配置的 error 文本。需要机器可靠错误契约时使用完整协议输出。

## 阅读文案配置

已定义配置项不包含阅读文本模板、`readable-view` header 模板或任意可改写 readable 字段 shape 的模板。`readable-view` 的 renderer config（block 字段声明和 framing 规则）是仓库内代码契约，不受用户配置、项目配置、环境变量或 CLI flag 控制。

阅读文案配置如需扩展，必须把可配置项限制在提示文案、usage、guidance 或包装文案，不得改变 protocol-json 的稳定字段、字段类型和错误 code，也不得改写 readable-json 的 documented shape。

## 通道

- `readable-view` 和 `readable-json` 写 stdout。
- `protocol-json` 写 stdout，且只输出一个 JSON 值。
- 诊断记录可投影到 stderr。
- automatic discovery 候选记录只有在全部候选失败时从属于 primary failure details；后续候选成功时不进入 success stdout payload。
- 默认配置路径缺失不产生诊断；显式或 present invalid config source 产生 failure projection，并阻断 document operation。
- 导航配置的诊断交接见 [导航配置](navigation-config.md#错误出口)；通道承载必须与该规则一致。
- 非 document machine output 若复用低层 JSON writer，仍由各自 owner 决定 schema、plain text/stderr 边界和 exit behavior。
