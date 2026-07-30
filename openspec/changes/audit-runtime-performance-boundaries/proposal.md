**本文是 `audit-runtime-performance-boundaries` 的临时提案：它建立 Docnav 广义运行性能的可复现审计与人工决策入口，不表示 Current baseline、已批准预算或优化方案。**

## Why

Docnav 的有限导航流程需要同时约束启动、计算、I/O、内存、输出、发布体积和极端输入伸缩，但当前没有一个长期 capability 负责把这些维度放进同一套可比较证据。已有局部观测不足以充当 baseline 或 gate；先建立独立审计边界，才能让后续性能判断基于可复现证据而不是单次数字或预选实现。

## What Changes

- 新增 `runtime-performance` capability，覆盖启动时间、端到端 wall time 与 CPU、I/O 与重复准备、峰值与 retained memory、输出体积与分页、package size，以及极端输入下的伸缩行为。
- 区分 representative workload 与 stress/adversarial workload；前者描述正常使用证据，后者寻找资源增长、边界失效和退化形状，二者不得互相冒充。
- 在任何测量前固定一个有限的 initial workload packet：按 format、operation、output/page 和 stress shape 分层抽样，不做笛卡尔积；未选组合显式留作 `unmeasured/future`，达到预先声明的 required cells 后停止首轮扩展。
- 要求每条可比较测量记录 binary、command、flags、fixture、output mode、page、limit、query/ref（适用时）、build、host、cache state 和 repeats，并区分 observation、可复现 baseline、已批准 budget 与 blocking gate。
- 建立 attribution categories，把证据归到 startup/process、input I/O、probe/routing、decode/parse/index、operation traversal/lookup/search、重复准备/composition、cost calculation、pagination/output construction、serialization/write、memory retention 或 package/dependency；不凭总耗时直接指定修复。
- 规定普通 observation 默认非阻断；数字 budget、退化阈值和 CI/merge gate 必须经过明确人工批准，并记录适用 workload、统计口径、环境与噪声处理。
- 要求发现修复需求时，把产品、协议、CLI、adapter、cost、find、state reuse、output、release 或依赖变更交还对应 owner change；本 change 只保留跨维度审计、归因和决策证据。
- 在后续获准 apply 时新增 `docs/runtime-performance.md` 作为稳定长期 owner，并从 `docs/navigation.md` 映射其读取时机：建立或解释 runtime 性能 baseline、budget、audit 或 optimization 时读取；现有 tooling 和静态 quality owner 不接管产品 runtime 性能。

### Non-goals

- 不选择或实现 parser、cache、document state reuse、token estimator、find model、pagination、renderer、buffering、allocator 或依赖优化。
- 不成为 `add-json-adapter`、`redesign-token-cost-estimation`、`reuse-adapter-document-state` 或 `redesign-find-result-model` 的前置；也不重复它们拥有的行为、机制或决策。
- 不把 `repository-quality-observability` 扩展成运行时性能 owner；未来可以由独立 integration 汇总两类报告，但静态代码质量 snapshot 与运行 workload 证据保持不同职责。
- 不设计 benchmark framework、dashboard、cache、通用 producer/sink、CI gate 或新的 public CLI/protocol surface。

## Capabilities

### New Capabilities

- `runtime-performance`: 定义 Docnav 运行性能 workload、测量记录、归因、报告、人工 budget/gate 决策与 owner handoff 的长期审计边界。

### Modified Capabilities

无。

## Impact

- 首阶段只影响本 change 的 OpenSpec artifacts；不修改 executable、adapter、shared library、protocol、schema、example、测试、release artifact 或既有 capability。
- 后续审计可能读取 core `docnav`、direct adapter、release package 和相关 owner change 的现有行为证据，但不得仅凭本提案改变这些表面。
- 只有 artifact audit、基线复现、归因报告和人工 workload/budget 决策依次完成后，才允许在对应 owner change 中创建或实施优化任务。
- 本 change 新增长期 capability 所需的 main spec、`docs/runtime-performance.md` 正文和 `docs/navigation.md` 读取/owner 映射，只能在后续获准实施和归档流程中同步并验证；当前 proposal 不修改 docs，也不证明能力已交付。
