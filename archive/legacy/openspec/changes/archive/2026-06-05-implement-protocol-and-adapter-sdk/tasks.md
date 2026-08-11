一句话核心：先实现稳定协议和 adapter SDK，后续实现不得绕过这层契约。

## 0. 审计门禁

- [x] 0.1 用户审计确认：用户已审计本 change 的 proposal、design、spec 和 tasks，并明确允许开始实现。

## 1. 协议类型与兼容性

- [x] 1.1 建立 Rust workspace 中的 `docnav-protocol` crate 结构和 v0 protocol 基础类型。
- [x] 1.2 实现 request/response envelope、operation arguments、operation result 和 page 类型。
- [x] 1.3 实现协议版本闭区间兼容判断和 `PROTOCOL_INCOMPATIBLE` 错误构造。
- [x] 1.4 实现 stable error code、必需 details 和 operation/null 失败响应规则。

## 2. Schema 与 Fixture 验证

- [x] 2.1 接入 protocol request/response、manifest、probe 和 readable schema 的 AJV strict 编译验证，并保持其作为 JS 开发校验工具。
- [x] 2.2 补充或整理协议 fixture，覆盖成功响应 operation/result 绑定和失败响应。
- [x] 2.3 验证 schema 与 Rust 共享类型的关键字段一致性。

## 3. Adapter SDK

- [x] 3.1 在 Rust `docnav-adapter-sdk` crate 中实现 stdin 单请求读取、operation 分发、stdout 单 JSON 响应和 stderr 诊断约束。
- [x] 3.2 实现 Rust manifest、probe 和 invoke 的 SDK 分发接口。
- [x] 3.3 实现 Rust SDK 层退出码和错误映射辅助。

## 4. 验证与审计

- [x] 4.1 运行 Rust 协议/SDK 单元测试和 schema 校验。
- [x] 4.2 运行文档示例校验脚本，确认现有 JSON 示例仍通过。
- [x] 4.3 用局部 diff 确认只修改协议/SDK/验证相关范围。
