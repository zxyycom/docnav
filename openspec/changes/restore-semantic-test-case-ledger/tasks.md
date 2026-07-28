本临时 tasks artifact 的目标是先通过阻塞审计，再把测试证据链硬切换为 `Topic -> Case(owner, proves, entities)`；未完成第 1 组门禁前不得执行任何实现任务。

## 1. 实现前阻塞审计

- [x] 1.1 核对 proposal、design、delta spec 与本 tasks 均围绕“语义 Case 直接拥有 owner/proves/entities，scanner 只拥有当前 scanned test entity（测试实体）事实”这一核心句，且没有把测试实体粒度重新表述为 Case 粒度。
- [x] 1.2 核对 proposal 只修改现有 capability `test-evidence-management`，delta spec 路径与 capability ID 完全一致，没有以 change name 创建同义 capability。
- [x] 1.3 核对全部 artifact 都明确是临时 change 材料，没有声称已批准、已实现或可绕过门禁直接 apply；`design.md` 的 Open Questions 没有未回答问题或残留歧义。
- [x] 1.4 核对迁移基线可执行：`2ec2de7:docs/testing/cases.md` 恰有 102 个 Case（101 implemented、1 planned），当前 committed scanner projection 恰有 548 个唯一 entity key（393 Cargo、128 Bun、27 smoke）；确认 101 个 implemented 记录只作为逐项 review seed，planned Case 留给规划 owner，最终任务覆盖实现期间重新扫描得到的完整集合。
- [x] 1.5 核对设计采用最小持久状态：`docs/testing/cases/topics.json` 只拥有稳定 topic，每 topic 一个允许为空的 Markdown Case owner；无 committed inventory/index、无 skill runtime catalog、无双读，测试实体实现变更复审明确留给后续 change。
- [x] 1.6 核对本次 OpenSpec 创建只写入 `openspec/changes/restore-semantic-test-case-ledger/`，没有越过 change 目录修改长期 owner 或其它 change，并运行 `openspec validate restore-semantic-test-case-ledger --type change --json --strict --no-interactive` 通过；以上任一项失败时不得按本 tasks 执行第 2 组及后续任务。

## 2. 先建立可执行契约证据

- [x] 2.1 在修改旧证据链前运行当前项目 wrapper 的完整 `check`，保存不提交的 scanned test entity key 基线并证明 static/runtime/映射闭合；实现期间新增的测试实体必须进入最终重新扫描集合。
- [x] 2.2 先为 topic catalog/topic-file parser 添加 focused failing fixtures，覆盖 workspace-safe root/source、嵌套目录、符号链接、未知 `.md`、可忽略的非 Markdown 普通文件、H1-only 空 topic、合法 Case blocks、malformed/其它 H2、Case block 外 prose、全局唯一 ID、单一精确 Owner、非空 Entities/Proves 以及非法字段/owner 诊断；证明 `Status` 和多 Owner 形态不被接受。
- [x] 2.3 先为 Case/test-entity join 添加 focused failing tests，覆盖 uncovered test entity、unknown entity、无实体 Case、一个测试实体支持多个 Case和一个 Case 使用多个测试实体。
- [x] 2.4 先为 `topics`、仅按 topic/Owner/entity key/text/pagination 过滤的有界 `list`、按 Case ID 精确查询的单 Case `show` 和 `check` 添加 CLI contract tests，并证明 Case-ID list filter、旧 `sync`、`changes` 与 Entry/Claim filters 被明确拒绝而不是静默兼容。
- [x] 2.5 更新完整当前树 required check 入口，使它直接证明 scanner closure 与 Topic/Case 双向 coverage，不再通过 committed inventory、Claim 或 index 新鲜度间接证明。

## 3. 实现项目内 Case 账本

- [x] 3.1 在 `scripts/test-evidence/` 将当前入口事实收敛为 `TestEntity` 与确定性 entity key，保留 Cargo/Bun/smoke profile、静态/runtime/映射闭合和必要定位诊断，删除 `NativeTestEntry`、machine Case/Entry 术语。
- [x] 3.2 在 `scripts/test-evidence/` 实现严格的 `topics.json`、per-topic Markdown parser 和 Case model：Case root 必须是 workspace-safe 非符号链接目录，`topics.json` 和 `.md` source 必须是 workspace-safe 非符号链接普通文件；嵌套目录、任意符号链接、未知 `.md` 与非法 topic 语法阻断，非 Markdown 普通文件忽略；校验稳定/空 topic、Case ID、单一 Owner、Entities 与 Proves，且不引入新的 committed projection。
- [x] 3.3 实现 scanned test entities 与 Cases 的 many-to-many 双向 coverage、owner heading 校验和阻断诊断；复用同一个测试实体的多个 Case 必须保持合法。
- [x] 3.4 把只读 `topics`、仅按 topic/Owner/entity key/text/pagination 过滤的有界 `list` 和按 Case ID 精确查询的单 Case `show` 直接接到 topic catalog 与 Case source，把 `check` 接到完整 scanner 与 Case validator；更新 JSON/可读输出、帮助和退出状态以统一使用 topic/Case/test-entity 术语。
- [x] 3.5 删除 inventory read/write、baseline change report、index sync/fallback 和对 skill runtime catalog 的导入；确认项目 wrapper 没有隐式写入路径。

## 4. 迁移语义 Case 与完整实体覆盖

- [x] 4.1 从 commit `2ec2de7` 提取 17 个 black-box、67 个 white-box 和 18 个 auxiliary 记录，按稳定 owner responsibility 选择/复用 topic ID；只把其中 101 个 implemented Case 作为逐项 review seed，并只在迁移进程内维护其 ID 到候选测试实体的工作表。
- [x] 4.2 逐项审查 16 个 implemented black-box seed：只有本 change 实现开始前已有当前测试实体直接支持的语义才保留连续 Case ID 并迁移当前单一 Owner/Entities/Proves；唯一 planned release Case 不迁入当前账本，并确认其继续由 Git 历史、OpenSpec 或其它规划 owner 承接。
- [x] 4.3 按 owner topic 逐项审查 67 个 white-box seed：只有本 change 实现开始前已有当前测试实体直接支持的语义才迁移；能力仍在但缺这种实体时不迁移且不在本 change 补产品测试，能力已移除时以明确 owner/source 依据退休，未迁移或退休 ID 均不复用。明确记录 `WB-TYPED-FIELDS-PRESENCE-001`、`WB-TYPED-FIELDS-METADATA-001`、`WB-TYPED-FIELDS-CONSTRAINTS-001`、`WB-TYPED-FIELDS-RANGES-001` 因缺起点直接实体未迁移，`WB-TYPED-FIELDS-PROJECTION-001`、`WB-TYPED-FIELDS-COMPILE-001` 因旧 API 已移除而退休。
- [x] 4.4 按 owner topic 逐项审查 18 个 auxiliary seed，遵循相同的起点直接证据迁移、独立 product test change 或有依据退休规则，并把 workspace、quality、release 和测试证据工具实体归入真实语义 Case 而不是通用模板；不得编造空 Case 或把未迁移/退休 ID 改作其它语义。
- [x] 4.5 对实现后的完整 scanner 集合执行反向 coverage，补充或修正历史种子未覆盖的当前语义；确认每个当前测试实体至少属于一个 Case、每个 Case 至少有一个当前测试实体，且不提交迁移映射。

## 5. 硬切换 owner、skill 与验证入口

- [x] 5.1 同步 `docs/navigation.md`、`docs/testing.md`、`docs/testing/case-maintenance.md`、`docs/testing/coverage.md` 和 `docs/tooling.md`，让稳定 owner 只描述 scanned test entity（测试实体）、Topic/Case、双向 coverage 与直接查询。
- [x] 5.2 更新根 `AGENTS.md` 和项目级 `test-evidence-review` skill 的读取时机、审查顺序与完成标准；skill 只保留 Case 质量指导，删除其 runtime catalog、声明文件和 Entry/Claim/index schemas。
- [x] 5.3 删除 `docs/test-evidence` 下 Claim files、claim topic catalog、native inventory 和 query index，确认当前 docs/code/tests 与本 change 不再依赖这些路径；历史 Git 与已完成 predecessor OpenSpec artifacts 保持原文。
- [x] 5.4 更新 package scripts、workspace check 定义、成功输出过滤和 verifier tests，使 required profile 只运行新 `test-evidence check` 且不重复调度完整 Bun 测试面。
- [x] 5.5 用局部搜索确认当前 owner、skill、代码、测试和本 change 没有继续依赖 Evidence Claim、hand-written Entry、machine case inventory、index sync 或兼容双读；本 change 的 REMOVED context 与已完成 predecessor/historical artifacts 的命中允许保留。

## 6. 验证与交付

- [x] 6.1 运行 test-evidence parser、scanner、CLI、rule 和当前 owner/docs verification，并用项目 scanner 证明新增/修改的测试实体均已纳入最终 Case coverage。
- [x] 6.2 运行 `bun run test-evidence -- check --root .`，记录最终 Cargo/Bun/smoke 测试实体数量、Case 数量、topic 数量和双向 coverage 全部通过。
- [x] 6.3 运行 `bun run verify:docnav-workspace:required` 和 `openspec validate restore-semantic-test-case-ledger --type change --json --strict --no-interactive`，修复所有 blocking diagnostics。
- [x] 6.4 运行 `bun run verify:docnav-workspace`，并按 `docs/coding-style.md` 的变更前后自检核对 owner、边界、失败诊断与验证证据。
- [x] 6.5 用 `dnm outline` 检查更新后的测试 owner 文档层级，并用局部 diff/路径过滤确认只存在目标模型、没有生成物或范围外重构；完成后再进入归档评估。
