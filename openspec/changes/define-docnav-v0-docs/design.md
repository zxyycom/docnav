## Context

Docnav 尚无实现代码。v0 文档必须先划清 `docnav` 核心 CLI、adapter invoke 原始协议、普通 CLI 阅读输出和 MCP 阅读输出的边界，并以 Markdown 纵向链路验证这些边界。

## Goals / Non-Goals

**Goals:**

- 建立原始协议层和阅读输出层。
- 让 `docnav` 拥有识别、路由、adapter 管理、配置、项目初始化、默认参数解析、输出模式和错误映射。
- 让 `docnav adapter install/update/remove/list` 成为正式 adapter 管理能力，首期安装来源限定为 GitHub 链接和本地可执行文件，并对本地 exe 做 hash 校验。
- 让 Markdown adapter 拥有具体格式识别、解析、导航、ref、展示和分页。
- 让 MCP、skill、AGENTS.md / system prompt 成为共享 `docnav` 契约的接入方式。
- 通过有限默认值避免默认返回全部。
- 让后续实现可以从 schema、示例和测试矩阵直接派生。

**Non-Goals:**

- 不创建实现代码。
- 不统一不同格式的展示字段。
- 不在 v0 首期实现 JSON、YAML、TOML 或 INI adapter。

## Decisions

### 1. 原始协议与阅读输出分层

`adapter invoke` 和 `docnav --output protocol-json` 使用稳定 envelope；响应 envelope 包含 operation，使成功响应可直接绑定 result 类型。这一层是机器稳定接口，不以可读性为目标。`docnav` 默认输出、`readable-json` 和 MCP 使用精简阅读结果，优先服务 AI 和人类阅读；`readable-json` 只是结构化阅读形态，不作为长期机器解析接口。两层复用 ref、内容、content_type、成本和 page，包装、展示形态和兼容目标分层定义。

### 2. `docnav` 是 core CLI router/manager

`docnav` 负责项目根、核心配置、adapter 发现、安装、更新、移除、选择、invoke 启动、协议校验、输出模式和错误映射。首期安装来源为 GitHub 链接和本地可执行文件。安装和更新必须读取 manifest、校验 manifest schema 与协议兼容性，并记录可执行入口；本地可执行文件必须记录并验证 SHA-256 hash；更新失败时保留旧版本。它按显式格式校验、扩展名匹配校验、全量 adapter probe 的顺序选择格式，ref 与格式原生 options 原样传递给 adapter。

### 3. `docnav-mcp` 是 MCP 接入层

`docnav-mcp` 使用 Node.js / JavaScript 实现，通过 stdio 提供 MCP transport，暴露 `document_outline`、`document_read`、`document_find`、`document_info`，并把 tool call 直接映射为核心 `docnav` CLI 调用。它读取 MCP 接入层配置，输出 TextContent 和 structuredContent，并依赖系统中可调用的 `docnav`；adapter 路由和下级适配层调用只由 `docnav` 完成。

### 4. Outline 永远扁平

outline 返回按文档顺序排列的 `ref + display` entries。

### 5. Ref 是可读唯一引用

`path` 定位文档并用于 `docnav` 选择 adapter。ref 由 adapter 定义，只定位当前文档内部区域；`docnav` 和接入层原样传递 ref。

### 6. 默认结果有限且必须可继续

Markdown v0 内置默认值为 outline/read/find 每页 6000 字符，Markdown outline 默认只展示 H1-H3。分页操作请求传入正整数 `page` 和 `limit_chars`，响应返回请求 page 加 1 或 null。

### 7. 每个 CLI 拥有独立配置域

固定优先级为显式参数、项目级 CLI 配置、用户级 CLI 配置、内置默认值。调用方在启动 invoke 前将最终参数显式传入。配置可以调整本 CLI 的输出文本模板、guidance、usage 和错误建议；完整协议字段、字段类型和错误 code 保持稳定，readable 输出保持 documented shape。

### 8. v0 首期 Markdown-first

v0 首期实现范围为 Markdown adapter 的 `outline`、`read`、`find` 和 `info` 全部能力；`outline -> ref -> read` 是首要纵向阅读链路。JSON、YAML、TOML 和 INI adapter 保留为后续格式能力，待 Markdown 链路和协议实现稳定后再分别补充格式语义、示例和测试。

### 9. Schema 按输出层拆分

原始协议、manifest、probe 和各 operation readable 输出使用独立 Draft 2020-12 schema。protocol response schema 使用 operation 绑定成功 result 类型；MCP structuredContent 使用 readable schema，不包含 invoke envelope。readable schema 用于示例、工具声明和实现自测，不把阅读输出提升为完整机器协议。

### 10. 脚本依赖使用项目包管理器

后续将 schema、JSON 示例和 fixture 校验转为脚本时，Node.js / JavaScript 工具使用 `pnpm` 管理，Python 工具使用 `uv` 管理；不依赖全局 `npm` 或 `pip` 安装。

### 11. 显式格式和 content type 优先

CLI 或 MCP 可提供显式格式提示，例如 format id 或 content type。`docnav` 必须先按该提示找到候选 adapter 并执行校验；失败后再尝试扩展名匹配，仍失败时逐个 probe 已安装 adapter。read 的 protocol、readable JSON 和 MCP structuredContent 均保留 adapter 返回的 `content_type`。

### 12. README 是导航入口，不是规范副本

README 保留项目目标、v0 范围、术语、角色化阅读路径和规则 owner 索引；职责、协议、CLI、adapter、ref、测试等细则由对应主规范拥有。Schema 和示例只作为校验材料；OpenSpec change 只作为变更依据、验收和审计历史，不作为日常实现主入口。

## Risks / Trade-offs

- [ref 可读格式可能随 adapter 变化] → ref 完全归 adapter 所有，`docnav` 只按 path 和 adapter 证据选择格式。
- [双层输出增加映射测试] → 测试明确验证业务语义一致但包装不同。
- [有限默认值可能需要多次调用] → page 直接给出下一页，null 表示结束。

## Migration Plan

1. 完成并审计 v0 文档。
2. 提出协议、正式 adapter 管理与 Markdown 纵向链路实现 change。
3. 使用 `pnpm` 或 `uv` 管理脚本依赖，并将 schema 和 JSON 示例转为自动化 fixtures。
4. 后续分别提出 JSON、YAML、TOML 和 INI adapter 变更。
