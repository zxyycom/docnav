本 spec delta 定义 `adopt-strict-input-boundaries` 对 `adapter-protocol` 的目标变更：让 adapter SDK direct CLI、invoke 和配置入口遵循严格公共输入边界。

## ADDED Requirements

### Requirement: SDK 直接 CLI 必须严格拒绝无效显式输入
`docnav-adapter-sdk` direct CLI MUST 默认拒绝 invalid caller intent。Unknown flags、extra positional arguments、selected operation 不支持的 flags、unregistered native options 和 invalid explicit config paths MUST 在 document operation execution 前产生 input diagnostics。

Adapter direct CLI 和 adapter `invoke` MUST 将 valid inputs 映射到同一个 canonical document operation input 或等价 semantic request。Adapter `invoke` stdin JSON MUST 保持 strict protocol direct input。Unknown request envelope fields、missing required fields 和 malformed request shapes MUST 在 protocol direct input boundary 失败。Known operation argument values 随后进入 standard parameter/typed-field validation；只有 explicit adapter-owned native option source 中的输入由 owner 校验或拒绝。

#### Scenario: 未知 argv 被拒绝
- **WHEN** adapter direct CLI 执行文档操作并收到未知 flag
- **THEN** SDK 返回输入错误
- **THEN** SDK 返回前停在 direct CLI input boundary
- **THEN** 诊断标出未知 token 并提供修复建议

#### Scenario: 多余 positional 被拒绝
- **WHEN** adapter direct CLI 执行文档操作并收到多余 positional
- **THEN** SDK 返回输入错误
- **THEN** SDK 返回前停在 direct CLI input boundary
- **THEN** 诊断标出多余 positional 的位置和值

#### Scenario: 当前 operation 不支持的参数被拒绝
- **WHEN** adapter direct CLI 收到当前 operation 不使用的已知参数
- **THEN** SDK 返回输入错误
- **THEN** 诊断说明该参数不适用于当前 operation
- **THEN** SDK 返回前停在 direct CLI input boundary

#### Scenario: direct CLI 和 invoke 共享文档操作语义归一
- **WHEN** adapter direct CLI input 与 adapter `invoke` direct input 经标准参数解析后表达同一个 outline/read/find/info 操作
- **THEN** 两者进入 canonical document operation input 或等价 semantic request
- **THEN** 默认值、native options、必需参数校验和 operation handler 不因入口不同分叉
- **THEN** 测试可通过等价 request、等价结果或共享 helper 覆盖该约束

#### Scenario: invoke request envelope 未知字段被拒绝
- **WHEN** adapter `invoke` 从 stdin 收到包含未知 envelope 字段、缺少必需字段或 request shape 错误的 JSON request
- **THEN** SDK 返回结构化 protocol failure
- **THEN** 该请求停在 protocol direct input boundary
- **THEN** 标准参数/typed-field processing 不接收该无效 envelope

#### Scenario: invoke operation arguments 进入 typed validation
- **WHEN** adapter `invoke` 从 stdin 收到 envelope 合法但 operation argument 类型错误、值不合法或无法映射的 JSON request
- **THEN** SDK 通过标准参数/typed-field processing 产生 validation diagnostic
- **THEN** failure 仍是 protocol-shaped failure response
- **THEN** adapter operation handler 不执行

## MODIFIED Requirements

### Requirement: Protocol 和 adapter SDK helper 必须保持进程边界契约
`docnav-protocol`、`docnav-json-io` 和 `docnav-adapter-sdk` MUST 只在保持 protocol、direct CLI 和 adapter process boundaries 时暴露 shared helpers。Adapter `invoke` stdin JSON MUST 保持 strict protocol direct input。Adapter direct CLI document command MAY reuse shared strict argv classification、diagnostics 和 document output helpers。

#### Scenario: Protocol input helper 区分 envelope 和 arguments 校验
- **WHEN** 共享代码 decode protocol request、protocol response、manifest 或 probe JSON value
- **THEN** protocol envelope、manifest 和 probe shape failures stay at the owning protocol or adapter boundary
- **THEN** valid request envelopes pass known operation arguments to standard parameter/typed-field processing
- **THEN** typed-field metadata、标准参数 mapping 和 owner semantic rules 产出 validation diagnostics for known operation arguments
- **THEN** 调用方 surface 保持既有 protocol diagnostic category、field path、diagnostic text 和 exit behavior

#### Scenario: Adapter invoke 保持严格 direct input 处理
- **WHEN** adapter `invoke` 收到 unknown envelope fields、missing required fields 或 malformed request shape
- **THEN** SDK 按 protocol direct input boundary 拒绝该请求
- **WHEN** adapter `invoke` 收到 envelope 合法但 argument wrong type、unmapped argument 或 invalid value
- **THEN** SDK 通过 standard parameter/typed-field processing 拒绝该请求
- **THEN** 两类失败都投影为 protocol-shaped failure responses

#### Scenario: Adapter direct CLI document command 复用共享 helper
- **WHEN** adapter direct CLI document operation succeeds or returns diagnostic failure outcome
- **THEN** SDK can use shared diagnostics for primary `DiagnosticRecord` projection and supplemental stderr diagnostic text when that surface permits it
- **THEN** SDK can use `docnav-output` for document output mode dispatch
- **THEN** manifest、probe 和 help output 保持既有 adapter contract 或 plain text boundary
- **THEN** invalid caller input 通过 owning failure surface 投影

### Requirement: Adapter SDK direct CLI 支持可覆盖配置路径
`docnav-adapter-sdk` direct CLI 在启用 adapter config loading 时 MUST 暴露 SDK-owned `--project-config-path <path>` 和 `--user-config-path <path>` document operation parameters。SDK MUST 在 document operation help 中展示这些参数。Missing default config paths MUST 表示 absent。Explicit override paths 按 strict config input 处理：missing、unreadable、non-file、invalid JSON 或 non-object JSON override sources MUST 使命令失败。

#### Scenario: 使用默认配置路径
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md` 且未覆盖配置路径
- **THEN** SDK 使用默认项目级路径 `.docnav/docnav-markdown.json`
- **THEN** SDK 使用默认用户级路径：默认用户配置目录下的 `docnav-markdown.json`
- **THEN** 路径参数保持为 SDK-owned config loading input

#### Scenario: 默认配置路径缺失表示 absence
- **WHEN** 默认项目级或用户级配置路径不存在
- **AND** 调用方未传入对应 config path override
- **THEN** SDK 将该 config source 记为 absent
- **THEN** 缺失默认路径表示该层配置 source absent

#### Scenario: 覆盖配置路径不可用时失败
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md --project-config-path missing.json`
- **THEN** SDK 尝试读取覆盖后的项目级配置路径
- **THEN** SDK 返回配置输入错误
- **THEN** SDK 返回前停在 config input boundary
- **THEN** 该显式配置 source 的诊断成为本次调用结果

#### Scenario: Help 展示配置路径参数
- **WHEN** 调用方执行 `docnav-markdown outline --help`
- **THEN** help 输出包含 `--project-config-path <path>` 和 `--user-config-path <path>`
- **THEN** help 输出由 CLI parser/help owner 生成

### Requirement: Adapter SDK direct CLI 配置只产出标准参数来源对象
Adapter direct CLI config MUST 支持 common `defaults.limit_chars`、`defaults.output` 和 adapter-owned `options` object。SDK MUST 按优先级把 argv、project config、user config 和 built-in defaults 合并为 standard direct CLI parameter source objects。Present 或 explicitly selected config sources 按 strict config loading 处理：unreadable files、invalid JSON、non-object root values、unknown top-level fields 和 unknown `defaults` fields MUST 产生 config diagnostics。

`options` object 是 explicit adapter-owned native option source。Declared `options` keys MUST 传递给 adapter/native option owner 校验；undeclared keys MUST 产生 native option diagnostics。

#### Scenario: defaults 字段投影为标准参数来源
- **WHEN** 配置文件包含 `defaults.limit_chars: 6000`
- **AND** 配置文件包含 `defaults.output: "readable-view"`
- **THEN** SDK 将 `defaults.limit_chars` 合并为 `limit_chars` 参数来源
- **THEN** SDK 将 `defaults.output` 合并为 `output` 参数来源

#### Scenario: 配置 options 合并为 adapter-owned 参数来源
- **WHEN** 配置文件包含 `options.max_heading_level: 2`
- **THEN** SDK 将 `options` object 合并为 adapter-owned native options 参数来源
- **THEN** native option 注册、value 处理和 operation 适用性由既有 native option 处理链路决定

#### Scenario: 配置读取层拒绝未知字段
- **WHEN** 配置文件包含未知顶层字段或未知 `defaults` 字段
- **THEN** SDK 返回配置输入错误
- **THEN** 诊断包含配置路径和字段路径
- **THEN** SDK 返回前停在 config input boundary

#### Scenario: 配置源内容无效时失败
- **WHEN** adapter direct CLI 读取到不可读、JSON 语法无效或顶层不是 JSON object 的 adapter 配置源
- **THEN** SDK 返回配置输入错误
- **THEN** 诊断 details 包含 `source_level`、`path_origin`、`path` 和 `reason_code`
- **THEN** SDK 返回前停在 config input boundary

#### Scenario: 参数来源对象交给标准参数处理链路
- **WHEN** 项目级 adapter 配置包含非法 `defaults.output`
- **AND** 调用方未显式传入 `--output`
- **THEN** SDK 将该值合并为 `output` 参数来源
- **THEN** direct CLI 复用 output typed validation 返回输入错误

## REMOVED Requirements

### Requirement: SDK 直接 CLI 必须兼容 CLI 扩展参数
**Reason**: 新契约要求 adapter direct CLI 在 document operation execution 前完成 strict input validation，并把 invalid caller intent 投影为 actionable diagnostics。

**Migration**: Adapter direct CLI 将 unknown argv、extra positional values、operation-inapplicable flags 和 undeclared native options 映射为 input diagnostics。Valid direct CLI 和 valid `invoke` JSON 继续共享同一个 document operation semantic pipeline。
