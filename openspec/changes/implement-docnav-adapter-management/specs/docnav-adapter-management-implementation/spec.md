## ADDED Requirements

### Requirement: 用户级安装 registry 必须按 id 和 version 保存 adapter
`docnav` MUST 在用户级安装 registry 中以 adapter id 为一级键、adapter version 为二级键保存安装记录；每个版本记录 MUST 包含可执行命令路径、install mode、manifest 快照、来源信息、健康状态和本地可执行文件 fingerprint 信息。

#### Scenario: 同一 adapter 多版本共存
- **WHEN** 用户安装同一 adapter id 的两个不同版本
- **THEN** 用户级 registry 保留两个 version 子记录
- **THEN** 两个版本的命令路径和 manifest 快照互不覆盖

### Requirement: 项目级 adapter 策略 registry 不得保存本机路径
项目级 adapter 策略 registry MUST 只保存 adapter id/version 级 allowlist、denylist 和当前版本选择，MUST NOT 保存可执行命令路径、绝对路径、用户名目录、来源 URL 或本地 fingerprint。

#### Scenario: 项目策略只引用 adapter id
- **WHEN** 项目启用 adapter allowlist 或 denylist
- **THEN** 策略条目只包含 adapter id 和可选 version
- **THEN** 项目文件中不出现本机 adapter 命令路径

### Requirement: allowlist 和 denylist 必须支持可选版本
项目级 adapter 策略 registry 中的 allowlist 和 denylist 条目 MUST 支持 adapter id 和可选 version；未指定 version 时 MUST 匹配该 id 的所有版本，指定 version 时 MUST 只匹配该 id 的该版本。

#### Scenario: denylist 未指定版本
- **WHEN** 项目 denylist 禁用 `docnav-markdown` 且未指定 version
- **THEN** 用户级 registry 中所有 `docnav-markdown` 版本都不得参与 adapter 选择

#### Scenario: allowlist 指定版本
- **WHEN** 项目 allowlist 只允许 `docnav-markdown@0.1.0`
- **THEN** `docnav-markdown@0.1.0` 可以参与 adapter 选择
- **THEN** `docnav-markdown` 的其它版本不得参与 adapter 选择

### Requirement: 当前版本选择必须约束 adapter id 的解析
项目级 adapter 策略 registry MAY 为 adapter id 指定当前使用 version；指定后，resolver MUST 只解析该 id 的该 version，且该 version 缺失、不可用、fingerprint 失配或协议不兼容时 MUST 返回结构化不可用错误。

#### Scenario: 当前版本不可用
- **WHEN** 项目选择 `docnav-markdown@0.1.0`
- **AND** 用户级 registry 中该版本不存在或不可用
- **THEN** resolver 返回结构化错误
- **THEN** resolver 不得静默选择 `docnav-markdown` 的其它版本

### Requirement: adapter install 必须只接受首期支持来源
`docnav adapter install <source>` MUST 只接受内置 adapter 下载简写和本地可执行文件来源；任意 URL、GitHub 链接和其它远程地址 MUST 失败且不得注册安装记录。

#### Scenario: 不支持的来源
- **WHEN** 调用方传入不属于内置下载简写或本地可执行文件的 source
- **THEN** install 返回结构化错误
- **THEN** 不写入 adapter 安装记录

#### Scenario: 任意 GitHub URL 不作为安装来源
- **WHEN** 调用方执行 `docnav adapter install https://github.com/example/adapter`
- **THEN** install 返回结构化错误
- **THEN** 错误 guidance 说明需要使用内置 source key 或本地可执行文件

### Requirement: adapter install 必须支持 managed 和 path 模式
`docnav adapter install <source>` MUST 支持 `--mode managed|path`；省略 `--mode` 时 MUST 使用 `managed`。`managed` 模式 MUST 将候选 adapter 可执行文件复制或原子移动到 `docnav` 管理的用户级 artifact 目录并记录托管后的命令路径；`path` 模式 MUST 不复制候选文件，只记录规范化绝对路径。managed artifact 目录 MUST 与配置文件目录分离。

#### Scenario: 本地 exe 使用默认 managed 模式
- **WHEN** 调用方执行 `docnav adapter install ./target/release/custom-adapter`
- **THEN** install 使用 `managed` 模式
- **THEN** 用户级 registry 记录的命令路径指向 `docnav` managed artifact 目录
- **THEN** 原始本地 exe 不被删除

#### Scenario: 本地 exe 使用 path 模式
- **WHEN** 调用方执行 `docnav adapter install ./target/debug/custom-adapter --mode path`
- **THEN** install 记录该本地 exe 的规范化绝对路径
- **THEN** install 不复制该本地 exe 到 managed artifact 目录

### Requirement: adapter register 必须等价于本地 path 模式安装
`docnav adapter register <local-exe>` MUST 作为 `docnav adapter install <local-exe> --mode path` 的语义别名；register MUST 只接受本地可执行文件来源。

#### Scenario: register 本地开发 adapter
- **WHEN** 调用方执行 `docnav adapter register ./target/debug/custom-adapter`
- **THEN** registry 中的 install mode 为 `path`
- **THEN** registry 中的命令路径为该文件的规范化绝对路径

### Requirement: 内置下载来源必须使用 managed 模式
内置 adapter 下载简写 install MUST 使用 `managed` 模式；调用方为内置下载 source 指定 `--mode path` 时 MUST 失败且不得注册安装记录。

#### Scenario: 内置下载来源拒绝 path 模式
- **WHEN** 调用方执行 `docnav adapter install markdown --mode path`
- **THEN** install 返回结构化错误
- **THEN** 不写入 adapter 安装记录

### Requirement: 安装必须校验 manifest 当前契约
adapter install MUST 执行候选 adapter 的 `manifest`，校验 manifest 当前 schema、必需字段、字段类型和语义后才注册。

#### Scenario: manifest schema 失败
- **WHEN** 候选 adapter manifest 不符合 schema
- **THEN** install 失败
- **THEN** 不注册该 adapter

### Requirement: SHA-256 fingerprint 必须低频验证
adapter install、register、update 和显式健康检查 MUST 记录或重新计算 SHA-256 fingerprint；fingerprint 算法 MUST 为 `sha256`，输入 MUST 为实际登记或托管的 adapter 可执行文件完整字节，保存值 MUST 为小写十六进制。普通 `outline`、`read`、`find` 和 `info` 操作 MUST NOT 为 fingerprint 校验读取整个 adapter 可执行文件。

SHA-256 fingerprint MUST 只用于检测已登记 adapter 文件是否发生静默变化，MUST NOT 被描述或实现为来源可信、作者可信或内容安全证明。

#### Scenario: 显式健康检查发现 fingerprint 失配
- **WHEN** 显式健康检查发现已安装 adapter 的当前 fingerprint 与安装记录不一致
- **THEN** `docnav` 将该 adapter 标记为不可用或返回不可用错误
- **THEN** 返回 `ADAPTER_UNAVAILABLE` 且 details reason 为 `hash_mismatch`

#### Scenario: 普通文档操作不刷新 fingerprint
- **WHEN** 调用方执行 `docnav outline <path>`
- **THEN** resolver 使用 registry 中的上次健康状态
- **THEN** resolver 不重新计算 adapter 可执行文件 fingerprint

### Requirement: 内置下载来源必须记录 source key 和解析制品
内置 adapter 下载来源 install MUST 记录 source key、解析后的制品信息、manifest 快照和可执行入口；下载得到的可执行制品 MUST 在校验通过后进入 managed artifact 目录。

#### Scenario: 内置下载来源无法解析
- **WHEN** 内置 source key 无法解析为可执行 adapter 制品
- **THEN** install 失败
- **THEN** 错误 guidance 说明当前支持的内置 source key

### Requirement: adapter list 必须展示安装和可用状态
`docnav adapter list` MUST 输出已安装 adapter 的 manifest 身份、version、支持格式、install mode、安装来源、项目策略命中和上次可用状态。默认 list MUST NOT 重新计算 fingerprint；显式健康检查模式 MAY 刷新 fingerprint 和可用状态。

#### Scenario: 列出本地 adapter
- **WHEN** 调用方执行 `docnav adapter list`
- **THEN** 输出包含 adapter id、version、格式、install mode、来源、策略状态和上次可用状态
- **THEN** 不重新计算 fingerprint

### Requirement: adapter update 必须先验证后替换
`docnav adapter update [adapter-id] [--version <version>]` MUST 使用已记录来源获取或重新验证候选版本，并 MUST 在 manifest 当前 schema、语义和 fingerprint 校验全部通过后才替换旧记录。

#### Scenario: update 校验失败
- **WHEN** 新候选 adapter 校验失败
- **THEN** update 返回结构化错误
- **THEN** 旧安装记录保持不变

#### Scenario: update 候选版本不匹配
- **WHEN** 调用方更新 `docnav-markdown@0.1.0`
- **AND** 新候选 manifest 声明的 id 或 version 不匹配
- **THEN** update 失败
- **THEN** 不覆盖其它 id/version 记录

#### Scenario: path 模式 update
- **WHEN** 调用方更新 install mode 为 `path` 的 adapter
- **THEN** update 重新执行记录路径的 manifest 校验
- **THEN** update 使用记录路径的当前文件字节刷新 fingerprint

#### Scenario: managed 本地来源缺失
- **WHEN** 调用方更新 install mode 为 `managed` 且来源为本地 exe 的 adapter
- **AND** 原始来源路径已经不可用
- **THEN** update 失败
- **THEN** 错误 guidance 说明需要重新安装或提供新的来源

### Requirement: adapter remove 必须清理安装记录并处理配置引用
`docnav adapter remove <adapter-id> [--version <version>]` MUST 注销 adapter 并清理 `docnav` 管理的用户级安装 registry 记录；若目标 id 或 id/version 仍被项目策略 registry 引用，MUST 失败或给出明确 guidance。

#### Scenario: remove 被项目策略引用的 adapter
- **WHEN** adapter id 或 id/version 仍被项目 allowlist、denylist 或当前版本选择引用
- **THEN** remove 不得静默删除
- **THEN** 输出说明需要先移除或修改引用

#### Scenario: remove path 模式 adapter
- **WHEN** remove 删除 install mode 为 `path` 的 adapter 记录
- **THEN** remove 清理 registry 记录
- **THEN** remove 不删除 registry 命令路径指向的用户文件

#### Scenario: remove managed 模式 adapter
- **WHEN** remove 删除 install mode 为 `managed` 的 adapter 记录
- **THEN** remove 清理 registry 记录
- **THEN** remove 清理 `docnav` 管理的对应 artifact 文件
