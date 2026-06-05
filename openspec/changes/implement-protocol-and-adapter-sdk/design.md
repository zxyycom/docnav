**一句话核心：本 change 只实现协议与 SDK 地基，保证所有后续制品通过同一套稳定 invoke 契约通信。**

## Context

Docnav v0 已经在主规范中定义原始协议层和阅读输出层。当前 change 处在实现序列第一位，后续 Markdown adapter、核心 CLI、adapter 管理和 MCP bridge 都会依赖这里提供的类型、校验和进程 I/O 基础。

## Goals / Non-Goals

**Goals:**

- 实现 `docnav-protocol`，覆盖 protocol request/response、manifest、probe、operation result、page 和稳定错误。
- 实现 `docnav-adapter-sdk`，让 adapter 可以用统一方式处理 manifest、probe、invoke 和直接命令分发。
- 提供 schema/example 自动化验证，确保协议字段、operation 绑定和错误 details 保持稳定。
- 明确协议兼容判断：`docnav` 与 adapter 的协议范围必须存在交集，并选择最高兼容版本。

**Non-Goals:**

- 不实现任何格式解析或 Markdown 行为。
- 不实现 `docnav` 核心 CLI 的 adapter 选择和输出映射。
- 不实现 adapter 安装管理和 MCP bridge。

## Decisions

1. `docnav-protocol` 只承载跨制品稳定协议。
   - 理由：共享库不能引入 Markdown 或其它格式展示字段，否则后续 adapter 会被错误耦合。
   - 替代方案：把 Markdown outline 类型放入共享库；拒绝，因为文档明确共享协议只把 `ref` 和紧凑结果当作业务语义。

2. `docnav-adapter-sdk` 负责 invoke 生命周期和命令分发，不负责格式策略。
   - SDK 提供 stdin 读取、stdout 单 JSON 响应、stderr 诊断、退出码和 operation 分发。
   - adapter 仍自行实现 parser、ref、probe 证据和格式 options。

3. schema 校验作为测试和边界防线。
   - 类型构造用于实现期约束，Draft 2020-12 schema 用于 fixture、示例和跨语言输出校验。
   - readable schema 不提升为机器稳定协议，只用于后续 CLI/MCP 阅读输出验证。

4. 错误处理保留稳定 code 和必要 details。
   - 无法解析请求时 `operation: null`。
   - 可识别 operation 的失败响应必须回填对应 operation。
   - SDK 只提供错误构造和映射工具，具体错误原因由调用面传入。

5. 进程边界按 adapter invoke 单请求模型实现。
   - 每次 invoke 读取一个完整请求、输出一个 JSON 响应并退出。
   - SDK 不支持长连接、多请求复用或 adapter 内部服务常驻。

## Risks / Trade-offs

- [schema 与 Rust 类型漂移] → 在测试中同时验证 schema 编译、fixture 解析和类型往返。
- [SDK 过度抽象影响 adapter 灵活性] → SDK 只封装协议和进程共性，格式行为保留给 adapter。
- [错误 details 过早固化不足] → 只固化主规范要求的 code 和必需 details，保留可选 details 扩展。

## Migration Plan

1. 建立共享库和基础测试。
2. 接入现有 schema/example 验证脚本。
3. 后续 change 在实现前依赖该库，不复制协议结构。

## Open Questions

- 无必须在本 change 前解决的问题；具体包名和 crate 拆分可在实现时按仓库实际结构确定。
