本 design 定义 `replace-probe-traversal-with-inferred-routing` 已批准但尚未应用的 manifest-native pathname routing、精确 registry lookup、显式强制选择和完整 no-probe 迁移，并保存实施前审计证据。

## Reading Contract and Decision Status

| 状态 | 本 design 中的内容 | 可执行含义 |
| --- | --- | --- |
| 已确认 Target | Decisions 1–7、9 的 pathname signal、manifest-native/zero-dependency mechanism、exact diagnostics、explicit override、no-fallback 与完整 probe 删除 | 后续实现不得静默改回 content detection、外部 MIME table 或 probe traversal |
| 已完成 process gate | Tasks 0.1–0.11 的调查、批准、删除面、handoff 与 artifact audit | Sections 1–7 已解除 planning gate；尚未完成任何实现任务 |
| 已核对 handoff | Decision 8 与“实施前审计记录” | 下游 change 接收单向 rebase；不要求其先实现，也不形成反向依赖 |
| 独立后续 change | `support-jsonc-in-json-adapter` | JSONC grammar、parser dependency 与 exact JSON/JSONC semantics 不由本 change 决定 |

正式调查主题 `docs/investigations/dependencies/format-routing-inference.md` 是证据 owner，不是批准 owner。活动决策 `docs/decisions/adapter-selection/route-by-manifest-basename-hints.md` 保存跨 change 默认方向，并在本 change 应用前保持 `unaligned`；本 design 和十二份 delta specs 保存本 change 的已批准 Target。主规范、代码和测试在 apply 前仍描述 Current。

## Context

Current `docnav-navigation::select_adapter` 有两条路径：declared path 先按 adapter id lookup 再执行该 adapter 的 probe；automatic path 按 static registry 顺序逐个执行 probe，选择第一个 `supported: true` 的 definition，并把失败候选收集进 `FORMAT_UNKNOWN` details。`Adapter::probe` 和 `AdapterDefinition::probe` 因此同时承担格式识别入口和 selection mechanics。

本次 source audit 找到的 production 调用链是 `select_adapter -> AdapterDefinition::probe -> Adapter::probe`；没有找到 routing 之外的 owner-backed production probe caller。`adapter list` 从 definition manifest 读取 identity、format descriptors 与 implementation source。Probe 仍占有 shared protocol type/re-export/decode、runtime contract validator、consumer-local typed-field definitions、JSON Schema、examples/fixtures 和 candidate diagnostic vocabulary。长期 owner 规范仍把这些 surface 写成 Current，因此实现必须同步 architecture、adapter、navigation、diagnostics、protocol、contract-validation、typed-fields、schema/fixture 与测试证据，不能仅替换函数。

Current core runtime 还会在进入 navigation 之前调用 filesystem-backed document-path normalization；该 helper 执行 metadata、file open 和 canonicalize。获批 Target 要求先判断 basename 是否可路由，因此 implementation 必须把入口改为两阶段：先从调用路径与 cwd 词法派生 `routing pathname` 并完成 selection；只有命中 adapter 或存在显式 adapter intent 后，才执行 filesystem-backed path/access normalization并构造 selected operation input。

约束如下：

- Navigation 仍拥有 routing input、adapter selection、selected-operation resolution 和 dispatch；adapter 仍拥有真实 document decode/parse、格式语义、ref 和 operation algorithm。
- Automatic selection 只能执行一次纯 lexical pathname hint lookup；在 route 命中前不能对目标文档执行 metadata、open、canonicalize、read 或 parse，也不能把 registry-order probe traversal 换成另一种候选循环。
- Explicit `--adapter` 必须跳过 pathname lookup；registry lookup 不能冒充 document validity，selected operation 仍执行真实 parse。
- Pathname hints、derived indexes 和冲突信息不能成为 caller 参数、public detection API、ref 或 continuation。
- Routing 不新增 dependency。Cargo manifest、lockfile、production code、schema 和 owner docs 只能按 tasks 1–7 的测试、同步、实现与验证顺序修改。

## Goals / Non-Goals

**Goals:**

- 让 automatic selection 与 registry order 和 document bytes 无关，并把选择收敛为一次 pathname hint lookup 和一次 exact format lookup。
- 保持 selected adapter 的完整 parse/validation，且任何 selected failure 都不触发 adapter fallback。
- 用 manifest-owned basename suffixes 与 exact filenames 表达全部 automatic routing facts；允许 compound suffix，并用最长 suffix 规则确定重叠命中。
- 以 `FORMAT_UNKNOWN` 表达无 hint match，以 global `INTERNAL_ERROR` 表达逃过 release validation 的 registry invariant failure。
- 保持零新增 routing dependency、一组 manifest-derived lookup views 和一个 lookup helper，不建立 detector framework 或第二个 metadata owner。

**Non-Goals:**

- 不公开 pathname detection API，不允许 caller、adapter 或 plugin 注册 detector。
- 不设计 confidence score、content heuristic、fallback parse 或通用 media-type database。
- 不改变 ref、operation result、protocol envelope、readable rendering、pagination、full-read 或 caller parameter catalog。
- 不实现 JSONC、JSON5、JSON Lines/NDJSON 或其它 JSON-family grammar；`.code-workspace` 和 `.prettierrc` 在这里仅是 best-effort routing hints。
- 不解决 project-wide traversal、parser-state reuse、runtime benchmark owner、code adapter 实现或跨进程 adapter hosting。

## Decisions

### Decision 1: Automatic routing 使用 manifest-owned pathname hints

不存在 declared adapter id 时，core/navigation 先从调用路径与 cwd 词法派生 invocation-private `routing pathname`，再检查其完整 basename。这个阶段不对目标文档执行 metadata、open、canonicalize、read 或 parse。Core 从 static registry manifests 确定性派生：

```text
exact filename    -> normalized format id
normalized suffix -> normalized format id
format id         -> adapter definition
```

`FormatDescriptor.extensions[]` 保存带前导点、可包含多个点的 basename suffix hints。匹配不是先提取“最后一个 extension token”，而是把完整 basename 与声明 suffix 分别做 ASCII 大小写归一化，再比较 basename 末尾；该行为可由 suffix comparison 实现，不向 manifest 暴露 regex 或 glob。新增的 `FormatDescriptor.filenames[]` 保存不含目录分隔符的 exact basename hints，匹配时保持大小写敏感的 exact spelling。两个数组都是 routing metadata，不是 parser validity proof。

Automatic lookup 的固定顺序是：

1. exact basename 命中 `filenames[]` 时使用其 format identity；
2. 否则在完整 basename 上匹配 ASCII-normalized `extensions[]` suffixes；
3. 若多个不同 suffix 同时命中，选择字符数最长的声明；例如 `model.schema.JSON` 由 `.schema.json` 覆盖 `.json`，而 `settings.json.backup` 不命中 `.json`；
4. 命中后按 normalized format id exact lookup definition；
5. 没有 hint 命中时返回 `FORMAT_UNKNOWN / FORMAT_NOT_RECOGNIZED`，且不对目标文档执行 filesystem I/O；
6. 命中后才执行 filesystem-backed path/access normalization，并为 selected operation构造 normalized document path。

Exact filename 有意优先于通用 suffix，使未来 `devcontainer.json` 之类的具体 JSON-family filename 可以覆盖通用 route。不同长度 suffix 的重叠由 longest-match 解决，不是冲突；ASCII 归一化后完全相同的 suffix 不能映射到多个 format identities。相同 exact filename 或 normalized format identity 的既有 registry 冲突仍由 construction、doctor 与 release validation 阻断。Validated indexes 因而不会产生 document-level ambiguity，也不会产生“recognized format but no linked adapter”；若冲突状态仍逃到 runtime，它是 global internal registry failure。

Automatic path 在 route 命中前不访问目标文件，不调用 adapter probe、不运行“尝试下一个 adapter”，也不保留 registry-order candidate evidence。替代方案中的 content detector、外部 MIME table、通用 regex surface、probe-prefilter 和 adapter-owned inference trait都会重新引入 I/O、第二份 metadata、长期 pattern contract 或候选遍历，因此不采用。

### Decision 2: Explicit adapter intent 只做 exact id lookup，并跳过 pathname routing

存在 resolved declared adapter id 时，navigation 只在 static registry 中做 exact adapter-id lookup：

- 命中时选中该 definition，不执行 pathname lookup 或 probe。
- 未命中时返回现有 `ADAPTER_UNAVAILABLE` selection diagnostic。

Selection success 只证明 caller 指定的 linked strategy 存在，不证明文档有效。Navigation 完成既有 selected-operation input resolution 后，dispatch 该 strategy；strategy 必须执行本 operation 正常需要的 acquisition、decode、parse 和 semantic validation。显式选择错误 adapter 因而得到 selected adapter 的正常 diagnostic，而不是静默成功或另选 adapter。

替代方案是 explicit lookup 后继续 pathname/format equality check。它会削弱 caller override，因此不采用。

### Decision 3: 固定 project-owned codes、exact public details 与 project classification

Routing 不新增按 mechanism 命名的 public diagnostic code。`FORMAT_UNKNOWN`、`ADAPTER_UNAVAILABLE`、既有 document codes 和 `INTERNAL_ERROR` 继续由 diagnostics/protocol owner 投影。完整删除 JSON probe 后，Current `json-document-changed-after-probe` 不能再承担 selected parse/safety failure，因此新增通用 `DOCUMENT_CONTENT_INVALID`；JSON adapter 拥有其 stable reason mapping，routing 只透传。Exact outcome mapping 是：

| Outcome | Canonical code | Exact public `details` | `add-project-wide-find` handoff |
| --- | --- | --- | --- |
| routing basename 未命中 manifest hint | `FORMAT_UNKNOWN` | `{"path":"<routing-pathname>","reason":"FORMAT_NOT_RECOGNIZED","candidates":[]}` | normal filter；不进入 local failures |
| duplicate normalized format identity 逃过 construction/release validation | `INTERNAL_ERROR` | `{"error_id":"registry-format-identity-conflict"}` | global fatal failure；终止 project invocation |
| duplicate exact-filename/normalized-suffix hint 逃过 construction/release validation | `INTERNAL_ERROR` | `{"error_id":"registry-path-hint-conflict"}` | global fatal failure；终止 project invocation |
| explicit adapter id 不存在 | `ADAPTER_UNAVAILABLE` | `{"adapter_id":"<declared-id>","reason":"ADAPTER_NOT_FOUND","selection_source":"<resolved-source>","stage":"resolve"}` | global fatal failure |
| selected document missing/path/encoding failure | existing `DOCUMENT_NOT_FOUND` / `DOCUMENT_PATH_INVALID` / `DOCUMENT_ENCODING_UNSUPPORTED` | 对应 owner 的现有 canonical details | per-document local failure |
| selected JSON syntax/trailing-input/duplicate-member/depth failure | `DOCUMENT_CONTENT_INVALID` | `{"path":"<normalized-path>","reason":"<stable-json-reason>"}` | per-document local failure |
| selected ref/other semantic/operation failure | adapter-owned code | adapter owner 的 canonical details；routing 不重写 | per-document local failure |

Manifest-derived routing 不使用 `NO_SUPPORTED_ADAPTER + format`，也不产生 `FORMAT_AMBIGUOUS`；若 task 0.5 证明这些 vocabulary 没有其它 Current owner，其 routing-only schema/example/type surface随 probe candidate evidence 一并删除。`FORMAT_UNKNOWN` 禁止携带 `format` 或 `candidate_failures`。旧 `probe` stage、`PROBE_*` reason 和 candidate failure arrays 全部删除。

JSON 的 `<stable-json-reason>` 精确为 `JSON_SYNTAX_INVALID`、`JSON_TRAILING_INPUT`、`JSON_DUPLICATE_MEMBER` 或 `JSON_MAXIMUM_DEPTH_EXCEEDED`。Reason 只分类 owner-observable failure，不暴露 parser type、raw message、unstable offset、member name 或 dependency trace。Invalid UTF-8 继续使用 `DOCUMENT_ENCODING_UNSUPPORTED`，不折叠进 content-invalid。

一旦 definition 被选中，selection 生命周期结束。Selected strategy 的 document read、parse、semantic validation、operation error、invalid result 或 nested behavior failure继续走 owner contract；routing 不重新匹配 pathname、不检查 registry 后续成员、不 dispatch 第二个 adapter。Known hint 的 missing path、malformed content、encoding failure、document change 和 format-specific safety limit 因而始终由实际执行 owner 报告。

### Decision 4: Pathname 是 routing hint，不取代 adapter parse

Pathname lookup 只回答“是否能选择一个 registry definition”，不构造 AST/document model、不生成 ref、不验证完整 adapter grammar，也不向 selected strategy传递 matched hint、format identity 或其它 routing state。这个低成本选择事实可能与真实内容不一致，因此把它加入 `StandardOperationInput` 只会扩大错误事实的传播面。Route 命中后，adapter operation 必须从 closed standard input 和 filesystem-backed normalized document path 走既有真实处理路径。

因此 JSON duplicate decoded member、depth limit、raw number、source region 与 `DOCUMENT_CONTENT_INVALID` reason 仍由 JSON adapter 拥有；Markdown heading/ref 语义仍由 Markdown adapter 拥有。`.md` 文件即使包含 JSON 也选择 Markdown；`.json` 文件即使包含 Markdown 也选择 JSON。若文件在 selection 后变化，selected operation 观察它按 Current independent-operation model 打开的 document view，并返回 normal document 或 adapter diagnostic；不再存在 generic `json-document-changed-after-probe` 阶段名称。

Initial JSON pathname hints 包括 `.json`、`.code-workspace`、exact `.prettierrc` 和 exact `.watchmanconfig`。`.prettierrc` 官方允许 JSON 或 YAML，`.code-workspace` 允许 JSON comments；在相应 grammar support 落地前，它们只是 best-effort route，合法但非严格 JSON 的内容仍由 selected JSON adapter 正常拒绝。`support-jsonc-in-json-adapter` 可以扩展同一 JSON strategy 的 grammar 与 hints，但不得改变本决策的 no-content-detection/no-fallback 边界。

### Decision 5: 本 breaking change 完整删除 probe，不提供兼容或 inspection fallback

本 change 明确接受 probe surface 的破坏性删除；backward compatibility 不是本次迁移的约束。目标最小 contract 是完整删除：

- 从固定 `Adapter` strategy 和 `AdapterDefinition` 删除 probe method。
- 删除 selection-only candidate stage/evidence mechanics。
- 内置 Markdown/JSON 不再实现 selection probe。
- 删除 `ProbeResult`/`ProbeReason`/`ProbeValidationError`、protocol constants/re-exports/decode/schema entry、runtime contract validator、probe consumer-local typed-field definitions、JSON Schema、examples/fixtures 和 release/conformance references。
- `adapter list` 继续只投影 manifest identity、format descriptors、capabilities 和 implementation source。

实施前 removal inventory 仍必须枚举 Rust public/re-export surface、CLI inspection、schemas/examples、release package、docs、tests、scripts 和可识别的外部承诺，以确保 breaking migration 完整。发现额外 owner-backed consumer 时，将其记录为必须删除、迁移或明确标注 breaking impact 的工作面；这不重新打开 probe retention，也不要求因兼容性停止 current apply。Routing probe retention 和 inspection-only compatibility surface 都不是允许的 fallback。

这不是允许实现者二选一的 extension point；本 artifacts 的唯一 Target 是完整删除。保留无人消费的独立 probe 会增加永久接口与重复解析面，故不作为预防性设计。

### Decision 6: Dependency audit 已关闭为 manifest-native、零新增依赖

正式调查主题 `docs/investigations/dependencies/format-routing-inference.md` 比较了成熟 Rust format candidates、no-new-dependency baseline 和 common project filename aliases。本 change 只消费其形成时证据，不拥有、复制或用 change 状态改写正式报告。审计覆盖：

1. ecosystem adoption、反向依赖/下载量/可信 production use 等可获得的热度证据、维护者/发布活跃度、issue 与 release cadence；
2. security advisories、unsafe 使用、transitive dependency graph 与供应链风险；
3. license、notice/attribution 与项目分发兼容性；
4. Rust edition、workspace/CI toolchain compatibility、声明或实测 MSRV；
5. `x86_64-unknown-linux-gnu` 和 `x86_64-pc-windows-msvc` build/test；
6. default/minimal features、直接与传递依赖数量、编译负担、增量 binary/package size，用于判断候选是否属于重量级依赖；
7. clean/cold/warm startup measurement，命令、build、host、cache state 和 repeats 可复现；
8. JSON/Markdown 功能充分性与 coverage：正常、大小写 extension、extensionless、空文件、BOM、non-UTF-8、误导 extension、content/extension conflict、malformed、polyglot 和代表性大文件；
9. future code-format fit、false positive/negative、failure taxonomy，以及 raw library facts能否被完全封装；
10. `.prettierrc`、`.code-workspace` 等 common pathname aliases、alternatives、拒绝理由、回滚成本和 recommendation。

用户已经批准 manifest-native mapping 和零新增 routing dependency。`mime_guess = "=2.0.5"` 虽然通过普通 extension corpus 且属于轻量依赖，但不能识别 `.prettierrc`，其表也没有 `.code-workspace`；采用它仍需 project-owned alias layer并形成第二份 extension knowledge。因此它和其它 external/content candidates 都不进入 implementation。Task 0.4 记录这一关闭结果及 measurement limitations；tasks 6.3–6.4 仍负责最终跨 target、binary/package 和 startup verification。

`audit-runtime-performance-boundaries` 可以消费既有测量记录，但不是 gate 的 prerequisite 或数值 owner；本 change 不建立通用 benchmark framework。

### Decision 7: Production implementation 保持一个私有 helper 和一个 registry lookup seam

最小实现形状是 navigation routing module 中一个 private basename helper，以及 core registry construction 从 manifests 派生的 exact-filename、ASCII-normalized suffix 和 format lookup views。Suffix lookup按最长声明优先，可用有序 suffix comparison 实现，不需要 regex engine。Indexes 是 manifest facts 的 validated views，不是第二个 metadata owner；实现不增加 `FormatInferencer` trait、service object、plugin registration、scoring config、generic confidence type、adapter-side detector、cache 或 external crate。

Registry construction、doctor 和 release validation 阻断 duplicate format identity、duplicate ASCII-normalized suffix 与 duplicate exact filename。不同长度 suffix 可以重叠并按 Decision 1 的 longest-match 处理；exact filename 与 suffix 分属不同 hint kinds，同一 basename 同时命中二者时按 filename precedence 处理。Runtime 保留 `registry-format-identity-conflict` 与 `registry-path-hint-conflict` 防御性 global failures；不得把真正的重复冲突当作 `FORMAT_AMBIGUOUS` 或 order-based winner。

新增 pathname hint、format identity 或 filename precedence exception 必须通过 adapter/manifest owner review；不得由 filesystem、MIME database 或 parser dependency 自动扩大 Docnav support。

### Decision 8: Cross-change handoff 建立单向 rebase，不建立反向实现依赖

- `add-project-wide-find`：本 change 是其 implementation predecessor。Final seam 是 automatic pathname no-match `FORMAT_UNKNOWN` 作为 normal filter；selected parse/operation failure 作为 per-document local failure；registry format/path-hint conflict 与 explicit adapter missing 作为 global fatal。Routing task 0.6 核对并记录该 planning acceptance，不要求 project task 1.3、implementation 或 validation 完成；最终顺序保持 routing 完成后再由 project task 1.3 接收最终 seam。本 change 不编辑其 artifacts 或实现 project traversal。
- `reuse-adapter-document-state`：本 change 取代其“registry-order first-supported”和“每个 unsupported/invalid candidate cleanup”假设。Task 0.7 只记录 downstream no-probe handoff；state-reuse 推进时按最终 Current routing pipeline 重建候选与 lifecycle 证据，同时保留 selected document view、full-read、nested read、snapshot、cleanup、memory/lifetime 和 private-state价值。本 change 不要求 state-reuse 先选择机制，也不选择其 mechanism；只有 source-backed audit 证明 no-probe 会破坏该 change 核心目标时才升级为人工决策。
- `add-ast-grep-code-adapter`：task 0.8 只记录 downstream no-probe/exact-routing handoff。Code-adapter change 在修改 dependency 或 production 前按最终 Current routing contract 重写旧 probe/registry-order artifacts；该 handoff 不在本 change 预选其 dependency、language coverage 或 parser implementation。
- `support-jsonc-in-json-adapter`：该 change 是 downstream grammar consumer。它可以让 selected JSON strategy接受获批 JSONC grammar并增加 `.jsonc` 或其它 JSON-family pathname hints；不得让 routing 读取 content、恢复 probe/fallback，或把 JSONC parser state变成 selection state。本 change 不预选其 parser dependency或 exact grammar。
- `audit-runtime-performance-boundaries`：其 `probe/routing` attribution 后续应解释为 pathname routing；startup/package-size evidence可被引用，但该 audit 不是本 change prerequisite。
- archived `add-json-adapter`：其 probe content validation、candidate rejection 和 `json-document-changed-after-probe` 是 migration input，不是继续保留 probe 的依据。Apply 时更新 Current JSON owner docs/spec/tests，让 selected JSON strategy 的正常 parse/TOCTOU diagnostic 取代 stage-specific reload 特例；不得改写 archived record。

### Decision 9: Protocol 与 process boundary 不扩大

Pathname routing 仅在当前 linked `docnav` process 内发生，derived match 只存在于 invocation-private routing state。Protocol request/success response、CLI flags、config/env fields、readable output、ref、continuation 和 adapter process boundary 均不新增字段；manifest 新增 `formats[].filenames[]`，failure details 只采用 Decision 3 的 exact shape。Probe result 不再是 protocol type、schema 或 decode surface。

Selection diagnostics 和 invocation logs 都不投影 matched filename/suffix、matched format identity 或 derived-index internals；日志只保留既有稳定 selection layer、selected adapter id 和 outcome。低置信度 pathname hint 不进入 adapter input 或其它持久/public state。

未来 external/service adapter host 如需格式 routing，必须单独决定 host-local pathname index 和 registry ownership；本 change 不创建 wire-level routing request 或 public session。

## Completed Pre-Implementation Audit

本节是 change-owned 的实施范围与 handoff 证据。审计基于 2026-08-03 的仓库 `HEAD` `ebca55a8564e1ae478e96a3c90645ca3bd7cf2db`、Current source/docs/schema/tests，以及四个下游 change 的 on-disk artifacts。它证明 planning scope 已闭合，不表示 Target 已经写入 Current owner、production code 或 release artifact。

### Probe removal inventory

以下每组 Current consumer 都有唯一 apply disposition；不得保留 compatibility 或 inspection-only probe surface：

1. **Adapter strategy 与内置 adapters — 删除并迁移。** 从 `crates/shared/adapter-contracts/src/lib.rs` 和 `definition.rs` 删除 `Adapter::probe`、`AdapterDefinition::probe` 与 `ProbeResult` import；从 `crates/adapters/markdown/src/adapter.rs`、`crates/adapters/json/src/adapter.rs` 删除 probe implementation/helper。对应 adapter tests、`crates/shared/adapter-contracts/src/tests/support.rs`、core registry test 和 invocation-logging recording adapter 改为 manifest hints、linked handler availability 与 selected-operation parse 证据。
2. **Navigation 与 core sequencing — 替换。** `crates/shared/navigation/src/routing.rs`、`context.rs`、`lib.rs` 的 registry-order traversal、`CandidateEvidence`、`CandidateStage`、attempted set 和 evidence handoff 整体替换为 exact-filename/normalized-suffix/format lookup；navigation test support 与 auto-read tests 改为已选 definition。`crates/docnav/src/runtime.rs` 与 `project_paths.rs` 的 Current path normalization 顺序改为 route-first、I/O-after-selection，并同步 core path/registry/logging tests。
3. **Protocol、runtime contract validation 与 typed-field consumer — 删除。** 删除 `crates/shared/protocol/src/probe.rs`，以及 `constants.rs`、`lib.rs`、`decode.rs`、`schema.rs`、`contract_validation.rs`、`contract_validation/probe.rs`、`contract_validation/enums.rs` 中的 probe version、type re-export、decoder、schema entry、semantic validation和 probe reason field definitions；删除对应 protocol decode/schema tests。Reusable typed-fields API 保留，只删除 protocol consumer 建立的 probe `FieldDefSet` 路径。
4. **Diagnostics 与 routing-only vocabulary — 删除或替换。** 从 shared diagnostics/protocol code、rules、typed markers、details payload/conversion、metadata 和 tests 删除 probe boundary codes、`FormatCandidateDetails`、`candidate_failures` 与 routing-only `FORMAT_AMBIGUOUS`；审计未找到 `FORMAT_AMBIGUOUS` 的其它 Current owner，也未找到 Current `FORMAT_MATCH` producer。`FORMAT_UNKNOWN` 改为 `FORMAT_NOT_RECOGNIZED` 与空 `candidates`；selected JSON content failure 新增 `DOCUMENT_CONTENT_INVALID` 和四个 exact reasons。
5. **Schema、examples 与 validators — 删除或同步。** 删除 `docs/schemas/probe-result.schema.json`、`docs/examples/json/probe-result.json` 及其 schema/example indexes、`scripts/tools/validators/config.ts` 和 schema validator registry entries；更新 manifest schema/example的 `filenames[]` 与 compound-suffix rules；更新 protocol-response schema 和 format-unknown example，删除 format-ambiguous example与 candidate/probe vocabulary。
6. **Owner docs、main specs 与 Case evidence — 迁移。** 更新 `docs/adapter-contract.md`、`architecture.md`、`cli.md`、`navigation.md`、`navigation-input-resolution.md`、`protocol.md`、`output.md`、`testing.md`、`testing/coverage.md`、JSON/Markdown adapter owners、schema/example indexes，以及 core/navigation/protocol/JSON/Markdown Case ledgers。对应 `openspec/specs/` owners 在 apply/archive 同步时接收十二份 delta；历史 decision 与 formal investigation 保留形成时事实，不改写成 Current。
7. **Smoke、TOCTOU 与 release-adjacent material — 迁移或删除。** Core failure/real-JSON smoke 改为 pathname no-match和 selected JSON diagnostics；probe-specific `test/tools/json-toctou-supervisor.py`、`real-json-toctou.ts` 与 `json-document-changed-after-probe` expectations 删除或由单次 selected-operation document/content evidence取代；`scripts/quality/accepted-warnings.ts` 随 JSON suite职责更新。Release workflow 没有 direct probe token，仍需通过 package smoke 和 workspace/schema validation证明最终 package 不携带 probe surface。

限定搜索没有发现上述集合之外的 owner-backed production caller、CLI inspection command、package manifest或 release workflow probe promise。Fixture 正文中普通英文 “probe” 与 runtime progress observer 不属于 adapter routing；决策和调查中的 probe 文字是历史证据，保留不形成兼容义务。

### Cross-change handoff audit

| Consumer | On-disk acceptance | Audit result and downstream action |
| --- | --- | --- |
| `add-project-wide-find` | Decisions 5/12 接受 routing predecessor、unknown/filter、selected/local 与 registry/explicit fatal seam；task 1.3 要求 predecessor 成为 Current 后重写重叠 requirements | Final reachable outcomes 与本 change 一致。Decision 5 中旧 inference ambiguity/I/O branches 是明确待 rebase 的 provisional wording，不是本 change obligation；project implementation/validation 仍在下游，无 core-goal conflict。 |
| `reuse-adapter-document-state` | Context、Decision 5/7 与 task 1.1 明确以 final no-probe pipeline 重建 packet，并保留 selected/full-read/nested-read/private-state目标 | Probe/candidate rows 只是历史比较。Routing 删除不会破坏 reuse 核心目标；snapshot、cleanup、source view 与 mechanism gate 仍由该 change 独立决定。 |
| `add-ast-grep-code-adapter` | Context、Decision 9 与 task 1.2 明确要求 routing 成为 Current 后先重写 probe/registry-order baseline | 只需 downstream rebase；本 change 不选择 ast-grep dependency、语言集合、format mapping 或 parser implementation，无 core-goal conflict。 |
| `support-jsonc-in-json-adapter` | Decision 8、Remaining Gates 与 tasks 0.3/0.11 把 routing 定义为单向 predecessor | JSONC 可增加 suffix 并扩展 selected JSON grammar，但不能传递 dialect、恢复 content detection/fallback或反向阻塞 routing；parser dependency gate 保持独立。 |

`audit-runtime-performance-boundaries` 只接收 measurement attribution，archived `add-json-adapter` 只提供 probe/TOCTOU migration history；两者都不是 implementation prerequisite。

### Artifact integrity audit

- Proposal 的十二个 capability ids 与十二个 delta directories、现有 main-spec ids 完全一致：`adapter-contract`、`contract-validation`、`core-cli`、`diagnostics-contract`、`docnav-architecture`、`invocation-logging`、`json-adapter`、`markdown-adapter`、`navigation-input-resolution`、`output-contract`、`protocol-contract`、`typed-fields`。
- 每个 `MODIFIED` requirement 复用现有 main-spec heading并保留 Current clauses；JSON/protocol 的新增 requirement 与 JSON/Markdown 的 removed probe requirements 各自提供完整 Target、reason 和 migration。
- Architecture、navigation、diagnostics、protocol、JSON owner、project handoff 与 tasks 对 no-match、registry conflict、explicit missing、selected failure 的 code/details/classification一致；routing state 不进入 adapter input、protocol、readable output或 logs。
- `## Open Questions` 没有未回答或被措辞隐藏的 product/dependency/compatibility选择。审计未发现需要升级给用户的下游 core-goal incompatibility。
- Apply 前仍以主规范、代码、测试和 release artifact 为 Current；本 change 的 planning readiness 不冒充实现完成。

## Risks / Trade-offs

- [Risk] Manifest pathname hint 缺少常见 suffix/exact filename，导致本可解析文档返回 `FORMAT_UNKNOWN`。
  → Mitigation：hints 由 format owner 维护有限 allowlist；先覆盖 `.json`、`.md`、`.markdown`、`.code-workspace`、`.prettierrc` 和 `.watchmanconfig`，新增 aliases 走 owner review。
- [Risk] Best-effort aliases 可能选择一个无法解析实际 grammar 的 adapter。
  → Mitigation：明确 hint 不是真实性保证；selected adapter 返回正常 parse diagnostic且不 fallback。JSONC、YAML 等 grammar由独立 change支持。
- [Risk] Exact filename、compound suffix 与 generic suffix 的 precedence 在未来 JSON-family adapters 增加后产生意外覆盖。
  → Mitigation：固定 filename-first、suffix ASCII-normalized、longest-suffix、filename exact 规则；construction/doctor/release validation覆盖重复与预期 precedence cases。
- [Risk] breaking probe deletion 遗漏仓库内 consumer、文档、schema 或 release material，形成半迁移状态。
  → Mitigation：blocking removal inventory 将所有发现的 consumer 作为删除、迁移或 breaking-impact 记录面；不得保留 inspection fallback，也不得以兼容性为由留下旧 probe surface。
- [Risk] Derived routing index 与 manifest事实漂移。
  → Mitigation：index 只在 registry construction 从当前 manifests生成，不持久化第二份 mapping；schema/runtime/release tests证明一致性。
- [Risk] 同一 format id 或 pathname hint 被多个 adapter 声明。
  → Mitigation：registry construction、doctor 与 release validation 阻断；runtime 仅保留 global conflict defense，绝不使用 registry order。
- [Risk] 与 state-reuse change 的 probe assumptions 冲突。
  → Mitigation：在任一 implementation 前接受 Decision 8 handoff；本 change 删除 candidate traversal obligation，但不替代 selected-state lifecycle 决策。

## Migration Plan

1. 读取正式调查主题 `docs/investigations/dependencies/format-routing-inference.md`；记录已批准的 manifest-native/zero-dependency pathname mechanism，并完成 probe removal inventory和 cross-change handoff records。
2. 通过 blocking artifact audit，证明 capability IDs、delta requirements、manifest suffix/exact-filename schema、diagnostic taxonomy、probe outcome 和 implementation scope 完全一致。
3. 按项目 TDD/Case 流程先建立 automatic/explicit/failure/no-fallback 与 adapter parse evidence，再同步长期 owner docs、schema/examples/fixtures。
4. 实现 manifest schema/descriptor `filenames[]`、validated filename/suffix/format lookup views、route 前 lexical basename lookup、命中后的 filesystem-backed path/access normalization和 exact dispatch；迁移 explicit path；在同一 change 中完整删除 probe method/type/validator/typed-field/schema/evidence surface。
5. 运行 targeted tests、两 target build/release smoke、zero-dependency/binary/startup comparison 和 `verify:docnav-workspace`，再做 final diff 与 public-contract audit。
6. 回滚必须整体恢复旧 routing、adapter interface 和 schema/tests；不得只恢复 registry traversal 而留下两套 routing metadata或死 schema。

## Open Questions

没有未回答的 architecture、dependency、compatibility 或 exact-outcome 问题。用户已经批准纯 pathname 的 route-before-I/O 顺序、exact filename、完整-basename ASCII-normalized longest-suffix、private routing state、Decisions 1–7、9 和完整 breaking scope。Adapter id 的既有 identity contract 与 adapter 内部 parser/dialect mapping 不由本 change 新增或重述。

Probe removal、cross-change handoff 与 final artifact audit 已闭合；没有发现需要新增产品选择或人类裁决的 core-goal incompatibility。后续发现的新 owner-backed consumer 仍必须按本 design 的 deletion/migration规则纳入同一 breaking change，不能恢复 compatibility surface。
