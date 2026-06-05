# protocol-and-adapter-sdk-implementation Specification

## Purpose
定义 Docnav v0 原始协议共享类型、协议兼容判断、adapter SDK invoke 生命周期，以及 schema 和示例验证的实现要求。
## Requirements
### Requirement: 共享协议类型完整覆盖 v0 原始协议
`docnav-protocol` MUST 定义 v0 request envelope、response envelope、operation、operation arguments、operation result、page、stable error、manifest 和 probe 的共享类型，并 MUST 不包含格式专属解析字段。

#### Scenario: 构造 outline 成功响应
- **WHEN** 调用方使用共享协议类型构造 `outline` 成功响应
- **THEN** 响应包含 `protocol_version`、`request_id`、`operation: "outline"`、`ok: true` 和 outline result
- **THEN** result 只包含扁平 entries 和 page

#### Scenario: 拒绝格式专属字段进入共享协议
- **WHEN** 实现者需要表达 Markdown heading path
- **THEN** 该信息只能存在于 adapter 生成的 `ref` 或 `display`
- **THEN** `docnav-protocol` 不新增 Markdown 专属 result 字段

### Requirement: 协议版本兼容必须按闭区间判断
协议兼容判断 MUST 使用 `docnav` 支持范围与 adapter manifest 协议范围的闭区间交集，并 MUST 在无交集时产生 `PROTOCOL_INCOMPATIBLE`。

#### Scenario: 选择最高兼容版本
- **WHEN** `docnav` 支持 `0.1` 到 `0.2` 且 adapter 支持 `0.1` 到 `0.1`
- **THEN** 兼容版本为 `0.1`

#### Scenario: 无协议交集
- **WHEN** `docnav` 与 adapter 的协议范围没有交集
- **THEN** 协议层返回 `PROTOCOL_INCOMPATIBLE`
- **THEN** 错误 details 包含参与判断的版本范围

### Requirement: operation 必须绑定成功 result 类型
protocol response schema 和共享校验 MUST 使用响应 `operation` 绑定成功 result 类型，且成功响应 operation MUST 与请求 operation 一致。

#### Scenario: read 响应绑定 ReadResult
- **WHEN** 请求 operation 为 `read`
- **THEN** 成功响应 operation 为 `read`
- **THEN** result 必须符合 ReadResult

### Requirement: SDK 必须实现单请求 invoke 生命周期
`docnav-adapter-sdk` MUST 提供 adapter invoke 单请求生命周期：从 stdin 读取一个完整 request，分发到对应 operation handler，向 stdout 输出一个 protocol JSON 响应，并在完成后退出。

#### Scenario: invoke 输出单个响应
- **WHEN** adapter 通过 SDK 处理一次 invoke 请求
- **THEN** stdout 只输出一个 JSON 响应
- **THEN** 诊断信息只能写入 stderr

### Requirement: SDK 必须提供 manifest 和 probe 分发基础
`docnav-adapter-sdk` MUST 支持 adapter 实现 manifest 和 probe 命令，并 MUST 保持 manifest/probe 与 invoke protocol envelope 分离。

#### Scenario: manifest 输出专属 schema
- **WHEN** 调用方执行 adapter `manifest --output protocol-json`
- **THEN** 输出符合 manifest schema
- **THEN** 输出不包含 invoke request/response envelope

### Requirement: 自动化验证必须覆盖 schema 与示例
本 change MUST 提供自动化验证，覆盖 protocol request/response、manifest、probe 和 readable schema 的 strict 编译，以及关键示例 fixture 的解析和语义校验。

#### Scenario: 校验协议响应 fixture
- **WHEN** 验证脚本读取 protocol response 示例
- **THEN** 示例通过 protocol response schema
- **THEN** 响应 operation 与 result 类型匹配
