# 当前测试评估体系验收审计

审计日期：2026-07-27
Change：`enforce-native-test-evidence-coverage`

## 1. 结论

**Gate：Proceed**

最终体系只以完整当前树、runner 报告和当前 Evidence Claim 为输入。它不读取旧
case、旧 marker、迁移映射或历史编号。Git 与归档 OpenSpec 仍可用于显式历史审计，
但不参与日常验证。

## 2. AI 消费契约

项目 AI 在测试发生变化时应能够：

1. 运行项目 wrapper，确认静态入口、runtime 入口和 machine cases 一一闭合。
2. 用 `topics`、`list`、`show` 和显式 baseline 报告缩小阅读范围。
3. 从 owner、测试实现和可观察结果判断证明是否充分、可靠且有维护价值。
4. 只在长期判断有额外信息时创建或保留 Claim。
5. 同步派生 inventory/index，并运行目标测试与范围匹配的 workspace verification。

结构检查只证明入口与制品闭合，不证明产品变更已经被充分测试。

## 3. 当前实现边界

| 责任 | Owner |
| --- | --- |
| Cargo/Bun/smoke 纳入范围 | `scripts/test-evidence/supported-runner-profile.json` |
| 静态规则与 runner adapters | `scripts/test-evidence/` |
| Entry、Claim、index 与审查契约 | `.codex/skills/test-evidence-review/` |
| 当前入口事实 | 源码与 runner 报告 |
| 长期测试语义 | `docs/test-evidence/claims/` |
| 当前 topic | `docs/test-evidence/claim-topics.json` |
| 可重建投影 | `native-test-inventory.json`、`test-evidence-index.json` |
| 稳定维护流程 | `docs/testing/case-maintenance.md` |

## 4. 当前评估规则

### 4.1 入口闭合

严格检查比较：

```text
static entries <-> runtime entries <-> committed machine inventory
```

所有不等价状态都带稳定 origin、code 和定位字段返回。完整性检查不以 Git diff 或
既有 inventory 作为发现范围。

### 4.2 Claim 信息增量

Claim 只有在精确 owner、稳定 statement、调用方可观察结果和当前 Entry 支持全部
成立时保留。当前 Claim ID 按语义命名；topic 表只包含实际使用分类。测试名称已经
完整表达的简单证明不建立 Claim。

### 4.3 当前变化审查

source fingerprint、owner fingerprint 和显式 baseline 只比较两个当前状态，用于
定位实现变化、重命名候选和 Claim 陈旧。它们不承担旧系统兼容，也不能代替 AI 的
充分性判断。

## 5. 工具与产品隔离

- `@ast-grep/cli` 精确锁定为 `0.45.0`。
- external ast-grep 只能由 `scripts/test-evidence/ast-grep.ts` 调用。
- 项目 rules、developer CLI 和 skills 不进入 canonical release file set。
- Docnav CLI、adapter、protocol、ref 和输出 contract 不因本 change 改变。

## 6. 验收范围

最终验收覆盖：

- ast-grep rule tests；
- discovery、closure、inventory、Claim、query、baseline report 与 toolchain tests；
- 当前树 `sync --write` 和 strict `check`；
- docs validation、TypeScript typecheck/lint、skill validation；
- OpenSpec strict validation；
- canonical release file-set 与 workspace verification；
- 当前代码与文档中版本/迁移残留的范围搜索。

实际命令、计数与 warning 记录在 `verification.md`。
