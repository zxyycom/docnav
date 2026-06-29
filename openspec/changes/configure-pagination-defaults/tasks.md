本 tasks 清单记录 pagination 默认配置的审核、方案细化、实施和验证入口。

## 1. 审计门禁

- [x] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“统一 pagination 默认配置，并把预算默认值表达为 adapter-owned numeric limit”这一核心目标。
- [x] 1.2 确认 protocol `limit` 与 `page` 已由现有协议 owner 承接，当前 change 不新增 protocol `pagination` 字段。
- [x] 1.3 确认 capability ID 只复用 `core-cli`、`adapter-protocol`、`markdown-navigation`。
- [x] 1.4 收敛临时说明和历史字段描述，只保留当前 change 的 owner、边界和验收项。

## 2. 方案细化

- [x] 2.1 确定 `defaults.limit` 到 `defaults.pagination.limit` 的迁移策略，并写清 hard switch、过渡 alias 或诊断提示的可观察行为。
- [ ] 2.2 定义 core 与 SDK direct CLI 的 pagination 参数身份、CLI spelling、config path、invoke argument binding、来源优先级和 built-in defaults。
- [x] 2.3 定义 `enabled` 与 `limit` 来自不同优先级来源时的最终归一矩阵，特别覆盖显式 `--limit` 与低优先级 `enabled=false` 的组合。
- [x] 2.4 定义 `enabled=false` 时使用的最大正整数常量、schema facet 和 typed validation 行为，禁止入口各自硬编码不同 magic number。
- [ ] 2.5 确认 Markdown adapter 配置 schema/example、smoke 和矩阵测试如何表达 `defaults.pagination.enabled` 与 `defaults.pagination.limit`。
- [ ] 2.6 确认 CLI/help、schema/example 和 readable 文案中哪些位置需要解释 adapter-owned `limit`，并避免把预算单位写成 core/SDK 语义。

## 3. 实施与验证

- [ ] 3.1 同步 `standard-parameters`、CLI、adapter contract、Markdown adapter、schema/example 和测试说明。
- [ ] 3.2 更新 core config、document command parser、help/default 文案和 invoke request construction。
- [ ] 3.3 更新 adapter SDK direct CLI config projection、argv parser、参数来源合并、disabled finalization 和 typed validation。
- [ ] 3.4 更新 Markdown adapter config schema/example、fixture 和 smoke/matrix 测试。
- [ ] 3.5 更新或新增测试，覆盖配置迁移策略、来源优先级矩阵、invalid pagination values、默认 `page=1` 不可配置和 disabled finalization。
- [ ] 3.6 覆盖 `--pagination disabled` 不泄漏为 protocol `pagination` 字段、adapter 只接收最终 `limit` 和 `page`。
- [ ] 3.7 运行范围匹配的 Rust、schema/example、CLI smoke 和 workspace 验证。
