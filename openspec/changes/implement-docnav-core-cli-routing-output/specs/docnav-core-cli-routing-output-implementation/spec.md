## ADDED Requirements

### Requirement: 核心 CLI 必须实现文档操作命令
`docnav` MUST 实现 `outline`、`read`、`find` 和 `info`，并 MUST 支持对应的 `--format`、`--page`、`--limit-chars` 和 `--output` 参数约束。

#### Scenario: 执行 outline 命令
- **WHEN** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 解析最终 page 和 limit_chars
- **THEN** `docnav` 启动选中 adapter 的 invoke

### Requirement: path 必须规范化并限制在项目根内
`docnav` MUST 将输入 path 解析为使用 `/` 的项目相对路径，并 MUST 拒绝指向项目根外的路径。

#### Scenario: 路径逃逸
- **WHEN** 调用方传入会解析到项目根外的 path
- **THEN** `docnav` 返回输入错误
- **THEN** 不启动 adapter 进程

### Requirement: adapter 选择必须按阶段校验
`docnav` MUST 按显式 format/content type、扩展名候选、全量 probe 的顺序选择 adapter，并 MUST 以 adapter 校验结果为准。

#### Scenario: 显式 format 失败后继续
- **WHEN** 调用方传入 `--format markdown` 但候选 adapter 校验失败
- **THEN** `docnav` 保留失败证据
- **THEN** `docnav` 继续进入扩展名匹配阶段

#### Scenario: 所有阶段失败
- **WHEN** 没有 adapter 能校验目标文档
- **THEN** `docnav` 返回 `FORMAT_UNKNOWN`
- **THEN** 错误 details 包含候选证据

### Requirement: invoke 请求必须包含最终显式参数
`docnav` MUST 在启动 adapter invoke 前解析配置和默认值，并 MUST 将最终 page、limit_chars、ref、query 和 options 显式写入 invoke 请求。

#### Scenario: 省略 page
- **WHEN** 调用方省略 page
- **THEN** `docnav` 传给 invoke 的 page 为 `1`

### Requirement: 输出模式必须按协议层和阅读层分离
`docnav --output protocol-json` MUST 输出完整原始协议 envelope；默认 text 和 readable-json MUST 输出阅读层结果，且 MUST NOT 包含 `protocol_version`、`request_id`、`operation` 或 `ok`。

#### Scenario: readable-json outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md --output readable-json`
- **THEN** 输出只包含 entries 和 page
- **THEN** 输出不包含 protocol envelope 字段

#### Scenario: protocol-json read
- **WHEN** 调用方执行 `docnav read docs/guide.md --ref <ref> --output protocol-json`
- **THEN** 输出包含完整 protocol response envelope

### Requirement: 核心 CLI 必须保留 adapter 业务语义
`docnav` MUST 在输出映射中保留 adapter 返回的 ref、display、content、content_type、cost 和 page。

#### Scenario: read 保留 content_type
- **WHEN** adapter read 返回 `content_type: "text/markdown"`
- **THEN** `docnav` readable-json read 输出包含相同 content_type

### Requirement: 核心 CLI 必须实现稳定错误和退出码映射
`docnav` MUST 将输入错误、文档/ref/格式错误、协议或 adapter 进程错误、内部错误映射到主规范定义的退出码，并 MUST 保持错误 code 稳定。

#### Scenario: ref 不存在
- **WHEN** adapter 返回 `REF_NOT_FOUND`
- **THEN** `docnav` 保留该错误 code
- **THEN** CLI 退出码为文档/ref/格式错误对应值
