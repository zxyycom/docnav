**一句话核心：先把 Docnav 的稳定原始协议和 adapter SDK 做成可校验、可复用、可被后续 adapter 与核心 CLI 共同依赖的地基。**

## Why

当前 v0 文档已经定义 `adapter invoke` 原始协议、manifest、probe、page、稳定错误和进程边界，但仓库尚无实现。必须先实现协议与 SDK 基础，后续 Markdown adapter、`docnav` 核心 CLI 和 MCP bridge 才能共享同一套稳定契约。

## What Changes

- 新增 Rust `docnav-protocol` 共享协议实现，覆盖请求/响应 envelope、operation/result 绑定、page、稳定错误、协议版本协商和 schema 校验入口。
- 新增 Rust `docnav-adapter-sdk` 基础能力，覆盖单请求 stdin/stdout invoke、manifest/probe/document operation 分发、stderr 诊断约束和退出码映射。
- 将文档中的 protocol、manifest、probe 和 readable schema 纳入自动化验证流程。
- 为后续 adapter 和 core CLI 提供可复用类型、校验函数和测试 fixture。
- 非目标：本 change 不实现 Markdown 解析、不实现 `docnav` 核心路由、不实现 adapter 安装管理、不实现 MCP bridge。

## Capabilities

### New Capabilities

- `protocol-and-adapter-sdk-implementation`: 实现原始协议、manifest/probe 基础校验、adapter SDK invoke 生命周期和共享测试支撑。

### Modified Capabilities

- 无。

## Impact

- 影响共享库与协议校验面：Rust `docnav-protocol`、Rust `docnav-adapter-sdk`、schema/example 验证脚本和测试 fixture。
- 明确实现栈边界：核心 CLI、adapter、protocol 和 adapter SDK 以 Rust 为主；JavaScript/Node.js 用于 MCP bridge 和 AJV 文档/schema 校验；Python 仅作为开发或 fixture 辅助工具。
- 影响后续 change 的依赖顺序：Markdown adapter、核心 CLI、adapter 管理和 MCP bridge 都必须依赖本 change 产出的协议与 SDK 基础。
