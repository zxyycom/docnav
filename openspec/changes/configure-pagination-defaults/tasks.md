本 change 目标是将分页默认值统一收敛到 `defaults.pagination`，并让 core `docnav` 和 adapter SDK direct CLI 使用同一 pagination 配置/argv 行为；本文档只是 `openspec/changes/configure-pagination-defaults/` 下的未审核临时 tasks，不影响现有其它文档或主规范。

## 1. 审计门禁

- [ ] 1.1 阻塞级审计：在执行任何实现任务前，审计 proposal、design、specs 和 tasks 是否都围绕“统一 pagination 默认配置，并让 core 与 SDK direct CLI 使用同一 pagination 配置/argv 行为”这一核心句；确认 capability ID 只复用 `core-cli`、`adapter-protocol`、`markdown-navigation`；确认提案阶段只新增 `openspec/changes/configure-pagination-defaults/` 下未审核临时 artifacts，尚未修改主规范、schema、examples 或代码。审计未完成前不得执行 2.x 及后续任何实现任务。

执行边界：除 1.1 外，所有实现、文档、测试和验证任务都以 1.1 完成为前置条件。

## 2. 主规范和验证材料

- [ ] 2.1 更新 `docs/cli.md`、`docs/architecture.md` 中 core 配置所有权和默认值解析说明，把分页默认值描述为 `defaults.pagination.enabled` 与 `defaults.pagination.limit_chars`，记录 `--pagination enabled|disabled`，并保留 page 不可配置规则。
- [ ] 2.2 更新 `docs/protocol.md` 和 `docs/adapter-contract.md`，说明 protocol request shape 不新增 pagination 字段，调用方仍必须显式传入正整数 `limit_chars` 和 `page`，禁用分页只在 pagination 参数归一阶段映射为 `PositiveInteger` 最大值。
- [ ] 2.3 更新 `docs/adapters/markdown.md`，说明 `docnav-markdown` direct CLI 配置支持 `defaults.pagination.enabled`、`defaults.pagination.limit_chars`、`defaults.output`、`options.max_heading_level` 和 `--pagination enabled|disabled`，其中 `defaults.pagination.*` 由 SDK direct CLI pagination 参数处理消费。
- [ ] 2.4 更新 `docs/schemas/docnav-markdown-config.schema.json` 和 `docs/examples/json/docnav-markdown-config.json`，使用 `defaults.pagination` 嵌套结构作为配置示例和参考 shape。
- [ ] 2.5 更新 `docs/testing.md`、`docs/testing/cases.md` 或相邻验证说明，覆盖 core 与 SDK pagination 参数处理、pagination 配置、`--pagination enabled|disabled`、禁用分页、`--limit-chars` 作为 `defaults.pagination.limit_chars` 显式来源、`--pagination disabled --limit-chars` 仍按 disabled 归一、invoke 不读配置。

## 3. Core CLI 实现

- [ ] 3.1 在 core pagination 参数处理中支持 `defaults.pagination.enabled` 与 `defaults.pagination.limit_chars`，并声明 config path、flag、help/default 文案、validation、source priority 和 finalization。
- [ ] 3.2 更新 `crates/docnav/src/config` 配置模型、supported keys、get/set/unset/list 和验证逻辑，使其支持 `defaults.pagination.enabled` 与 `defaults.pagination.limit_chars`。
- [ ] 3.3 更新 core document command parser 和 help，使 pagination 参数规则驱动 `--pagination enabled|disabled` 与 `--limit-chars` 的 argv 映射，并将 `--pagination` 映射为 `defaults.pagination.enabled` 的显式参数来源。
- [ ] 3.4 更新 core document default resolution，使最终 `defaults.pagination.enabled=false` 时，最终 `limit_chars` 初始化为协议 `PositiveInteger` 最大值；显式 `--pagination disabled` 触发同样归一。
- [ ] 3.5 更新 `config list --path ... --operation ...` 的 document context 输出，展示 pagination 配置来源、显式 argv 覆盖和最终 `limit_chars`。
- [ ] 3.6 补充 core 单元测试或 CLI 测试，证明项目/用户配置优先级、pagination flag/config 映射、pagination disabled、`--pagination enabled|disabled`、显式 `--limit-chars` 只映射为 `defaults.pagination.limit_chars`、`--pagination disabled --limit-chars` 仍按 disabled 归一、page 仍固定从 `1` 开始、invoke request shape 不新增 pagination 字段。

## 4. Adapter SDK 实现

- [ ] 4.1 在 SDK direct CLI pagination 参数处理中支持 `defaults.pagination.enabled` 与 `defaults.pagination.limit_chars`，并确保与 core 同名参数使用一致的 canonical key、config path、flag semantics、validation semantics、source priority 和 disabled finalization。
- [ ] 4.2 更新 `crates/docnav-adapter-sdk/src/direct/config` 配置源模型和字段投影，支持 `defaults.pagination.enabled` 与 `defaults.pagination.limit_chars`。
- [ ] 4.3 更新 SDK direct CLI argv parser 和 help，使 pagination 参数规则驱动 `--pagination enabled|disabled` 与 `--limit-chars` 的 argv 映射，并将 `--pagination` 映射为 `defaults.pagination.enabled` 的显式参数来源。
- [ ] 4.4 更新 SDK direct CLI 参数来源合并和 typed validation，使 explicit argv、项目配置、用户配置和内置默认值按字段优先级解析 `defaults.pagination.enabled` 与 `defaults.pagination.limit_chars`，再统一初始化最终 `limit_chars`。
- [ ] 4.5 确认 SDK `invoke` 路径仍只接受 stdin protocol request，不读取 direct CLI 配置，也不补全缺失的 `limit_chars`。
- [ ] 4.6 补充 SDK tests，覆盖 pagination flag/config 映射、pagination 配置投影、`--pagination enabled|disabled`、disabled 映射为 `PositiveInteger` 最大值、显式 `--limit-chars` 只映射为 `defaults.pagination.limit_chars`、`--pagination disabled --limit-chars` 仍按 disabled 归一、非法 pagination 值通过 direct CLI 参数处理链路报错、invoke 不读配置。

## 5. Markdown Adapter 集成

- [ ] 5.1 更新 `docnav-markdown` direct CLI 默认值、help/default 文案和测试 fixture，使配置示例使用 `defaults.pagination`，help 展示由 SDK direct CLI pagination 参数处理提供的 `--pagination enabled|disabled`。
- [ ] 5.2 更新 Markdown smoke 和矩阵测试，证明配置预算分页、`enabled: false` 对外不启用默认分页、`--pagination enabled|disabled`、显式 `--limit-chars` 只映射为 `defaults.pagination.limit_chars`、`--pagination disabled --limit-chars` 仍按 disabled 归一、配置路径覆盖和配置源 warning 仍有效。
- [ ] 5.3 确认 Markdown adapter 的 outline/read/find 分页 helper 和 operation result `page` 语义不被移动到 readable 输出层。

## 6. 验证

- [ ] 6.1 运行格式化和与改动范围匹配的 Rust 单元测试，至少覆盖 `docnav` core config、`docnav-adapter-sdk` direct args/config、`docnav-markdown` pagination/config。
- [ ] 6.2 运行 schema/example 验证，确认 `docnav-markdown` config schema 与示例同步。
- [ ] 6.3 运行 Markdown CLI smoke/matrix 或等价局部验证，覆盖 direct CLI 配置和 invoke 不读配置边界。
- [ ] 6.4 改动跨 core、SDK、schema/example 和 Markdown adapter 时，最终优先运行 `bun run verify:docnav-workspace`；无法运行时记录原因和未覆盖风险。
- [ ] 6.5 用局部 diff 确认只修改本 change 范围内的 pagination 规范、验证材料、配置、SDK、core 和 Markdown adapter 文件。
