本 change 准备把 Docnav 测试证据迁移到 test-evidence-review v7；本 proposal 只拥有迁移动机、范围和影响，不证明 change 已审核、已实施，也不改变现有测试、账本、主规范或验证行为。

## Why

当前集中式 `docs/testing/cases.md` 以稳定语义 case 聚合多个测试函数，并依赖源码 `@case` marker 和定制 validator；它与 v7 的“一个保留的最小原生测试入口对应一个独立 case”契约不兼容。迁移必须先逐条恢复 runner 原生入口、测试证明价值和 topic 归属，不能通过目录搬移或自动生成直接替换。

## What Changes

- **BREAKING**：用固定的 `docs/test-evidence/` topic 目录和一 case 一 Markdown 模型替换集中账本、`Status`/`Code` 字段、源码 `@case` marker 与现有 case-catalog 采集校验。
- 首次整包接入 `.codex/skills/test-evidence-review` v7，并以其 catalog CLI 生成和严格检查可删除重建的统一索引。
- 逐个审查当前保留的最小原生测试入口；聚合 case 按 runner 报告粒度拆分，工程校验和 supporting helper 不迁入测试证据目录。
- 保留仍适用的稳定 case ID、契约和证明语义；需要拆分时为新增入口分配新 ID，并明确旧 case 的去向。
- 同步 `docs/navigation.md`、测试策略、case 维护规则、覆盖材料、AGENTS 指令、验证脚本与 package/workspace 入口。
- 不新增缺少明文 owner 契约的产品断言，不改变 Docnav CLI、adapter、协议、输出或 release 产品行为。

## Capabilities

### New Capabilities

- `test-evidence-management`: 定义最小原生测试入口、topic、case、派生索引、查询和严格验证的项目级测试证据契约。

### Modified Capabilities

- 无。

## Impact

- Skill 与工具：新增 `.codex/skills/test-evidence-review/`，替换 `scripts/tools/validators/case-catalog/` 及相关验证集成。
- 测试证据：迁移 `docs/testing/cases.md` 到 `docs/test-evidence/test-evidence-topics.json`、独立 case Markdown 和派生索引。
- 源码与测试：移除 `@case` marker；必要时拆分混合多个原生测试意图的测试入口，但保持产品行为不变。
- Owner 文档与 active changes：同步所有仍引用旧账本、marker 或维护流程的稳定文档和在途 change。
