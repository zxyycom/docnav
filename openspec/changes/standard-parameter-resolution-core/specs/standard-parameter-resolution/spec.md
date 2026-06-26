## ADDED Requirements

### Requirement: Pipeline 以 typed field definitions 为入口
标准参数 pipeline MUST 接受 caller-defined typed field definitions 作为普通入口，从 typed-field metadata 和 direct/config strategy 构建内部 catalog/index，并保持 field identity、type、requiredness、defaults、constraints 和 extraction paths 由 `docnav-typed-fields` 拥有。

#### Scenario: Caller-defined FieldDefSet 是字段事实源
- **WHEN** caller 从 `docnav-typed-fields` definitions 构建 `FieldDefSet`，并向标准参数 pipeline 提供 direct/config strategy ids
- **THEN** pipeline 从 `schema_metadata()`、`strategy_metadata("direct")` 和 `strategy_metadata("config")` 派生 standard parameter identities、typed validation metadata 和 source paths
- **THEN** 普通路径不要求 caller 手工构造 catalog/index entry

#### Scenario: Direct/config role 映射到 typed-field strategies
- **WHEN** caller 将 direct role 绑定到 direct strategy id，并将 config role 绑定到 config strategy id
- **THEN** direct input 通过 direct strategy paths 读取
- **THEN** project/user config input 通过 config strategy paths 读取
- **THEN** default candidates 来自 typed-field defaults 和 caller-provided dynamic defaults，而不是第三个 extraction strategy role
- **THEN** field validation 继续使用同一份 typed-field metadata

#### Scenario: Catalog/index 是内部编译产物
- **WHEN** pipeline 已经从 typed-field metadata 派生出 catalog/index
- **THEN** source construction 使用该 catalog/index
- **THEN** 普通 caller 不需要手工构造中间 catalog/index

### Requirement: 标准参数解析从 constructed sources 产生 typed values
标准参数 resolver MUST 用 typed-field metadata 合并 constructed direct input、project config、user config 和 default sources，生成 typed runtime values，并保留 source information 与 validation diagnostic events。

#### Scenario: Direct input 覆盖低优先级 sources
- **WHEN** 同一 standard parameter identity 同时存在于 direct input、project config、user config 和 default sources
- **THEN** resolver 返回 direct input value 作为最终 typed value
- **THEN** resolver 记录 direct input 作为该最终值的 source info

#### Scenario: Project config 覆盖 user config 和 default
- **WHEN** standard parameter identity 不在 direct input 中，但同时存在于 project config 和 user config
- **THEN** resolver 返回 project config value 作为最终 typed value
- **THEN** resolver 记录 project config 作为该最终值的 source info

#### Scenario: Default 补齐缺失声明值
- **WHEN** standard parameter identity 没有 mapped direct input、project config 或 user config value
- **THEN** resolver 在存在 declared static 或 dynamic default 时使用 default
- **THEN** default result 通过与其它 mapped values 相同的 typed-field metadata validation

#### Scenario: Invalid mapped value 产生 validation diagnostic
- **WHEN** mapped source value 违反 declared typed-field kind、enum、range、requiredness 或 default constraint
- **THEN** resolver 为该 identity、source 和 validation failure 交接 diagnostic event
- **THEN** invalid mapped value 不暴露为 safe typed runtime value

### Requirement: Source construction 使用 pipeline-derived catalog/index
标准参数 source layer MUST 从 pipeline-derived catalog/index 构造 direct input、project config、user config 和 default sources，然后进入 resolution。

#### Scenario: Config JSON 通过 derived config path 映射
- **WHEN** project 或 user config JSON object 在 derived config path 上包含值
- **THEN** source construction 将该值映射到 catalog/index 中的 standard parameter identity
- **THEN** source 记录 project config 或 user config 作为 source kind

#### Scenario: Unmapped config field 保持 passthrough
- **WHEN** config JSON object 包含未映射到任何 derived standard parameter entry 的字段
- **THEN** source construction 不把该字段作为 standard parameter validation
- **THEN** source construction 接受 caller passthrough processing result 并原样交接
- **THEN** caller 可以用 raw-minus-mapped 处理函数删除已消费 mapped paths，并让剩余 JSON 子树按原结构和 entry passthrough policy 被 retained、discarded 或 delegated

#### Scenario: Direct input 通过 derived direct binding 映射
- **WHEN** direct CLI input 或 adapter invoke arguments 包含 direct input strategy 映射到的值
- **THEN** source construction 将该值映射到 derived catalog/index 中的 standard parameter identity 并标记为 direct input
- **THEN** unmapped direct input 保持在 standard parameter validation 之外

#### Scenario: Defaults 进入标准参数解析
- **WHEN** 字段存在 static default 或 caller-provided dynamic default
- **THEN** default handling 将该值作为 default source candidate
- **THEN** default 通过与其它 mapped source values 相同的 typed-field metadata validation

### Requirement: Config source loading 交接 skipped sources 且不拥有输出
标准参数 source layer MUST 加载 configured project/user config sources，按标准参数规则跳过 unavailable 或 invalid sources，并交接 diagnostic events；diagnostic formatting、output channel 和 exit behavior 仍由 entry owner 处理。

#### Scenario: Pipeline 拥有普通 config loading
- **WHEN** pipeline 收到 project/user config paths 或 source descriptors
- **THEN** 标准参数层读取 JSON，校验顶层 value 是 object，并构造 config source
- **THEN** caller 不需要为普通路径提供 separately loaded JSON

#### Scenario: Missing default config source 视为 absent
- **WHEN** default project 或 user config path 不存在
- **THEN** config source 被视为 absent
- **THEN** 该 missing default source 不返回 skipped-source diagnostic event

#### Scenario: Invalid explicit config source 被 skipped
- **WHEN** explicit project 或 user config override 缺失、不可读、invalid JSON 或不是 JSON object
- **THEN** config source 被 skipped
- **THEN** diagnostic handoff 包含 source level、path origin、path、reason code 和 operation-continued semantics
- **THEN** 其它可用 sources 继续进入 standard parameter resolution

#### Scenario: Loaded config 只复用标准参数 loading 结果
- **WHEN** caller 已经持有由 standard parameter config loader 产生的 loaded config source
- **THEN** pipeline 可以复用该 loaded source 而不重复读取同一文件
- **THEN** post-load source construction 和 diagnostic handoff semantics 与 path-based pipeline path 保持一致

### Requirement: Standard parameter passthrough 保持 owner-scoped
标准参数 resolver MUST 让 unmapped input 保持在 standard parameter validation 之外，并按 entry policy 返回 passthrough，使 owning CLI、adapter、protocol 或 config layer 可以 retain、discard、warn 或 validate。

#### Scenario: Unmapped input 不参与标准参数 validation
- **WHEN** input field 未映射到 standard parameter identity
- **THEN** standard parameter resolver 不把它作为 standard parameter validation
- **THEN** resolver 通过 entry policy 返回 caller passthrough processing result
- **THEN** raw-minus-mapped passthrough 由 caller processing function 产生，并保留未映射 JSON 子树结构

#### Scenario: Adapter native option 保持 delegated
- **WHEN** adapter direct CLI 或 invoke argument 包含没有 standard parameter mapping 的 native option
- **THEN** resolver 让该 option 保持在 typed-field standard parameter validation 之外
- **THEN** entry owner 仍负责 native option validation 或 ignored-argument warning

### Requirement: Operation argument binding 保留 source semantics
标准参数 resolver MUST 把 operation argument binding 建模为 standard parameter identity 到 protocol request `arguments` path 的映射，并保留 resolution 产生的 source info。

#### Scenario: Bound direct argument 可序列化到 protocol arguments
- **WHEN** direct input value 映射到带有 operation argument binding 的 standard parameter identity
- **THEN** binding 标识该 direct value 对应的 protocol request `arguments` path
- **THEN** resolver 保留该 value 的 direct source info

#### Scenario: Config 和 default values 保留 resolved source info
- **WHEN** final standard parameter value 来自 project config、user config 或 default
- **THEN** operation argument binding 保留 resolved source info
- **THEN** 是否将该 value 序列化或省略到 protocol request 仍由后续 request construction layer 拥有
