本 change 目标是统一 core `docnav`、`docnav-adapter-sdk` direct CLI、adapter `invoke` 和 MCP tool mapping 使用的标准参数共享基础层。本文档是 `openspec/changes/unify-standard-parameter-definitions/` 下的 change-local proposal；主规范同步由 tasks 中的文档任务承接。标准参数机制由新增的 `docs/standard-parameters.md` 完整承接，入口主规范只保留消费边界摘要和引用。

## Why

Core CLI、adapter SDK direct CLI、adapter `invoke` 和 MCP tool mapping 现在各自维护输入映射、配置读取、来源合并、argv/help、tool input schema、protocol argument binding 和类型校验链路。`defaults.output`、`defaults.limit_chars` 这类跨入口复用的标准参数容易在 key、flag、config path、schema、来源和展示行为上漂移。

需要把这些机械参数能力收敛到共享实现，并把长期规则集中到 `docs/standard-parameters.md`。CLI argv、MCP tool input、invoke request arguments、项目配置、用户配置和默认值先归一为标准参数来源，再按固定优先级合并和校验。未映射输入由入口策略决定：core 作为转发和路由层可以保留下游字段，adapter 层作为最终消费者可以丢弃未映射 native 输入或执行 adapter-owned validation。`invoke` 不再把 protocol request `arguments` 视为调用方已经完成配置/default 解析的最终参数，而是把显式 request arguments 作为 adapter `invoke` 的输入来源。具体业务参数 change 只声明自己的参数行为，并引用 `docs/standard-parameters.md` 的共享规则。

## What Changes

- 新增 `docs/standard-parameters.md`，完整承接标准参数机制规则，并在 `docs/navigation.md` 规则所有权表中登记；`docs/architecture.md`、`docs/cli.md`、`docs/adapter-contract.md`、`docs/mcp.md` 和 `docs/protocol.md` 只记录各自入口如何消费该机制。
- 新增或扩展共享 Rust 标准参数实现，承接 base definition registry、入口字段映射、类型化配置路径、配置读取、来源合并、透传策略、来源追踪、schema-backed validation、operation argument binding、MCP tool input mapping、类型化结果和 schema metadata。
- 标准参数 base definition 使用 builder-style API 声明 `ParamKey<T>`、canonical key、value type、default facet、必选 schema facet、基础 validator 和 schema metadata。
- Consumer、CLI command、operation 和 MCP tool 不从 base definition 隐式继承全局 `.applies_to`；它们通过 registration set 或 tool mapping 明确声明自己暴露的 config path、CLI flag mapping、operation argument binding 或 MCP tool input mapping。
- `standard_params` 或等价解析器先把 CLI argv、MCP tool input、invoke request arguments、项目配置、用户配置和默认值归一为带来源标记的标准参数来源，再按固定顺序合并并生成 `ResolvedStandardParams` 或等价类型化结果。调用方可通过 `ParamKey<T>` 取得已校验的 `T` 值，并复用同一结果做 request construction、context 输出和测试断言。
- Config path 仅允许 typed path builder 作为 registration 输入。dotted path 只作为显示、序列化或 schema path 输出。
- Schema facet 是基础 value validation 和 schema metadata 的共同来源。默认值先由 default facet 产出，再进入同一 schema facet 校验；静态默认值在 build/register 阶段校验，动态默认值在 runtime 产出后校验。Schema metadata MUST 支持 entry-specific schema view：protocol request schema view 只校验 envelope、operation、document path、raw arguments object 和已出现已注册标准参数字段的基础 JSON 类型；未映射 argument 字段由入口策略处理。解析器、MCP tool 和 config schema view MAY 使用同一 schema facet 表达更完整的 enum、range、requiredness 和 default metadata。
- Operation argument binding 是 protocol/invoke arguments 与标准化参数之间的映射来源；CLI flag 属于 CLI registration；MCP tool input 属于 MCP tool mapping。简单参数可以复用 stable name 派生 CLI flag、protocol argument path 和 MCP tool input path，但不同入口仍拥有各自 spelling 与包装职责。
- Core 和 SDK 在 request construction 前先完成各自入口的标准参数解析和正常数据处理；跨 protocol 序列化时使用 operation argument binding 和来源追踪，只写入需要跨 protocol 传递的显式字段和入口策略明确保留的透传字段。已解析的配置值或默认值不得仅因 request construction 被重新分类为 direct source；下游 adapter `invoke` 作为独立入口会再次按共享规则处理自己的 request arguments、配置、默认值和透传策略。
- MCP tool input metadata 与 tool input 到标准参数的映射由 tool -> operation 映射、operation registration set、MCP metadata 和可选 CLI argv transport metadata 共同生成或同步。MCP bridge 推荐消费 Rust 生成的 JSON artifact；也可以消费 runtime metadata，或接受人工同步的等价映射并用映射测试/artifact diff 防止漂移。当前实现仍可把 tool input 转成 core `docnav` CLI argv 作为传输路径，但语义上不再维护一套独立参数定义。
- Core 和 SDK 分别注册 owned 参数集合并提供配置来源描述。跨 consumer 复用的 canonical key 来自共享 base definition 或 builder factory；consumer registration 引用共享 base identity，并只补充 owner-specific registration。Registry 使用 definition identity/fingerprint 校验同名 key 的语义唯一性。

## Scope Boundaries

- 具体业务配置 key 由对应业务参数 change 声明。
- `docs/standard-parameters.md` 只承接标准参数机制、入口字段映射、配置字段映射、来源标记、合并顺序、validation、operation binding 和 metadata 规则；core CLI、adapter SDK、MCP bridge、protocol envelope 和测试验证边界仍由对应主规范承接。
- Protocol request/result envelope、readable output、MCP structuredContent、ref 和 MCP transport shape 保持当前契约；protocol request `arguments` 的标准参数语义会从“最终显式参数”调整为 adapter `invoke` 的显式输入来源，对应 schema/examples 必须同步。
- 配置管理命令入口、warning 承载、request construction、operation build 和 exit behavior 仍由对应入口 owner 承接；配置路径发现、入口策略、类型化配置路径和配置值映射由 `docs/standard-parameters.md` 承接。
- Schema generation 可以分阶段交付；本 change 要求 definition 携带可生成 schema、可供 runtime 校验复用的结构化 metadata。

## Capabilities

### New Capabilities

- `args-config-parameters`：新增共享标准参数能力，拥有 base definition model、registration set、类型化配置路径、配置读取、来源合并、透传策略、source tracking、default facet、flag argv facet、schema-backed validation、operation argument binding、MCP tool input mapping、类型化结果、MCP metadata 输出和 schema metadata 输出；归档后由 `docs/standard-parameters.md` 承接主规范。

### Modified Capabilities

- `core-cli`：core 配置、document argv、help、context 输出、类型化结果和 invoke request argument construction 消费 `args-config-parameters`。
- `adapter-protocol`：adapter SDK direct CLI 的配置/argv/help/schema-backed validation、类型化结果、request argument construction，以及 adapter `invoke` 的 request arguments、配置/default 和类型化结果消费 `args-config-parameters`；protocol request `arguments` 的标准参数语义同步从最终参数改为直接输入来源。
- `docnav-contracts`：CLI argv、invoke request arguments 和 MCP tool input 都先归一为标准化参数来源；MCP tool input schema 和 tool input 映射从 tool -> operation 映射、operation registration set、MCP metadata 和可选 CLI argv transport metadata 生成。

## Impact

- 新增 `docs/standard-parameters.md` 并更新 `docs/navigation.md` 规则所有权，使标准参数机制只有一个主规范归属；入口主规范同步为消费方说明，避免把共享规则分散在 architecture/CLI/adapter/MCP/protocol 文档中。
- 影响 `crates/docnav` 中 core 标准参数注册、配置 key 管理、配置源读取、document argv、help/default 文案、`config get/set/unset/list`、document context 输出和 invoke request argument construction。
- 影响 `crates/docnav-adapter-sdk` 中 direct CLI 和 adapter `invoke` 的标准参数注册、入口策略、配置读取、argv/request arguments、help/default 文案、schema-backed validation、warning、invoke request argument construction 和 operation 参数生成。
- 影响 `docnav-mcp` 的目标 tool input 映射来源：tool inputs 应从 tool -> operation 映射、operation registration set、MCP metadata 和可选 CLI argv transport metadata 生成或同步，并作为直接输入标准化参数来源；现阶段可继续映射到 core `docnav` CLI argv 作为传输路径。
- 影响 `docs/protocol.md`、protocol request schema 和 examples 中 request `arguments` 的标准参数语义：envelope 保持稳定，标准参数字段表达 adapter `invoke` 的显式输入；protocol request schema 使用较窄 schema view，完整标准参数 required/default/range/enum 校验由解析器承接，未映射 argument 字段由入口策略处理。
- 可能扩展 `docnav-cli-args` 或新增共享 Rust 模块/crate 作为标准参数实现位置。
