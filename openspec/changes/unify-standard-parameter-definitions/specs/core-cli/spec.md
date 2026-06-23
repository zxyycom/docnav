本 change 目标是统一 core `docnav` 和 `docnav-adapter-sdk` direct CLI 的标准参数定义机制；本文档只是 `openspec/changes/unify-standard-parameter-definitions/` 下的未审核临时 core-cli delta，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Core CLI 标准参数必须由共享定义模型驱动
`docnav` core CLI 中可同时来自 CLI flag、core 配置文件或内置默认值的标准参数 MUST 由共享 Rust 标准参数定义模型声明。每个 core 标准参数定义 MUST 至少表达 canonical key、配置文件 path、可选 CLI flag、help/default 文案、value kind、typed parser 或 validator、operation applicability、source priority、default provider、finalization rule 和 schema metadata。Core `docnav` 的 argv parsing、config supported key listing、`config get/set/unset/list`、document context 输出、help/default 文案、typed validation 和最终 operation 参数生成 MUST 消费同一组 core 标准参数定义。

#### Scenario: Core 定义驱动同一参数的多个 surface
- **WHEN** core 注册一个标准参数定义，例如 `defaults.output`
- **THEN** CLI flag 映射、配置 key 支持、help/default 文案、`config list` 展示和 typed validation 都引用该定义
- **THEN** core 不为同一个参数维护独立的 flag/config/help/validation 映射表

#### Scenario: Core 定义保留参数来源信息
- **WHEN** 调用方通过显式 argv、项目配置、用户配置或内置默认值提供同一个 core 标准参数
- **THEN** core 按定义声明的 source priority 解析最终值
- **THEN** document context 输出能展示最终值和来源

#### Scenario: Core 定义可提供 schema metadata
- **WHEN** core 标准参数定义声明 JSON 配置 path、value kind、enum 或数值范围
- **THEN** 该 metadata 可用于生成或校验 core 配置 schema 参考材料
- **THEN** core runtime 不要求先加载生成后的 schema 文件才能解析配置

### Requirement: Core CLI 和 SDK 同名标准参数语义必须一致
当 core `docnav` 和 `docnav-adapter-sdk` direct CLI 都注册同一个 canonical key 时，双方 MUST 使用共享定义模型表达一致的 config path、CLI flag semantics、value kind、validation semantics 和 source priority semantics。两边 MAY 拥有不同参数集合和不同 finalization rule，但 MUST NOT 让同名 key 在 core 和 SDK 中代表不同业务含义。

#### Scenario: 同名 key 在 core 和 SDK 中保持一致
- **WHEN** core 和 SDK 都注册 `defaults.output`
- **THEN** 两边使用相同 canonical key 和 output value set
- **THEN** 显式 argv、项目配置、用户配置和内置默认值的优先级语义一致

#### Scenario: Core 不解释 adapter native options
- **WHEN** adapter direct CLI 配置包含 `options.max_heading_level`
- **THEN** core 标准参数定义集合不注册或解释该 key
- **THEN** format-specific options 仍由 adapter/SDK native option 机制拥有
