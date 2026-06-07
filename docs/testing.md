# 测试策略

本文将“原始协议保证机器接口稳定；阅读输出保证信息密度和可读性”映射到自动化验证。

## 测试层级

| 层级 | 核心目标 |
| --- | --- |
| schema | 原始协议、manifest、probe 和各 operation readable 输出分别通过独立 schema；readable schema 用于示例和工具输出校验，不作为完整机器协议 |
| 单元 | parser、扁平 outline、ref、有限默认值、page |
| 集成 | `docnav` 配置优先级、adapter 选择、adapter 管理、invoke 单请求、三种输出模式、进程通道 |
| 端到端 | 直接 CLI、MCP bridge、紧凑协议映射、精简输出和分页 |

## 脚本与工具依赖

验证脚本和临时工具依赖按生态选择包管理器：

- Node.js / JavaScript 使用 `pnpm`，例如 schema 校验工具和 MCP bridge 相关脚本。
- Python 使用 `uv`，例如文档 fixture 生成、批量 JSON 检查或辅助审计脚本。
- 不要求依赖预先全局安装；脚本应通过项目命令或临时工具执行保持可复现。

## 统一验证入口

常规交付前使用 Docnav workspace 综合验证入口：

```bash
pnpm run verify:docnav-workspace
```

该入口一次性覆盖常用门禁类型：Rust 格式化、生成物一致性、文档/schema/示例校验、Rust 静态检查、workspace 测试、OpenSpec 严格校验和 diff 空白检查。具体子命令和输出忽略规则由 `scripts/verify-docnav-workspace.mjs` 的 `checks` 配置维护，避免在文档中复制可执行命令清单。

终端默认透出各子命令输出，保留命令自身报告的状态信息；脚本只过滤已配置的无行动价值输出，例如 Git 的 CRLF 换行提示。输出忽略规则必须按子命令配置，避免全局吞掉真实失败或状态信息。某个检查失败时，脚本记录该检查并继续运行后续检查；全部检查结束后统一汇总失败项、每个失败命令的完整未过滤输出、通过项和日志提示。

局部改动仍可先运行范围更小的命令；但最终交付跨 Rust、文档、OpenSpec、schema、示例或输出层边界时，应运行 `pnpm run verify:docnav-workspace`。若需要完整命令输出，按终端提示查看，或手动重跑对应子命令。

## 必须验证的架构边界

- invoke protocol envelope 不出现在 MCP structuredContent 或 readable JSON。
- MCP 和 CLI 阅读输出不复制完整原始协议 JSON。
- protocol 响应 envelope 包含 operation；成功响应的 operation 与 result 类型必须由 schema 绑定。
- outline readable 结果只包含 entries 与 page；每条 entry 包含 ref 和 display。
- find readable 结果只包含 matches 与 page；每条 match 包含 ref 和 display。
- read readable 结果保留 ref、content、content_type、cost 和 page。
- `docnav` 根据 path 选择 adapter，并原样传递 ref。
- `docnav` 的 adapter 选择顺序为确定一个预选 adapter、校验预选 adapter、预选失败后按 registry 顺序 probe 并返回第一个成功项；配置预选优先于 core 推断。
- `docnav` 的候选失败证据必须包含 adapter_id、stage、code、reason 和 details，并覆盖字段不对齐、probe 不支持和进程失败。
- `docnav adapter install/update/remove/list` 是正式管理流程，安装或更新必须校验 manifest schema 和当前协议字段 shape。
- `docnav adapter install` 首期只接受 GitHub 链接和本地可执行文件；本地可执行文件安装、运行前健康检查和更新必须验证 SHA-256 hash。
- `docnav` 保留 adapter 生成的 ref、display、内容、content_type、成本和 page。
- 每个 CLI 只读取自身配置域。
- 配置优先级固定且最终 invoke 参数显式完整。
- 配置可以改变本 CLI 的阅读文本模板和 guidance；完整协议字段和错误 code 保持稳定，readable 输出保持 documented shape 以服务阅读展示和工具声明。
- `docnav-mcp` 通过 Node.js / JavaScript bridge 直接调用核心 `docnav` CLI，不拥有文档解析、adapter 管理、项目初始化、核心配置或 adapter 路由职责。

## Markdown 最低测试

- outline 是扁平条目，ref 和 display 可直接阅读。
- 默认 outline 每页最多 6000 字符，且 Markdown 默认只展示 H1-H3。
- 默认 read 每页最多 6000 字符。
- 默认 find 每页最多 6000 字符。
- page 从 1 开始；有更多信息时返回请求 page 加 1，否则返回 null。
- 使用相同语义参数和响应中的 page 可继续读取。
- 请求超过末尾时返回空结果和 null。
- 配置不能改变初始 page；入口省略 page 时 invoke 请求显式使用 `1`。
- 分页按 Unicode 字符预算验证，不按行数验证；outline/find 的超长 display 必须压缩到预算内，ref 保持完整。若完整 ref 本身超过预算，单条记录可超出预算但必须消耗该记录并让分页前进。
- 代码围栏伪 heading 和 frontmatter 不进入 outline。
- 重复 heading 和重复路径仍生成唯一 ref。
- ref 无匹配和多匹配分别返回稳定错误。

## 输出测试

| 入口 | 最低要求 |
| --- | --- |
| `adapter invoke` | 完整 protocol envelope、显式参数、stdout 单响应 |
| `docnav --output protocol-json` | 与 invoke 使用相同原始协议 schema |
| `docnav` 默认输出 | 紧凑可读、包含 page 状态、不含 envelope |
| `docnav --output readable-json` | 通过 operation readable schema、不含 envelope，仍属于阅读输出 |
| MCP TextContent | 精简阅读文本和 page 状态 |
| MCP structuredContent | 通过 operation readable schema、不含 envelope，用于工具展示和客户端消费，不替代完整协议接口 |

request/response fixture 或集成测试必须验证请求 operation 与响应 operation 一致；无法解析 operation 的失败响应使用 `operation: null`。

## 每个 Capability 的最低要求

| capability | `docnav` CLI | adapter invoke | MCP bridge |
| --- | --- | --- | --- |
| `outline` | 默认/`readable-json` 扁平可读；`protocol-json` envelope；字符预算和 page | 显式 page/limit_chars/options、扁平 entries | 调用 `docnav` 并返回精简 entries 和 page |
| `read` | path/ref 原样输入、有限内容、content_type 和 page | 显式 ref/page/limit_chars、唯一定位、ref 错误 | 调用 `docnav` 并返回精简内容、content_type 和 page |
| `find` | query、有限匹配和 page | 显式 query/page/limit_chars、ref/display matches | 调用 `docnav` 并返回精简匹配和 page |
| `info` | 格式原生可读摘要 | 紧凑 display/capabilities | 调用 `docnav` 并返回精简摘要 |
| `manifest` | 发现 adapter 身份、格式 id、扩展名、content type、capabilities 和当前 manifest 字段 shape | 不通过 invoke | 不拥有该能力 |
| `probe` | 获取格式支持度和候选判断依据 | 不通过 invoke | 不拥有该能力 |
| `adapter install/update/remove/list` | 正式安装、更新、移除和列出 adapter；支持 GitHub 链接和本地可执行文件；校验 manifest、当前协议字段 shape 和本地 exe hash | 不通过 invoke | 不拥有该能力 |

## 端到端验收

1. `docnav outline` 根据 `--adapter` 或 core 简易推断确定预选 adapter，预选失败后遍历 registry 选择第一个 probe 成功的 adapter。
2. `docnav` 将最终 page、limit_chars 和调用方显式 options 写入 invoke 请求，且不从 manifest 生成格式专属 options。
3. adapter 返回带 operation 的 protocol envelope、扁平 entries 和 page。
4. `docnav` 保留 operation 与 entries，并映射为默认文本、`readable-json` 或 `protocol-json`。
5. 从 outline 取得 ref 并原样调用 `docnav read`。
6. read 继续按 path 选择 adapter，并由 adapter 解析 ref。
7. page 非 null 时，使用该 page 继续读取。
8. `docnav-mcp` 将 MCP tool call 直接映射为核心 `docnav` CLI 调用，并把 readable 结果转为 TextContent 和 structuredContent。
9. 同一业务结果在 protocol 与 readable 层语义一致，但包装、字段集合和兼容目标不同；只有 protocol 层作为机器稳定接口。

## 一致性审计

- [文档导航](navigation.md) 只作为入口导航、文档分层、规则 owner 和术语索引，不重复定义细则。
- [架构](architecture.md) 独占职责、配置域和双层原则。
- [原始协议](protocol.md) 独占 invoke envelope、紧凑结果、错误和 page。
- [Ref](refs.md) 独占 ref 语义。
- [CLI](cli.md) 独占 `docnav` 命令、输出模式与入口转换。
- [Schema](schemas/README.md) 分别校验 protocol 和 readable 输出，不定义新的业务语义。
- [示例](examples/README.md) 只验证端到端链路和输出映射，不成为新的规范来源。
- OpenSpec change 只作为变更依据、验收和审计历史，不作为日常实现主入口。
