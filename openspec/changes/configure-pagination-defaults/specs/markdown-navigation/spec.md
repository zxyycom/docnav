本 change 目标是将分页默认值统一收敛到 `defaults.pagination`，让 `docnav-markdown` direct CLI 通过 SDK direct CLI pagination 参数处理消费统一分页配置；本文档只是 `openspec/changes/configure-pagination-defaults/` 下的未审核临时 markdown-navigation delta，不影响现有其它文档或主规范。

## MODIFIED Requirements

### Requirement: docnav-markdown direct CLI 支持 JSON 配置文件
`docnav-markdown` direct CLI MUST 读取项目级 `.docnav/docnav-markdown.json` 和默认用户配置目录下的 `docnav-markdown.json` 配置，并 MUST 支持 SDK-owned `--project-config-path <path>` 和 `--user-config-path <path>` 覆盖这两个配置文件路径；默认用户配置目录未提供时使用当前调用位置（启动 cwd）。首期配置 MUST 支持 `defaults.pagination.enabled`、`defaults.pagination.limit_chars`、`defaults.output` 和 `options.max_heading_level`。Document operation help MUST 展示两个配置路径参数和 `--pagination enabled|disabled`。导致配置源被跳过的读取失败 MUST 产生 direct CLI warning，同时 operation MUST 使用其余来源继续执行。

`docnav-markdown` direct CLI MUST 消费 SDK direct CLI pagination 参数规则中的 `defaults.pagination.enabled`、`defaults.pagination.limit_chars`，并继续通过现有 output 处理规则消费 `defaults.output`。`options.max_heading_level` 仍是 adapter-owned `options` object 下的 Markdown native option，并 MUST 继续使用 native option registration 和 operation applicability handling。

#### Scenario: Markdown direct CLI 通过 SDK pagination 参数规则消费 pagination
- **WHEN** SDK 支持 `defaults.pagination.enabled` 和 `defaults.pagination.limit_chars`
- **THEN** `docnav-markdown` help、argv parsing 和 config projection 使用这些规则处理 `--pagination enabled|disabled` 和 `--limit-chars`
- **THEN** Markdown-specific code 只把 `options.max_heading_level` 作为 native option 处理

#### Scenario: pagination limit_chars 来自配置
- **WHEN** `.docnav/docnav-markdown.json` 包含 `defaults.pagination.enabled: true`
- **AND** `.docnav/docnav-markdown.json` 包含 `defaults.pagination.limit_chars: 120`
- **AND** 调用方执行 `docnav-markdown outline docs/guide.md` 且未传入 `--limit-chars`
- **THEN** outline 使用 `120` 作为本次字符预算
- **THEN** 该结果与显式传入 `--limit-chars 120` 的分页行为一致

#### Scenario: pagination disabled 来自配置
- **WHEN** `.docnav/docnav-markdown.json` 包含 `defaults.pagination.enabled: false`
- **AND** 调用方执行 `docnav-markdown read docs/guide.md --ref H:L1:H1` 且未传入 `--limit-chars`
- **THEN** direct CLI 在进入 Markdown read 前使用协议 `PositiveInteger` 可表示的最大值
- **THEN** 对外行为是不按默认字符预算拆分该 read 结果

#### Scenario: 显式 limit_chars 不隐式启用分页
- **WHEN** `.docnav/docnav-markdown.json` 包含 `defaults.pagination.enabled: false`
- **AND** 调用方执行 `docnav-markdown outline docs/guide.md --limit-chars 120`
- **THEN** direct CLI 将 `defaults.pagination.limit_chars` 解析为显式 `120`
- **THEN** direct CLI 保持 `defaults.pagination.enabled: false`
- **THEN** direct CLI 在进入 Markdown outline 前使用协议 `PositiveInteger` 可表示的最大值

#### Scenario: 显式 pagination 覆盖配置
- **WHEN** `.docnav/docnav-markdown.json` 包含 `defaults.pagination.enabled: false`
- **AND** 调用方执行 `docnav-markdown outline docs/guide.md --pagination enabled`
- **THEN** direct CLI 启用本次调用的分页
- **THEN** direct CLI 使用生效 `defaults.pagination.limit_chars` 或内置默认预算

#### Scenario: 显式 pagination disabled 接受显式预算来源
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md --pagination disabled --limit-chars 120`
- **THEN** direct CLI 将 `defaults.pagination.enabled` 解析为显式 `false`
- **THEN** direct CLI 将 `defaults.pagination.limit_chars` 解析为显式 `120`
- **THEN** direct CLI 在进入 Markdown outline 前使用协议 `PositiveInteger` 可表示的最大值

#### Scenario: max_heading_level 来自配置
- **WHEN** `.docnav/docnav-markdown.json` 包含 `options.max_heading_level: 2`
- **AND** 调用方执行 `docnav-markdown outline docs/guide.md`
- **THEN** outline 只显示当前 max_heading_level 下可见的 heading entries
- **THEN** 该结果与显式传入 `--max-heading-level 2` 的行为一致

#### Scenario: find 使用配置中的 max_heading_level
- **WHEN** `.docnav/docnav-markdown.json` 包含 `options.max_heading_level: 2`
- **AND** 调用方执行 `docnav-markdown find docs/guide.md --query install`
- **THEN** find 只搜索当前 max_heading_level 下可见的 heading entries
- **THEN** 该结果与显式传入 `--max-heading-level 2` 的行为一致

#### Scenario: 显式 max_heading_level 覆盖配置
- **WHEN** `.docnav/docnav-markdown.json` 包含 `options.max_heading_level: 2`
- **AND** 调用方执行 `docnav-markdown outline docs/guide.md --max-heading-level 4`
- **THEN** direct CLI 使用显式 argv 值 `4`
- **THEN** 配置值 `2` 不覆盖显式 argv

#### Scenario: 配置路径覆盖生效
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md --project-config-path fixtures/project.json`
- **AND** 覆盖路径中的 JSON 包含 `options.max_heading_level: 2`
- **AND** 默认项目级路径中的 JSON 包含 `options.max_heading_level: 4`
- **THEN** direct CLI 使用覆盖路径中的 `max_heading_level: 2`
- **THEN** 默认项目级路径中的值不参与本次合并

#### Scenario: output 默认值来自配置
- **WHEN** `.docnav/docnav-markdown.json` 包含 `defaults.output: "readable-json"`
- **AND** 调用方执行 `docnav-markdown info docs/guide.md` 且未传入 `--output`
- **THEN** stdout 使用 readable-json 输出
- **THEN** 输出不使用 readable-view block framing

#### Scenario: Help 展示配置路径参数
- **WHEN** 调用方执行 `docnav-markdown outline --help`
- **THEN** help 输出包含 `--project-config-path <path>`、`--user-config-path <path>` 和 `--pagination enabled|disabled`
- **THEN** help 不读取 `.docnav/docnav-markdown.json`

### Requirement: docnav-markdown 配置必须由 smoke 和矩阵测试覆盖
`docnav-markdown` black-box CLI smoke 和矩阵 MUST 覆盖配置文件读取、优先级、配置源不可用时继续合并并输出 warning、help 参数展示、pagination 默认值和 invoke 不读配置的边界。

#### Scenario: Smoke 覆盖配置优先级
- **WHEN** smoke suite 使用项目级配置和默认用户配置目录下的 `docnav-markdown.json`
- **THEN** 测试证明显式 argv 覆盖项目级配置
- **THEN** 项目级配置覆盖用户级配置
- **THEN** 用户级配置覆盖内置默认值
- **THEN** 测试证明 `outline` 和 `find` 都消费适用的 `options.max_heading_level`

#### Scenario: Smoke 覆盖 pagination 配置
- **WHEN** smoke suite 使用设置了 `defaults.pagination.limit_chars` 的项目级配置
- **THEN** 测试证明 direct CLI 使用该字符预算分页
- **WHEN** smoke suite 使用设置了 `defaults.pagination.enabled: false` 的项目级配置
- **THEN** 测试证明 direct CLI 对外表现为不启用默认分页

#### Scenario: Smoke 覆盖 pagination argv
- **WHEN** smoke suite 执行 `docnav-markdown outline <path> --pagination disabled`
- **THEN** 测试证明 direct CLI 使用 `PositiveInteger` 最大值作为最终字符预算
- **WHEN** smoke suite 执行 `docnav-markdown outline <path> --pagination enabled --limit-chars 120`
- **THEN** 测试证明 direct CLI 使用显式预算分页
- **WHEN** smoke suite 执行 `docnav-markdown outline <path> --pagination disabled --limit-chars 120`
- **THEN** 测试证明 direct CLI 接受两个显式参数来源，并使用 `PositiveInteger` 最大值作为最终字符预算

#### Scenario: Smoke 覆盖配置路径覆盖
- **WHEN** smoke suite 提供默认配置路径和覆盖配置路径
- **THEN** 测试证明覆盖路径中的配置参与合并
- **THEN** 被覆盖的默认路径不参与本次合并

#### Scenario: 矩阵覆盖配置源不可用
- **WHEN** smoke 或矩阵 fixture 提供语法无效的 JSON 配置源
- **AND** 其它配置来源或内置默认值可用
- **THEN** `docnav-markdown` 继续按其余来源合并 direct CLI 参数来源对象
- **THEN** 测试证明配置源跳过 warning 出现在当前输出模式允许的 warning 通道

#### Scenario: 矩阵覆盖显式配置路径不可用
- **WHEN** smoke 或矩阵 fixture 显式传入不存在或不可读的 `--project-config-path`
- **THEN** 覆盖后的项目级配置源不参与本次合并
- **THEN** 用户级配置和内置默认值仍可参与 direct CLI 参数来源对象合并
- **THEN** 测试证明配置源跳过 warning 出现在当前输出模式允许的 warning 通道

#### Scenario: Invoke 不受配置影响
- **WHEN** 项目级 `docnav-markdown.json` 设置 `options.max_heading_level`
- **AND** 项目级 `docnav-markdown.json` 设置 `defaults.pagination.enabled: false`
- **AND** smoke suite 通过 `docnav-markdown invoke` 提交未携带 options 的 outline request
- **THEN** invoke path 不读取该配置
- **THEN** 行为只由 stdin request 中显式携带的参数和 adapter document operation 线路决定

### Requirement: docnav-markdown 配置提供 schema 和示例参考
`docs/schemas/docnav-markdown-config.schema.json` MUST 描述 `docnav-markdown` JSON 配置文件的参考 shape，包含 `defaults.pagination.enabled`、`defaults.pagination.limit_chars`、`defaults.output` 和 `options.max_heading_level`。`docs/examples/json/docnav-markdown-config.json` MUST 提供符合该 schema 的配置示例。该 schema/example MUST 用于文档校验、编辑器提示或 adapter package 打包参考，MUST NOT 改变 adapter direct CLI runtime 是否读取或校验配置文件。

#### Scenario: 配置示例通过 schema 校验
- **WHEN** docs validator 校验 `docs/examples/json/docnav-markdown-config.json`
- **THEN** 示例符合 `docs/schemas/docnav-markdown-config.schema.json`
- **THEN** schema 约束 `defaults.pagination.enabled` 为 boolean
- **THEN** schema 约束 `defaults.pagination.limit_chars` 为正整数
- **THEN** schema 约束 `defaults.output` 为 direct CLI output mode
- **THEN** schema 约束 `options.max_heading_level` 为 1 到 6 的整数

#### Scenario: schema 不改变 direct CLI runtime 行为
- **WHEN** adapter direct CLI 读取 `docnav-markdown.json`
- **THEN** runtime 不要求加载 `docs/schemas/docnav-markdown-config.schema.json`
- **THEN** 配置读取和 direct CLI 参数处理链路仍由 adapter direct CLI 实现负责
