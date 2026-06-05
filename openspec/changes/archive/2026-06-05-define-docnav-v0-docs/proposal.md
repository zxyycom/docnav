## Why

Docnav 需要在实现前明确 CLI-first 架构、稳定原始协议和高信息密度阅读输出的边界。`docnav` 核心 CLI 统一承担识别、路由、管理、配置和项目初始化；MCP、skill、AGENTS.md / system prompt 作为接入方式共享该契约。

## What Changes

- 定义“原始协议保证系统稳定；阅读输出保证信息密度”的架构原则。
- 明确 `docnav` 是 core CLI router/manager，负责 adapter 选择、默认参数解析、输出模式和错误映射。
- 明确 `docnav-mcp` 是 Node.js / JavaScript MCP bridge，负责 MCP stdio、tool 到核心 `docnav` CLI 的映射、TextContent 和 structuredContent。
- 使用扁平 outline 和可读 ref。
- 将原始协议响应定义为带 operation 的自描述 envelope。
- 定义有限且具体的默认参数，以及统一的 page 分页状态。
- 为原始协议和 readable/MCP 输出提供独立 schema 与示例。
- 拆分每个可执行 CLI 的配置所有权。
- 将 `docnav adapter install/update/remove/list` 定义为正式 adapter 管理能力，首期支持 GitHub 链接和本地可执行文件来源，并要求本地 exe hash 校验。
- 明确 adapter 选择顺序为显式格式校验、扩展名匹配校验、全量 probe。
- 在 read 的 readable/MCP 输出中保留 `content_type`。
- 明确 Markdown v0 首期实现 `outline`、`read`、`find` 和 `info` 全部能力；`outline -> ref -> read` 是首要纵向阅读链路。
- 收窄 README 为角色化阅读入口，明确主规范 owner、校验材料和 OpenSpec 历史边界。
- 本变更不创建实现代码。

## Capabilities

### New Capabilities

- `v0-contract-documentation`
- `markdown-reference-baseline`

## Impact

- 新增项目级规范文档、schema 和测试向量。
- 为后续 `docnav`、正式 adapter 管理、`docnav-protocol`、`docnav-adapter-sdk`、`docnav-markdown` 和 `docnav-mcp` 实现变更提供输入。
- JSON、YAML、TOML 和 INI adapter 作为后续格式能力另行提出变更。
