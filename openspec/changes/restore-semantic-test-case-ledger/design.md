本临时 design 的目标是以最少持久状态实现 `Topic -> Case(owner, proves, entities)`，并把当前 scanned test entity（测试实体）的完整覆盖作为严格门禁；它不表示实现或迁移已经完成。

## Context

当前实现已经能从版本化 runner profile 发现 Cargo、Bun 和 smoke 原生测试入口，并闭合 static、runtime 与当前映射。当前提交的投影包含 548 个唯一入口（393 Cargo、128 Bun、27 smoke），随后又把这些入口写入 `native-test-inventory.json`，把 21 个 Evidence Claim 和四个 topic 写成独立文件，再生成一份 committed query index。

这套结构把“当前有什么测试”与“这些测试共同证明什么”拆成 Entry/Claim 两种人工概念，却没有恢复 `2ec2de7:docs/testing/cases.md` 中 102 个稳定语义 Case（101 implemented、1 planned）。历史 `Code:` 路径和 smoke task 可以帮助定位候选实体，但一个源码文件通常包含多个当前实体，不能机械地把整文件实体都归入同一 Case。

本 change 只影响仓库内测试证据维护和开发验证，不经过产品 `docnav` CLI、adapter 或 protocol 进程边界。长期规则最终仍由测试 owner 文档拥有，change artifacts 只保存本次设计、迁移和验收依据。

## Goals / Non-Goals

**Goals:**

- 让稳定语义 Case 直接拥有当前 owner、可观察证明陈述和精确测试实体 key。
- 保留完整当前树 scanner 以及 static/runtime/映射闭合，不再提交扫描结果副本。
- 用双向覆盖阻断遗漏，同时允许一个 Case 由多个测试实体支持、一个测试实体支持多个 Case。
- 让受控 topic catalog 成为稳定、可为空的责任分组 owner，并避免一 Case 一文件造成的碎片化。
- 处理历史 102 Case：以其中 101 个 implemented Case 为审查种子，明确排除唯一 planned Case；只有本 change 实现开始前已有当前测试实体直接支持的语义才迁移，其余按明确处置完成硬切换。

**Non-Goals:**

- 不修改测试实现、产品行为、public CLI、adapter、protocol 或 release artifact。
- 不因历史 Case 所述生产能力仍存在，就在本 change 中反向新增或改写产品测试；缺少起点直接测试实体的语义由独立的 owner-driven product test change 评估。
- 不保留 Entry/Claim 兼容层、双读、committed inventory/index 或一次性迁移映射。
- 不在本 change 中实现 fingerprint 驱动的实体实现变更复审、rename 推断、split/merge 自动化或通用审查工作流。
- 不把每个扫描测试实体机械提升为一个语义 Case，也不要求测试实体与 Case 一一对应。

## Decisions

### Decision 1: Case 是语义事实源，scanned test entity 是当前事实

每个 scanned test entity（测试实体）表示 runner 能稳定独立报告或选择的最小当前测试节点，包含确定性的 entity key 和必要定位事实。Case 表示一个可长期命名的当前 implemented 证明目标，保存单一精确 `Owner`、`Proves` 与支持它的当前 entity keys。严格检查在内存中连接两者；scanner 不生成 Case，Case 也不创建当前测试实体。

保留 Entry/Claim 分层并补 coverage 会继续要求维护两套语义名词和 Claim 门槛；为每个实体生成 Case 则会把扫描粒度错误地当成证明粒度。选定模型只保留当前事实与语义事实两个不可替代的 owner。

### Decision 2: Topic catalog 拥有稳定集合，每个 topic 使用一个 Markdown 文件

固定布局为 `docs/testing/cases/topics.json` 与 `docs/testing/cases/<topic>.md`。`topics.json` 是 topic ID、说明和顺序的唯一 owner；每个受控 topic 恰有一个同名 Markdown 文件，该文件包含 topic 标题和零个或多个 Case。因此 topic 可以在暂时没有 Case 时继续稳定存在。

cases root 必须词法位于 workspace 内、解析后仍位于同一 workspace，且自身是 no-symlink directory。`topics.json`、受管理 topic Markdown 以及任何未知 `.md` 都必须通过同一 workspace-safe path 边界；`topics.json` 和 Markdown source 必须是 regular no-symlink file。cases root 中的 nested directory、任何 symlink、未知 `.md` 或受控 topic 缺失都阻断；无关 non-Markdown regular file 不属于受管理来源，可以忽略。

单一集中账本会重新形成大型冲突热点，一 Case 一文件则复制当前 Claim 的碎片化维护成本。只有文件名无法独立保存空 topic 的说明和受控顺序；一个小型 topic catalog 加按 topic 分文件是在稳定空 topic、局部阅读和文件数量之间维护面最小的方案。它不是 Case/entity 的重复 index。

### Decision 3: Case 使用显式、可严格解析的最小字段

每个 Case 使用全局唯一且稳定的 ID；该 ID 跟随测试目的，语义连续时保留，原目的退休后不得换义复用。每个 Case 恰好包含：

- 一个 `Owner`，精确引用当前 Markdown owner 的 heading；
- 非空 `Entities` 列表，逐项保存 scanner 返回的完整 entity key；
- 非空 `Proves` 列表，只写该 Owner 下调用方或责任层可观察的判断。

`Owner` 不只需要解析到当前 Markdown heading，还必须真实拥有每条 `Proves` 所陈述的契约。账本只保存当前 implemented Case，因此每个 Case 都至少有一个当前测试实体，不需要 `Status` 字段。planned 行为留在 OpenSpec、owner 文档或其它规划载体，不能用空实体 Case 占位。跨多个 owner 的历史 Case 必须选择能够完整拥有当前证明的单一精确 Owner；无法由一个 Owner 承接时拆分或收窄 Case，而不是引入多 Owner 变体。

不加入 source fingerprint、迁移来源、测试命令、Code 路径、marker 或派生反向引用。需要定位时由 entity key 和 scanner 当前事实给出；历史材料只在迁移时读取。

每个 topic Markdown 的 grammar 只允许同名 H1、空白和合法 `## Case <ID>: <title>` blocks。H1-only 文件是合法空 topic；拼错的 Case H2、其它 H2 或 Case block 之外的正文都产生阻断诊断，不能被跳过或当作普通说明。

### Decision 4: 严格检查直接计算 many-to-many 双向覆盖

`check` 每次从完整 supported runner profile 取得已闭合、去重的当前测试实体集合，解析 topic catalog 与全部 topic files，并阻断至少以下状态：

- static/runtime/映射未闭合或 entity key 重复；
- Case ID、字段、owner ref、topic 布局或实体 key 非法；
- Case 没有当前测试实体；
- Case 引用不存在的实体；
- 当前测试实体没有任何 Case。

同一测试实体出现在多个 Case 是合法关系，不产生重复错误。检查结果直接从 scanner 与 Case source 计算，不读取 committed inventory 或 index。

### Decision 5: 查询直接读取 Case，不建立 committed projection

保留 `topics`、有界 `list` 和单 Case `show` 作为只读维护入口；它们直接解析 `topics.json` 与 topic files。`list` 只按精确 topic、owner、entity key 和文本缩小结果并支持 limit/offset；精确 Case ID 查询只由 `show <CASE-ID>` 承担。`topics` 能返回空 topic，`check` 是唯一需要执行完整项目 scanner 的严格入口。

删除 `sync`、`changes`、Entry/Claim kind filters 和索引回退。后续若实体实现变更复审成为明确需求，应另立 change 选择 transient baseline 或其它机制，不能为该潜在需求在本轮保留 inventory/index。

### Decision 6: 历史 101 个 implemented Case 是逐项复核种子

迁移逐一读取 `2ec2de7:docs/testing/cases.md` 的 102 个记录。对其中 101 个 `implemented` Case，历史状态只证明它是 review seed，不证明它今天应成为 current Case。只有本 change 实现开始前已经存在、且能直接产生该历史语义可观察信号的当前测试实体时，才保持语义连续的 ID，再按当前 owner 文档修正单一 Owner/Proves 并精确关联 entity key。生产能力仍存在但没有这种起点直接实体时，该 seed 不迁移，也不构成本 change 新增或改写产品测试的义务；是否补足直接测试必须由独立的 owner-driven product test change 评估。生产能力已移除时，可以用明确的当前 owner/source 依据退休该 seed。未迁移或退休的 ID 都不得用于不同测试目的，也不能用空 Case 假装仍有当前证据。

本次逐项复核据此区分两类处置：

- `WB-TYPED-FIELDS-PRESENCE-001`、`WB-TYPED-FIELDS-METADATA-001`、`WB-TYPED-FIELDS-CONSTRAINTS-001`、`WB-TYPED-FIELDS-RANGES-001` 所述生产能力仍存在，但本 change 实现开始前没有直接测试实体；它们不迁入 current ledger，本 change 不为它们补产品测试。
- `WB-TYPED-FIELDS-PROJECTION-001` 与 `WB-TYPED-FIELDS-COMPILE-001` 所依赖的旧 FieldDefs derive/projection API 已移除；它们按生产能力移除规则退休，而不是被解释成缺测试待办。

唯一 `planned` release Case 不迁入当前账本，继续由 Git 历史、OpenSpec 或当前规划 owner 承接。历史 `Code:`、smoke task 和 marker 只用于寻找候选，不自动选择整个文件或生成 owner 文案。迁移后对全部当前测试实体做反向缺口检查，并为历史账本未覆盖的当前语义补充或调整 Case；一次性对照结果不作为 committed artifact。

### Decision 7: 一次硬切换，不维护兼容双读

实现先让新 parser、scanner join 和 focused tests在隔离 fixture 上通过，再迁移 Case 并同步 owner 文档/skill，最后删除 Claim、topic catalog、native inventory、query index 及其专用 schema/code path。required workspace check 在同一变更中只接受新模型。

旧模型没有外部运行时消费者，兼容双读只会增加错误分支和权威冲突。若需要回退，直接 revert 本 change 恢复完整旧文件集合；不做在线数据迁移或外部状态回滚。

### Decision 8: 可执行验证归项目 wrapper，skill 只保留审查指导

Topic/Case parser、scanner join、query 和 diagnostics 放在 `scripts/test-evidence/`，与本项目 runner profile 和 required check 共同拥有。项目级 `test-evidence-review` skill 保留通用审查顺序、Case 质量门槛和维护说明，但不再分发一份独立 runtime catalog、声明文件或持久 schema 集合。

继续从 skill runtime 模块导入项目校验会保留两个代码 owner 和跨目录接口；为只有当前项目消费的 Markdown grammar 再建立通用 library 或 JSON projection schema也没有当前义务。局部项目实现与 focused tests 是总维护面更小的正确候选。

## Risks / Trade-offs

- **[历史 Case 与当前测试树或生产能力已漂移]** → 对 101 个 historical implemented Case 逐项核对 current owner、production source 与本 change 起点已有的精确 entity keys；只有起点直接实体支持的语义才保留 ID 并迁移，能力仍在但缺直接实体者不迁移且留给独立 owner-driven product test change，能力已移除者以明确依据退休，planned Case 不迁移。路径匹配只生成候选，最终以双向 coverage 和目标测试审查验收。
- **[一个测试实体被多个 Case 引用可能掩盖过宽证明]** → 允许关系复用，但每个 Case 的 Owner/Proves 必须独立通过可观察性审查；不把“已被任意 Case 引用”等同于语义充分。
- **[全树 `check` 继续执行多个 runner，查询却不执行 scanner]** → 明确职责：查询只浏览账本，交付门禁才验证当前性；不为查询速度提交缓存。
- **[completed 但未归档的旧 OpenSpec changes仍记录 Entry/Claim 方案]** → 把它们视为历史设计依据；本 change 的 owner 同步与归档顺序必须让最终 `test-evidence-management` spec 只保留新契约，不修改旧 change 原文。
- **[硬切换使旧命令和过滤器立即失效]** → 在主规范、skill、help、tests 和 required check 同步替换；仓库外没有受支持的 public consumer，不增加兼容层。

## Migration Plan

1. 以当前完整 scanner 输出建立只存在于迁移进程内的测试实体集合，并再次证明 static/runtime/映射闭合。
2. 实现 topic catalog/topic-file parser、Case model、owner/entity/coverage diagnostics 和 read-only query；用 synthetic fixtures 先证明空 topic、单一 Owner、many-to-many 与全部失败边界。
3. 从 commit `2ec2de7` 逐项复核 101 个 implemented seed，并明确排除唯一 planned Case；只有本 change 实现开始前已有直接当前测试实体的语义才按稳定 owner 划分 topic files并保留 ID，能力仍在但缺这种实体者不迁移且不在本 change 反向补产品测试，生产能力已移除者以 owner/source 依据退休。随后校正已迁移 Case 的 Owner、Proves 与当前 entity keys，对 scanner 全集做反向检查并补齐由当前实体实际证明的语义缺口。
4. 把项目 wrapper、test-evidence-review skill、测试策略、维护文档和 workspace check 切到新模型；删除 skill runtime catalog/schema、Claim/Entry/inventory/index 及旧命令实现，不保留双读。
5. 运行 focused schema/parser/scanner tests、目标 `test-evidence check`、required workspace profile和范围匹配的完整 workspace 验证；用局部 diff 确认旧模型只留在历史 Git/OpenSpec artifact 中。
6. 若实现无法在同一提交满足全部当前测试实体与 Case 双向覆盖，停止切换并 revert 整个 change；不得提交部分迁移或临时豁免。

## Open Questions

无未回答开放问题，可以进入实现前审计。
