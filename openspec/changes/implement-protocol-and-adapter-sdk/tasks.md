一句话核心：先实现稳定协议和 adapter SDK，后续实现不得绕过这层契约。

## 0. 审计门禁

- [ ] 0.1 用户审计确认：用户已审计本 change 的 proposal、design、spec 和 tasks，并明确允许开始实现；未完成本项前，1.x 及后续任务全部处于阻塞状态。

## 1. 协议类型与兼容性

- [ ] 1.1 （阻塞：等待 0.1 用户审计确认）建立 `docnav-protocol` 模块结构和 v0 protocol 基础类型。
- [ ] 1.2 （阻塞：等待 0.1 用户审计确认）实现 request/response envelope、operation arguments、operation result 和 page 类型。
- [ ] 1.3 （阻塞：等待 0.1 用户审计确认）实现协议版本闭区间兼容判断和 `PROTOCOL_INCOMPATIBLE` 错误构造。
- [ ] 1.4 （阻塞：等待 0.1 用户审计确认）实现 stable error code、必需 details 和 operation/null 失败响应规则。

## 2. Schema 与 Fixture 验证

- [ ] 2.1 （阻塞：等待 0.1 用户审计确认）接入 protocol request/response、manifest、probe 和 readable schema 的 strict 编译验证。
- [ ] 2.2 （阻塞：等待 0.1 用户审计确认）补充或整理协议 fixture，覆盖成功响应 operation/result 绑定和失败响应。
- [ ] 2.3 （阻塞：等待 0.1 用户审计确认）验证 schema 与共享类型的关键字段一致性。

## 3. Adapter SDK

- [ ] 3.1 （阻塞：等待 0.1 用户审计确认）实现 stdin 单请求读取、operation 分发、stdout 单 JSON 响应和 stderr 诊断约束。
- [ ] 3.2 （阻塞：等待 0.1 用户审计确认）实现 manifest、probe 和 invoke 的 SDK 分发接口。
- [ ] 3.3 （阻塞：等待 0.1 用户审计确认）实现 SDK 层退出码和错误映射辅助。

## 4. 验证与审计

- [ ] 4.1 （阻塞：等待 0.1 用户审计确认）运行协议、schema、SDK 单元测试。
- [ ] 4.2 （阻塞：等待 0.1 用户审计确认）运行文档示例校验脚本，确认现有 JSON 示例仍通过。
- [ ] 4.3 （阻塞：等待 0.1 用户审计确认）用局部 diff 确认只修改协议/SDK/验证相关范围。
