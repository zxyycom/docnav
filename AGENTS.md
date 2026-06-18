# AGENTS.md

## 项目定位

Docnav 是 CLI-first 的结构化文档导航系统。核心入口是 `docnav` CLI，所有接入方式共享同一契约，通过有限、可继续的流程读取大型文档：

```text
outline -> ref -> read
```

当前实现、规划和 adapter 能力边界以 `docs/navigation.md` 指向的主规范为准；涉及产品、协议、CLI、adapter、schema 或测试细节时，按对应主规范判断。

## 架构边界摘要

- `docnav`：核心 CLI，负责格式识别、adapter 路由和管理、配置、项目初始化、默认参数解析、输出模式和错误映射。
- `docnav-mcp`：Node.js / JavaScript MCP bridge，职责是将 MCP tool call 映射到 `docnav` CLI；不得复制解析、adapter 路由或 ref 生成逻辑。
- 格式 adapter：负责本格式识别、解析、导航策略、ref、分页结果和 adapter 直接 CLI 输出。
- 共享协议：原始协议保证稳定校验；阅读输出保证信息密度。两层复用业务语义，但不复用传输包装。
- ref 由 adapter 生成和解析；`docnav`、MCP 和其它接入层只原样传递。

## 需求澄清与决策透明

- 当用户直接给出实现方案，但缺少目标、用户场景或成功标准，且改动会影响 public contract、CLI 行为、adapter、schema、MCP 映射或长期架构时，先区分用户目标、当前方案、可选方案和推荐方案。
- 能从现有文档、OpenSpec 或相邻实现可靠推断时，说明假设后继续；不能可靠推断且风险较高时，只问必要问题。
- 涉及架构、协议、数据模型、CLI/API surface、adapter 边界、MCP 映射、依赖或验证链路的实现前，简短说明技术判断依据、影响范围、可能受影响模块和验证方式。
- 当存在多个可行实现路径，且选择会影响兼容性、扩展性或维护成本时，比较方案复杂度、扩展性、风险和开发成本后再推荐一个。
- 不为短期跑通引入长期难维护的临时方案；确需临时处理时，必须标注 TODO、说明原因、约束范围和后续移除条件。

## 上下文获取规则

- 读取项目文档时，先从 `docs/navigation.md` 的“如何阅读这些文档”进入，只读取当前任务角色匹配的主规范。
- `openspec/changes/` 只在处理 OpenSpec change、审计历史、验收或用户明确要求时读取；涉及 OpenSpec 时先运行 `openspec list --json`。
- `docs/schemas/` 和 `docs/examples/` 是契约与校验材料；在实现、修改或验证字段、示例、schema、输出 shape 时读取。
- 探索仓库内产品规范、设计文档、OpenSpec 文档，或需要理解 Markdown 层级结构时，先运行 `pnpm --silent dnm outline <path>` 获取 ref，再用 `pnpm --silent dnm read <path> --ref "<ref>"` 按 ref 读取；短小配置、入口提示词或工具说明可直接读取，验证 Docnav 导航行为时除外。仓库命令不可运行时，回退到常规文件读取。
- 后续交互引用已读内容时只提炼关键结论和文件位置，不展开原文；跟踪变化用局部搜索和 diff。
- 优先使用 CodeGraph 理解代码结构：先用 `codegraph_search` 定位符号或文件，再用 `codegraph_node` 查看签名、位置和调用 trail。
- 调用链优先使用 `codegraph_callers`、`codegraph_callees` 和 `codegraph_impact`；需要跨符号路径且当前工具列表暴露 `codegraph_trace` 时，才使用 `codegraph_trace`。
- 需要源码时，先用 `codegraph_node` 且优先 `includeCode=false` 确认目标；确认后再用 `includeCode=true`，并用 `file` 或 `line` 消除同名歧义。
- CodeGraph 返回源码可视为已读；索引陈旧、工具未暴露或结果不够精确时，再用有路径过滤的 `rg` / `rg --files` 补充。
- 搜索按任务过滤路径、扩展名和关键词，并排除 `.git`、`target`、`node_modules`、`.venv`、`dist`、`build` 和缓存目录；避免无过滤遍历整个仓库。
- 大文件、长输出和目录列表先做摘要或筛选，再读取具体片段。

## 实现与验证规则

- 涉及实现代码、重构、测试脚本、验证脚本或跨模块修改时，除对应主规范外，必须先读取 `docs/CODING_STYLE.md`；交付前按其中“变更前后自检”检查本次改动。
- CLI 命令优先选择只读、可复现、范围明确的命令；验证命令按改动范围选择，避免无关全量操作。
- 改动跨 Rust 行为、OpenSpec、schema、示例、输出契约或多个包边界时，最终优先运行 `pnpm run verify:docnav-workspace`；纯提示词或说明文档改动可用 `dnm outline`、局部 diff 等范围匹配的验证。
- 新增或运行脚本依赖时，Node.js / JavaScript 使用 `pnpm`，Python 使用 `uv`；不默认使用全局 `npm` 或 `pip` 安装。
- 涉及协议、schema、示例、CLI、adapter 或 MCP 映射时，同步更新对应主规范和验证材料。
- 当实现与 docs、OpenSpec、schema 或 examples 发生偏离时，说明偏离点、原因和潜在影响；能确定正确方向时同步修正，不能确定时让用户选择更新代码、更新文档或记录偏差原因。
- 修改后运行与范围匹配的格式化、静态检查、schema、单元或集成验证；无法运行时在最终说明中写明。
- 修改后用局部 diff 确认只改了目标范围。
