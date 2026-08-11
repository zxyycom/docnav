# AGENTS.md

## 项目不变量

Docnav 是 CLI-first 的结构化文档导航系统。核心入口是 `docnav` CLI，所有调用入口共享同一契约，并通过有限、可继续的流程读取大型文档：

```text
outline -> ref -> read
```

当前稳定规范以 `docs/` owner 文档为准；活动决策保存已确认的跨 change 方向；被当前任务选中的 Change Plan 保存 change-local 计划；调查报告保存明确要求沉淀的形成时证据与认识快照；代码、测试和 release artifact 证明当前实现状态。Change Plan 的目录存在、status、stage、assessment 和 checkbox 都不表示当前优先级、实施授权或二进制能力。目标性 `MUST` / `SHALL` 只有在主规范明确标注 Current 或已实现时才表示当前二进制能力。

当前实现、规划和 adapter 能力边界，以 `docs/navigation.md` 指向的主规范和状态语义为准。涉及产品、协议、CLI、adapter、schema、示例或测试细节时，先按对应主规范判断。

长期决策、调查报告、Change Plan 与 owner 规范的项目级分工和交接顺序，以 `docs/navigation.md` 的“长期决策、调查报告与 Change Plan 分工”为准；各载体的触发/写入授权、结构和生命周期由对应项目 skill 拥有，项目规则不扩张这些授权。

## 架构边界

- `docnav` 负责格式识别、adapter 路由和管理、配置、项目初始化、默认参数解析、输出模式和错误映射。
- 格式 adapter 负责本格式识别、解析、导航策略、ref、分页结果和 adapter operation result。
- 输出分层保持独立：原始协议用于稳定校验，阅读输出用于信息密度；两者复用业务语义，但不复用传输包装。
- ref 由 adapter 生成和解析；`docnav` 和其他调用入口只原样传递。

## 工作方式

1. 需求不完整时，先从 owner 文档和当前实现证据恢复当前事实，再按 `docs/navigation.md` 读取适用的决策、Change Plan 或调查证据；这些载体都不自行补成当前需求或扩大授权。仍不能可靠判断时只问必要问题。
2. 用户给出实现方案但缺少目标、场景或成功标准，且会影响 public contract、CLI 行为、adapter、schema 或长期架构时，先区分用户目标、当前方案、可选方案和推荐方案。
3. 多个实现路径会影响兼容性、扩展性或维护成本时，先比较复杂度、风险和开发成本，再推荐一个。
4. 不为短期跑通引入长期难维护的临时方案；确需临时处理时，写明 TODO、原因、约束范围和移除条件。
5. 风险高且不能可靠推断时，只问必要问题。

## 上下文获取

1. 读取项目文档时，从 `docs/navigation.md` 的“如何阅读这些文档”进入，只读取当前任务角色匹配的主规范。
2. 探索仓库内产品规范、Change Plan、调查报告、历史设计文档或 Markdown 层级结构时，优先用：

   ```powershell
   bun --silent run dnm outline <path>
   bun --silent run dnm read <path> --ref "<ref>"
   ```

   短小配置、入口提示词和工具说明可直接读取；验证 Docnav 导航行为时除外。仓库命令不可运行时，回退到常规文件读取。
3. 新增、拆分或合并文档前，先明确 owner、读取时机和验证方式；能归入已有 owner 文档时，优先更新已有文档。
4. 用户明确要求创建、查询、推进、搁置、恢复、审阅或归档持久 Change Plan 时，先读取项目级 `change-plan` skill，再从 `changes/` 定位目标。工作复杂度只能触发创建提醒或建议，不能代替用户对新增持久 Change 的明确要求。`archive/legacy/openspec/` 只在审计切换前历史或用户明确要求时读取，其中未完成目录和 CLI 状态不属于当前计划。
5. 修改或验证字段、示例、schema、输出 shape 时，读取 `docs/schemas/` 和 `docs/examples/` 中对应材料。
6. 理解代码结构和调用关系时，优先使用当前可用的 CodeGraph 工具；结果不精确、索引陈旧或工具不可用时，再用有路径过滤的 `rg` / `rg --files` 补充。
7. 搜索时按任务过滤路径、扩展名和关键词，并排除 `.git`、`target`、`node_modules`、`.venv`、`dist`、`build` 和缓存目录。
8. 后续交互引用已读内容时，只提炼关键结论和文件位置；跟踪变化用局部搜索和 diff。

## 实现与验证

1. 涉及实现代码、重构、测试脚本、验证脚本或跨模块修改时，除对应主规范外，先读取 `docs/coding-style.md`；交付前按其中“变更前后自检”检查。
2. 涉及架构、协议、数据模型、CLI/API surface、adapter 边界、依赖或验证链路的实现前，简短说明技术判断依据、影响范围、可能受影响模块和验证方式。
3. 新增、修改、删除或审查测试前，按 `docs/testing.md`、
   对应行为 owner、`docs/testing/case-maintenance.md`、项目级
   `test-evidence-review` skill 的顺序恢复测试层级、当前契约、账本流程和通用评审
   方法；修改前先用项目 wrapper 证明完整当前树的静态实体、runner 实体和 Case
   映射闭合。Case 细节以 `case-maintenance.md` 为准。历史 Case、已归档 Change Plan、legacy OpenSpec 或旧
   账本只作审计与风险输入，不形成当前 coverage 缺口或当前任务的产品测试义务。
4. 涉及协议、schema、示例、CLI 或 adapter 时，同步更新对应主规范和验证材料。
5. 当实现与文档载体看似偏离时，先按 `docs/navigation.md` 分类当前 owner、长期方向、change-local 计划、形成时调查和历史材料，再在本次授权内按对应 skill 处理；不能确定正确载体或同步方向时让用户选择。
6. CLI 命令优先选择只读、可复现、范围明确的命令；验证命令按改动范围选择，避免无关全量操作。
7. 改动跨 Rust 行为、Change Plan、schema、示例、输出边界或多个包边界时，优先运行 `bun run verify:docnav-workspace`。
8. 纯提示词或说明文档改动，可用 `dnm outline`、局部 diff 等范围匹配的验证。
9. 新增 Node.js / TypeScript 脚本依赖时使用 `pnpm`；运行项目脚本或本地依赖提供的 CLI 时使用 `bun run`；Python 工具使用 `uv`；不默认使用全局 `npm` 或 `pip` 安装。
10. 修改后运行范围匹配的格式化、静态检查、schema、单元或集成验证；无法运行时在最终说明中写明。
11. 修改后用局部 diff 确认只改了目标范围。
