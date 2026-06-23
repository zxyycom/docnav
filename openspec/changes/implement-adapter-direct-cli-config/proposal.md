本 change 目标是让 adapter direct CLI 通过 SDK 支持可覆盖的项目级/用户级配置文件路径，并把配置内容合并为标准 direct CLI 参数来源对象；除本 change 明确新增的配置 schema/example 参考材料外，它只在 `openspec/changes/implement-adapter-direct-cli-config/` 下形成未审核临时文档，不改变现有主规范或实现语义。

## Why

当前规范已经把 `docnav-markdown` 和其它 adapter direct CLI 的配置域列为长期契约，但当前实现只有静态默认值和 argv/native option，没有真正读取 `.docnav/<adapter-id>.*` 或用户级 `<adapter-id>.*` 配置。

后续非结构化 outline 和 Markdown frontmatter metadata 都需要 adapter direct CLI 能稳定读取自身配置；如果每个 feature 自行实现配置读取，会造成重复逻辑和配置优先级不一致。

## What Changes

- `docnav-adapter-sdk` 增加 adapter direct CLI 配置读取和合并基础设施，并把 `--project-config-path <path>` 与 `--user-config-path <path>` 建模为 SDK-owned standard direct CLI 参数。
- SDK 为两个配置文件路径提供默认值：项目级默认指向项目根下 `.docnav/<adapter-id>.json`，用户级默认指向用户配置目录下的 `<adapter-id>.json`；调用方可以覆盖这两个路径。
- Adapter direct CLI document operation 使用一条内部参数归一化和 operation 执行线路；argv、配置文件和 `invoke` stdin JSON 都只是参数来源，进入业务逻辑前必须显式化为标准 operation 参数。
- Adapter direct CLI 文档操作先解析配置路径参数并读取两个配置文件，再按“显式 argv > 项目级 adapter 配置 > 用户级 adapter 配置 > 内置默认值”解析最终 `limit_chars`、`output` 和当前 operation 适用的 native options。
- Adapter direct CLI 在进入 document operation request construction 或 operation handler 前，将配置内容合并为标准 direct CLI 参数来源对象；配置贡献 `defaults.limit_chars`、`defaults.output` 和 `options`，`path`、`ref`、`query` 与 `page` 仍来自 argv 或入口固定默认。
- Adapter direct CLI help 必须暴露 SDK-owned 配置路径参数；显式覆盖配置路径不可用、配置 JSON 无法解析等配置源跳过原因必须产生 direct CLI warning，并按既有 warning 通道承载；新增 warning family 必须同步 `docnav-diagnostics` 和 readable warning schema。
- `docnav-markdown` 作为首个使用者，支持配置 `defaults.limit_chars`、`defaults.output` 和 `options.max_heading_level`。
- 新增 `docs/schemas/docnav-markdown-config.schema.json` 与 `docs/examples/json/docnav-markdown-config.json`，作为配置文件填写提示、打包分发和文档校验参考；该 schema/example 不改变 runtime 是否读取或校验配置的边界。
- Adapter direct CLI 把配置文件读取结果按优先级合并为标准 direct CLI 参数来源对象；配置读取层只处理配置源、字段映射和来源优先级，后续标准参数处理链路负责类型、范围、枚举、native option 注册和 operation 适用性处理。
- Adapter `invoke` stdin JSON 保持严格协议输入；它不读取项目/用户配置，而是把 stdin 中已经显式携带的参数送入同一 adapter document operation 线路。
- Core `docnav` 和 `docnav-mcp` 只决定自身入口可传递的参数，不读取 adapter direct CLI 配置，不从 adapter manifest、adapter 配置或隐式默认值合成格式专属 `options`。
- 非目标：不实现 core `docnav` 的 adapter-specific config 映射；不改变 manifest schema；不改变 protocol request/response 字段；不为 MCP bridge 增加独立配置解析；不让 config schema 成为 runtime 配置读取前置校验。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `adapter-protocol`: 增加 adapter direct CLI 配置读取、合并和 native options 显式化要求，并保持 invoke 严格协议边界。
- `docnav-contracts`: 补强“每个 CLI 只读取自身配置域”的可验证行为，明确 adapter direct CLI 配置不会被 core 或 MCP 重新解释。
- `markdown-navigation`: 增加 `docnav-markdown` 配置文件支持、配置键、优先级、读取边界、参考 schema/example 和 smoke 覆盖。

## Impact

- 受影响 public surface：`docnav-markdown outline/read/find/info` 的默认 `limit_chars`、默认 `output`、`max_heading_level` 来源，adapter direct CLI 配置文件路径覆盖参数 `--project-config-path` / `--user-config-path`，document operation help，配置源跳过 warning/config diagnostics，以及 Markdown adapter config schema/example 参考材料。
- 受影响代码：`crates/docnav-adapter-sdk` direct CLI config path parameters、config file discovery、JSON parsing、standard operation parameter merge、native option default resolution，以及 `crates/docnav-markdown` direct CLI wiring。
- 受影响文档与验证材料：`docs/architecture.md`、`docs/cli.md`、`docs/adapter-contract.md`、`docs/adapters/markdown.md`、`docs/testing.md`、`docs/schemas/docnav-markdown-config.schema.json`、`docs/examples/json/docnav-markdown-config.json`、docs validator、Markdown CLI smoke fixtures 和 OpenSpec delta specs。
- 不受影响范围：core `docnav` config keys、adapter install/registry 管理、adapter `invoke` strict JSON transport、protocol envelope shape、MCP bridge pass-through 职责和非 Markdown adapter 的具体 native option keys。
