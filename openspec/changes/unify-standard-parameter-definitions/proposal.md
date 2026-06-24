
## Why



## What Changes

- 标准参数 base definition 使用 builder-style API 声明 `ParamKey<T>`、canonical key、value type、default facet、必选 schema facet、基础 validator 和 schema metadata。
- Config path 仅允许 typed path builder 作为 registration 输入。dotted path 只作为显示、序列化或 schema path 输出。
- Core 和 SDK 在 request construction 前先完成各自入口的标准参数解析和正常数据处理；跨 protocol 序列化时使用 operation argument binding 和来源追踪，只写入需要跨 protocol 传递的显式字段和入口策略明确保留的透传字段。已解析的配置值或默认值不得仅因 request construction 被重新分类为 direct source；下游 adapter `invoke` 作为独立入口会再次按共享规则处理自己的 request arguments、配置、默认值和透传策略。
- Core 和 SDK 分别注册 owned 参数集合并提供配置来源描述。跨 consumer 复用的 canonical key 来自共享 base definition 或 builder factory；consumer registration 引用共享 base identity，并只补充 owner-specific registration。Registry 使用 definition identity/fingerprint 校验同名 key 的语义唯一性。

## Scope Boundaries

- 具体业务配置 key 由对应业务参数 change 声明。
- 配置管理命令入口、warning 承载、request construction、operation build 和 exit behavior 仍由对应入口 owner 承接；配置路径发现、入口策略、类型化配置路径和配置值映射由 `docs/standard-parameters.md` 承接。
- Schema generation 可以分阶段交付；本 change 要求 definition 携带可生成 schema、可供 runtime 校验复用的结构化 metadata。

## Capabilities

### New Capabilities


### Modified Capabilities

- `core-cli`：core 配置、document argv、help、context 输出、类型化结果和 invoke request argument construction 消费 `args-config-parameters`。
- `adapter-protocol`：adapter SDK direct CLI 的配置/argv/help/schema-backed validation、类型化结果、request argument construction，以及 adapter `invoke` 的 request arguments、配置/default 和类型化结果消费 `args-config-parameters`；protocol request `arguments` 的标准参数语义同步从最终参数改为直接输入来源。

## Impact

- 影响 `crates/docnav` 中 core 标准参数注册、配置 key 管理、配置源读取、document argv、help/default 文案、`config get/set/unset/list`、document context 输出和 invoke request argument construction。
- 影响 `crates/docnav-adapter-sdk` 中 direct CLI 和 adapter `invoke` 的标准参数注册、入口策略、配置读取、argv/request arguments、help/default 文案、schema-backed validation、warning、invoke request argument construction 和 operation 参数生成。
- 影响 `docs/protocol.md`、protocol request schema 和 examples 中 request `arguments` 的标准参数语义：envelope 保持稳定，标准参数字段表达 adapter `invoke` 的显式输入；protocol request schema 使用较窄 schema view，完整标准参数 required/default/range/enum 校验由解析器承接，未映射 argument 字段由入口策略处理。
- 可能扩展 `docnav-cli-args` 或新增共享 Rust 模块/crate 作为标准参数实现位置。
