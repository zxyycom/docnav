## ADDED Requirements

### Requirement: adapter install 必须只接受首期支持来源
`docnav adapter install <source>` MUST 只接受 GitHub 链接和本地可执行文件来源；其它来源 MUST 失败且不得注册安装记录。

#### Scenario: 不支持的来源
- **WHEN** 调用方传入不属于 GitHub 链接或本地可执行文件的 source
- **THEN** install 返回结构化错误
- **THEN** 不写入 adapter 安装记录

### Requirement: 安装必须校验 manifest 和协议兼容
adapter install MUST 执行候选 adapter 的 `manifest`，校验 manifest schema，并确认协议范围与 `docnav` 兼容后才注册。

#### Scenario: manifest schema 失败
- **WHEN** 候选 adapter manifest 不符合 schema
- **THEN** install 失败
- **THEN** 不注册该 adapter

### Requirement: 本地可执行文件必须记录并验证 SHA-256 hash
本地可执行文件来源 install MUST 记录 SHA-256 hash；list 健康检查、update 和运行前检查 MUST 重新计算 hash，hash 不一致时 MUST 返回不可用错误。

#### Scenario: 本地 exe hash 失配
- **WHEN** 已安装本地 adapter 的当前 hash 与安装记录不一致
- **THEN** `docnav` 不得继续调用该 adapter
- **THEN** 返回 `ADAPTER_UNAVAILABLE` 且 details reason 为 `hash_mismatch`

### Requirement: GitHub 来源必须记录原始 URL 和解析制品
GitHub 来源 install MUST 记录来源 URL、解析后的制品信息、manifest 快照和可执行入口。

#### Scenario: GitHub 来源无法解析
- **WHEN** GitHub source 无法解析为可执行 adapter 制品
- **THEN** install 失败
- **THEN** 错误 guidance 说明当前支持的 GitHub 发布形态

### Requirement: adapter list 必须展示安装和可用状态
`docnav adapter list` MUST 输出已安装 adapter 的 manifest 身份、支持格式、协议范围、安装来源和可用状态。

#### Scenario: 列出本地 adapter
- **WHEN** 调用方执行 `docnav adapter list`
- **THEN** 输出包含 adapter id、格式、来源和可用状态

### Requirement: adapter update 必须先验证后替换
`docnav adapter update [adapter-id]` MUST 使用已记录来源获取或重新验证候选版本，并 MUST 在 manifest、schema、协议兼容和来源校验全部通过后才替换旧记录。

#### Scenario: update 校验失败
- **WHEN** 新候选 adapter 校验失败
- **THEN** update 返回结构化错误
- **THEN** 旧安装记录保持不变

### Requirement: adapter remove 必须清理安装记录并处理配置引用
`docnav adapter remove <adapter-id>` MUST 注销 adapter 并清理 `docnav` 管理的安装记录；若仍被项目配置显式引用，MUST 失败或给出明确 guidance。

#### Scenario: remove 被项目配置引用的 adapter
- **WHEN** adapter 仍被项目配置显式引用
- **THEN** remove 不得静默删除
- **THEN** 输出说明需要先移除或修改引用
