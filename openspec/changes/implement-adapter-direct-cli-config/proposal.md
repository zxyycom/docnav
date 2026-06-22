本 change 目标是让 adapter direct CLI 通过 SDK 支持可覆盖的项目级/用户级配置文件路径，并把配置内容合并为标准 direct CLI 调用参数；它只在 `openspec/changes/implement-adapter-direct-cli-config/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

当前规范已经把 `docnav-markdown` 和其它 adapter direct CLI 的配置域列为长期契约，但当前实现只有静态默认值和 argv/native option，没有真正读取 `.docnav/<adapter-id>.*` 或用户级 `<adapter-id>.*` 配置。

后续非结构化 outline 和 Markdown frontmatter metadata 都需要 adapter direct CLI 能稳定读取自身配置；如果每个 feature 自行实现配置读取，会造成重复逻辑和配置优先级不一致。

## What Changes

- `docnav-adapter-sdk` 增加 adapter direct CLI 配置读取和合并基础设施，并把 `--project-config-path <path>` 与 `--user-config-path <path>` 建模为 SDK-owned standard direct CLI 参数。
- SDK 为两个配置文件路径提供默认值：项目级默认指向项目根下 `.docnav/<adapter-id>.json`，用户级默认指向用户配置目录下的 `<adapter-id>.json`；调用方可以覆盖这两个路径。
- Adapter direct CLI 文档操作先解析配置路径参数并读取两个配置文件，再按“显式 argv > 项目级 adapter 配置 > 用户级 adapter 配置 > 内置默认值”解析最终 `limit_chars`、`output` 和当前 operation 适用的 native options。
- Adapter direct CLI 在进入 canonical document operation input、operation handler 或等价 invoke request 前，将配置内容合并为标准 direct CLI 调用参数；配置只贡献 `limit_chars`、`output` 和 adapter native options，`path`、`ref`、`query` 与 `page` 仍来自 argv 或入口固定默认。
- `docnav-markdown` 作为首个使用者，支持配置 `defaults.limit_chars`、`defaults.output` 和 `options.max_heading_level`。
- Adapter direct CLI 配置错误按已确定的 document output mode 渲染；显式 `--output protocol-json` 时 stdout 保持 protocol failure envelope，显式或配置确定 `readable-json` 时 stdout 保持 readable error JSON。
- Adapter `invoke` stdin JSON 保持严格协议输入，不读取项目/用户配置，也不使用 direct CLI 的配置容错规则。
- Core `docnav` 不读取 adapter 专属配置，不从 adapter manifest、adapter 配置或隐式默认值合成格式专属 `options`。
- 非目标：不实现 core `docnav` 的 adapter-specific config 映射；不改变 manifest schema；不改变 protocol request/response 字段；不为 MCP bridge 增加独立配置解析。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `adapter-protocol`: 增加 adapter direct CLI 配置读取、合并、校验和 native options 显式化要求，并保持 invoke 严格协议边界。
- `docnav-contracts`: 补强“每个 CLI 只读取自身配置域”的可验证行为，明确 adapter direct CLI 配置不会被 core 或 MCP 重新解释。
- `markdown-navigation`: 增加 `docnav-markdown` 配置文件支持、配置键、优先级、错误边界和 smoke 覆盖。

## Impact

- 受影响 public surface：`docnav-markdown outline/read/find/info` 的默认 `limit_chars`、默认 `output`、`max_heading_level` 来源，adapter direct CLI 配置文件路径覆盖参数 `--project-config-path` / `--user-config-path`、help/config diagnostics，以及配置错误的 stdout/stderr/exit code。
- 受影响代码：`crates/docnav-adapter-sdk` direct CLI config path parameters、config file discovery、JSON parsing/validation、standard operation parameter merge、native option default resolution，以及 `crates/docnav-markdown` direct CLI wiring。
- 受影响文档与验证材料：`docs/cli.md`、`docs/adapter-contract.md`、`docs/adapters/markdown.md`、`docs/testing.md`、Markdown CLI smoke fixtures 和 OpenSpec delta specs。
- 不受影响范围：core `docnav` config keys、adapter install/registry 管理、adapter `invoke` strict JSON transport、protocol envelope shape、MCP bridge pass-through 职责和非 Markdown adapter 的具体 native option keys。
