本 change 目标是将分页默认值统一收敛到 `defaults.pagination`，并让 core `docnav` 通过标准参数定义映射配置和 CLI flag，在进入 adapter invoke 前初始化最终 `limit_chars`；本文档只是 `openspec/changes/configure-pagination-defaults/` 下的未审核临时 core-cli delta，不影响现有其它文档或主规范。

## MODIFIED Requirements

### Requirement: 核心 CLI 必须实现文档操作命令
`docnav` MUST 实现 `outline`、`read`、`find` 和 `info`，并 MUST 支持各命令对应的 `--adapter`、`--page`、`--pagination enabled|disabled`、`--limit-chars` 和 `--output` 参数约束。`info` invoke 请求 MUST 只包含 info operation 需要的 core 参数。

Core document command 中可由配置文件和 CLI flag 共同提供的标准参数 MUST 由 core 标准参数定义声明。每个定义 MUST 包含 canonical path、value kind/validation、argv flag/parser、config projection、operation applicability、default source、source priority、help/context 元数据和 finalization rule。CLI argv 和配置文件 MUST 只作为同一标准参数的不同来源进入统一归一流程；parser、config projection、help、supported key listing 和 context 输出 MUST 消费同一个 core 标准参数定义。

`--pagination enabled|disabled` MUST 只适用于分页操作。`--pagination` MUST 映射为标准参数 `defaults.pagination.enabled` 的显式来源；`--limit-chars` MUST 映射为标准参数 `defaults.pagination.limit_chars` 的显式来源。`--pagination enabled` MUST 使本次调用使用解析后的 pagination limit。`--pagination disabled` MUST 使本次调用不启用分页，并在进入 invoke 前把最终 `limit_chars` 初始化为协议 `PositiveInteger` 可表示的最大值。`--pagination disabled` 与同一次调用中的 `--limit-chars` MUST 被接受，并 MUST 按“配置提供 `enabled: false` 和 `limit_chars`”相同的标准参数归一规则处理。

#### Scenario: 执行 outline 命令
- **WHEN** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 解析最终 page、pagination enabled 状态和 limit_chars
- **THEN** `docnav` 启动选中 adapter 的 invoke

#### Scenario: 标准参数定义驱动 flag 和配置映射
- **WHEN** core 注册 `defaults.pagination.enabled` 标准参数定义
- **THEN** `--pagination enabled|disabled`、配置 key `defaults.pagination.enabled`、help 文案、supported key listing 和 document context 输出都引用该定义
- **THEN** `docnav` 通过同一个 core 标准参数定义驱动该参数的 CLI/config/help/context 映射

#### Scenario: 执行 pagination disabled 命令
- **WHEN** 调用方执行 `docnav outline docs/guide.md --pagination disabled`
- **THEN** `docnav` 将最终 `limit_chars` 初始化为协议 `PositiveInteger` 可表示的最大值
- **THEN** `docnav` 启动选中 adapter 的 invoke

#### Scenario: pagination enabled 使用显式预算
- **WHEN** 调用方执行 `docnav outline docs/guide.md --pagination enabled --limit-chars 120`
- **THEN** `docnav` 使用显式 argv 值 `120` 作为最终 `limit_chars`
- **THEN** `docnav` 启动选中 adapter 的 invoke

#### Scenario: pagination disabled 接受显式预算来源
- **WHEN** 调用方执行 `docnav outline docs/guide.md --pagination disabled --limit-chars 120`
- **THEN** `docnav` 将 `defaults.pagination.enabled` 解析为显式 `false`
- **THEN** `docnav` 将 `defaults.pagination.limit_chars` 解析为显式 `120`
- **THEN** `docnav` 仍将最终 adapter request 的 `limit_chars` 初始化为协议 `PositiveInteger` 可表示的最大值

#### Scenario: 执行 info 命令
- **WHEN** 调用方执行 `docnav info docs/guide.md`
- **THEN** `docnav` 启动选中 adapter 的 invoke
- **THEN** invoke 请求不包含 page、pagination 或 limit_chars

### Requirement: 核心配置 MVP 必须有限且可审计
`docnav` MUST 在本 change 中支持 `defaults.adapter`、`defaults.pagination.enabled`、`defaults.pagination.limit_chars` 和 `defaults.output` 四类核心配置 key，并 MUST 按显式参数、项目配置、用户配置、内置默认值的优先级解析最终 core 参数值。Core 参数的默认值来源 MUST 限定为 CLI、配置和内置默认值；adapter manifest 只参与 adapter 识别和当前契约校验。

Core supported config keys MUST 由 core 标准参数定义提供。`config get/set/unset/list`、document context 输出和 help/default 文案 MUST 消费同一组定义，以保持新增标准参数时的配置路径、CLI flag、验证规则和展示行为一致。

`defaults.pagination.enabled` MUST 默认为 `true`。`defaults.pagination.limit_chars` MUST 是正整数，默认值 MUST 为 `6000`。当最终 `defaults.pagination.enabled` 为 `false` 时，`docnav` MUST 在进入 invoke 前将最终 adapter request 的 `limit_chars` 初始化为协议 `PositiveInteger` 可表示的最大值，并 MUST 对外表达为本次不启用分页；该规则不受 `defaults.pagination.limit_chars` 来源影响。`page` MUST NOT 成为配置默认值；省略 `--page` 时仍固定为 `1`。

#### Scenario: 配置 adapter 预选
- **WHEN** 调用方未传入 `--adapter`
- **AND** 项目配置设置了 `defaults.adapter`
- **THEN** `docnav` 使用该 adapter id 作为预选 adapter

#### Scenario: page 不可配置
- **WHEN** 调用方省略 `--page`
- **THEN** `docnav` 使用 page `1`
- **THEN** 项目配置和用户配置保持初始 page 为 `1`

#### Scenario: pagination limit_chars 作为默认字符预算
- **WHEN** 调用方未传入 `--limit-chars`
- **AND** 项目配置设置了 `defaults.pagination.enabled: true`
- **AND** 项目配置设置了 `defaults.pagination.limit_chars: 120`
- **THEN** `docnav` 将最终 `limit_chars` 解析为 `120`
- **THEN** 该值在进入 adapter invoke 前已经显式写入请求

#### Scenario: pagination disabled 初始化最大预算
- **WHEN** 最终 `defaults.pagination.enabled` 为 `false`
- **THEN** `docnav` 将最终 `limit_chars` 初始化为协议 `PositiveInteger` 可表示的最大值
- **THEN** adapter invoke 请求仍包含 `limit_chars`
- **THEN** adapter 继续使用现有 page 语义表达是否还有更多内容

#### Scenario: 显式 limit_chars 不隐式启用分页
- **WHEN** 生效配置设置了 `defaults.pagination.enabled: false`
- **AND** 调用方执行 `docnav outline docs/guide.md --limit-chars 120`
- **THEN** `docnav` 将 `defaults.pagination.limit_chars` 解析为显式 `120`
- **THEN** `docnav` 保持 `defaults.pagination.enabled: false`
- **THEN** `docnav` 将最终 adapter request 的 `limit_chars` 初始化为协议 `PositiveInteger` 可表示的最大值

#### Scenario: 显式 pagination 覆盖配置
- **WHEN** 生效配置设置了 `defaults.pagination.enabled: false`
- **AND** 调用方执行 `docnav outline docs/guide.md --pagination enabled`
- **THEN** `docnav` 启用本次调用的分页
- **THEN** `docnav` 使用生效 `defaults.pagination.limit_chars` 或内置默认预算

#### Scenario: 未知配置 key
- **WHEN** 调用方执行 `docnav config get unknown.key`
- **THEN** `docnav` 返回 `INVALID_REQUEST`

#### Scenario: 默认写项目配置
- **WHEN** 调用方执行 `docnav config set defaults.output readable-json`
- **THEN** `docnav` 写入 `<project-root>/.docnav/docnav.json`
- **THEN** 当前项目的生效配置包含 `defaults.output`

#### Scenario: 写 pagination 项目配置
- **WHEN** 调用方执行 `docnav config set defaults.pagination.enabled false`
- **THEN** `docnav` 写入 `<project-root>/.docnav/docnav.json`
- **THEN** 当前项目的生效配置包含 `defaults.pagination.enabled: false`

#### Scenario: 写用户配置
- **WHEN** 调用方执行 `docnav config set defaults.output readable-json --user`
- **THEN** `docnav` 写入用户配置文件
- **THEN** 未设置项目同名 key 时该用户配置成为生效值

#### Scenario: 列出当前生效配置
- **WHEN** 调用方执行 `docnav config list`
- **THEN** 输出包含所有支持 key 的当前生效值
- **THEN** 输出标明每个值来自项目配置、用户配置、内置默认值或未设置状态
- **THEN** 输出包含 `defaults.pagination.enabled` 和 `defaults.pagination.limit_chars`

#### Scenario: 按文档上下文列出最终配置
- **WHEN** 调用方执行 `docnav config list --path docs/guide.md --operation outline`
- **THEN** `docnav` 按文档命令规则解析 path 并选择 adapter
- **THEN** 输出包含选中 adapter id
- **THEN** 输出包含该文档和 operation 下的最终默认参数及其来源
- **THEN** 输出包含 pagination 配置和由其初始化后的最终 `limit_chars`

### Requirement: invoke 请求必须包含最终显式参数
`docnav` MUST 在启动 adapter invoke 前解析配置和默认值，并 MUST 将最终 page、limit_chars、ref 和 query 等 core 通用参数显式写入 invoke 请求。Core `docnav` MUST NOT 合成 format-specific `options`；adapter manifest MUST NOT 作为 options 或 default-parameter source。

当最终 `defaults.pagination.enabled` 为 `false` 时，`docnav` MUST 在构造 invoke 请求前把本次 adapter request 的 `limit_chars` 归一为协议 `PositiveInteger` 可表示的最大值。显式 `--pagination disabled` MUST 触发同样的归一。该归一只改变进入 adapter 的预算值，MUST NOT 改变 protocol argument 字段、response `page` 字段或 readable output shape。

#### Scenario: 省略 page
- **WHEN** 调用方省略 page
- **THEN** `docnav` 传给 invoke 的 page 为 `1`

#### Scenario: 不写入格式 options
- **WHEN** 选中 adapter manifest 通过当前 schema 校验
- **THEN** `docnav` 仍能解析 core 默认参数
- **THEN** invoke 请求不包含由 manifest、配置或隐式默认值合成的格式 options

#### Scenario: disabled pagination 不改变 protocol shape
- **WHEN** 生效配置设置了 `defaults.pagination.enabled: false`
- **AND** 调用方执行 `docnav read docs/guide.md --ref H:L1:H1 --output protocol-json`
- **THEN** `docnav` 在请求 arguments 中写入正整数 `limit_chars`
- **THEN** 请求 arguments 中仍写入正整数 `page`
- **THEN** 请求 arguments 不包含 `pagination` 字段

#### Scenario: invoke 当前契约不一致
- **WHEN** 选中 adapter 的 invoke 输出字段缺失、类型不符、operation/result shape 不匹配或语义校验失败
- **THEN** `docnav` 返回 adapter/protocol 错误
- **THEN** `docnav` 不继续尝试其它 adapter
