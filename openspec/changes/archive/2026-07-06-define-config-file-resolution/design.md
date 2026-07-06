本 design 说明 `docnav` 用户/项目配置文件路径解析 change 的实现方案；当前 change 只在 `openspec/changes/define-config-file-resolution/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

当前实现已有 `ProjectContext`，会记录 `cwd`、`project_root`、`project_config_path` 和 `user_config_path`。`docnav` core 为 navigation command 构造 project/user config source descriptors，`docnav-navigation` 再读取 raw config sources 并按 `explicit > project > user > built_in` 解析参数。缺口在于 CLI 没有 public flag 直接指定 project/user config 文件，且用户配置文件的环境变量与平台默认路径没有在主规范中形成完整 contract。

这个 change 触及 `docnav` core CLI surface、config command、doctor/init、navigation handoff 和 diagnostics，但不触及 adapter handler、ref、protocol envelope 或 readable output wrapper。

## Goals / Non-Goals

**Goals:**

- 增加 `--project-config <path>` 和 `--user-config <path>`，让调用方可以在真实 CLI 边界直接选择两个配置文件。
- 统一 document operation、`config`、`doctor` 和 `init` 的配置文件路径解析。
- 让 `docnav-navigation` 能区分 explicit config path 与 default config path，以正确处理 missing source。
- 保留参数值来源优先级 `explicit > project > user > built_in`。
- 使测试可以传入长期保留的配置 fixture 路径，只有会写入的场景再复制到临时文件。

**Non-Goals:**

- 不新增或修改配置 JSON 字段形状。
- 不改变 `defaults.*`、`options.*`、outline selector 或 adapter-owned native option 语义。
- 不新增 adapter direct CLI 配置路径。
- 不改变 project root discovery、document path normalization、ref 或 output mode contract。
- 不把 config path flag 变成 navigation parameter direct input。

## Decisions

### Decision 1: CLI flags are exact config file paths

`--project-config` 和 `--user-config` 都表示当前 invocation 使用的 exact JSON file path。它们不是目录，不自动追加 `.docnav/docnav.json`。这样测试、脚本和用户都能直接传入已存在的 fixture 或临时副本。

Alternative considered: 使用 `--config-dir` 风格的目录 flag。这个方案和“直接传配置文件路径”的目标不一致，并且对 project/user 两个来源仍需额外命名规则。

### Decision 2: User config env fallback keeps `DOCNAV_CONFIG_DIR` directory semantics

用户配置路径解析顺序为 `--user-config`、`DOCNAV_CONFIG_DIR/docnav.json`、平台用户默认 `.docnav/docnav.json`。`DOCNAV_CONFIG_DIR` 保留历史目录语义，因为现有 smoke 和隔离环境已经通过目录变量选择 user config root。直接指定单个文件的新增能力由 `--user-config` 承担。

Alternative considered: 新增 `DOCNAV_USER_CONFIG` 作为文件路径环境变量。该方案会同时存在两个环境变量控制同一来源，增加迁移和优先级解释成本；本 change 暂不引入。

### Decision 3: Project config default follows existing project context

未传 `--project-config` 时，project config 使用当前 project context 下的 `.docnav/docnav.json`。这保留从子目录启动时复用上层 project context 的既有行为；如果调用方需要绕过 project context，可以传 `--project-config` 指向任意可访问文件。

Alternative considered: 将默认 project config 改为 process cwd 下的 `.docnav/docnav.json`。这会破坏已有 project root discovery 语义，因此不作为本 change 的默认行为。

### Decision 4: Config path selection is a descriptor property, not parameter source

Core 应在 handoff descriptor 中表达 source level、resolved path 和 path origin，例如 explicit CLI、env default 或 platform/project default。`docnav-navigation` 只根据 descriptor 读取 raw source；配置文件内部的字段仍按 project/user source level 合并。由 CLI flag 选择的 config file 里的值不会升级为 direct argv source。

Alternative considered: 把 `--project-config` 和 `--user-config` 纳入 navigation direct input。该方案会混淆“选择来源文件”和“提供参数值”，并破坏现有来源优先级。

### Decision 5: Missing behavior depends on path origin

Default path missing 继续表示该 config source absent，不产生 diagnostic。Explicit config path missing、unreadable、invalid JSON 或顶层非 object 必须作为 blocking config source failure。调用方显式传入路径意味着希望该文件参与本次调用，静默忽略会隐藏配置错误。

Alternative considered: 所有 missing config source 都 absent。该方案对显式路径不够可审计，测试也无法证明用户传错文件时是否真的被使用。

## Risks / Trade-offs

- [Risk] 用户可能把 `DOCNAV_CONFIG_DIR` 误当成文件路径。→ Mitigation: CLI/help/docs 明确 `DOCNAV_CONFIG_DIR` 是目录；需要文件路径时使用 `--user-config`。
- [Risk] config path flags 分散加到多个命令，help 或 parser 行为不一致。→ Mitigation: 在 parser 层使用共享 helper，但只挂载到文档化支持该 flag 的命令。
- [Risk] explicit path missing 从 absent 变成 failure 可能暴露原本被忽略的测试问题。→ Mitigation: tests 分开覆盖 default missing absence 和 explicit missing failure。
- [Risk] 平台默认目录迁移可能影响已有本地用户配置。→ Mitigation: implementation audit 先确认当前已发布 contract；如需要保留历史路径，提供兼容读取或迁移说明后再实现。

## Migration Plan

1. 更新 `docs/cli.md`、`docs/navigation-input-resolution.md` 和相关测试文档，明确 flags、默认路径、origin 和 failure semantics。
2. 扩展 core CLI parser 和 command model，给 document operations、`config`、`doctor` 和 `init` 增加共享 config path args。
3. 调整 project/user config path resolution，产出带 origin 的 project/user descriptor。
4. 调整 `docnav-navigation` config source loading，使 default missing absent、explicit missing failure。
5. 更新 config command、doctor 和 init，使读写目标来自同一 resolution 结果。
6. 增加 Rust unit/integration tests 和 core CLI smoke，覆盖 explicit file path、env fallback、default absence、explicit missing failure 和 config set/list target。
7. 运行范围匹配验证；如跨 CLI、docs 和 navigation 边界，优先运行 `bun run verify:docnav-workspace`。

Rollback strategy: 移除新 flags 并恢复 descriptor origin 为 default-only；已有配置 JSON shape 不变，因此 rollback 不需要迁移文件内容。

## Open Questions

无未回答开放问题，可以进入实现前审计。
