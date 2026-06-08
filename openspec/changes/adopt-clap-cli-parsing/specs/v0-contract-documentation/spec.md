**一句话核心：本 delta 将 v0 文档契约中的 CLI 参数解析目标调整为 AI 友好的宽松成功路径，并把 `clap` 确认为 Rust CLI 的首选实现基础。当前 change 只在 `openspec/changes/adopt-clap-cli-parsing/` 下形成未审核临时文档，不影响现有其它文档或主规范。**

## ADDED Requirements

### Requirement: Rust CLI 参数解析必须服务 AI 维护和一次成功调用
Rust CLI 参数解析文档 MUST 将 `clap` 定义为核心 `docnav` CLI、adapter 直接 CLI 和后续 Rust CLI 扩展的首选参数解析基础。CLI argv 容错目标 MUST 是在必需语义参数正确时优先执行成功；未知 flag、多余 positional 和无关参数 MUST NOT 作为成功路径的主失败原因。文档 MUST 保留 protocol/invoke 严格校验边界，并 MUST 说明 `clap` 是 CLI 入口实现选择，不改变共享协议字段、schema 或 adapter 格式解析所有权。

#### Scenario: AI 调用包含未知参数但必需语义完整
- **WHEN** AI agent 调用 Rust CLI 时传入未知 flag 或额外 positional
- **AND** path、ref、query、page、limit_chars、output 等当前 operation 必需语义可被解析
- **THEN** CLI 继续执行对应 operation
- **THEN** 未知或多余输入最多产生阅读层 warning 或诊断
- **THEN** protocol envelope、readable schema 和 adapter ref 语义保持不变

#### Scenario: Protocol 入口仍严格校验
- **WHEN** adapter `invoke` 从 stdin 收到包含未知字段、缺少必需字段或参数类型错误的 JSON request
- **THEN** invoke 按 protocol schema 返回结构化失败
- **THEN** CLI argv 容错规则不用于忽略该 JSON request 的错误字段

#### Scenario: 文档 owner 同步 CLI 解析规则
- **WHEN** 实现者更新 CLI 参数解析、warning 行为或 `clap` 依赖
- **THEN** docs/cli.md 描述用户可见 CLI 行为和容错边界
- **THEN** docs/adapter-contract.md 描述 adapter direct CLI 与 invoke 的边界
- **THEN** docs/testing.md 描述成功路径、必要失败和 schema 边界的自动化验证
