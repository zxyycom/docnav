本临时 change proposal 的目标是恢复由语义 Case 直接拥有测试证明关系的账本；它只界定拟议变更，不表示实现已经完成。

## Why

当前测试证据链把 548 个扫描入口投影为 machine Entry，再从 21 个 Evidence Claim 补充少量长期语义；这既没有恢复旧账本中 102 个经过人工组织的证明目标，也让 Entry、Claim、inventory 和 index 的维护模型反过来主导语义。现在需要让稳定 Case 重新成为测试意图的直接 owner，同时保留已经闭合的全树扫描能力来证明没有测试实体遗漏。

## What Changes

- **BREAKING**：把旧 `NativeTestEntry -> Evidence Claim` 模型替换为 `Topic -> Case(owner, proves, entities)`；Case 是长期语义单元，扫描器只提供当前 scanned test entity（测试实体）事实，`NativeTestEntry` 不再是长期模型。
- 以 `docs/testing/cases/topics.json` 定义稳定、允许为空的 topic，并由每个 `<topic>.md` 保存该 topic 的全部当前 Case；受管理来源保持 workspace-safe、regular、no-symlink，未知 Markdown、嵌套目录和符号链接阻断，无关 non-Markdown regular file 不进入账本来源。
- Case ID 全局唯一、稳定且不得换义复用；每个 Case 使用真正拥有 `Proves` 的单一精确 Owner，并列出至少一个当前实体 key。
- 严格检查继续在完整当前树上闭合 static、runtime 与实体映射，并增加双向覆盖：每个 scanned test entity（测试实体）至少属于一个 Case，每个 Case 至少引用一个当前测试实体；同一实体可以支持多个 Case。
- **BREAKING**：删除 Evidence Claim、手写/提交的 Entry 或 machine inventory、一 Case 一文件布局以及重复 committed query index；不提供兼容双读或同步生成的第二权威源。
- 把 `2ec2de7:docs/testing/cases.md` 中 101 个 historical implemented Case 只作为逐项复核种子：仅当本 change 实现开始前已经存在直接支持该语义的当前测试实体时，才保持语义连续 ID 并迁移；生产能力仍存在但缺少这种直接实体时不迁移，也不反向为账本补产品测试，而是留给独立的 owner-driven product test change 评估；当前生产能力已移除时可以用明确 owner/source 依据退休。不得硬造空 Case 或把旧 ID 换义复用。唯一 historical planned Case 不进入当前账本，继续由 OpenSpec 或其它规划 owner 承接。
- 收敛项目测试证据命令为直接读取 Topic/Case 并按需运行当前扫描的 `topics`、`list`、`show` 与 `check`；`show <CASE-ID>` 是精确 Case ID 查询入口，`list` 只提供 topic、owner、entity 和文本有界过滤。移除依赖 committed inventory/index 的 `sync` 和 baseline `changes` 流程。
- 同步测试策略、维护规范、项目级 test-evidence-review skill、项目 parser/validator、focused tests 和 workspace required check 的术语及验收语义。
- 非目标：不改变产品 `docnav` CLI、adapter、protocol/schema 或现有测试行为，不为了迁移历史 Case 新增或改写产品测试；不在本 change 中引入实体实现变更 fingerprint 复审、rename 推断或其它后续审查抽象。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `test-evidence-management`：把 Entry/Claim/派生 inventory-index 契约改为 scanned test entity（测试实体）与 Topic/Case 语义账本的双向覆盖契约。

## Impact

- 长期 owner 与说明：`docs/navigation.md`、`docs/testing.md`、`docs/testing/case-maintenance.md`、`docs/testing/coverage.md`、`docs/tooling.md` 和项目级 `test-evidence-review` skill。
- 数据与校验：现有 `docs/test-evidence/` Evidence Claim、topic、inventory/index 制品将由 `docs/testing/cases/{topics.json,<topic>.md}` 取代；`scripts/test-evidence/` 继续拥有 runner profile 和全树发现，并改为直接验证 Case。
- 开发者工具：`bun run test-evidence -- check --root .` 与 workspace required check 保持单一严格入口；只读查询改为 Case 语义，旧 `sync` / `changes` 和 Entry/Claim 过滤器退出。
- 验证材料：删除旧 Entry/Claim/index 专用 schema 与 runtime catalog 分发，更新项目 parser/validator focused tests、完整当前树集成测试与 workspace verifier 断言。产品运行时和 release artifact 不受影响。
