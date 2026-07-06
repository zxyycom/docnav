本 proposal 定义 `docnav` 用户/项目配置文件路径的显式 flag 与默认解析顺序；当前 change 只在 `openspec/changes/define-config-file-resolution/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

当前规范已经定义 project/user config source 如何参与 navigation input resolution，但没有完整定义配置文件路径自身的发现顺序，也没有定义 CLI 直接指定 user/project config 文件的位置。测试和实现只能依赖默认目录或临时环境隔离，无法用 public CLI surface 明确表达“本次调用使用哪个配置文件”。

## What Changes

- 为 `docnav` core CLI 增加显式配置文件路径输入：`--user-config <path>` 和 `--project-config <path>`。
- 定义用户配置文件路径解析顺序：CLI 直接输入优先，其次环境变量默认，再其次平台用户目录下 `.docnav/docnav.json`。
- 定义项目配置文件路径解析顺序：CLI 直接输入优先，其次当前 project context 下 `.docnav/docnav.json`。
- 要求 document operation、`config` 命令和 `doctor` 使用同一套 user/project config path resolution；`init` 对 project config 创建目标也遵守 `--project-config`。
- 保留配置内容来源合并顺序 `explicit > project > user > built_in`，不改变配置 JSON shape、adapter-owned option 语义、schema 示例字段或 protocol output shape。
- 非目标：不引入 adapter direct CLI 配置路径，不新增 outline-mode override flag，不改变 ref、pagination envelope、readable/protocol 输出契约。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `core-cli`: 补齐 user/project config file path flag、默认路径发现、config/doctor/init 使用目标和 strict argv 行为。
- `navigation-input-resolution`: 补齐 core-supplied config source descriptor/path 的显式/默认来源归因，以及 navigation 对缺失和 present invalid config source 的处理边界。

## Impact

- Affected surface: `docnav` CLI flags、help 文本、project/user config source descriptor/path handoff、`config` 命令写入/读取目标、`doctor` 检查目标和 `init` 项目配置创建目标。
- Affected code: core CLI parser、project context/user config path resolution、config command handling、doctor checks、navigation handoff tests 和 smoke fixtures。
- Affected docs/tests: `docs/cli.md`、`docs/navigation-input-resolution.md`、测试策略/case ledger、CLI smoke 和 Rust unit/integration tests。
- Compatibility: 缺省路径继续使用现有 project context；新增 flags 是 optional public input。显式路径存在但不可读、JSON 无效或顶层非 object 时必须作为 present config source failure，而不是静默回退。
