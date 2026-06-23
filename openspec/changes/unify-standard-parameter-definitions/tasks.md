本 change 目标是交付共享 args/config 标准参数基础层；本文档是 `openspec/changes/unify-standard-parameter-definitions/` 下的未审核临时 tasks，主规范同步由第 2 节承接。

## 1. 审计门禁

- [ ] 1.1 审计 proposal、design、specs 和 tasks，确认本 change 只围绕 `args-config-parameters` 共享 capability，以及 `core-cli`、`adapter-protocol`、`docnav-contracts` 消费它；确认没有引入具体业务参数变更或主规范修改。
- [ ] 1.2 审计 `design.md#key-decision-log` 的 D1-D7，确认 proposal、spec delta、tasks 和后续主规范同步保持同一决策；如发现偏离，先更新 decision log 和受影响 artifacts，再继续实现。

## 2. 主规范和验证材料

- [ ] 2.1 更新 `docs/architecture.md`，说明共享 args/config 参数 owner 的职责：标准参数 base definition、registration set、source-to-standard-parameter-object projection、配置读取与投影、标准参数对象合并、来源追踪、schema-backed validation、operation argument binding、typed runtime values 和 schema metadata。
- [ ] 2.2 更新 `docs/cli.md`，说明 core CLI 标准参数、配置命令支持 key、document argv、help/default 文案、context 输出和 invoke request direct-source serialization 消费共享 registration、source tracking 与 typed standard params。
- [ ] 2.3 更新 `docs/adapter-contract.md`，说明 SDK direct CLI 标准参数、adapter `invoke` request arguments、配置读取与标准参数对象投影、argv/help/schema validation 和 request direct-source serialization 都消费共享 registration、source profile、source tracking 与 typed standard params。
- [ ] 2.4 更新 `docs/mcp.md`，说明 MCP tool input schema 和 tool input -> direct standard param source 映射从 tool -> operation 映射、operation registration set 和 MCP/CLI surface metadata 生成或同步；当前 transport 可继续映射到 core CLI argv。
- [ ] 2.5 更新 `docs/protocol.md`，说明 protocol request/result envelope 不变，但 request `arguments` 的标准参数字段从调用方最终 resolved params 调整为 resolver direct input source；同步 operation argument requiredness、schema view owner、examples 和错误分类边界。
- [ ] 2.6 更新 `docs/schemas/`、`docs/examples/` 或相邻验证材料，使 protocol request schema/example、MCP tool schema metadata 和配置 schema/example 与标准参数 source projection 语义一致；protocol request schema 必须使用较窄 schema view，只校验 envelope、operation、document path、raw arguments object、已出现字段基础 JSON 类型和字段可识别性。
- [ ] 2.7 更新测试说明，记录 definition-driven surface 的唯一来源、core/SDK 同名 key 复用共享 base definition、typed standard params 复用，以及 MCP metadata、invoke operation binding 与 operation/CLI registration 一致。

## 3. 共享 args/config 参数层

- [ ] 3.1 实现 builder-style 标准参数 base definition，支持 `ParamKey<T>`/canonical key、value type、default facet、必选 schema facet、基础 validator 和 schema metadata。
- [ ] 3.2 实现 registration set，使 consumer、CLI command、operation 和 MCP tool mapping 显式声明自己暴露的 config path、CLI surface、operation argument binding 或 MCP tool input surface；base definition 不使用全局 `.applies_to` 隐式决定暴露范围。
- [ ] 3.3 实现共享 base definition 或 builder factory，使跨 consumer canonical key 能从同一个 base 派生，consumer 只补充 owner-specific registration、配置域描述、CLI surface 或 operation registration。
- [ ] 3.4 实现 build/register 结构校验：必需槽位、schema、canonical key fingerprint、flag、config path、operation argument binding、静态默认值、flag argv facet 与 schema 兼容性、no-value flag 与 boolean schema 匹配关系都必须可验证。
- [ ] 3.5 实现 typed config path builder 作为唯一 config path 输入来源，并输出 path segments、显示路径和 schema path。
- [ ] 3.6 实现定义集合查询，支持按 canonical key、flag、config path、operation 和 operation argument binding 查询。
- [ ] 3.7 实现配置源读取和标准参数对象投影：调用方提供配置域描述和路径策略；共享层读取 JSON、校验顶层 object，并按 typed config path 将项目配置、用户配置分别投影为 standard parameter object；source profile 只声明入口可用 source provider 和 transport metadata。
- [ ] 3.8 实现所有来源到标准参数对象的投影：direct input、project config、user config 和 default 都必须先映射为 standard parameter object；no-config registration 的运行时值来自 direct input standard parameter object 或 default standard parameter object。
- [ ] 3.9 实现 typed runtime values：共享 resolver 按统一全局来源优先级合并标准参数对象并解析最终值：direct input standard parameter object、project config standard parameter object、user config standard parameter object、default standard parameter object；返回 `ResolvedStandardParams` 或等价 typed object，调用方可通过 `ParamKey<T>` 取得已校验的 `T` 值，并附带来源信息。
- [ ] 3.10 实现 direct input binding registry，支持 CLI argv、operation argument 和 MCP tool input 到 `ParamKey<T>` 的投影；binding name 可默认派生，也可由 typed operation argument path 或 MCP tool input path 显式覆盖。
- [ ] 3.11 实现 MCP metadata 生成或同步：tool-level operation mapping 消费 operation registration set 和 MCP/CLI surface registration，生成 tool input path、stable serialized param identity、canonical key、value kind、direct source projection、可选 CLI argv spelling、schema facet metadata、default metadata 和 operation registration membership；CLI argv spelling 只作为当前 transport projection metadata。JS 消费形态推荐 Rust generated JSON artifact；runtime metadata 或人工同步也可接受，但必须有映射测试或 artifact/schema diff 防止漂移。
- [ ] 3.12 实现 schema facet 输出、surface-specific schema view 和 runtime validation 复用，至少覆盖 type、enum、minimum/maximum、description、requiredness、default metadata，以及 protocol request schema view 与 resolver/MCP/config schema view 的差异。
- [ ] 3.13 补充共享层单元测试，覆盖 builder 声明、base definition 复用、registration set、同名 key fingerprint mismatch、typed config path、typed runtime values、source-to-standard-parameter-object projection、operation binding 派生/覆盖、MCP metadata 生成、重复检测、schema 必选、默认值校验、flag/schema 兼容、配置读取与投影、标准参数对象合并、来源追踪、schema view 输出和 schema-backed validation。

## 4. Core CLI 接入

- [ ] 4.1 将 core-owned 标准参数迁移到共享 base definition 和 core registration；跨 consumer 参数必须使用共享 base definition，至少覆盖当前 `defaults.output` 或其等价参数。
- [ ] 4.2 更新 core document argv parsing、help/default 文案和 context 输出，使其消费 core 标准参数 registration set 和 typed standard params。
- [ ] 4.3 更新 core config supported keys、配置读取与标准参数对象投影、`config get/set/unset/list` 和配置验证，使其消费共享层。
- [ ] 4.4 更新 core invoke request construction，使 core 先完整运行共享 resolver 并完成 core-owned 数据处理，再将需要跨 protocol 传递的 direct standard param source fields 通过 operation argument binding 和来源追踪写入 request `arguments`；core 已解析的配置值或默认值不得被重新标记为 adapter `invoke` direct source，下游 adapter `invoke` 作为独立入口再次运行共享 resolver。
- [ ] 4.5 补充 core tests/smoke，证明 flag/config/help/context/request binding/schema validation 均来自共享 base/registration，typed standard params 可复用，且 observable behavior 保持稳定。

## 5. Adapter SDK 接入

- [ ] 5.1 将 SDK direct CLI 标准参数迁移到共享 base definition 和 SDK registration；跨 consumer 参数必须使用共享 base definition，至少覆盖当前 `defaults.output`。
- [ ] 5.2 更新 SDK direct CLI 配置读取与标准参数对象投影、argv parsing、help/default 文案、warning 和 schema-backed validation，使其消费共享层。
- [ ] 5.3 更新 SDK request construction 和 adapter `invoke` source profile，使 SDK request construction 只序列化需要跨 protocol 传递的 direct standard param source fields；adapter `invoke` 将 request `arguments`、项目/用户配置和默认值分别投影为 standard parameter object，再通过共享 resolver 和统一全局来源优先级合并生成 typed standard params。
- [ ] 5.4 补充 SDK tests，证明 direct CLI 配置、argv、invoke request arguments、adapter invoke 标准参数对象合并、schema-backed validation、request argument projection 和 operation 参数生成消费共享层 typed standard params。

## 6. MCP 标准参数映射

- [ ] 6.1 为 `docnav-mcp` 提供 Rust generated JSON artifact、runtime metadata 或人工同步的等价方案，表达 tool -> operation 映射、tool input、stable serialized param identity、canonical key、value kind、direct source projection、可选 CLI argv spelling、schema facet metadata、default metadata 和 operation registration membership；CLI argv spelling 只作为当前 transport projection metadata。
- [ ] 6.2 更新 MCP bridge 参数映射或目标实现任务，使 document tool input 先映射为 direct input standard parameter object；当前 transport 可继续把该 source 映射为 core CLI argv。MCP bridge 仍不构造 adapter invoke request。
- [ ] 6.3 补充 MCP mapping tests 或 artifact tests，证明 `document_read.limit_chars` 从 `document_read -> read`、read operation registration、MCP tool metadata 和可选 core CLI registration 生成或被人工同步；人工同步路径必须证明 stable serialized param identity、schema facet 和 direct source projection 未漂移。

## 7. 验证

- [ ] 7.1 运行格式化和范围匹配的 Rust 单元测试，覆盖共享层、core config/args、SDK direct args/config。
- [ ] 7.2 运行 core CLI、adapter direct CLI、adapter invoke projection/source merge 和 MCP metadata focused smoke 或 artifact tests，证明当前配置 key、flag、help、invoke request argument mapping、MCP tool input mapping 和输出行为符合新标准参数契约。
- [ ] 7.3 运行 schema/example 或等价 docs 验证，确认 protocol request/result envelope、MCP structuredContent shape 保持稳定，并确认 protocol request `arguments` 标准参数 schema/examples 已同步为 direct input source 语义和较窄 protocol request schema view。
- [ ] 7.4 若改动跨共享 crate、core、SDK、MCP artifact 和 docs，优先运行 `bun run verify:docnav-workspace`；无法运行时记录原因和风险。
- [ ] 7.5 用局部 diff 确认修改范围保持在标准参数机制相关的规范、共享 helper、core、SDK、MCP metadata 和测试文件。
- [ ] 7.6 完成前复核 `design.md#key-decision-log` 的 D1-D7，确认实现、主规范、测试和验证材料没有偏离关键决策；如有偏离，先更新 decision log、proposal、spec delta 和 tasks，再继续验收。
