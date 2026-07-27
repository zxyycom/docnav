# Change: 建立当前树驱动的测试评估体系

## Why

逐入口手写测试账本既不能证明完整当前树没有漏项，也会诱导维护者为每个测试填充
低信息模板。项目需要一个以源码和 runner 为事实源、由机器保证入口闭合、由 AI
只审查高信息语义的测试评估体系。

## What Changes

- 把 supported runner profile 覆盖的最小原生测试入口规范化为
  `NativeTestEntry`，每个当前 Entry 恰好形成一个 machine case。
- 用项目 ast-grep 规则发现静态入口，并与 Cargo、Bun 和 smoke runner 报告双向
  核对；漏项、悬空、重复和不支持形态必须阻断严格检查。
- 生成可删除重建的 inventory 和 query index，提供按入口、runner、路径、topic、
  owner 与 Claim 的有界查询。
- 只为不能由 owner requirement 加测试入口直接恢复的长期判断维护 Evidence
  Claim；普通入口不需要 Claim。
- 用 source/owner fingerprint 和显式 baseline 报告缩小 AI 对当前变更的审查范围，
  但不把结构闭合或测试通过冒充为充分性证明。
- 将 `validate:docs -- cases`、workspace verifier、测试策略、维护指南、AGENTS 和
  相关 active change 接到同一当前链路。
- 不修改 Docnav CLI、adapter、protocol、ref、输出或 release 产品行为；不为旧
  测试账本提供兼容读取、迁移映射或恢复协议。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `test-evidence-management`：改为完整当前树驱动的 Entry、machine case、Evidence
  Claim 与 AI 审查体系。

## Impact

- Skill 与工具：项目级 `ast-grep`、`test-evidence-review` 和精确锁定的开发期
  `@ast-grep/cli`。
- 发现与验证：`scripts/test-evidence/`、docs validator 和 workspace verifier。
- 当前证据：`docs/test-evidence/` 的 inventory、受控 Claim、topic 与 query index。
- 稳定文档：`docs/navigation.md`、`docs/testing.md`、
  `docs/testing/case-maintenance.md`、`docs/testing/coverage.md`、`docs/tooling.md`
  和 `AGENTS.md`。
