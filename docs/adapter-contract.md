# 适配器契约

本文是格式适配器命令、默认值声明、invoke、manifest 和 probe 的主规范。

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
protocol.min
protocol.max
formats[].id
formats[].extensions[]
formats[].content_types[]
capabilities[]
recommended_parameters
```

`recommended_parameters` 由适配器拥有，用于让 `docnav` 在 invoke 前显式化格式原生参数。`docnav` 可以选择并原样传入。Markdown v0 adapter 必须声明并实现 `outline`、`read`、`find` 和 `info` 全部能力。

Markdown v0 manifest 推荐参数：

```json
{
  "outline": {
    "limit_chars": 6000,
    "options": {
      "max_heading_level": 3
    }
  },
  "read": {
    "limit_chars": 6000
  },
  "find": {
    "limit_chars": 6000,
    "options": {
      "max_heading_level": 3
    }
  }
}
```

适配器直接 CLI 使用相同数值作为内置默认值，允许其项目级和用户级配置覆盖。
Markdown find 返回的 match ref 必须与当前导航粒度一致，并可被 read 原样消费；没有局部导航区域时，可以返回 adapter 定义的全文 ref。`max_heading_level` 等格式原生 options 只影响 adapter 的导航粒度，具体归属策略由 Markdown adapter 自有契约定义。

## Probe

probe 只识别格式，不执行导航。机器结果包含：

```text
probe_version
adapter_id
path
supported
format
confidence
reasons[]
```

每次判断至少包含一个 reason。调用方提供显式格式或 content type 时，adapter 应先按该提示校验；不支持或内容不匹配时返回 `supported: false` 并给出 reason。格式歧义时 `docnav` 返回所有候选及判断依据。

## Invoke

`invoke` 不读取适配器直接 CLI 配置，也不选择隐式默认参数。请求必须已包含调用方最终解析的有限参数。

适配器必须：

- 校验原始协议版本和请求 schema。
- 只处理一个请求。
- stdout 只返回原始协议 envelope。
- 为分页操作返回下一页页码，结束时返回 null。
- 按 `limit_chars` 字符预算分页；display 可压缩，ref 不得截断。
- 不输出 CLI 阅读文本或 MCP 结构。

## 默认值所有权

- 适配器直接 CLI 默认值属于适配器配置域。
- manifest 推荐参数属于适配器声明，但不是 invoke 的隐式默认值。
- `docnav` 按自身配置域决定核心参数；未配置时可采用 manifest 推荐参数并显式传入。
- 格式原生 `options` 对 `docnav` 和接入层保持 opaque。
- page 不属于配置默认值；入口省略 page 时固定从 `1` 开始。

## 协议兼容

manifest 的协议范围是闭区间。`docnav` 与适配器没有版本交集时返回 `PROTOCOL_INCOMPATIBLE`。

正式 schema：

- [manifest.schema.json](schemas/manifest.schema.json)
- [probe-result.schema.json](schemas/probe-result.schema.json)
