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

普通 CLI、readable JSON 和 invoke 复用业务逻辑，但不复用输出包装或展示形态。默认文本和 `readable-json` 以阅读为主；`invoke` 和 `protocol-json` 属于完整协议接口，不以可读性为目标。
文档操作的直接 CLI 支持默认文本、`readable-json` 和 `protocol-json` 输出；`manifest`、`probe` 和 `protocol-json` 输出各自专属机器 schema。
适配器可复用 SDK 的直接 CLI 基础能力完成通用命令分发、`<path>`、`--page`、`--limit-chars`、`--ref`、`--query`、`--output` 解析、protocol request 构造、输出分流和稳定错误映射。格式 adapter 只声明格式原生 CLI flag 到 protocol `options` 的映射，并保留这些 options 的业务语义、ref 策略和文本展示。

适配器直接 CLI argv 必须复用 Docnav 兼容参数规则：未知 flag、多余 positional 和当前 operation 不使用的已知 flag 生成 warning 后忽略；已知必需参数缺失、已知 flag 缺少值或值非法仍失败。warning item 必须包含原始 `ignored_tokens`、`kind` 和 `reason`。未知 flag 不消费后续 token，`--unknown=value` 作为一个 token 忽略，`--unknown value` 中的 `value` 继续按普通 token 处理。需要值的已知 flag 消费紧跟 token，即使该 token 以 `--` 开头。

warning 的承载由输出层决定：text 在正常阅读文本后拼接 warning；`readable-json` 可增加顶层 `warnings` 数组；文档操作 `protocol-json`、`manifest` 和 `probe` stdout 不增加 `warnings` 字段，CLI warning 写 stderr。

## 适配器职责

- 使用成熟 parser 解析格式。
- 生成扁平 outline 和可读 ref。
- 定义格式原生导航参数、直接 CLI 默认值与展示文本。
- 返回有限结果和下一页 page。
- 解析 ref 的格式定位部分并唯一读取。
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
Markdown find 返回的 match ref 必须与当前导航粒度一致，并可被 read 原样消费；没有局部导航区域时，可以返回 adapter 定义的全文 ref。`max_heading_level` 等格式原生 options 只影响 adapter 的导航粒度，具体归属策略由 Markdown adapter 自有契约定义。

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

直接 CLI 兼容参数规则不适用于 `invoke` stdin JSON。`invoke` 必须按 protocol request schema 严格校验请求，未知字段、缺少必需字段或参数类型错误不得被 warning 后忽略。

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

`docnav` 不在 adapter 选择阶段做协议版本协商。候选适配器的 manifest、probe 和 invoke 响应必须通过当前 schema、必需字段、字段类型、operation/result shape 和语义校验；字段缺失、字段类型不符或 shape 不对齐时记录候选失败证据，并继续 adapter 选择流程；没有候选可用时由 `docnav` 按 adapter 选择失败返回稳定错误。

正式 schema：

- [manifest.schema.json](schemas/manifest.schema.json)
- [probe-result.schema.json](schemas/probe-result.schema.json)
