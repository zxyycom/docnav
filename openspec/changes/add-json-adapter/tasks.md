**一句话核心：以最小静态 JSON adapter 完整走通现有产品链路并记录真实扩展摩擦，所有实现任务必须等待阻塞审计完成；本文件是仅位于本 change 目录下的未审核临时任务清单，不影响现有主规范或其它 change。**

## 0. 实现前阻塞审计

- [ ] 0.1 阻塞审计 proposal、design、`json-adapter`/`release-artifacts` delta 和本 tasks 是否都围绕“第二个真实静态 adapter”核心句，`json-adapter` 是否为稳定新 capability、`release-artifacts` 是否复用现有 owner；0.1、0.2 和 0.3 未全部完成前不得执行任何 1.x 及后续实现任务。
- [ ] 0.2 阻塞审计本 change 是否只包含 `openspec/changes/add-json-adapter/` 下的未审核临时 artifacts、未修改主规范/实现/其它 change，`## Open Questions` 是否无未回答问题或已收敛歧义，并确认 dynamic registration、adapter-owned public parameter、第三格式预抽象、streaming/index 和无关重构不在范围。
- [ ] 0.3 产品启动门禁：确认 `ship-markdown-cli-beta` 已公开、从 public asset 复验 Quick Start，并由用户基于初步使用或反馈明确记录继续第二格式的理由；若该决定不成立，本 change 保持草案且不得开始实现。

## 1. JSON owner 文档与证明目标

- [ ] 1.1 新增 `docs/adapters/json.md`，定义 probe、重复 key、deterministic traversal、canonical ref、outline/read/find/info、full-read、pagination、cost 和错误 owner 语义，并把它接入 `docs/navigation.md`。
- [ ] 1.2 按测试策略、case maintenance 和 coverage owner，先写 JSON parser/ref/operation、core selection、CLI smoke 与 package smoke 的“owner 语义 -> 可观察结果”证明目标，更新 case ledger/coverage plan。
- [ ] 1.3 准备最小长期 fixtures，覆盖 mixed tree、special pointer tokens、empty roots、root scalar、duplicate keys、invalid/trailing input、Unicode pagination 和 long result；同一等价类不重复枚举。

## 2. Adapter-private document 与 ref mechanics

- [ ] 2.1 新增 `docnav-json` workspace crate 和最小模块边界，只依赖既有 adapter contracts、protocol、text-cost、serde/serde_json。
- [ ] 2.2 先写 owner tests，再实现 UTF-8/BOM load、完整输入 decode、重复 object member rejection、稳定 root kind/node count/max depth 和 adapter diagnostic mapping。
- [ ] 2.3 先写 owner tests，再实现 `json:`/RFC 6901 canonical ref encode/parse/resolve，覆盖 `~0`/`~1`、root、canonical array index、`REF_INVALID` 与 `REF_NOT_FOUND`。
- [ ] 2.4 先写 owner tests，再实现 object-key 排序、array-index 顺序和 depth-first preorder traversal，不把通用树 framework 提取到 shared crate。
- [ ] 2.5 实现 deterministic pretty JSON 与 original-source full-read 两条 text 路径，并复用现有 cost/pagination mechanics 验证 Unicode-safe continuation。

## 3. Fixed adapter strategy

- [ ] 3.1 实现 manifest 与 probe，使用 `docnav-json`/`json`/`.json`/`application/json`，覆盖 automatic/declared selection 所需的 supported 与 conflict reasons。
- [ ] 3.2 实现 outline，覆盖 descendant preorder、value kind、完整 ref、root scalar/empty-container fallback、超长 item 和 page termination。
- [ ] 3.3 实现 read，覆盖规范化 selected value、输入 ref 保留、content type、完整 cost、pagination 及 parse/ref failure。
- [ ] 3.4 实现 find，覆盖非空 literal query、pointer/scalar corpus、每 node 最多一个 match、确定顺序、bounded label 和 find-to-read roundtrip。
- [ ] 3.5 实现 info 与 declared unstructured full-read capabilities，覆盖 stable metadata、raw source、content type 和 lines/bytes/tokens cost。
- [ ] 3.6 导出唯一 registry-facing `json_adapter_definition()`，验证 definition semantics，并确认 adapter crate 没有参数声明、raw source resolution 或 protocol/output orchestration。

## 4. Core 静态集成

- [ ] 4.1 将 `docnav-json` 加入 workspace/core dependency 和现有 static factory slice，保持 registry 顺序确定且不增加动态 discovery path。
- [ ] 4.2 更新 registry、adapter list 和 doctor 的 owner tests，证明 Markdown/JSON 两个 `core_static` definition、manifest/probe metadata 和无独立 adapter executable。
- [ ] 4.3 更新 navigation/core integration tests，证明 `.json` automatic selection、`--adapter docnav-json` declared selection、probe rejection 和 closed operation dispatch。
- [ ] 4.4 增加显式 parity 断言或审计，证明 core parameter catalog inventory、CLI/env/config/protocol accepted input 和 `StandardInputBinding` 未因注册 JSON 扩大。

## 5. CLI、release 与验证材料

- [ ] 5.1 扩展真实 core CLI smoke，用实际 JSON fixture 覆盖 automatic selection、outline/ref/read、find/ref/read、readable-view、`protocol-json` 和代表性 failure。
- [ ] 5.2 扩展 canonical package smoke，用 package 中同一个 `docnav` binary 覆盖 Markdown 与 JSON roundtrip 及双 adapter list；不得查找独立 JSON executable。
- [ ] 5.3 同步更新测试策略、release docs、case ledger、coverage mapping、源码 `@case` 标记以及受 JSON manifest/probe/example 影响的验证材料。
- [ ] 5.4 运行 JSON crate focused tests、core/navigation focused tests、CLI smoke、package verify/smoke，并确认 pagination/ref roundtrip 在终止前持续前进。

## 6. 架构观察与交付审计

- [ ] 6.1 在 `design.md` 追加 `## Implementation Observations`，记录实际接入点、shared contract/catalog 是否变化、跨 adapter 重复、不可预测修改点、职责绕行和被推迟问题，不写第三格式假设。
- [ ] 6.2 对实现 diff 做 minimal-implementation 审计；删除 dynamic/generic escape hatch、未使用 option、第三格式 helper 和与 JSON 产品语义无关的 shared refactor。
- [ ] 6.3 运行 `cargo fmt --check`、范围匹配的 clippy/test 后运行 `bun run verify:docnav-workspace` 与 `openspec validate add-json-adapter --type change --strict --no-interactive`。
- [ ] 6.4 根据 Implementation Observations 判断是否存在有证据的后续结构 change；不存在重复或阻塞级摩擦时明确结束通用化验证，不创建预防性优化任务。
