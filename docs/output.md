# 输出模式

本文是 `docnav` 文档操作输出模式、readable rendering、primary diagnostic 投影、阅读文案配置和输出通道的主规范。CLI 命令面见 [CLI](cli.md)；navigation input resolution 见 [Navigation Input Resolution](navigation-input-resolution.md)；原始协议 envelope、result 和 error 字段见 [原始协议](protocol.md)。

## 输出层边界

`--output` 只选择已经形成的 document response 如何输出，不改变上游解析、dispatch 或业务结果判断。Document operation 只接受 public values `readable-view` 和 `protocol-json`；`readable-json` 不是 accepted value、alias 或 fallback。

Shared output 使用两个封闭路径：

```text
ProtocolResponse
  -> ProtocolJson
       -> protocol JSON
  -> Rendered(RenderStrategy)
       -> complete UTF-8 text
```

在执行任一 document output plan 前，成功或失败必须已经表示为一个不可变 `ProtocolResponse`。`ProtocolJson` 序列化该 response；`Rendered` 把同一个 response 原样传给其选定 renderer。`ProtocolResponse` 是 renderer 的完整输入 contract；它的 envelope、result、error、operation 和 request context 仍由 [原始协议](protocol.md) 拥有，本文件不定义第二套 outcome 或 context model。提前发生的 document failure 由 core 使用既有 protocol error projection 构造成 `ProtocolResponse::Failure` 后再进入选定 plan。

每个 `Rendered` plan 在输出编排开始前携带一个由 linked code 选择的 renderer function 或 trait value。Core CLI 对省略 output 或 `readable-view` 固定注入内置 renderer；直接使用 shared output API 的 linked caller 可以注入自定义 renderer，但 renderer identity 不进入 CLI、config 或 serialized contract。

Renderer contract 为：

```text
RenderStrategy(&ProtocolResponse) -> Result<String, RenderFailure>
```

Renderer 必须在第一次 stdout write 前返回一份完整 UTF-8 `String` 或 `RenderFailure`。`docnav-output` 负责执行 plan 和 document stdout/stderr：成功时把 renderer 返回值原样写出，不增加 wrapper、separator 或尾随换行；`RenderFailure` 时 stdout 保持为空，不调用第二个 renderer，也不 fallback 到其它 output path。Renderer 成功后的 writer failure 是独立 I/O failure，不得重新分类为 `RenderFailure`。

Document operation failure projection 由本文件与 [原始协议](protocol.md)、[CLI](cli.md) 等 surface owner 分别承担；`docnav-diagnostics` 继续只提供 diagnostic/error model helper primitives 和 record invariants。Output 与 renderer failure mapping 不新增或重命名 stable diagnostic code。

Runtime invocation log event 不是 document output，不由 `readable-view` 或 `protocol-json` 承载。启用 invocation logging 后，日志事件只能写入 [CLI](cli.md#invocation-logging) 解析出的独立 sink/path，不得作为 rendered text、protocol field、manifest/probe field 或 linked adapter handler payload 注入。

## `protocol-json`

用途：完整接口、脚本、调试和稳定接口校验；不以可读性为目标。正常阅读不使用该模式。

```text
docnav outline docs/guide.md --output protocol-json
docnav read docs/guide.md --ref "<ref-from-outline>" --output protocol-json
```

文档操作输出完整原始协议 envelope，字段语义按 [原始协议](protocol.md) 的 envelope、result 和 protocol error object。Public `protocol-json` mode 构造 `ProtocolJson`；该 plan 直接序列化传入的 `ProtocolResponse`，不调用 renderer，并把结果作为 stdout 的唯一 JSON 值。

`protocol-json` stdout 只承载 protocol response 或 failure envelope。只要 argv 或请求能确定 `--output protocol-json`，document operation 的 input、selection 或 dispatch failure 都先按既有 projection 形成 `ProtocolResponse::Failure`，再由 `ProtocolJson` 序列化，而不是退回文本错误。无法确定 operation 时，failure envelope 使用 `operation: null`。

成功 response 的 `result` 保留 adapter 返回的结构化事实。内置 `readable-view` renderer 需要的 `display`、成本摘要和精简字段从这些事实派生；原始协议不反向接受 presentation-only 字段。

当 navigation 返回包含 successful `auto_read` 的 composed outline/find response 时，`ProtocolJson` 直接序列化完整 outer response，并原样保留 protocol-owned `auto_read` object；outer operation 仍为 `outline` 或 `find`。Navigation 返回 base response 时，输出中不增加 sibling auto-read metadata。该 plan 不执行 read，也不构造 selection、skipped 或 failed facts。

## `readable-view`

用途：文档操作的默认输出模式。人类和 AI 直接阅读，信息密度高，开箱即可定位内容。省略 output 或选择 `readable-view` 时，core 构造携带内置 renderer 的 `Rendered`。内置 renderer 接收完整 `ProtocolResponse::Success` 或 `ProtocolResponse::Failure`，并返回一个由 pretty JSON header 和零个或多个 length-delimited block section 组成的完整文本。调用方和测试通过字段名和值、block pointer 和 UTF-8 byte length 判断语义；JSON header object key 顺序和多个 block section 的输出顺序不作为稳定契约。

成功 header 始终只包含阅读层操作字段（ref、display、content_type、cost、page 等）和该 operation 拥有的 success payload 字段。outline/find 的 `display` 由 response 中的 raw item facts 派生；read 的 `cost` 是由 `cost.measurements[]` 派生的人类可读摘要。renderer config 声明为 block 的字符串字段（例如 read 的 `/content`、readable error 的 `/error`）在 header 中以 `{"$block": "<pointer>", "bytes": <utf8-byte-length>}` 引用替代；实际字符串内容写入 `[block <pointer> bytes=<n>]` ... `[endblock <pointer>]` section。

内置 renderer 可以使用 private presentation helper 和 renderer config 派生最终文本；这些 helper value 不形成 public output shape。Renderer config 是仓库内提交的代码契约，不通过用户配置、项目配置、环境变量或 CLI flag 控制。声明：

| View Kind | Block Pointers |
| --- | --- |
| `outline` | `/auto_read/read/content`，仅在 successful `auto_read` 存在时 |
| `outline-unstructured` | `/content` |
| `read` | `/content` |
| `find` | `/auto_read/read/content`，仅在 successful `auto_read` 存在时 |
| `info` | 无 block |
| `error` | `/error` |

Structured outline 和 find 的 `readable-view` 始终保留既有 base header fields。Protocol response 包含 `auto_read` 时，renderer 在 header 中增加同名 object：保留 `reason`，并把 nested read 的 `ref`、`content_type`、`page` 和由 `cost.measurements[]` 派生的 readable cost summary 放入 `auto_read.read`；nested `content` 替换为 `{"$block": "/auto_read/read/content", "bytes": <utf8-byte-length>}`。Renderer 恰好输出一个同 pointer 的 length-delimited block，payload bytes 等于 protocol `auto_read.read.content`。

Protocol response 不包含 `auto_read` 时，structured outline/find 使用原有 base projection，不增加 auto-read header field 或 block。Navigation pre-dispatch 产出的 unstructured outline result 仍由内置 renderer 表示为 `kind: "unstructured"`、`reason`、`content_type`、稳定 `cost` facts 和 `content` 的 `{"$block": "/content", "bytes": <n>}` 引用；`/content` block payload 等于 response 中该 result 的 content。它是 content，不得被表示为 outline entries 或分页状态；header 不包含 entries、ref、page、continuation 或 `auto_read`。

Unstructured outline 的 `readable-view` 示例：

```text
{
  "kind": "unstructured",
  "reason": "path_rule",
  "content": {
    "$block": "/content",
    "bytes": 10
  },
  "content_type": "text/markdown",
  "cost": {
    "measurements": []
  }
}

[block /content bytes=10]
plain note
[endblock /content]
```

`readable-view` framing 在所有平台使用 LF byte `0x0A`；header 以 LF 结束，存在 block 时 header 结束 LF 后有一个空 separator LF。block marker 行以 LF 结束；正文不含尾部换行时，renderer 在 block marker 前插入不属于 payload 的 framing LF。正文中的 marker 字样（`[block ...]` 等）不改变以 byte length 定界的 block 边界。

内置 renderer 在写 stdout 前完成内存渲染。Block pointer（包括 `/auto_read/read/content`）缺失、目标值非字符串、pointer 重复或 identity 冲突时返回 `RenderFailure`；core 沿用现有 `readable_view_render_failed` output failure mapping。此时 stdout 为空，stderr 包含诊断，不调用第二个 renderer；进程退出码按 [CLI](cli.md#退出码) 映射。渲染成功后，output layer 把完整文本原样写入，不追加 framing 或换行。

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

## 阅读文案配置

已定义配置项不包含 renderer implementation id、阅读文本模板、`readable-view` header 模板或任意可改写 readable 字段 shape 的模板。Core CLI 的内置 renderer 与其 renderer config 是 linked code contract，不受用户配置、项目配置、环境变量或 CLI flag 控制。

直接调用 shared output API 的 linked code 可以为 `Rendered` 提供自定义 renderer；该 renderer 拥有自己的 presentation contract，但不会创建新的 public output value 或 serialized strategy id。阅读文案配置如需扩展，必须把可配置项限制在提示文案、usage、guidance 或包装文案，不得改变 `protocol-json` 的稳定字段、字段类型和错误 code。

## 通道

- `ProtocolJson` 成功序列化 `ProtocolResponse::Success` 或 `ProtocolResponse::Failure` 后写 stdout，且只输出一个 JSON 值。
- `Rendered` 在 renderer 成功后把返回的完整 UTF-8 text 原样写 stdout；内置 renderer 的成功或可投影 document failure 文本遵循 `readable-view` contract。
- Renderer 返回 `RenderFailure` 时 stdout 为空，诊断写 stderr，且不 fallback；renderer 成功后的 writer failure 保持独立 I/O failure。
- Runtime invocation log 写入独立 sink/path，不能写 stdout。日志写入失败诊断如需可见，必须是 bounded 且不得破坏 machine-readable stdout。
