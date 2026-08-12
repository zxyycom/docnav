# Tasks

按“Current shared capability 与 public baseline 重审 → bounded prefix → protocol/input/adapter producer 原子迁移 → navigation/report/presentation 集成 → end-to-end evidence → stable owner 同步”推进；任何 public 中间状态都不得同时承诺 pagination 和 output constraint。

当前任务表停在第一处代码改动前：规划、架构选择、owner handoff、测试起点和启动审计均已完成，下一步直接从 `1.1` 开始实现 bounded-prefix contract，不再等待产品决策或额外的 Change 生命周期动作。Implementation 与 Verification 仍未勾选，因为对应代码和验证尚未执行。

## Readiness

- [x] 0.1 Proposal、design 和 tasks 已删除旧 post-result `OutputWindow`/field-traversal 中心，并统一采用 producer → `OutputSession` → typed result/report → navigation/protocol → presentation 的目标调用链。
- [x] 0.2 已恢复并分类三条直接决策：output-limit 与 OutputSession 是本 Change 实施的 active future directions；保留 current tokenizer 是必须遵守的 aligned baseline，不再存在 tokenizer replacement gate。
- [x] 0.3 已核对 Current shared implementation 与归档证据：`CostUnit`、requested-unit `TextMeter`、Limited/Unbounded Gate、InputCost/Projection、Collector、producer stop 和 OutputReport 已存在并通过 focused tests；真实 adapter integration 与 bounded prefix 尚未实现。
- [x] 0.4 Public input/output、`tokens:6000` preset、`0.2` hard cutover、fixed metadata、任意正数 limit、Unbounded bypass、无 continuation 和 scope exclusions 均有唯一 change-local 语义。
- [x] 0.5 Entry sequence、read/unstructured text 和 nested auto-read 已分别固定 input atom、measurement、Collector、completion 与 report-composition 义务；shared core 不再推断业务字段或遍历 formed result。
- [x] 0.6 已按测试 owner 和 Case-maintenance 规则运行完整起点检查：577 个 Current entities 全部由 161 个 Cases 映射；design 登记了保留、改写、删除和新增证据的语义目的。
- [x] 0.7 CLI/config/navigation/protocol/adapter/output/Markdown/JSON/schema/example/testing/release/stable owner 的 handoff 已列明；fast-read、streaming、serializer-size budgeting、tokenizer replacement 和 decision lifecycle 写入保持范围外，Open Questions 为无。
- [x] 0.8 用户已确认整体方案以及三项会改变实现方向的选择：Entry 按完整 canonical compact JSON 原子计量；text 返回 deterministic exact bounded prefix、但不承诺 public maximal-fill；base incomplete 仍可从 admitted current result 触发 nested auto-read，但最终 complete 保持 false。
- [x] 0.9 已完成 AI-ready 启动交接审计：后续 agent 可仅凭本 Change 恢复 Current/Target、术语与 owner、首个代码任务 `1.1`、依赖顺序、验证出口和范围边界；全部 Implementation 与 Verification 任务保持未勾选。

## Implementation

`1.1`–`1.9` 共同形成一个不可分发中间协议的 breaking migration。可以按依赖顺序局部编译和测试，但 release artifact 只能在全部实现与 Verification 门禁通过后更新为 current candidate。

- [ ] 1.1 扩展 `docnav-text-cost` bounded-prefix contract：对 lines/bytes/tokens 返回 deterministic UTF-8 prefix end、exact recounted cost 与 complete；保持唯一 `o200k_base` ordinary-text token semantics，覆盖 empty/Unicode/newline/merge-sensitive/adversarial whitespace，并用代表 workload 证明非退化输出与可接受资源行为。
- [ ] 1.2 在 shared protocol/navigation 类型与 semantic validation 中建立 `Limit`、`OutputConstraint::{Limited, Unbounded}`、protocol `0.2` request union、success `output` union 和 invocation-level report mapping；删除 operation arguments/result 中的 page/continuation 与 read/unstructured full-selection common cost，并在 adapter dispatch 前拒绝非法/旧 shape。
- [ ] 1.3 迁移 core parameter catalog、strict CLI parser/help、project/user config extraction/schema、source resolution、protocol request construction 和 invocation metadata：实现 `--limit <unit>:<positive-integer>` XOR `--ignore-limit`、machine union、`defaults.output_limit` union 与 built-in `tokens:6000`，并让旧 CLI/config/numeric limit 只产生稳定迁移诊断。
- [ ] 1.4 重构 object-safe adapter contract：从 `OutlineInput`、`ReadInput`、`FindInput` 和 standard bindings 移除 page/numeric limit；把 normalized `OutputConstraint` 作为独立 execution control；让 content operations 返回 concrete typed result + `OutputReport`，Info 保持原路径；提供共享 canonical compact-Entry InputCost 与 report/error plumbing。
- [ ] 1.5 把 Markdown outline/find/read 与 unstructured full-read 改为 operation-owned producer/Projection/Collector：Entry 原子接纳、text 使用 bounded prefix、Limited stop 后不访问 tail、Unbounded 不构造 InputCost；删除 Unicode page slicing、entry soft truncation、next-page 和分页测试 helpers。
- [ ] 1.6 把 JSON outline/find/read 与 unstructured full-read 改为同一 Session contract：保持 JSON selection/ref/source-order owner、复用 canonical Entry InputCost、让 lazy find/outline producer 在 stop 后不生成 tail、text 使用 bounded prefix；删除 paging module、lookahead continuation 与 selection-scoped common cost。
- [ ] 1.7 重构 navigation execution 与 auto-read composition：先校验 base typed result/report，再从 admitted current result 计算 unique-ref eligibility，以同 unit 的剩余 budget 或 Unbounded 执行 nested read，checked 聚合 cost/complete，并在最终 `ProtocolResponse` 前验证 phase/report invariants；zero remainder 时合法省略 nested。
- [ ] 1.8 迁移 `docnav-output`、protocol-json、readable-view、failure mapping 与 invocation logging：两种 presentation 只消费同一个 `0.2` response/common output facts；fixed metadata、Info、failure、framing 和 log content 保持预算外；measurement/Collector/validation/serialization/I/O failure 不伪装为 incomplete success。
- [ ] 1.9 删除剩余 pagination production surface，原子同步 protocol/config JSON Schema、contract examples、fixtures、CLI/package help、migration guidance、release-package materials、Rust/Bun/smoke tests 与 Semantic Cases；纯 paging Case 随实体删除，新 integration Case 只在直接证据存在后建立，不保留 `0.1` fixture 作为 current compatibility promise。

## Verification

`2.1`–`2.8` 证明 implementation 与 change-local Target 一致；只有这些证据通过后才执行 `2.9` stable owner 同步，随后用 `2.10` 重新验证最终 Current 状态。

- [ ] 2.1 对 `docnav-text-cost`、`docnav-output-session`、protocol 与 adapter-contracts 运行 Rust format、Clippy 和 focused tests；证明三 unit dispatch、prefix UTF-8/recount/complete、Entry atomic admission、checked budget state、failure non-mutation、Unbounded measurement bypass 与 Collector exactly-once。
- [ ] 2.2 运行 Markdown/JSON focused adapter tests，覆盖 structured Entry、read text、unstructured outline、empty incomplete、first-item rejection、exact exhaustion、producer early stop、full Unbounded output、opaque ref round-trip 和既有 format-owned selection semantics。
- [ ] 2.3 运行 core parameter、config、navigation、protocol decode/validation 与 nested composition tests；覆盖 CLI/project/user/built-in precedence、互斥/非法/legacy input、original-limit report、base-first remainder、zero-remainder omission、nested truncation、success-only nested failure 和 checked cost aggregation。
- [ ] 2.4 运行 output/rendering/invocation tests，证明 protocol-json 与 readable-view 的 mode/limit/cost/complete 一致，Info/failure/log/framing 不获得普通 sidecar，host/I/O failure 保持 failure，任何正数 limit 都不产生专用 too-small error。
- [ ] 2.5 运行 protocol/config schema、example、fixture、help、docs link/Markdown、Change Plan 和 release-material validators；确认所有 current artifact 使用 `0.2` closed unions，同步移除 page/continuation/pagination 与旧 numeric limit promise。
- [ ] 2.6 运行 `bun run smoke:docnav` 和 canonical `bun run smoke:docnav-package`，以真实进程覆盖 outline/read/find、unstructured full-read、nested auto-read 的 limited complete/incomplete 与 unbounded success，以及三种 input source 和 legacy rejection；关闭 rebaseline 的 `CORE-CONFIG-PATH-002` package failure。
- [ ] 2.7 按真实 owner 与可观察结果审阅变更测试和 Case，先运行最窄 runner，再执行 `bun run test-evidence -- check --root .`，证明完整 Current static/runtime entities、Cases、Topics 和 mappings 重新闭合。
- [ ] 2.8 运行 `bun run verify:docnav-workspace`；任一 schema、Rust、CLI、package、docs、quality 或 evidence failure 都返回对应 Implementation task，不把部分通过解释为 hard cutover 完成。
- [ ] 2.9 在 `2.1`–`2.8` 全部通过后，把 proposal 所列 stable owner 从 Current pagination 同步为 Current OutputSession-backed unit limits，并更新 `AGENTS.md` / navigation 的“有限、可继续”不变量；decision lifecycle/alignment 只报告，不在未获授权时写入。
- [ ] 2.10 对同步后的 owner、schemas/examples、代码、tests、Cases、release artifact 和本 Change 执行最终 AI semantic recovery、局部 diff、whitespace、docs/Change validators、完整 test-evidence 与 `bun run verify:docnav-workspace`，记录最终 evidence 和明确非目标后再请求归档授权。
