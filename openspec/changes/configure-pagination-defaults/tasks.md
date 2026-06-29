本 tasks 清单记录通用 pagination limit 默认配置的探索和实施入口；当前只在 `openspec/changes/configure-pagination-defaults/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 审计门禁

- [ ] 1.1 阻塞级审计：在执行任何实现任务前，审计 proposal、design、specs 和 tasks 是否都围绕“统一 pagination 默认配置，并把预算默认值表达为 adapter-owned numeric limit”这一核心句。
- [ ] 1.2 审计本 change 是否等待 `explore-structured-protocol-fields` 确认 `limit` 字段迁移方式。
- [ ] 1.3 审计 capability ID 是否只复用 `core-cli`、`adapter-protocol`、`markdown-navigation`。
- [ ] 1.4 审计当前 change 是否只包含 `openspec/changes/configure-pagination-defaults/` 下的未审核临时 artifacts。

## 2. 方案细化

- [ ] 2.1 确认 `limit_chars` 到 `limit` 的兼容策略、schema/example 迁移和 CLI 文案。
- [ ] 2.2 确认 core 与 SDK direct CLI 的 pagination 参数来源模型、优先级和 disabled 归一。
- [ ] 2.3 确认 Markdown adapter 配置示例、smoke 和矩阵测试如何表达 `defaults.pagination.limit`。

## 3. 实施与验证

- [ ] 3.1 更新 core config、document command parser、help/default 文案和 invoke request construction。
- [ ] 3.2 更新 adapter SDK direct CLI config projection、argv parser、参数来源合并和 typed validation。
- [ ] 3.3 更新 Markdown adapter config schema/example、fixture 和 smoke/matrix 测试。
- [ ] 3.4 同步主规范、schema/example 和测试说明。
- [ ] 3.5 运行范围匹配的 Rust、schema/example、CLI smoke 和 workspace 验证。
