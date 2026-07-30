本 design 是 `replace-probe-traversal-with-inferred-routing` 的临时技术工件：它定义单次内部格式推断、精确 registry 路由和无 fallback dispatch 的目标形状，并把依赖选择与 probe 兼容性保留为实施前人工 gate。

## Context

Current `docnav-navigation::select_adapter` 有两条路径：declared path 先按 adapter id lookup 再执行该 adapter 的 probe；automatic path 按 static registry 顺序逐个执行 probe，选择第一个 `supported: true` 的 definition，并把失败候选收集进 `FORMAT_UNKNOWN` details。`Adapter::probe` 和 `AdapterDefinition::probe` 因此同时承担格式识别入口和 selection mechanics。

本次 source audit 找到的 production 调用链是 `select_adapter -> AdapterDefinition::probe -> Adapter::probe`；没有找到 routing 之外的 owner-backed production probe caller。`adapter list` 从 definition manifest 读取 identity、format descriptors 与 implementation source。Probe 仍占有 shared protocol type/re-export/decode、runtime contract validator、consumer-local typed-field definitions、JSON Schema、examples/fixtures 和 candidate diagnostic vocabulary。长期 owner 规范仍把这些 surface 写成 Current，因此实现必须同步 architecture、adapter、navigation、diagnostics、protocol、contract-validation、typed-fields、schema/fixture 与测试证据，不能仅替换函数。

约束如下：

- Navigation 仍拥有 routing input、adapter selection、selected-operation resolution 和 dispatch；adapter 仍拥有真实 document decode/parse、格式语义、ref 和 operation algorithm。
- Automatic selection 只能执行一次格式推断，不能把 registry-order probe traversal 换成另一种候选循环。
- Explicit `--adapter` 必须跳过 inference，但 registry lookup 不能冒充 document validity；selected operation 仍执行真实 parse。
- 候选库的 enum、confidence、message、error 或 detection trace 不能进入 protocol、readable output、diagnostic details、logs、ref 或 continuation。
- 新 dependency 尚未批准。Cargo manifest、lockfile、production code、schema 和 owner docs 在 gate 通过前都不属于本 proposal 阶段的写入。

## Goals / Non-Goals

**Goals:**

- 让 automatic selection 与 registry order 无关，并把选择收敛为一个 inference outcome 和一个 exact lookup。
- 保持 selected adapter 的完整 parse/validation，且任何 selected failure 都不触发 adapter fallback。
- 以现有 selection diagnostic family 表达 unknown、unsupported 与 ambiguous，且只使用 Docnav-owned reason/details。
- 选择总维护面最小的实现：直接调用获批 inference library、显式 normalization、精确 registry match；不建立 detector framework。
- 在依赖、兼容性、target、size、startup 和格式覆盖证据齐全后由人类决定是否实施。

**Non-Goals:**

- 不公开格式推断 API，不允许 caller、adapter 或 plugin 注册 detector。
- 不设计 confidence score、tie-breaker、heuristic chain、fallback parse 或通用 media-type database。
- 不改变 ref、operation result、protocol envelope、readable rendering、pagination、full-read 或 caller parameter catalog。
- 不解决 project-wide traversal、parser-state reuse、runtime benchmark owner、code adapter 实现或跨进程 adapter hosting。

## Decisions

### Decision 1: Automatic routing 固定为一次 inference、一次 normalization 和一次 exact match

不存在 declared adapter id 时，navigation 对 normalized document path 调用获批 inference implementation **恰好一次**。调用结果立即映射为 invocation-private、project-owned outcome：

```text
Recognized(normalized_format_id)
Unknown
Ambiguous(normalized_format_ids[])
DocumentFailure
InternalFailure
```

`Recognized` 中只保留 Docnav format identity。Normalization 使用有限、显式、可审阅的 match/table 把获批库的私有结果映射到 manifest format ids（例如 `markdown`、`json`）；它不是 public enum、extension point 或 adapter callback。

Registry construction/release validation 先从 `AdapterDefinition.manifest.adapter.formats[].id` 派生唯一 format index；同一 normalized format id 出现在多个 definitions 中是 release blocker。Automatic routing 随后按 normalized format id 做 exact equality：

- 恰好一个 definition 命中：选中该 definition。
- 没有 definition 命中：`unsupported` selection failure。
- inference 返回多个 normalized identities：`FORMAT_AMBIGUOUS` selection failure；details 只列出其中能精确映射的 project adapter candidates，可能为空或只有一个，且不得据此猜 winner。
- 已验证 registry 仍在 runtime 出现 duplicate identity：global internal registry failure；不得降级为 per-document ambiguity。

`Unknown`、inference ambiguity 和 registry invariant failure 都在 dispatch 前结束。Automatic path 不调用任何 adapter probe，不运行“尝试下一个 adapter”，也不保留 registry-order candidate failure evidence。

替代方案是保留 probe traversal 但先按 extension 缩小 candidates，或让每个 adapter 实现新的 inference trait。这两者仍有候选顺序、重复 I/O 和第二套 adapter framework，故不采用。

### Decision 2: Explicit adapter intent 只做 exact id lookup，并跳过 inference

存在 resolved declared adapter id 时，navigation 只在 static registry 中做 exact adapter-id lookup：

- 命中时选中该 definition，不执行 format inference 或 probe。
- 未命中时返回现有 `ADAPTER_UNAVAILABLE` selection diagnostic。

Selection success 只证明 caller 指定的 linked strategy 存在，不证明文档有效。Navigation 完成既有 selected-operation input resolution 后，dispatch 该 strategy；strategy 必须执行本 operation 正常需要的 acquisition、decode、parse 和 semantic validation。显式选择错误 adapter 因而得到 selected adapter 的正常 diagnostic，而不是静默成功或另选 adapter。

替代方案是 explicit lookup 后继续 inference/format equality check。它会削弱 caller override，并恢复第二次识别，因此不采用。

### Decision 3: 复用既有 codes，并固定 exact public details 与 project classification

Routing 不新增按 inference mechanism 命名的 public diagnostic code。`FORMAT_UNKNOWN`、`FORMAT_AMBIGUOUS`、已有 document codes、`ADAPTER_UNAVAILABLE` 和 `INTERNAL_ERROR` 继续由 diagnostics/protocol owner 投影。Exact outcome mapping 是：

| Outcome | Canonical code | Exact public `details` | `add-project-wide-find` handoff |
| --- | --- | --- | --- |
| inference 没有 project-normalized identity | `FORMAT_UNKNOWN` | `{"path":"<normalized-path>","reason":"FORMAT_NOT_RECOGNIZED","candidates":[]}` | normal filter；不进入 local failures |
| 已识别 normalized identity，但 registry 无 adapter | `FORMAT_UNKNOWN` | `{"path":"<normalized-path>","reason":"NO_SUPPORTED_ADAPTER","format":"<normalized-format-id>","candidates":[]}` | normal filter；不进入 local failures |
| inference 返回多个 normalized identities | `FORMAT_AMBIGUOUS` | `{"path":"<normalized-path>","candidates":[{"adapter_id":"<mapped-project-adapter-id>","stage":"resolve","reason":"FORMAT_MATCH"},...]}`；只含能精确映射的 adapters，按 `adapter_id` 排序去重，允许空/单元素 | per-document local selection failure |
| inference path acquisition/permission/encoding failure | existing `DOCUMENT_*` code | 分别保持 `{"path":"..."}`、`{"path":"...","reason":"<Docnav-owned>"}` 或 `{"path":"...","encoding":"..."}` 的既有 exact shape | per-document local failure |
| inference implementation 无法归入 document failure | `INTERNAL_ERROR` | `{"error_id":"format-routing-failed"}` | per-document local failure |
| static registry duplicate normalized format identity 逃过 construction/release validation | `INTERNAL_ERROR` | `{"error_id":"registry-format-identity-conflict"}` | global fatal failure；终止 project invocation |
| explicit adapter id 不存在 | `ADAPTER_UNAVAILABLE` | `{"adapter_id":"<declared-id>","reason":"ADAPTER_NOT_FOUND","selection_source":"<resolved-source>","stage":"resolve"}` | global fatal failure |
| selected adapter parse/semantic/operation failure | adapter-owned existing code | adapter owner 的现有 canonical details；routing 不重写 | per-document local failure |

`FORMAT_UNKNOWN` 的 `format` 只在 `reason = "NO_SUPPORTED_ADAPTER"` 时存在；unknown outcome 禁止携带该字段。`FORMAT_AMBIGUOUS.candidates[]` 只包含 project-owned adapter id、固定 `resolve` stage 与固定 `FORMAT_MATCH` reason；旧 `candidate_failures`、`probe` stage 和 `PROBE_*` reason 全部删除。第三方 library enum、message、confidence、debug/error 和 detection trace 不进入 details、message、guidance、logs 或 project result。

一旦 definition 被选中，selection 生命周期结束。Selected strategy 的 document parse、semantic validation、operation error、invalid result 或 nested behavior failure继续走既有 owner contract；routing 不重新 inference、不检查 registry 后续成员、不 dispatch 第二个 adapter。这样 malformed content、TOCTOU 和 format-specific safety limit 始终由实际执行 owner 报告。

### Decision 4: Inference 是 routing hint，不取代 adapter parse

Inference 只回答“哪个 normalized format identity 应进入 exact lookup”，不构造 AST/document model、不生成 ref、不验证完整 adapter grammar，也不向 selected strategy传递 library state。Adapter operation 必须从 closed standard input 和 document path 走既有真实处理路径。

因此 JSON duplicate decoded member、depth limit、raw number、source region 等仍由 JSON adapter 拥有；Markdown heading/ref 语义仍由 Markdown adapter 拥有。若文件在 inference 后变化，selected operation 观察它按 Current independent-operation model 打开的文档 view，并返回 normal document 或 adapter diagnostic；不再存在 generic `json-document-changed-after-probe` 阶段名称。

把 inference parse tree 传给 adapter 可减少重复工作，但会泄漏第三方类型、耦合 parser、与 `reuse-adapter-document-state` 争夺 lifecycle owner，故不在本 change 中采用。

### Decision 5: 最小目标是删除 probe；兼容性证据不成立时必须先修订 artifacts

当前树没有 routing 外的 owner-backed production probe consumer，因此目标最小 contract 是完整删除：

- 从固定 `Adapter` strategy 和 `AdapterDefinition` 删除 probe method。
- 删除 selection-only candidate stage/evidence mechanics。
- 内置 Markdown/JSON 不再实现 selection probe。
- 删除 `ProbeResult`/`ProbeReason`/`ProbeValidationError`、protocol constants/re-exports/decode/schema entry、runtime contract validator、probe consumer-local typed-field definitions、JSON Schema、examples/fixtures 和 release/conformance references。
- `adapter list` 继续只投影 manifest identity、format descriptors、capabilities 和 implementation source。

实施前 audit 必须枚举 Rust public/re-export surface、CLI inspection、schemas/examples、release package、docs、tests、scripts 和可识别的外部兼容承诺。若发现真实 owner-backed consumer，当前实施必须停止并回到 artifacts 与人工批准，决定取消 change 或重新提出有独立 owner 的替代 change；不得在 apply 中临时保留 inspection surface。Routing probe retention 不是允许的 fallback。

这不是允许实现者二选一的 extension point；本 artifacts 的唯一 Target 是完整删除。保留无人消费的独立 probe 会增加永久接口与重复解析面，故不作为预防性设计。

### Decision 6: Dependency audit 和人工批准先于任何 Cargo 或 production 修改

Apply 的第一份记录必须是 change-local `dependency-audit.md`，并至少比较一个维护成熟的现有 Rust inference library、可行替代库和 no-new-dependency baseline。每个候选使用同一证据表：

1. ecosystem adoption、维护者/发布活跃度、issue 与 release cadence；
2. security advisories、unsafe 使用、transitive dependency graph 与供应链风险；
3. license、notice/attribution 与项目分发兼容性；
4. Rust edition、workspace/CI toolchain compatibility、声明或实测 MSRV；
5. `x86_64-unknown-linux-gnu` 和 `x86_64-pc-windows-msvc` build/test；
6. default/minimal features、增量 binary/package size 和 dependency count；
7. clean/cold/warm startup measurement，命令、build、host、cache state 和 repeats 可复现；
8. JSON/Markdown coverage：正常、大小写 extension、extensionless、空文件、BOM、non-UTF-8、误导 extension、content/extension conflict、malformed、ambiguous/polyglot 和代表性大文件；
9. future code-format fit、false positive/negative、failure taxonomy，以及 raw library facts能否被完全封装；
10. alternatives、拒绝理由、回滚成本和 recommendation。

Audit 只能形成 recommendation，不能批准 dependency。用户或指定 architecture/product owner 必须显式批准精确 crate、version requirement、features、normalization mapping 和接受的 size/startup/coverage trade-off。若没有候选通过，不得在本 change 下临时编写 custom detector；必须修订 artifacts 或结束 change。

`audit-runtime-performance-boundaries` 可以消费测量记录，但不是 gate 的 prerequisite 或数值 owner；本 change 只为依赖决定收集 bounded comparative evidence，不建立通用 benchmark framework。

### Decision 7: Production implementation 保持一个私有 helper 和一个 registry lookup seam

获批后，最小实现形状是 navigation routing module 中一个 private inference helper、一个显式 normalization mapping 和 registry 的 exact format lookup。Library 直接调用，不增加 `FormatInferencer` trait、service object、plugin registration、scoring config、generic confidence type 或 adapter-side detector。

Static registry 很小；registry construction 从 manifest 派生一个 validated format index，并让 doctor/release validation 阻断 duplicate identity。该 index 是 manifest facts 的派生 lookup view，不是第二个 metadata owner。Runtime 保留 `registry-format-identity-conflict` 防御性 global failure；不得把 duplicate identity 当作 `FORMAT_AMBIGUOUS` 或 order-based winner。除非 profiling 证明需要，不引入 cache 或其它 metadata store。

Dependency features 只启用批准 coverage 所需集合。Library upgrade、format mapping expansion 和新 format support 必须走现有 dependency/owner review，而不能由 raw upstream enum 自动扩大 Docnav support。

### Decision 8: Cross-change owner acceptance 是 implementation gate

- `add-project-wide-find`：本 change 是其 implementation predecessor。Project owner 必须先明确接受并完成该 predecessor gate，并记录：automatic `FORMAT_UNKNOWN` unknown/unsupported 是 normal filter；inference operational failure、inference ambiguity和 selected parse/operation failure 是 per-document local failure；registry format identity conflict 是 global fatal；explicit adapter missing 仍是 global fatal。本 change 不编辑其 artifacts 或实现 project traversal。
- `reuse-adapter-document-state`：本 change 取代其“registry-order first-supported”和“每个 unsupported/invalid candidate cleanup”假设。State-reuse owner 必须在本 change 实施前记录接受或拒绝 handoff；接受时把 candidate traversal/cleanup scope 缩为一次 inference 加一个 selected adapter，同时保留 selected document view、full-read、nested read、snapshot、cleanup、memory/lifetime 和 private-state价值。拒绝时双方 implementation 均保持阻断，直到人工决定 ordering/contract；本 change 不选择 state mechanism。
- `add-ast-grep-code-adapter`：owner 必须在本 change 实施前接受或拒绝 no-probe 与 exact multi-format mapping handoff。接受时每个 normalized language format exact match 到同一 `docnav-code` definition，parser types remain private；拒绝时本 change 不假设 code formats 可用，并保持 implementation gate 未通过。
- `audit-runtime-performance-boundaries`：其 `probe/routing` attribution 后续应解释为 inference/routing；startup/package-size evidence可被引用，但该 audit 不批准本 dependency，也不是本 change prerequisite。
- archived `add-json-adapter`：其 probe content validation、candidate rejection 和 `json-document-changed-after-probe` 是 migration input，不是继续保留 probe 的依据。Apply 时更新 Current JSON owner docs/spec/tests，让 selected JSON strategy 的正常 parse/TOCTOU diagnostic 取代 stage-specific reload 特例；不得改写 archived record。

### Decision 9: Protocol 与 process boundary 不扩大

Inference 仅在当前 linked `docnav` process 内发生，输出只存在于 invocation-private routing state。Protocol request/success response、CLI flags、config/env fields、readable output、ref、continuation 和 adapter process boundary 均不新增字段；failure details 只采用 Decision 3 的 owner-approved exact shape。Probe result 不再是 protocol type、schema 或 decode surface。

Selection diagnostics可投影 normalized format id 和 Docnav-owned reason，但不能投影 library crate name/version、enum debug、raw error、confidence 或 detection evidence。Invocation logs同样只允许稳定 selection layer、normalized identity（若已识别）、selected adapter id 和 outcome。

未来 external/service adapter host 如需格式 routing，必须单独决定 host-local inference 和 registry ownership；本 change 不创建 wire-level inference request 或 public session。

## Risks / Trade-offs

- [Risk] 候选库对 Markdown、JSON 或未来 code format 的识别不足，导致 Current 可用路径退化。
  → Mitigation：同一 corpus 做 coverage/false-result audit；任何 required cell 不满足即不批准，不能靠 adapter fallback 掩盖。
- [Risk] Automatic selection 从内容严格 probe 变为较粗 inference，使 malformed 文档在不同 layer 失败。
  → Mitigation：delta specs明确 selection 与 selected parse 分层；更新 JSON/Markdown owner cases，并保持单一 primary diagnostic。
- [Risk] 删除 probe 破坏未发现的外部 consumer。
  → Mitigation：blocking compatibility inventory；发现真实 owner-backed consumer 时停止 current apply并回到 artifacts/人工决定，禁止先删后补或临时保留 inspection。
- [Risk] 新 dependency 增加供应链、MSRV、package size 或 startup 负担。
  → Mitigation：Decision 6 的可复现 audit、精确 features 与人工 gate；未获批时 Cargo/lockfile保持不变。
- [Risk] 同一 format id 被多个 adapter 声明。
  → Mitigation：registry construction、doctor 与 release validation 阻断；runtime 仅保留 global `registry-format-identity-conflict` 防御性 failure，绝不使用 registry order。
- [Risk] 与 state-reuse change 的 probe assumptions 冲突。
  → Mitigation：在任一 implementation 前接受 Decision 8 handoff；本 change 删除 candidate traversal obligation，但不替代 selected-state lifecycle 决策。

## Migration Plan

1. 完成 `dependency-audit.md`、probe compatibility inventory、三个 cross-change owner acceptance 和人工 approval；随后更新所有 artifact 决策并关闭 Open Questions。
2. 通过 blocking artifact audit，证明 capability IDs、delta requirements、diagnostic taxonomy、dependency choice、probe outcome 和 implementation scope 完全一致。
3. 按项目 TDD/Case 流程先建立 automatic/explicit/failure/no-fallback 与 adapter parse evidence，再同步长期 owner docs、schema/examples/fixtures。
4. 实现一次 inference、normalization、validated registry index 和 exact lookup；迁移 explicit path；在同一 change 中完整删除 probe method/type/validator/typed-field/schema/evidence surface。
5. 运行 targeted tests、两 target build/release smoke、dependency/size/startup comparison 和 `verify:docnav-workspace`，再做 final diff 与 public-contract audit。
6. 回滚必须整体恢复旧 routing、adapter interface、schema/tests 和 dependency state；不得只恢复 registry traversal 而留下两套 detector 或死 schema。

## Open Questions

以下问题都有明确 owner 和关闭动作；在回答并回写 Decisions/specs 前，task 0 的 blocking audit 不能勾选，任何 implementation task 都不能开始。

1. **哪个精确 dependency/version/features 通过 Decision 6，并获得谁的显式批准？** `dependency-audit.md` 提供证据，用户或指定 architecture/product owner 决定；无批准或 no-dependency 结论等于不实施，并不授权 custom detector。
2. **probe compatibility inventory 是否发现真实 owner-backed consumer？** 当前唯一 Target 是完整删除。若发现，current apply 停止，用户/owner 决定取消本 change 或重新提出独立 owner change；不能在实施中临时保留 inspection。
3. **`add-project-wide-find`、`reuse-adapter-document-state` 和 `add-ast-grep-code-adapter` 的 owner 是否分别接受 Decision 8 handoff？** 三项 acceptance 都必须形成可审计记录；任一未接受或拒绝后未获人工处理时，本 change implementation 保持阻断。
