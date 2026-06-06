# AGENTS.md

## 用途

本文件只约束 agent 在本仓库中的工作方式，不作为 Docnav 完整规范副本。产品、协议、CLI、adapter、schema 和测试细节从 `README.md` 的“如何阅读这些文档”进入，并按角色读取对应主规范。

## 项目快照

Docnav 是 CLI-first 的结构化文档导航系统。核心入口是 `docnav` CLI，所有接入方式共享同一契约，通过有限、可继续的流程读取大型文档：

```text
outline -> ref -> read
```

v0 首期聚焦 Markdown 纵向链路。JSON、YAML、TOML 和 INI 作为后续 adapter 能力推进。

## 文档读取规则

- 读取项目文档时，先从 `README.md` 的“如何阅读这些文档”进入，只读取当前任务角色匹配的主规范。
- `openspec/changes/` 只在处理 OpenSpec change、审计历史、验收或用户明确要求时读取；涉及 OpenSpec 时先运行 `openspec list --json`。
- `docs/schemas/` 和 `docs/examples/` 是校验材料，只在验证字段、示例、schema 或测试时读取。
- 已读取过的内容不要重复展开；后续使用局部搜索和 diff 跟踪变化。

## 架构边界摘要

- `docnav`：核心 CLI，负责格式识别、adapter 路由和管理、配置、项目初始化、默认参数解析、输出模式和错误映射。
- `docnav-mcp`：Node.js / JavaScript MCP bridge，通过 stdio 暴露 tools，只把 MCP tool call 映射到 `docnav`，不复制解析或路由逻辑。
- 格式 adapter：负责本格式识别、解析、导航策略、ref、分页结果和 adapter 直接 CLI 输出。
- 共享协议：原始协议保证稳定校验；阅读输出保证信息密度。两层复用业务语义，但不复用传输包装。
- ref 由 adapter 生成和解析；`docnav`、MCP 和其它接入层只原样传递。

## 工作规则

- 优先使用 CodeGraph 理解结构；索引缺失或不够时使用 `rg` / `rg --files` 补充。
- 搜索按任务过滤路径、扩展名和关键词，并排除 `.git`、`target`、`node_modules`、`.venv`、`dist`、`build` 和缓存目录；避免无过滤遍历整个仓库。
- 大文件、长输出和目录列表先做摘要或筛选，再读取具体片段。
- 文档读取按 README 角色路径进入；不要因为存在 OpenSpec、schema、examples 就默认全部读取。
- CLI 命令优先选择只读、可复现、范围明确的命令；验证命令按改动范围选择，避免无关全量操作。
- 跨 Rust、文档、OpenSpec、schema、示例或输出层边界的交付，最终优先运行 `pnpm run verify:docnav-workspace`；局部验证可先运行范围更小的命令。
- 新增或运行脚本依赖时，Node.js / JavaScript 使用 `pnpm`，Python 使用 `uv`；不默认使用全局 `npm` 或 `pip` 安装。
- 涉及协议、schema、示例、CLI、adapter 或 MCP 映射时，同步更新对应主规范和验证材料。
- 修改后运行与范围匹配的格式化、静态检查、schema、单元或集成验证；无法运行时在最终说明中写明。
- 修改后用局部 diff 确认只改了目标范围。

## CodeGraph MCP 使用规则

- 结构性问题先使用 `codegraph_context`。
- 查找符号使用 `codegraph_search`，查看单个符号使用 `codegraph_node`，调用链使用 `codegraph_trace`。
- CodeGraph 返回源码可视为已读；索引陈旧或缺失时再用本地搜索。
