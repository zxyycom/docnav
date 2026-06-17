# 覆盖矩阵

本文记录跨入口、命令族和 capability 的最低测试覆盖目标。它不定义稳定字段、错误码、命令语义或 schema shape；这些规则以 [文档导航](../navigation.md#规则所有权) 指向的 owner 文档为准。

## 架构边界

- 输出分层：protocol 输出通过原始协议 schema；readable-json、readable-view 和 MCP structuredContent 不携带 invoke envelope；warning 承载边界按 [输出模式](../output.md) 验收。
- 入口转换：直接 CLI 兼容参数、输出模式、stdout/stderr 分流和 warning 行为由 [CLI](../cli.md#直接-cli-兼容参数规则) 定义；测试覆盖成功路径、代表性失败和 strict 分界，不复制 token 消费细节。
- adapter 路由：`docnav` 根据配置、路径和 registry 确定 adapter；候选失败证据和选择顺序按 [架构](../architecture.md) 与 [适配器契约](../adapter-contract.md) 验收。
- ref 与分页：ref 由 adapter 生成和解析，core、CLI 和 MCP 只原样传递；分页继续读取按 [Ref](../refs.md) 和对应 adapter 规范验收。
- 配置边界：每个 CLI 只读取自身配置域，最终 invoke 参数显式完整；配置不得改变协议字段、错误 code 或 readable 输出 shape。
- adapter 管理：安装、更新、移除和列出 adapter 的流程按 CLI owner 和 adapter management 规格验收；测试覆盖 manifest 校验、fingerprint 边界和普通文档操作不重复安装期校验。
- MCP bridge：MCP 只映射到核心 `docnav` CLI，不拥有文档解析、adapter 管理、项目初始化、核心配置或 adapter 路由职责；交付验收按 [MCP Handoff](../mcp.md)、[输出模式](../output.md) 和 [原始协议](../protocol.md) owner 文档执行。

## 输出测试

输出测试只证明入口之间的包装边界和 schema 映射，不在本节定义字段全集。

| 入口 | 最低要求 |
| --- | --- |
| `adapter invoke` | 原始 protocol envelope、显式参数和 stdout 单响应 |
| `docnav --output protocol-json` | 与 invoke 使用同一原始协议 schema；warning 行为按 CLI owner 规则验收 |
| `docnav` 默认输出 / `readable-view` | 使用 readable-view renderer config；包含 page 状态和 warning 展示；不携带 protocol envelope |
| `docnav --output readable-json` | 通过 operation readable schema；不携带 protocol envelope |
| MCP TextContent / structuredContent | TextContent 保持精简阅读文本；structuredContent 通过 readable schema；不携带 protocol envelope |

request/response fixture 或集成测试必须验证请求 operation 与响应 operation 一致；无法解析 operation 的失败响应使用 `operation: null`。

## 命令族

| 命令族 | 覆盖目标 |
| --- | --- |
| Core document operations：`docnav outline/read/find/info` | help 可用；成功 readable/protocol 输出；代表性兼容参数；当前 operation 使用参数非法失败；warning stdout/stderr 边界 |
| Core non-document commands：`config/init/doctor/version` | 类型化命令成功和关键失败；不进入 adapter routing 或文档 invoke |
| Core adapter management：`docnav adapter list/install/update/remove` | 正式流程、manifest 校验、fingerprint 边界和错误映射；未交付能力只审计 owner，不作为验收通过项 |
| Adapter direct document operations：`docnav-markdown outline/read/find/info` | help 可用；direct CLI 成功；代表性兼容参数；实际使用参数非法失败；warning 承载边界 |
| Adapter direct machine commands：`manifest/probe/invoke` | manifest/probe schema stdout；invoke protocol envelope；malformed/unknown field/type 返回结构化失败 |
| Help commands | root help 和子命令 help 暴露关键参数、默认值或可选值；不读取文档、不选择 adapter、不启动 invoke |
| MCP bridge | tool call 映射为核心 `docnav` CLI；TextContent/structuredContent 不含 protocol envelope；structuredContent 通过 readable schema |

## Capability

Capability 测试按入口类型覆盖，不在本节重复字段 shape：

1. 文档能力（`outline`、`read`、`find`、`info`）：`docnav` CLI 覆盖默认 readable、readable-json 和 protocol-json；adapter invoke 覆盖显式参数和原始协议结果；MCP bridge 覆盖到 `docnav` 的映射和精简 readable 结果。
2. adapter 机器能力（`manifest`、`probe`）：直接 adapter 命令覆盖 schema stdout、warning stderr 和候选判断依据；不通过 invoke 或 MCP 暴露。
3. adapter 管理能力（`adapter install/update/remove/list`）：core CLI 覆盖正式流程、校验边界和错误映射；不通过 adapter invoke 或 MCP 暴露。

## 端到端链路

1. `docnav outline` 根据调用方输入、配置和 core 推断确定 adapter，并在预选失败后按 registry 策略继续选择。
2. `docnav` 将最终 page、limit_chars 和调用方显式 options 写入 invoke 请求，且不从 manifest 生成格式专属 options。
3. adapter 返回带 operation 的 protocol envelope、紧凑结果和 page。
4. `docnav` 保留 operation 与结果语义，并映射为默认 `readable-view`、`readable-json` 或 `protocol-json`。
5. 调用方从 outline 或 find 取得 ref，并原样调用 `docnav read`。
6. read 继续按 path 选择 adapter，并由 adapter 解析 ref。
7. page 非 null 时，使用该 page 继续读取。
8. `docnav-mcp` 的 tool call 映射、TextContent 和 structuredContent 包装按 [MCP Handoff](../mcp.md)、[输出模式](../output.md) 和 [JSON Schema 索引](../schemas/json-schema.md) 验收；当前端到端验收只把 MCP bridge 作为 handoff，不表示已交付 bridge E2E。
9. 同一业务结果在 protocol 与 readable 层语义一致，但包装、字段集合和兼容目标不同；只有 protocol 层作为机器稳定接口。
