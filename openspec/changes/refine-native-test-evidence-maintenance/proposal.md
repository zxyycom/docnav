本 change 提议把测试证据维护收敛为目录驱动的完整发现、可独立归因的原生入口和稀疏高信息 Claim；本文是临时提案，不代表实现已经完成。

## Why

当前 Bun runner profile 逐文件列出普通测试面，新增测试文件若未登记就可能同时避开静态与运行时发现。现有证据虽然已经机械闭合，但部分 smoke leaf 与 Claim 仍保留旧式“大 case 一对一 Claim”形态，降低失败归因和语义审查密度。

## What Changes

- **BREAKING**：将 supported runner profile 升级为目录与 include/ignore pattern 驱动的 Bun 测试面；单文件只作为目录规则之外的特殊补充，不提供旧 profile 双读。
- 对 profile 展开的完整文件集合执行确定性归一、路径安全校验和空匹配/重复匹配检查，再用同一集合执行静态扫描与 Bun runner report。
- 只拆分混合了可独立命名、独立失败契约的原生测试节点；共同证明一个不变量的多步骤或代表矩阵继续保持单一 Entry。
- **BREAKING**：允许因合理 split/rename 直接重建 smoke/Cargo `entryKey` 和派生 inventory，不维护旧 Entry 身份兼容层。
- 删除无长期信息增量的 Claim；让保留 Claim 能关联一个或多个当前 Entry，并确保 statement、observations 与实际支持范围一致。
- 修正 smoke task、command audit label 与 machine selector 的身份一致性。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `test-evidence-management`：runner profile 的测试面从逐文件登记改为目录匹配与显式忽略为主、特殊文件补充为辅，并收紧 Entry 粒度和 Claim 信息增量要求。

## Impact

影响 `scripts/test-evidence/` 的 profile/schema、Bun discovery 与验证测试，`test/smoke/core/` 的 leaf task 组织，少量 Rust 测试入口，`docs/testing*`、Evidence Claims、machine inventory/index 和本 change 的 OpenSpec artifacts。不会改变 Docnav 产品 CLI、protocol、adapter 或 release artifact；实现以已完成但尚未归档的 `enforce-native-test-evidence-coverage` 为前置基线，归档时必须先处理该基线 change。
