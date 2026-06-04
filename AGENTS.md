# AGENTS.md

## 项目目标

Docnav 是一套面向 AI 和普通用户的结构化文档导航工具，使大型文档能够通过以下流程按需读取，避免调用方直接加载全文：

```text
outline -> selector -> read
```

首期支持 Markdown、JSON、YAML、TOML 和 INI。

## 架构原则

1. 使用统一 MCP 网关和独立格式适配器。
2. 最终提供 `docnav-mcp`、`docnav-markdown`、`docnav-json`、`docnav-yaml`、`docnav-toml`、`docnav-ini` 独立可执行制品。
3. 使用共享 library 定义统一协议和适配器通用能力。
4. 适配器能够独立安装、更新、调用和发布。
5. `docnav-mcp` 是可独立调用的 CLI，同时提供 stdio MCP 入口和安装、更新、诊断、配置等管理命令。
6. stdio MCP 模式由 AI Client 启动，并随会话结束而退出。
7. 每次工具调用按需启动对应适配器的 `invoke` 命令；适配器处理单个请求后立即退出。
8. 进程通信使用 stdin、stdout 和 stderr，不依赖端口、Socket、后台服务、WASM 或动态库。

调用链：

```text
AI Client <-> docnav-mcp stdio -> adapter invoke -> result
```

## 职责边界

### docnav-mcp

`docnav-mcp` 提供两类独立入口：

```text
stdio
init
doctor
version
adapter list
adapter install
adapter update
adapter remove
config
```

- `stdio` 启动 MCP stdio transport，面向 AI Client。
- 其他命令作为管理 CLI，面向用户、安装脚本和自动化工具。
- 暴露统一 MCP tools：`document_outline`、`document_read`、`document_find`、`document_info`。
- 根据项目配置、适配器 manifest、文件扩展名和 `probe` 结果选择适配器。
- 发现、安装、更新、移除、启动和调用适配器，并管理适配器版本。
- 校验协议版本与响应 schema。
- 初始化和管理 `.docnav/` 项目配置。
- 通过 `doctor` 检查运行环境、适配器可用性、版本和协议兼容性。
- 将适配器错误转换为统一 MCP 响应。
- 不解析具体文档内容。

### 格式适配器

每个适配器只负责对应格式的识别和解析，并提供统一命令：

```text
outline
read
find
info
invoke
manifest
probe
```

- 普通 CLI 面向用户、调试和脚本，支持人类可读输出和统一 JSON 输出。
- `invoke` 从 stdin 读取单个 JSON 请求，向 stdout 写入单个 JSON 响应，然后退出。
- `manifest` 返回适配器名称、版本、协议版本、扩展名和能力。
- `probe` 返回格式支持情况、识别置信度和判断依据。
- 普通 CLI 与 `invoke` 复用相同的解析逻辑、业务逻辑和数据 schema。

### 共享协议与 SDK

- `docnav-protocol` 定义请求、响应、selector、结果、错误、能力声明和协议版本。
- `docnav-adapter-sdk` 提供命令分发、invoke 输入输出、manifest、协议校验、错误转换和配置读取等通用能力。
- MCP 与适配器只要求协议兼容，不要求软件版本一致。

## 格式识别

`docnav-mcp` 按以下优先级选择适配器：

1. 使用 `.docnav/` 项目配置明确指定的适配器。
2. 根据文件扩展名匹配适配器 manifest。
3. 扩展名缺失或存在歧义时，调用候选适配器的 `probe`。
4. 无法唯一识别时返回候选信息和判断依据，由调用方决定。

## 协议与输出规则

1. `outline` 返回可直接交给 `read` 使用的统一 selector。
2. selector 对调用方隐藏不同格式的原生路径差异。
3. 稳定数据 schema 不随项目配置改变。
4. 请求携带 `protocol_version`。
5. guidance、usage 和错误建议允许通过 `.docnav/` 配置定制。
6. `docnav-mcp stdio` 的 stdout 仅输出 MCP 协议消息，诊断日志写入 stderr。
7. `docnav-mcp` 管理 CLI 默认输出人类可读结果，并通过 `--output json` 提供稳定的机器可读结果。
8. 适配器 `invoke` 的 stdout 仅输出单个机器可读 JSON 响应，诊断日志写入 stderr。
9. `invoke` 成功时返回退出码 0；失败时返回非零退出码，并尽可能输出结构化错误响应。

## 开发规则

1. 先定义并测试稳定协议，再实现或扩展适配器。
2. 优先完成 Markdown 适配器和完整的 `outline -> selector -> read` 调用链，再扩展其他格式。
3. 文档结构解析使用成熟解析库，不使用正则表达式模拟完整格式语法。
4. MCP 层保持格式无关；格式特有逻辑归属对应适配器。
5. 共享 SDK 只承载跨适配器一致且可复用的行为，格式特有能力保留在适配器内。
6. 新增或修改协议字段时，补充协议兼容性和 schema 测试。
7. 新增适配器能力时，同时覆盖普通 CLI、`invoke` 和 MCP 调用链测试。
8. 用户可见文案以中文为主，保留必要的命令、路径、参数和协议字段英文。

## 验收标准

1. 每个适配器均可独立执行普通 CLI 和 `invoke`。
2. `docnav-mcp stdio` 能通过 stdio transport 运行，并自动选择、调用正确适配器。
3. `docnav-mcp` 管理 CLI 能独立执行初始化、诊断、配置和适配器版本管理。
4. `outline` 返回的 selector 可直接用于 `read`。
5. `invoke` 每次只处理一个请求，并在响应后退出。
6. 所有成功与失败响应均通过统一 schema 校验。
7. 未知或歧义格式不会被静默猜测。
8. 适配器可以独立安装和更新，不影响其他格式。
9. 协议兼容、格式识别、selector、错误响应、管理 CLI 和 MCP 调用链具有自动化测试。

## 上下文读取与验证原则

- 先查看 `git status --short` 和 `git diff --stat`，再读取或修改文件。
- 涉及 OpenSpec 时，先运行 `openspec list --json` 查看当前变更状态。
- 优先使用 CodeGraph 理解代码结构；按符号和目标区域读取，避免无过滤遍历。
- 搜索时优先使用 `rg`，并排除 `.git`、`target`、`node_modules`、`.venv`、`dist`、`build` 和缓存目录。
- 长输出先摘要，只展开与当前任务相关的部分。
- 修改后运行与改动范围匹配的格式化、静态检查、单元测试和集成测试。
- 同一信息不重复展开；已读取内容后使用局部搜索和 diff 跟踪变化。

## CodeGraph MCP 使用规则

- 结构性问题先使用 `codegraph_context` 定位相关区域。
- 查找符号使用 `codegraph_search`，查看单个符号使用 `codegraph_node`，分析调用链使用 `codegraph_trace`。
- CodeGraph 返回的源码和关系可视为已读上下文；索引陈旧或缺失时再使用本地搜索补充。
- 修改代码后根据需要运行 `codegraph sync .` 同步索引。
