本 change 目标是统一 core `docnav`、`docnav-adapter-sdk` direct CLI、adapter `invoke` 和 MCP tool mapping 使用的 args/config 标准参数基础层。本文档是 `openspec/changes/unify-standard-parameter-definitions/` 下的 change-local proposal；主规范同步由 tasks 中的文档任务承接。标准参数机制由新增的 `docs/standard-parameters.md` 完整承接，入口主规范只保留消费边界摘要和引用。

## Why

Core CLI、adapter SDK direct CLI、adapter `invoke` 和 MCP tool mapping 现在各自维护直接输入映射、配置读取与投影、来源对象合并、argv、help、tool input schema、protocol argument binding 和类型校验链路。`defaults.output`、`defaults.limit_chars` 这类跨入口复用的标准参数容易在 key、flag、config path、schema、来源和展示行为上漂移。

需要把这些机械参数能力收敛到共享实现，并把长期规则集中到 `docs/standard-parameters.md`：CLI argv、MCP tool input、invoke request arguments、项目配置、用户配置和默认值先分别映射为标准参数对象；最后由同一 resolver 按统一全局来源优先级合并这些对象、校验并交给正常调用逻辑。该优先级固定为直接输入值（CLI argv、MCP tool input 或 invoke request arguments）、项目配置、用户配置、默认值。`invoke` 不再把 protocol request `arguments` 视为调用方已经完成配置/default 解析的最终参数，而是和 CLI/MCP 一样把显式 request arguments 作为 direct input standard parameter object 进入标准参数解析。具体业务参数 change 只声明自己的参数行为，并引用 `docs/standard-parameters.md` 定义的共享 base definition、registration set、标准参数对象投影和来源合并规则。

## What Changes

- 新增 `docs/standard-parameters.md`，完整承接标准参数机制规则，并在 `docs/navigation.md` 规则所有权表中登记；`docs/architecture.md`、`docs/cli.md`、`docs/adapter-contract.md`、`docs/mcp.md` 和 `docs/protocol.md` 只记录各自入口如何消费该机制。
- 新增或扩展共享 Rust args/config 参数实现，承接标准参数 base definition registry、standard parameter object projection、typed config path builder、配置读取与投影、标准参数对象合并、来源追踪、schema-backed validation、operation argument binding、MCP tool input mapping、typed runtime values 和 schema metadata。
- 标准参数 base definition 使用 builder-style API 声明 `ParamKey<T>`、canonical key、value type、default facet、必选 schema facet、基础 validator 和 schema metadata。
- Consumer、CLI command、operation 和 MCP tool 不从 base definition 隐式继承全局 `.applies_to`；它们通过 registration set 或 tool mapping 明确声明自己暴露的 config path、CLI surface、operation argument binding 或 MCP tool input surface。
- `standard_params` 或等价 resolver 先把 CLI argv、MCP tool input、invoke request arguments、项目配置、用户配置和默认值分别投影为标准参数对象，再按统一全局来源优先级合并并生成 `ResolvedStandardParams` / typed runtime object。调用方可通过 `ParamKey<T>` 取得已校验的 `T` 值，并复用同一 typed object 做 request construction、context 输出和测试断言。
- Config path 仅允许 typed path builder 作为 registration 输入。dotted path 只作为显示、序列化或 schema path 输出。
- Schema facet 是基础 value validation 和 schema metadata 的共同来源。默认值先由 default facet 产出，再进入同一 schema facet 校验；静态默认值在 build/register 阶段校验，动态默认值在 runtime 产出后校验。Schema metadata MUST 支持 surface-specific schema view：protocol request schema view 只校验 envelope、operation、document path、raw arguments object、已出现标准参数字段的基础 JSON 类型和字段可识别性；resolver schema view、MCP tool schema view 和 config schema view MAY 使用同一 schema facet 表达更完整的 enum、range、requiredness 和 default metadata。
- Operation argument binding 是 protocol/invoke arguments 与标准化参数之间的映射来源；CLI flag 属于 CLI registration；MCP tool input 属于 MCP tool mapping。简单参数可以复用 stable name 派生 CLI flag、protocol argument path 和 MCP tool input path，但不同 surface 仍拥有各自 spelling 与包装职责。
- Core 和 SDK 在 request construction 前先完成各自入口的完整标准参数解析和正常数据处理；跨 protocol 序列化时使用 operation argument binding 和来源追踪，只把需要跨 protocol 传递的 direct standard param source fields 写成 adapter `invoke` direct source。已解析的配置值或默认值不得仅因 request construction 被重新分类为 direct source；下游 adapter `invoke` 作为独立入口会再次按共享 resolver 处理自己的 request arguments、配置和默认值。
- MCP tool input metadata 与 tool input -> direct standard param source 映射由 tool -> operation 映射、operation registration set 和 MCP/CLI surface registration 共同生成或同步。MCP bridge 推荐消费 Rust 生成的 JSON artifact；也可以消费 runtime metadata，或接受人工同步的等价映射并用映射测试/artifact diff 防止漂移。当前实现仍可把 tool input 转成 core `docnav` CLI argv 作为传输路径，但语义上不再维护一套独立参数定义。
- Core 和 SDK 分别注册 owned 参数集合并提供配置域描述。跨 consumer 复用的 canonical key 来自共享 base definition 或 builder factory；consumer registration 引用共享 base identity，并只补充 owner-specific registration。Registry 使用 definition identity/fingerprint 校验同名 key 的语义唯一性。

## Scope Boundaries

- 具体业务配置 key 由对应业务参数 change 声明。
- `docs/standard-parameters.md` 只承接标准参数机制、共享注册/投影/resolver/schema metadata 规则；core CLI、adapter SDK、MCP bridge、protocol envelope 和测试验证边界仍由对应主规范承接。
- Protocol request/result envelope、readable output、MCP structuredContent、ref 和 MCP transport shape 保持当前契约；protocol request `arguments` 的标准参数语义会从“最终显式参数”调整为“resolver 的直接输入来源”，对应 schema/examples 必须同步。
- 配置域路径发现、配置管理命令、warning 承载、request construction、operation build 和 exit behavior 仍由对应入口 owner 承接。
- Schema generation 可以分阶段交付；本 change 要求 definition 携带可生成 schema、可供 runtime 校验复用的结构化 metadata。

## Capabilities

### New Capabilities

- `args-config-parameters`：新增共享 args/config 参数能力，拥有标准参数 base definition model、registration set、standard parameter object projection、typed config path builder、配置读取与投影、标准参数对象合并、source tracking、default facet、flag argv facet、schema-backed validation、operation argument binding、MCP tool input mapping、typed runtime values、MCP metadata 输出和 schema metadata 输出；归档后由 `docs/standard-parameters.md` 承接主规范。

### Modified Capabilities

- `core-cli`：core 配置、document argv、help、context 输出、typed runtime values 和 invoke request argument projection 消费 `args-config-parameters`。
- `adapter-protocol`：adapter SDK direct CLI 的配置/argv/help/schema-backed validation、typed runtime values、request argument construction，以及 adapter `invoke` 的 request argument projection、配置/default standard parameter object projection 和 typed runtime values 消费 `args-config-parameters`；protocol request `arguments` 的标准参数语义同步从最终参数改为直接输入来源。
- `docnav-contracts`：CLI argv、invoke request arguments 和 MCP tool input 都先投影为标准化参数来源；MCP tool input schema 和 tool input 映射从 tool -> operation 映射、operation registration set 和 MCP/CLI surface registration 生成。

## Impact

- 新增 `docs/standard-parameters.md` 并更新 `docs/navigation.md` 规则所有权，使标准参数机制只有一个主规范归属；入口主规范同步为消费方说明，避免把共享规则分散在 architecture/CLI/adapter/MCP/protocol 文档中。
- 影响 `crates/docnav` 中 core 标准参数注册、配置 key 管理、配置源读取与标准参数对象投影、document argv、help/default 文案、`config get/set/unset/list`、document context 输出和 invoke request argument projection。
- 影响 `crates/docnav-adapter-sdk` 中 direct CLI 和 adapter `invoke` 的标准参数注册、source profile、配置读取与标准参数对象投影、argv/request arguments、help/default 文案、schema-backed validation、warning、invoke request argument projection 和 operation 参数生成。
- 影响 `docnav-mcp` 的目标 tool input 映射来源：tool inputs 应从 tool -> operation 映射、operation registration set 和 MCP/CLI surface registration 生成或同步，并作为直接输入标准化参数来源；现阶段可继续映射到 core `docnav` CLI argv 作为传输路径。
- 影响 `docs/protocol.md`、protocol request schema 和 examples 中 request `arguments` 的标准参数语义：envelope 保持稳定，标准参数字段表达 resolver direct input source；protocol request schema 使用较窄 schema view，完整标准参数 required/default/range/enum 校验由 resolver 承接。
- 可能扩展 `docnav-cli-args` 或新增共享 Rust 模块/crate 作为 args/config 参数实现位置。
