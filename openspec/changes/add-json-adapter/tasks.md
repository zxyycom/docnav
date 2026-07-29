**目标：以最小静态 JSON adapter 完整走通现有产品链路并记录真实扩展摩擦。**

**状态：本文是 change-local candidate task list。[Decision Map](design.md#decision-map) 标出已确认方向和唯一剩余 gate；0.5 source-order 实验完成后进入 1.x。当前 change 使用 generic `readable-view`，格式专用自定义渲染由 6.4 交接到必需的后续 change。Current 能力仍以 `docs/`、代码和测试为准。**

## 0. 实施入口

- [x] 0.1 完成 artifact owner 审计：proposal 定义目标和范围，design 映射决策与 mechanics，`json-adapter` delta 定义目标行为，`release-artifacts` delta 修改 package 验证，本文件管理执行顺序。
- [x] 0.2 完成范围审计：change 内容与 design 的 Target Scope 一致；现有 static delivery、closed input、shared protocol 和 owner boundaries 构成实施基线。
- [x] 0.3 确认启动依据：第二个真实 adapter 用于架构边界验证，JSON 是已确认样本；Decision Map A0 连接长期理由与本 change。
- [x] 0.4 完成长期决策维护：Decision Map 中可独立修订的活动判断均为 active unaligned，决策关系与索引检查通过。
- [ ] 0.5 执行 E1 object source-order 有界实验。在 A2/A3/A7 所需 adapter-private decode model 与 source regions 上比较局部表示、memory、branching 和 maintenance 成本；成本保持当前 model 量级时保留 source order，否则采用 parser/model 的确定性顺序。把结果同步到 design、delta、tasks 和测试目标后进入 1.x。
- [x] 0.6 确认 JSON 自定义渲染的交付顺序：当前 change 用既有 generic `readable-view` 走通全部 fixed operations 并记录格式假设；信息密度、层级、标点、preview、分页显示和 renderer mechanics 由相连的后续 change 完整确定，作为 adapter 边界验证的必需阶段。

## 1. JSON owner 文档与证明目标

- [ ] 1.1 将跨格式活动决策同步到稳定 owner：`docs/architecture.md` 承接真实异构 adapter 的完整行为证据门槛以及 structured/full-read/raw/readable 分层，`docs/adapter-contract.md` 承接由真实重复职责形成共享抽象和按证据选择源码顺序的 adapter 边界，`docs/output.md` 承接 generic `readable-view`、格式专用自定义渲染与 raw machine contract 的分层。
- [ ] 1.2 新增 `docs/adapters/json.md`，定义 probe/reason、重复 key、原始 number token、E1 选定顺序、canonical `json:#` ref、outline、原文 find 与 source-region-to-ref 映射、read/info、full-read、pagination、cost 和错误 owner 语义，并把它接入 `docs/navigation.md`。
- [ ] 1.3 依次读取 `docs/testing.md`、JSON/adapter/ref/core/release 行为 owner、`docs/testing/case-maintenance.md`、`docs/testing/coverage.md` 和项目级 `test-evidence-review` skill；修改任何测试前运行 `bun run test-evidence -- check --root .` 证明完整当前树闭合。随后写清 JSON parser/ref/operation、core selection、CLI smoke 与 package smoke 的“owner 语义 -> 可观察结果”，查询并规划当前 Case/实体映射与覆盖维度。
- [ ] 1.4 准备最小长期 fixtures，覆盖 mixed tree、`~`/`/`/空 key/control/非 ASCII pointer tokens、object `"01"` 与 array `01`、empty container roots、root scalar、duplicate decoded keys、invalid/trailing input、`max_depth` 127/128 边界、Unicode pagination、大数/指数原始 token、key/scalar/结构/空白/跨节点原文命中、重复 ref occurrence 和 long result；每个 fixture 对应一个独立等价类或跨层复用目标。

## 2. Adapter-private document 与 ref mechanics

- [ ] 2.1 新增 `docnav-json` workspace crate 和最小模块边界，依赖集合限定为既有 adapter contracts、protocol、text-cost、serde/serde_json；启用 `serde_json` `raw_value` 或等价私有 mechanics 保存 number token。
- [ ] 2.2 先写 owner tests，再实现 UTF-8/BOM load、原文件 byte size、完整输入 decode、所有层级的 duplicate decoded member rejection、adapter-private 单一硬编码配置源中的 `max_depth <= 127`、原始 number token、node/member source regions、稳定 root kind/node count/max depth，以及 probe/operation-time diagnostic mapping。
- [ ] 2.3 先写 owner tests，再实现 `json:#`/RFC 6901 URI-fragment canonical ref encode/parse/resolve，覆盖 `~0`/`~1`、大写 canonical percent encoding、root/空 key/control/non-ASCII key、context-sensitive array index、超范围 canonical index、`REF_INVALID` 与 `REF_NOT_FOUND`。
- [ ] 2.4 先写 owner tests，再按 E1 选定的 object member 顺序实现 traversal，array 保持 index 顺序并执行 depth-first preorder；tree representation 保持 JSON adapter-private。
- [ ] 2.5 使用 workspace-pinned parser/serializer 的自然 scalar/escape/尾随换行结果生成两空格 structured JSON，只对原始 number token 做已批准的特殊保留；同时实现 BOM-stripped original-source full-read。复用现有 cost/pagination mechanics，分别证明完整 structured text cost、actual full-read text cost 和 Unicode-safe continuation。

## 3. Fixed adapter strategy

- [ ] 3.1 实现 manifest 与 probe，使用 `docnav-json`/`json`/`.json`/`application/json`，固定 supported/confidence/reason mapping，并覆盖 automatic/declared selection 和选择后文档竞争修改。
- [ ] 3.2 实现 outline，覆盖 descendant preorder、六种 value kind、确定性 label、完整 ASCII-safe ref、empty object/array 的空 entries、root scalar 的 `json:#` entry、无 JSON-specific metadata、超长 item 和 page termination。
- [ ] 3.3 实现 read，覆盖确定性 selected value、原始 number token、输入 ref 保留、content type、完整 cost、pagination 及 parse/ref failure。
- [ ] 3.4 实现 find，覆盖长度至少为一的大小写敏感原文 literal query、BOM-stripped source、source-order occurrence、key-to-value 与最深覆盖 region 映射、结构/空白/root 归属、同 ref 重复 occurrence、bounded source excerpt、line location、entry pagination 和 find-to-read roundtrip。
- [ ] 3.5 实现 info 与 declared unstructured full-read capabilities，覆盖 stable metadata、包含 BOM 的原文件 size、BOM-stripped raw source、content type 和实际返回 text 的 lines/bytes/tokens cost。
- [ ] 3.6 导出唯一 registry-facing `json_adapter_definition()`，验证 manifest、strategy、full-read capability 和 closed-input semantics；raw source resolution 与 protocol/output orchestration 继续由现有 owner 承接。

## 4. Core 静态集成

- [ ] 4.1 将 `docnav-json` 加入 workspace/core dependency 和 static factory slice；追加 JSON definition，并按 registry owner 的当前 membership、ordering 和 static discovery contract 保留其它 definitions。
- [ ] 4.2 更新 registry、adapter list 和 doctor 的 owner tests，证明 Markdown/JSON 两个必需 `core_static` definition、manifest/probe metadata 和单一 core executable delivery；整体 registry 断言使用 owner-defined membership 与 ordering 语义。
- [ ] 4.3 更新 navigation/core integration tests，证明 `.json` automatic selection、`--adapter docnav-json` declared selection、probe rejection 和 closed operation dispatch。
- [ ] 4.4 增加显式 parity 断言或审计，证明注册 JSON 前后的 core parameter catalog、CLI/env/config/protocol accepted input 和 `StandardInputBinding` inventory 相等。

## 5. CLI、release 与验证材料

- [ ] 5.1 扩展真实 core CLI smoke，用实际 JSON fixture 覆盖 automatic selection、ASCII-safe outline/ref/read、原文 find/ref/read、原始 number token、generic `readable-view`、`protocol-json` 和代表性 selection/ref/TOCTOU failure。
- [ ] 5.2 扩展 canonical package smoke，用 package 中同一个 `docnav` binary 覆盖 Markdown 与 JSON roundtrip 及两个必需 adapter id；其它内置格式按 release owner 的当前 registry contract 保留代表性 roundtrip。
- [ ] 5.3 同步更新测试策略、release docs、完整当前树的语义 Case 与当前测试实体映射、coverage mapping，以及受 JSON manifest/probe/example 影响的验证材料；按 `docs/testing/case-maintenance.md` 只为当前实体直接证明的独立 owner 契约或可观察结果新增/拆分 Case，历史材料不建立额外测试义务。
- [ ] 5.4 运行 JSON crate focused tests、core/navigation focused tests、CLI smoke、package verify/smoke，再运行完整 `bun run test-evidence -- check --root .`；确认 pagination/ref roundtrip 在终止前持续前进。

## 6. 架构观察与交付审计

- [ ] 6.1 在 `design.md` 追加 `## Implementation Observations`，记录实际接入点、shared contract/catalog 变化、跨 adapter 重复、不可预测修改点、职责绕行，以及 generic `readable-view` 在 JSON outline/read/find/info 上暴露的格式假设。
- [ ] 6.2 对实现 diff 做 minimal-implementation 审计；最终 diff 只保留 JSON 产品语义、static integration、owner evidence 及由实际阻塞证明必要的 shared change。
- [ ] 6.3 运行 `cargo fmt --check`、范围匹配的 clippy/test 后运行 `bun run verify:docnav-workspace` 与 `openspec validate add-json-adapter --type change --strict --no-interactive`。
- [ ] 6.4 基于稳定 raw facts 和 Implementation Observations 建立相连的 JSON 自定义渲染 follow-up change；该 change 完整承接信息密度、层级、标点、preview、分页显示和 renderer mechanics，并继续 adapter 完整行为的边界验证。
- [ ] 6.5 根据 Implementation Observations 判断其它后续结构 change；只有重复职责或阻塞级摩擦形成证据时建立，否则以当前 static boundary 结束本轮 raw adapter 交付。
- [ ] 6.6 将 Decision Map 中所有 active unaligned 决策分别与稳定 owner、代码、测试和 release evidence 做完整事实核对；只对已全部成为 Current 基线的记录执行 `mark-aligned`，其余保持 unaligned 并写明实际差距。完整行为证据与格式专用渲染有关的决策保持 unaligned，直到后续 presentation change 完成。
