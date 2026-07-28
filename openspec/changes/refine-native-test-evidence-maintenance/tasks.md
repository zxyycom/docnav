本清单把目录驱动测试发现、Entry 粒度收敛和 Claim 稀疏化拆成可验证步骤；它是临时实施计划，勾选项只表示已取得对应证据。

## 1. 实现前审计

- [x] 1.1 阻塞级审计 proposal、design、delta spec 与 tasks 是否围绕目录驱动发现和 Entry/Claim 分层，确认 capability ID、归档前置、兼容性取舍和验证范围一致；本项完成前不得执行任何实现任务

## 2. Bun 测试面

- [x] 2.1 先增加 profile v2 展开测试，证明目录 include、新文件自动纳入、ignore、特殊文件补充、冗余补充和无效边界
- [x] 2.2 实现安全、确定性的 Bun source roots/include/ignore/supplemental files 解析与展开，并让静态扫描和 runner report 共用同一文件集合
- [x] 2.3 将当前 profile/schema 和测试维护文档迁移到 v2，确认目录规则覆盖当前全部 Bun test files

## 3. 原生入口粒度

- [x] 3.1 拆分混合独立契约的 core smoke leaf，保留不可分割的 round trip/parity/source matrix，并修正 task selector 与 command audit label
- [x] 3.2 拆分当前 Claim 支持范围内混合独立错误契约的 Rust tests，使 runner failure 可归因
- [x] 3.3 独立运行所有变化后的 smoke leaf 与目标 Rust tests，确认每个 selector 可单独报告

## 4. Claim 与派生制品

- [x] 4.1 删除无信息 Claim，校准保留 Claim 的 statement、observations 和一对多 `supportedBy`
- [x] 4.2 解除 sync 对旧 inventory 新鲜度的自举依赖，从完整当前树重建 native inventory/index，并用 changes 与 strict check 审查 split、rename、stale 和未知支持

## 5. 验证与收尾

- [x] 5.1 运行 profile/schema、test-evidence rules、catalog、文档和 OpenSpec 的目标验证
- [x] 5.2 运行 `bun run verify:docnav-workspace`，记录通过项、warning 边界与未验证风险
- [x] 5.3 对照编码规范、测试策略和 AI-ready 文档原则审查最终 diff，确认无旧 profile 双读、无模板 Claim 和无范围外产品行为变化

## 6. 额外审核修复

- [x] 6.1 用失败测试固定正向 glob 语义、配置路径中间符号链接阻断和 smoke helper 实现变化可被 fingerprint 观察
- [x] 6.2 收紧 Bun profile glob 与 checkout 路径边界，并保持静态扫描和 runtime runner 共用同一文件集合
- [x] 6.3 让 smoke Entry fingerprint 覆盖实际 `run` 实现，拆分 Markdown option 的成功与失败 leaf，并同步关联 Claim
- [x] 6.4 把 profile 展开与 sync 自举不变量同步到稳定 owner，校准 change spec/design，重建 inventory/index
- [x] 6.5 运行目标测试、changes 报告、严格账本检查、文档/OpenSpec 校验和完整 workspace verification
