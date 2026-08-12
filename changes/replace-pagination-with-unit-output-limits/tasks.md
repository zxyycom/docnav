# Tasks

按“Current 重新基线与依赖能力门 → public contract 和 production 原子迁移 → 行为与分发验证 → Current owner 同步”推进。`introduce-budgeted-output-window` 与 `adopt-low-constant-reference-tokenizer` 的实现能力是 public cutover 的硬门禁；相邻 Change 的目录或 stage 不证明能力存在，也不授权本 Change 修改其 artifacts。

## Readiness

- [x] 0.1 Proposal、design 和 tasks 共享一个目标：用 `lines | bytes | tokens` 的单单位内容上限及显式 `Unbounded` 一次性替换 `outline`、`read`、`find` 的 public pagination contract。
- [x] 0.2 Product、architecture、preset 与 runtime/host 分层已经固定；`tokens:6000` 是可调整默认值，不是架构不变量。
- [x] 0.3 CLI、config、navigation、protocol、adapter、output、schema/example、testing、release 与稳定 owner 的影响边界已经列明，Current 与 change-local Target 没有混写。
- [x] 0.4 Limited/unbounded input、common output sidecar、内容字段预算、complete、极小 limit、nested ordering、scope 和 `0.2` hard cutover 均有唯一语义，没有剩余产品开放问题。
- [x] 0.5 相邻 OutputWindow/token calculator 的 owner、关闭动作和被阻塞任务已经声明；fast-read probing 已分类为默认不阻塞的后续消费者。
- [x] 0.6 已按测试策略证明完整测试实体与 Semantic Case 映射闭合，并识别 pagination、navigation、protocol、adapter、output 与 text-cost 的受影响 Case。

## Implementation

`1.2` 是 public cutover 的硬门禁；它未完成时，`2.1` 及之后任务全部保持阻塞。推进相邻 Change 需要独立授权，本 Change 的实施授权不扩张到相邻 artifacts 或 production。

- [x] 1.1 已从稳定 owner、schema/examples、Rust types、CLI help、fixtures、Semantic Cases、开发 CLI 和 release package 核对 `0.1` pagination、numeric limit、cost、auto-read 与 raw/readable 基线；design 的 Implementation Observations 记录 Current 能力和 package baseline failure。
- [ ] 1.2 取得并验证 `introduce-budgeted-output-window` 与 `adopt-low-constant-reference-tokenizer` 各自 owner 要求的 production 证据：共享 `lines | bytes | tokens` calculator、只为 `Limited` 建立的 OutputWindow、内容字段 traversal/atomic item policy、budget report，以及 `Unbounded` 直接 bypass；field classification 必须与本 design 一致。
- [x] 1.3 已重读测试策略、行为 owner、Case maintenance 与 `test-evidence-review` skill，完整 test-evidence check 通过；design 逐项登记现有 Case 的保留、改写或删除，以及新 shared budget Case。
- [x] 1.4 Design 已作为唯一 change-local Target 登记 `AGENTS.md`、`docs/navigation.md`、architecture、CLI、navigation input resolution、protocol、adapter contract、output、Markdown/JSON adapter、schema/example、testing 和 release owner 的 Current handoff；稳定 owner 保持未修改。

`2.1`–`2.7` 共同完成同一 breaking release 的原子迁移。不得发布同时接受 page/pagination 与新 output constraint 的中间 public contract。

- [ ] 2.1 在 shared cost/protocol/navigation 类型与 semantic validation 中建立 closed `CostUnit`、`Limit`、`OutputConstraint::{Limited, Unbounded}`、protocol `0.2` request 和 affected-success `output` union；删除 page/continuation、read/unstructured full-selection common cost 与 `0.1` runtime shape，并证明非法组合在 adapter dispatch 前失败。
- [ ] 2.2 迁移 core parameter catalog、CLI parser/help、project/user config schema与加载、source precedence、request construction 和 invocation metadata：支持 `--limit <unit>:<positive-integer>` XOR `--ignore-limit`、machine object union、`defaults.output_limit` discriminated union 与 built-in `tokens:6000`；旧 `--page`、`--pagination`、numeric-only limit 和 `defaults.pagination.*` 只返回迁移诊断，不兼容接受或自动转换。
- [ ] 2.3 迁移 adapter contract 与 built-in Markdown/JSON adapters，使 adapter 形成完整 operation-owned typed selection 并移除 adapter-private page/continuation 与 calculator 解释；保持 opaque ref、format selection 和 navigation strategy 的既有 owner，不把 output constraint 变成 adapter selection 参数。
- [ ] 2.4 在 typed result 后、raw/readable rendering 前接入 Limited OutputWindow：只预算新返回的 text、sequence item 与 optional nested content，固定 envelope/root identity metadata 位于预算外；base content 先于 nested content，sequence item 原子接纳，text 可为空 prefix，nested payload 可整体省略，任何正数 limit 都返回合法 success 且不定义 `OUTPUT_LIMIT_TOO_SMALL`。
- [ ] 2.5 实现 `Unbounded` 直接 bypass，不创建隐藏 limit、emergency ceiling 或 unit-specific measurement；资源、allocation、render 和 write 故障继续走既有 failure mapping。确保 `info` success、failure envelope 与 invocation log 不获得普通 output sidecar。
- [ ] 2.6 让 protocol-json 与 readable-view 消费同一个 budgeted typed result 和 common output facts；Limited cost 只描述所选 unit 下已接纳的内容字段，Unbounded 不要求 cost，presentation framing 不形成第二个预算或 complete owner。
- [ ] 2.7 同步 protocol/config JSON Schema、contract examples、fixtures、CLI/package help、迁移诊断与 release materials，并按 `1.3` 登记的独立证明目的修改 Rust/Bun/smoke tests 和 Semantic Cases；所有 artifact 都使用 `0.2` 新 shape，不保留旧 pagination fixture 作为当前兼容承诺。

## Verification

`3.1`–`3.6` 先证明 design Target 已由实现和分发行为成立；全部通过后才能执行 `3.7` 的稳定 owner 同步。

- [ ] 3.1 运行 shared calculator、constraint/source validation、protocol semantic validation、OutputWindow traversal、adapter contract、Markdown/JSON operation 与 output renderer 的范围匹配 Rust format、Clippy 和 focused tests；覆盖三种 unit 的边界、原子 sequence item、empty text、nested omission、base-first 和 budget cost/complete invariant。
- [ ] 3.2 运行真实开发 CLI、`bun run smoke:docnav` 与 canonical release-package smoke，覆盖 `outline`、`read`、`find`、unstructured full-read 和 nested auto-read 的 limited complete/incomplete 及 unbounded success；对 CLI、config、machine 三种 source 验证 precedence、互斥、默认值和稳定错误。
- [ ] 3.3 证明 raw/readable 使用同一 output facts，`info`、failure、invocation log 与 host/I/O failure 保持范围外；证明旧 page/pagination/numeric limit/`0.1` shape 被拒绝，且任何正数 limit 不触发专用 too-small failure。
- [ ] 3.4 运行 protocol/config schema、example、fixture、docs link 与 Change Plan validators，确认 protocol `0.2`、三单位枚举、closed unions、固定 metadata 预算外语义和 release guidance 一致。
- [ ] 3.5 按修改后的真实测试目的审阅受影响 Semantic Cases，运行最窄目标 runner 后执行 `bun run test-evidence -- check --root .`，证明完整当前树的静态实体、runner 实体与 Case 映射重新闭合。
- [ ] 3.6 运行 `bun run verify:docnav-workspace`，并保存足以证明 schema、实现、真实 CLI、package、tests 与 change-local Target 一致的结果；任一失败都返回对应 Implementation task，不把部分通过解释为 hard cutover 已完成。
- [ ] 3.7 只有 `3.1`–`3.6` 全部通过，才把 `1.4` 登记且已由证据证明的 delta 同步到稳定 owner 为 Current，并更新 `AGENTS.md` / navigation 中“有限、可继续”的不变量；长期 decision alignment 只在用户明确授权维护决策时按 `decision-records` skill 写入。
- [ ] 3.8 对同步后的稳定 owner、schema/examples、实现、tests、release artifacts 和本 Change 做最终一致性、局部 diff 与 whitespace 审计；重新运行受影响 docs/Change validators 和 `bun run verify:docnav-workspace`，再记录 implementation observations 与剩余非目标。
