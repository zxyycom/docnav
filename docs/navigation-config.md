# 导航配置

本文定义 `docnav` document operation 的配置读取、校验、来源合并和运行时补全。读者应能从本文判断哪些配置字段合法、各来源如何覆盖、缺失或非法配置如何失败，以及 core 在 request construction 前交出哪些已补全值。

当前实现的主线是：`docnav` core 先发现项目上下文并加载 project/user config，再把显式 document command 值、project config、user config 和内置默认值合并为本次 document operation 的运行时配置。runtime 在选中内置 adapter 后补全 native options，并把完成后的 operation input 交给后续 request construction。

## 行为范围

导航配置拥有这些规则：

1. 配置来源：项目配置、用户配置、内置默认值，以及本次调用的显式 command 值。
2. 配置文件读取：默认缺失的 project/user config 表示空配置；存在但不可读、JSON 无效或 shape 非法时阻断 document operation。
3. 支持字段：`defaults.adapter`、`defaults.pagination.enabled`、`defaults.pagination.limit`、`defaults.output` 和已注册 native option config key。
4. 来源优先级：`explicit > project > user > built_in`。
5. 运行时补全：产出 adapter、pagination、page、limit、output 和 native options 的最终值与来源信息。

导航配置只解释可配置的 document operation 默认值。`path`、`ref` 和 `query` 是当前调用的 document identity input，不从配置取得。

## 配置文件

`docnav` 读取两个配置来源：

| 来源 | 路径 |
| --- | --- |
| Project config | `<project-root>/.docnav/docnav.json` |
| User config | 用户级 `docnav.json` |

默认路径缺失不是错误，等价于空配置。显式存在的配置文件必须满足这些条件：

1. 文件可读。
2. 内容是有效 JSON。
3. 顶层是 object。
4. 字段只使用本文支持的配置域。
5. 已注册的 core 字段满足基础校验。
6. `options.*` key 能在当前 core release 的内置 adapter registry 中找到 registered native option config key。

## 配置字段

Core 配置字段：

| 字段 | 含义 |
| --- | --- |
| `defaults.adapter` | 默认 declared adapter id。 |
| `defaults.pagination.enabled` | 默认分页状态。 |
| `defaults.pagination.limit` | 默认分页预算，必须是正整数。 |
| `defaults.output` | 默认输出模式。 |
| `options.*` | Adapter-owned native option raw value。 |

`defaults.limit` 不是合法字段；应使用 `defaults.pagination.limit`。未知顶层字段、未知 `defaults.*` 字段、未知 `defaults.pagination.*` 字段和未注册 `options.*` key 都是配置错误。

## 来源合并

Document operation 的运行时配置按固定优先级合并：

```text
explicit command value > project config > user config > built-in default
```

显式 command 值由调用入口提供，例如 `--adapter`、`--pagination`、`--limit`、`--page` 和 `--output`。本文只规定这些值进入运行时配置后的优先级。

内置默认值：

| 值 | 默认 |
| --- | --- |
| `defaults.pagination.enabled` | `true` |
| `defaults.pagination.limit` | `6000` |
| `page` | `1` |
| `defaults.output` | `readable-view` |
| `defaults.adapter` | unset |

`page` 是运行时调用位置，不是配置字段；配置文件不能设置 `defaults.page`。

## 运行时补全

Core 合并配置后，为 document operation 产出这些运行时值：

| 值 | 来源规则 |
| --- | --- |
| `adapter` | explicit/project/user/unset。 |
| `pagination.enabled` | explicit/project/user/built-in。 |
| `limit` | explicit/project/user/built-in；pagination disabled finalization 后可能变为最大正整数预算。 |
| `page` | explicit 或 built-in `1`。 |
| `output` | explicit/project/user/built-in。 |
| `options` | runtime 在 adapter selection 附近单独补全。 |

当最终 `pagination.enabled` 为 `false` 时，core 在 request construction 前把 operation input 的 `limit` 归一为最大正整数预算。该 finalization 不改写配置文件，也不把配置值重新标记为显式输入。

## Native Options

Native options 是 adapter-owned 配置值。Core 只负责来源合并和 selected-adapter projection：

1. 从当前 core release 的内置 adapter registry 读取 native option specs。
2. 按 `explicit > project > user` 查找 CLI native option input 和 config `options.*` raw value。
3. 保留 `identity`、`owner`、`namespace`、`key`、`source`、`type_variant` 和 raw value。
4. Adapter selection 后，只把 selected adapter 支持的 entries 传给 adapter handler。
5. 不属于 selected adapter 的 option 是 native option unsupported error。

Core 对 native option 的处理到 selected support projection 为止；通过 projection 的 raw value 原样进入 operation input。

## 补全结果

导航配置完成后，core 交出的 operation input 包含 operation、document path、ref/query、page、limit 和 options。配置来源只影响 adapter、pagination、limit、output 和 native options。

配置或 built-in default 补足的值只进入新的 operation input；不回写原始 CLI argv 或配置文件。

## 错误出口

| 位置 | 结果 |
| --- | --- |
| 配置源 | 存在但不可读、JSON 无效或顶层非 object 时，返回配置错误。 |
| 配置字段 | 未知字段、旧字段名或未注册 native option key 返回配置错误。 |
| 配置值 | `defaults.pagination.limit` 非正整数、`defaults.output` 非法或 adapter id 为空时返回输入错误。 |
| Native option projection | 已合并 option 不属于 selected adapter 时返回 native option unsupported error。 |

## 维护注意事项

维护导航配置时，重点保持这些不变量：

1. 配置文件只表达 document operation 默认值和 adapter-owned native option raw values。
2. `path`、`ref`、`query` 和 `page` 不进入配置文件字段集合。
3. Project config 优先于 user config；显式 command 值优先于所有配置。
4. Missing default config 是空配置；present invalid config 是 blocking error。
5. Native option enrichment 不回写配置文件或已解析的 core defaults。
6. 下游调用只消费补全后的值，不重新执行配置读取或来源合并。
