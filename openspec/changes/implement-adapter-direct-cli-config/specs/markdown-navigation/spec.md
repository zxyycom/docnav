本 delta 定义 `docnav-markdown` 作为首个 adapter direct CLI 配置使用者的配置键和验证边界；它只在 `openspec/changes/implement-adapter-direct-cli-config/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: docnav-markdown direct CLI 支持 JSON 配置文件
`docnav-markdown` direct CLI MUST 读取项目级 `.docnav/docnav-markdown.json` 和用户级 `docnav-markdown.json` 配置，并 MUST 支持 SDK-owned `--project-config-path <path>` 和 `--user-config-path <path>` 覆盖这两个配置文件路径。首期配置 MUST 支持 `defaults.limit_chars`、`defaults.output` 和 `options.max_heading_level`。

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

### Requirement: docnav-markdown 配置必须由 smoke 和矩阵测试覆盖
`docnav-markdown` black-box CLI smoke 和负向矩阵 MUST 覆盖配置文件读取、优先级、非法配置和 invoke 不读配置的边界。

#### Scenario: Smoke 覆盖配置优先级
- **WHEN** smoke suite 使用项目级和用户级 `docnav-markdown.json`
- **THEN** 测试证明显式 argv 覆盖项目级配置
- **THEN** 项目级配置覆盖用户级配置
- **THEN** 用户级配置覆盖内置默认值
- **THEN** 测试证明 `outline` 和 `find` 都消费适用的 `options.max_heading_level`

#### Scenario: Smoke 覆盖配置路径覆盖
- **WHEN** smoke suite 提供默认配置路径和覆盖配置路径
- **THEN** 测试证明覆盖路径中的配置参与合并
- **THEN** 被覆盖的默认路径不参与本次合并

#### Scenario: 负向矩阵覆盖非法配置
- **WHEN** smoke 或矩阵 fixture 提供非法 `defaults.limit_chars`、非法 `defaults.output` 或非法 `options.max_heading_level`
- **THEN** `docnav-markdown` document operation 非零退出
- **THEN** 错误 payload 或诊断包含配置 key 和失败原因
- **THEN** `--output protocol-json` 下的非法配置仍输出 protocol failure envelope

#### Scenario: Invoke 不受配置影响
- **WHEN** 项目级 `docnav-markdown.json` 设置 `options.max_heading_level`
- **AND** smoke suite 通过 `docnav-markdown invoke` 提交未携带 options 的 outline request
- **THEN** invoke path 不读取该配置
- **THEN** 行为只由 stdin request 和 adapter 内部协议默认边界决定
