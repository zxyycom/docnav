# 适配器契约

本文是格式适配器命令、默认值所有权、invoke、manifest 和 probe 的主规范。

## 命令

每个适配器提供：

```text
outline
read
find
info
invoke
manifest
probe
```

普通 CLI、readable JSON 和 schema-valid `invoke` request 在传输层解析成功后进入 canonical document operation input 或等价 semantic request，并复用业务逻辑；它们不复用输出包装或展示形态。`readable-view`（默认）和 `readable-json` 以阅读为主；`invoke` 和 `protocol-json` 属于完整协议接口，不以可读性为目标。
文档操作的直接 CLI 支持 `readable-view`（默认）、`readable-json` 和 `protocol-json` 输出；`manifest`、`probe` 和 `protocol-json` 输出各自专属机器 schema。
适配器可复用 SDK 的直接 CLI 基础能力完成通用命令分发、`<path>`、`--page`、`--limit-chars`、`--ref`、`--query`、`--output` 解析、protocol request 构造、输出分流和稳定错误映射。SDK 直接 CLI 使用 `clap` 或 `clap` builder API 承载命令、固定参数、默认值、枚举和 help；SDK 在确定 operation 后只校验当前 operation 实际使用的参数。格式 adapter 只声明格式原生 CLI flag 到 protocol `options` 的映射，并保留这些 options 的业务语义、ref 策略和 readable payload 字段语义。

适配器直接 CLI argv 必须复用 [CLI](cli.md#直接-cli-兼容参数规则) 定义的直接 CLI 兼容参数规则。

`manifest`、`probe` 和文档操作 `protocol-json` 的 stdout 仍使用本文件定义的专属机器 schema；存在 CLI warning 时按直接 CLI 规则写 stderr。`--help` 和子命令 help 只输出可纠错参数说明，不执行文档导航业务。

## 适配器职责

- 使用成熟 parser 解析格式。
- 生成扁平 outline 和可读 ref。
- 定义格式原生导航参数和直接 CLI 默认值。
- 返回有限结果和下一页 page。
- 按自身契约解析 ref 并读取，将非法 ref、无匹配 ref 等失败映射为稳定错误。
- 在 invoke 中返回紧凑原始协议结果。

adapter 直接提供本格式的 ref、display、内容、成本和 page，供 `docnav` 原样映射到阅读输出。

## Manifest

`manifest --output protocol-json` 返回稳定 manifest：

```text
manifest_version
adapter.id
adapter.name
adapter.version
formats[].id
formats[].extensions[]
formats[].content_types[]
capabilities[]
```

manifest 只声明 adapter 身份、支持格式、扩展名、content type 和 capabilities，不声明协议范围或格式默认参数。旧字段 `protocol.min`、`protocol.max` 和 `recommended_parameters` 必须被当前 manifest schema 拒绝。Markdown v0 adapter 必须声明并实现 `outline`、`read`、`find` 和 `info` 全部能力。

Markdown adapter 直接 CLI 使用 `limit_chars: 6000` 和 `max_heading_level: 3` 作为内置默认值，允许其项目级和用户级配置覆盖。
Markdown find 返回的 match ref 可按共享调用流程原样传给 read；没有局部导航区域时，可以返回 adapter 定义的全文 ref。find 的 ref 归属策略和 read 对该 ref 的接受与解释行为，由 [Markdown Adapter](adapters/markdown.md) 定义。`max_heading_level` 等格式原生 options 只影响 adapter 的导航粒度。

## Probe

probe 只识别格式，不执行导航。probe 输入只包含 path；`docnav` 在调用 probe 前解析 `--adapter` 或 core 推断得到的预选 adapter，但 adapter 选择提示不会作为 probe 参数传入。机器结果包含：

```text
probe_version
adapter_id
path
supported
format
confidence
reasons[]
```

每次判断至少包含一个 reason。不支持或内容不匹配时返回 `supported: false` 并给出 reason。`docnav` 必须以 probe 结果为准，不能只凭 adapter id、扩展名或 manifest 静默选中。

## Invoke

`invoke` 不读取适配器直接 CLI 配置，也不选择隐式默认参数。请求必须已包含调用方最终解析的有限参数。

直接 CLI 兼容参数规则不适用于 `invoke` stdin JSON。`invoke` 必须在进入 canonical document operation input 或等价 semantic request 前按 protocol request schema 严格校验请求；malformed JSON、未知字段、缺少必需字段或参数类型错误不得被 warning 后忽略。schema-valid `outline/read/find/info` request 必须与 direct CLI 文档操作共享语义归一、request 构造或统一 operation handler，不得维护第二套业务参数解释规则。

适配器必须：

- 校验 `protocol_version` 字段和当前请求 schema。
- 只处理一个请求。
- stdout 只返回原始协议 envelope。
- 为分页操作返回下一页页码，结束时返回 null。
- 按 `limit_chars` 字符预算分页；display 可压缩，ref 不得截断。
- 不输出 CLI 阅读文本或 MCP 结构。

## 默认值所有权

- 适配器直接 CLI 默认值属于适配器配置域。
- manifest 只声明 adapter 能力，不提供默认参数。
- `docnav` 按自身配置域决定 path、ref、query、page、limit_chars、output 和 adapter 等 core 通用参数。
- 格式原生 `options` 对 `docnav` 和接入层保持 opaque。
- adapter 直接 CLI 可以由 adapter 自有 flag 或配置生成 `arguments.options`，并在进入 invoke 前显式写入请求。
- page 不属于配置默认值；入口省略 page 时固定从 `1` 开始。

## 协议字段对齐

`docnav` 不在 adapter 选择阶段做协议版本协商。候选适配器的 manifest 和 probe 输出必须通过当前 schema、必需字段、字段类型和语义校验；字段缺失、字段类型不符、shape 不对齐、语义校验失败、进程不可用或 `supported: false` 时，`docnav` 必须能形成包含 adapter id、阶段和原因的候选失败证据。候选遍历策略由 [架构](architecture.md#adapter-选择) 定义；选择成功或全部候选失败后的输出映射由 `docnav` 输出层负责。

选定 adapter 后的 `invoke` 响应不再属于候选选择阶段。`invoke` 响应必须通过当前 protocol response schema、必需字段、字段类型、operation/result shape 和语义校验；校验失败时返回 adapter/protocol 稳定错误，不能把已经选定 adapter 的 invoke 失败当作普通候选失败继续静默切换。

正式 schema：

- [manifest.schema.json](schemas/manifest.schema.json)
- [probe-result.schema.json](schemas/probe-result.schema.json)
