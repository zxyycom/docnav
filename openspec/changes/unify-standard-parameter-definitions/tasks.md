本 change 目标是交付标准参数共享基础层；本文档是 `openspec/changes/unify-standard-parameter-definitions/` 下的 change-local tasks，主规范同步由第 2 节承接。标准参数机制由新增的 `docs/standard-parameters.md` 完整承接。

## 1. 审计门禁

- [x] 1.1 审计 proposal、design、specs 和 tasks，确认本 change 只围绕 `args-config-parameters` 共享 capability，以及 `core-cli`、`adapter-protocol`、`docnav-contracts` 消费它；确认没有引入具体业务参数变更或主规范修改。
- [x] 1.2 审计 `design.md#key-decision-log` 的 D1-D7，确认 proposal、spec delta、tasks 和后续主规范同步保持同一决策；如发现偏离，先更新 decision log 和受影响 artifacts，再继续实现。

## 2. 主规范和验证材料

- [x] 2.2 更新 `docs/architecture.md`，只摘要标准参数共享层的制品职责和跨制品边界，并链接 `docs/standard-parameters.md`；不得在 architecture 中重新定义合并顺序、registration 规则或 schema view。
- [x] 2.3 更新 `docs/cli.md`，说明 core CLI 标准参数、配置命令支持 key、document argv、help/default 文案、context 输出和 invoke request construction 消费 `docs/standard-parameters.md` 定义的共享规则。
- [x] 2.4 更新 `docs/adapter-contract.md`，说明 SDK direct CLI 标准参数、adapter `invoke` request arguments、配置读取、argv/help/schema validation 和 request construction 都消费 `docs/standard-parameters.md` 定义的共享规则。
- [x] 2.6 更新 `docs/protocol.md`，说明 protocol request/result envelope 不变，但 request `arguments` 的标准参数字段从调用方最终 resolved params 调整为 adapter `invoke` 的显式输入；同步 operation argument requiredness、schema view 归属、未映射字段策略、examples 和错误分类边界，并链接 `docs/standard-parameters.md`。

## 3. 共享标准参数层

- [ ] 3.1 实现 builder-style 标准参数 base definition，支持 `ParamKey<T>`/canonical key、value type、default facet、必选 schema facet、基础 validator 和 schema metadata。
- [ ] 3.3 实现共享 base definition 或 builder factory，使跨 consumer canonical key 能从同一个 base 派生，consumer 只补充 owner-specific registration、配置来源描述、CLI registration 或 operation registration。
- [ ] 3.4 实现 build/register 结构校验：必需槽位、schema、canonical key fingerprint、flag、config path、operation argument binding、静态默认值、flag argv facet 与 schema 兼容性、no-value flag 与 boolean schema 匹配关系都必须可验证。
- [ ] 3.5 实现 typed config path builder 作为唯一 config path 输入来源，并输出 path segments、显示路径和 schema path。
- [ ] 3.6 实现定义集合查询，支持按 canonical key、flag、config path、operation 和 operation argument binding 查询。
- [ ] 3.7 实现配置源读取和已注册字段映射：调用方提供配置来源描述、路径策略和透传策略；共享层读取 JSON、校验顶层 object，并按 typed config path 将项目配置、用户配置分别映射为标准参数来源。
- [ ] 3.8 实现所有来源到标准参数来源的映射：direct input、project config、user config 和 default 都必须作为独立来源进入解析；no-config registration 的运行时值来自 direct input 或 default；未映射输入按入口策略保留、丢弃或交给调用方校验。
- [ ] 3.9 实现类型化运行值：共享解析器按固定合并顺序合并标准参数来源并解析最终值：direct input、project config、user config、default；返回 `ResolvedStandardParams` 或等价 typed object，调用方可通过 `ParamKey<T>` 取得已校验的 `T` 值，并附带来源信息和透传字段。

## 4. Core CLI 接入

- [ ] 4.1 将 core-owned 标准参数迁移到共享 base definition 和 core registration；跨 consumer 参数必须使用共享 base definition，至少覆盖当前 `defaults.output` 或其等价参数。
- [ ] 4.2 更新 core document argv parsing、help/default 文案和 context 输出，使其消费 core 标准参数 registration set 和类型化标准参数。
- [ ] 4.3 更新 core config supported keys、配置读取与标准参数来源映射、`config get/set/unset/list` 和配置验证，使其消费共享层。
- [ ] 4.4 更新 core invoke request construction，使 core 先完整运行共享解析器并完成 core-owned 数据处理，再将需要跨 protocol 传递的显式字段和 core 入口策略明确保留的透传字段通过 operation argument binding、透传策略和来源追踪写入 request `arguments`；core 已解析的配置值或默认值不得被重新标记为 adapter `invoke` direct source，下游 adapter `invoke` 作为独立入口再次运行共享解析器。
- [ ] 4.5 补充 core tests/smoke，证明 flag/config/help/context/request binding/schema validation 均来自共享 base/registration，类型化标准参数可复用，且 observable behavior 保持稳定。

## 5. Adapter SDK 接入

- [ ] 5.1 将 SDK direct CLI 标准参数迁移到共享 base definition 和 SDK registration；跨 consumer 参数必须使用共享 base definition，至少覆盖当前 `defaults.output`。
- [ ] 5.2 更新 SDK direct CLI 配置读取与标准参数来源映射、argv parsing、help/default 文案、warning 和 schema-backed validation，使其消费共享层。
- [ ] 5.3 更新 SDK request construction 和 adapter `invoke` 入口策略，使 SDK request construction 只序列化需要跨 protocol 传递的显式字段和入口策略明确保留的透传字段；adapter `invoke` 将 request `arguments`、项目/用户配置和默认值作为独立来源，再通过共享解析器和固定合并顺序生成类型化标准参数，并按 adapter policy 丢弃或校验未映射 native 输入。
- [ ] 5.4 补充 SDK tests，证明 direct CLI 配置、argv、invoke request arguments、adapter invoke 标准参数来源合并、schema-backed validation、request argument mapping 和 operation 参数生成消费共享层类型化标准参数。



## 7. 验证

- [ ] 7.1 运行格式化和范围匹配的 Rust 单元测试，覆盖共享层、core config/argv、SDK direct config/argv。
- [ ] 7.6 完成前复核 `design.md#key-decision-log` 的 D1-D7，确认实现、主规范、测试和验证材料没有偏离关键决策；如有偏离，先更新 decision log、proposal、spec delta 和 tasks，再继续验收。
