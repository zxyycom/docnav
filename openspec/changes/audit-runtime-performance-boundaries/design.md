**本文是 `audit-runtime-performance-boundaries` 的临时设计：它定义证据、归因、决策和 owner handoff 的最小流程，不选择优化机制，也不把任何现有数字提升为 Current baseline 或 budget。**

## Context

Docnav 的产品承诺是通过有限、可继续的 `outline -> ref -> read` 流程导航大型文档。一次命令的可用性不只由 wall time 决定：进程启动、CPU、文件读取、probe/routing、adapter decode/parse、重复准备、结果构造、序列化、输出字节、分页、峰值与跨调用 retained memory、发布包体积，以及输入增长时的伸缩都可能成为主导成本。

现有 owner 分散承担具体行为：

- core、navigation、adapter、protocol/output 和 release owner 分别拥有自己的可观察契约；
- `redesign-token-cost-estimation`、`reuse-adapter-document-state` 和 `redesign-find-result-model` 分别拥有 estimator、同调用状态复用和 find 模型选择；
- `add-json-adapter` 拥有 JSON-specific 导航行为；
- `repository-quality-observability` 只拥有静态代码质量 snapshot，不拥有 runtime workload 或性能 budget。

因此，本 change 新增 `runtime-performance` capability 作为审计 owner，只组织可比较证据、归因、人工决策和 handoff。它不成为上述 change 的依赖，也不接管它们的实现。

## Goals / Non-Goals

**Goals:**

- 用最小、可复现的记录覆盖启动时间、端到端 wall time/CPU、I/O/重复准备、peak/retained memory、输出体积/分页、package size 和极端输入伸缩。
- 明确 representative 与 stress/adversarial workload 的不同用途，并让报告保留这种分类。
- 把单次 observation、可复现 baseline、人工批准 budget 和 blocking gate 分开。
- 在选择修复前完成 attribution，并把修复交还拥有该行为的 owner change。
- 让后续人类能够基于完整 workload packet 决定是否需要预算、门禁或 owner-specific 优化。

**Non-Goals:**

- 不选择 benchmark framework、dashboard、长期存储、cache、通用 producer/sink、CI integration 或 merge gate。
- 不新增 public CLI/protocol/schema 字段，不改变 ref、pagination、ordering、output 或 error behavior。
- 不选择 parser、index、state reuse、token calculator、find model、renderer、buffer、allocator、dependency或 package 优化。
- 不把当前工作树、单次实验或未批准依赖产生的数据表述成 release baseline。

## Decisions

### Decision 1: 使用独立的 `runtime-performance` capability

`runtime-performance` 拥有 workload 分类、测量记录、evidence state、归因报告、人工 budget/gate 决策和 owner handoff。`repository-quality-observability` 继续只拥有静态代码质量 snapshot，现有 tooling owner 继续只拥有工具链和验证入口；未来 integration 可以并列展示这些结果，但不得让它们接管产品 runtime 性能。

替代方案是复用 `repository-quality-observability`。这会把静态源码指标与依赖真实 binary、fixture、host、cache 和 process boundary 的运行证据混为一体，因此不采用。

### Decision 2: representative 与 stress/adversarial workload 分开建账

Representative workload 用于描述已确认的正常使用形状，例如常见格式、文件规模、结构密度、operation、output mode 和分页请求。Stress/adversarial workload 用于主动寻找边界，例如超大节点数、超长 key/label/ref、深层或宽层结构、高匹配/零匹配 search、later page、完整 root read、重复调用和输入规模阶梯。

每个 workload 必须明确分类。Representative 结果不得证明极端情况安全；stress 结果也不得替代正常使用的基线。输入伸缩至少比较多个明确规模或结构阶梯，并报告 wall time、CPU、I/O、memory 与 output/package 中适用维度的增长，而不是只给最大输入的一点。

Initial workload packet 必须在任何测量前写入 `audit-report.md`，并固定以下 required cells；这些 cells 是分层抽样，不互相做笛卡尔积：

1. **Control/package**：一个成功的最小 core CLI startup cell，以及一个 Current release binary/package size cell。
2. **Primary representative format**：选择 Current evidence 最稳定的内置格式（当前预期为 Markdown），只测三个 core cells：一次 outline、一次 find、一次使用 outline ref 的 read。`protocol-json` 与 `readable-view` 分配到这三个 cells，确保两种 output 都出现但不为每个 operation 重复；first page 必测，并从实际产生 continuation 的一个 operation 增加恰好一个 later-page cell。
3. **Secondary representative format**：存在其它 Current 内置格式时只选择一个，并只增加一个 outline cell；JSON 只有在 Current release/source evidence 成立时才可被选中，否则该 cell 标记 unavailable，不能用未批准 worktree 代替。
4. **Stress/adversarial**：在一个 Current 格式上为同一个 outline cell 选择 small、representative、stress 三个明确规模；在最大层级上各增加一个 find-miss cell 和一个 root-read cell；另加一个超长 key/label/ref 在小 limit 下的 output/pagination cell。只有 Current process surface 支持同进程重复调用时才增加一个 retained-memory lifecycle cell，否则明确记为 not-applicable。

Packet 同时列出未选择的 format × operation × output × page × stress 组合并统一标记 `unmeasured/future`。首轮停止规则是：每个 required cell 都已得到完整 record，或以 unavailable/not-applicable 和证据闭合；三层伸缩已报告；其余组合已列入 future。归因不足可以保留 `unattributed`，不以继续扩张矩阵换取表面闭合；只有后续报告证明某个缺口会改变人工决定时，才另行批准扩展。

替代方案是维护一张不区分用途的 benchmark 列表。它无法说明数字代表常态还是边界探索，容易产生错误预算，因此不采用。

### Decision 3: 每条可比较测量使用自足 measurement record

Measurement record 至少包含：

1. **Invocation**：binary identity/path、subcommand 或完整 command、全部 flags、input path、output mode，以及适用的 page、limit、query 和 ref。
2. **Fixture**：format、生成或来源说明、字节数，以及与成本相关的节点/heading 数、深度、宽度、重复项、长 section、长 key/label/ref、匹配数量等结构事实。
3. **Build/process**：commit 或 source identity、debug/release/profile、package/binary identity、linked/direct/service/subprocess boundary，以及相关 dependency state。
4. **Host/runtime**：OS/architecture、CPU、可用 memory、storage/location、runtime/tool versions、相关 env、cache state（cold/warm/unknown）和并发背景；未知值显式记为 unknown。
5. **Sampling**：测量工具与定义、warmup、repeats、每轮原始值、聚合统计和噪声/异常值处理；单轮结果只标记 observation。
6. **Results**：startup 与 end-to-end wall time、CPU（可得时区分 user/system）、I/O count/bytes、完整准备次数、peak memory、适用进程边界内静置后的 retained memory、stdout/stderr bytes、page/continuation、package 的压缩/解压或 binary/dependency size 定义，以及成功/错误结果。

短生命周期 CLI 无法有意义地报告跨调用 retained memory 时必须记为 not-applicable；同进程重复调用或 service workload 才在明确静置点和生命周期下比较 retention。缺字段的历史数字可以保留为 seed observation，但不能与完整记录直接比较。

替代方案是只保存命令和最终时间/RSS。它不能排除 build、host、cache、output 或 process boundary 差异，因此不采用。

### Decision 4: 先按稳定 categories 归因，再决定 owner

报告把已测成本归到一个或多个有证据支持的 categories：

- `startup-process`
- `input-io`
- `probe-routing`
- `decode-parse-index`
- `operation-traversal-lookup-search`
- `repeated-preparation-composition`
- `cost-calculation`
- `pagination-output-construction`
- `serialization-write`
- `memory-retention`
- `package-dependency`
- `unattributed`

Attribution 必须说明比较、profile、instrumentation、计数或排除证据，并保留未归因部分；总耗时、RSS 或 output bytes 本身不证明某个内部机制有错。一个现象跨 categories 时可以拆分或标记多重贡献，不强迫单一归因。

修复任务按最终责任返回 owner：adapter parse/ref/search 回 adapter change，core routing/process 回 core/navigation/architecture owner，cost 回 token-cost owner，重复准备回 document-state reuse owner，find work budget/model 回 find owner，output/pagination 回 protocol/output/adapter owner，package/dependency 回 release/dependency owner。没有合适 active change 时另建 owner-specific change；本 change 只记录 handoff，不实现修复。

替代方案是在性能 change 中直接实现测得的最快候选。它会绕过产品契约和现有 change 的人工决策门，因此不采用。

### Decision 5: observation 默认非阻断，budget 和 gate 只由人工批准

Evidence state 依次区分：

1. `seed observation`：历史或不完整记录，只用于待复现问题清单；
2. `observation`：本次可审计记录，但未获准作为比较基线；
3. `reproducible baseline`：同一 workload packet 已重复执行并明确允许后续 before/after 比较；
4. `approved budget`：人类明确批准的 workload、指标、数值/范围、统计口径、host/build/cache、噪声规则与复核周期；
5. `approved gate`：人类另行批准 enforcement owner、适用入口、失败语义和解除/更新流程。

普通 observation 和 baseline 默认不阻断产品验证、发布或 merge。任何数字阈值、回归容忍度或 CI gate 都不得从 seed、工具默认值或代理建议自动推导。

替代方案是把首轮 baseline 直接转成阈值。这样会把偶然环境和当前缺陷固化成 policy，因此不采用。

### Decision 6: 采用 audit → measurement → decision → owner optimization 的有序流程

执行顺序为：

1. 先创建 change-local `audit-report.md`；其首句和固定章节明确它是 runtime performance artifact audit、workload、measurement、attribution、human decision 与 owner handoff 的实际记录，不是 README 或通用项目说明。
2. 在该报告记录 proposal、design、spec 和 tasks 的 scope、capability、状态词、独立性和未批准内容审计结论。
3. 按 Decision 2 固定有限 initial workload packet 与停止规则，再复现 seed observations 和建立 measurement records；报告创建或 artifact audit 不能在 task 2 中倒置补做。
4. 做 attribution，形成同时包含证据、未知项、伸缩与 owner handoff 的报告。
5. 由人类选择 accepted workloads，并决定是否批准 baseline、budget 或 gate。
6. 只有批准后，才在对应 owner change 创建或实施优化任务，并使用相同 workload 做 before/after 验证。

本 change 的 task completion 不授权在其它 change 创建任务；跨 change 写入仍需相应授权。已有相关 change 不依赖本 change 才能继续其已批准工作，本 change 也不依赖它们完成才能开始审计。

替代方案是先搭建通用 benchmark infrastructure 或直接优化可疑路径。当前没有已批准消费者或预算支撑该维护面，因此不采用。

### Decision 7: 现有数字只作为待复现 seed observations

下列用户提供数字保留为调查入口；它们缺少本设计要求的完整 command、flags、build、host、cache、repeats 和部分指标定义，因此不是 Current baseline、budget 或收益声明：

- 约 `20.78 MiB`、`1,500,000` 节点 JSON：outline 约 `0.742 s / 453224 KiB RSS`，find miss 约 `0.412 s / 263404 KiB RSS`，root read 约 `1.018 s / 343556 KiB RSS`。
- `1,000,000` 字符 key 在 `limit 10` 下仍产生约 `1000293` bytes 输出。
- 旧 tokenizer 路径曾观察到约 `310 MiB`；具体 memory 定义与完整 workload 仍待恢复。
- `5 MiB` / BPE 工作树数据受未批准依赖影响，不得表述为 Current 或 release baseline。

用户已接受正常数据下当前 JSON 表现作为非阻断 observation，由本独立 change 后续承接；该接受不覆盖 stress/adversarial 输入，也不批准任何数字预算或具体修复。

替代方案是删除不完整数字或直接当 baseline 使用。前者会丢失复现线索，后者会虚构可比性；保留并显式降级为 seed observation 更准确。

### Decision 8: `docs/runtime-performance.md` 是未来稳定 docs owner

本 change 获准 apply 时新增 `docs/runtime-performance.md`，由它长期拥有 runtime performance workload 分类、measurement record、evidence state、attribution、baseline/budget/gate 语义和 owner handoff。读者在建立或解释 runtime 性能 baseline、budget、audit 或 optimization 时必须读取该文档；`docs/navigation.md` 的“如何阅读这些文档”和“规则所有权”负责映射这个读取时机与 owner。

`docs/tooling.md` 继续拥有工具版本、运行方式和验证入口，`repository-quality-observability` 及其 owner 文档继续拥有静态代码质量 snapshot；它们可以被 runtime audit 引用，但不拥有产品 runtime 性能语义。当前 change 只规划该 owner，任务 5.2 获准执行前不得创建或修改上述 docs。

替代方案是把稳定性能语义附加到 tooling 或 quality owner。两者都缺少 workload、运行环境、资源边界和人工 budget 决策的完整消费上下文，因此不采用。

## Risks / Trade-offs

- **[跨主机数字不可比]** → measurement record 固定 host/build/cache/process/output 条件；条件不同只并列报告，不计算收益。
- **[维度过多导致审计膨胀]** → 每个 workload 只记录适用维度，但 coverage report 必须显式列出未测、not-applicable 和未知项。
- **[stress 结果被误读为常态]** → workload classification 与报告标题同时保留 `representative` 或 `stress/adversarial`。
- **[审计变成优化总包]** → attribution 之后只记录 owner handoff；没有人工决定时不创建机制或实现任务。
- **[measurement 改变被测行为]** → 报告记录 instrumentation、redirect/output handling 与 process boundary，并把有 instrumentation 和无 instrumentation 结果分开。
- **[package size 与 runtime 指标混淆]** → package/binary/dependency size 使用独立定义和 artifact identity，不用它推断运行时 memory。

## Migration Plan

1. 当前 change 只完成临时 artifacts；task 1 先创建 `audit-report.md` 并记录 artifact audit，不迁移 main spec、owner docs 或实现。
2. 审计获准后，按 tasks 先固定有限 initial workload packet，再生成可复现 measurement records、attribution report 和人类 decision packet。
3. 人类批准 capability 内容后，在获准的 apply/archive 流程中创建并同步 `docs/runtime-performance.md`、更新 `docs/navigation.md` 的读取/owner 映射，并同步 `runtime-performance` main spec。
4. 任何优化都通过对应 owner change 独立实施和回滚；本 change 不提供运行时迁移或回滚机制。

## Open Questions

无未回答的 artifact 起草问题，可以进入 task 1 的实现前审计。Decision 2 已固定首轮 packet 的有限 selection rule；哪些测得的 workload/tier 成为 accepted baseline，以及数字 budget、gate 和优化 owner 是否获批，仍在证据形成后由 tasks 中的人类门禁决定，不因 artifacts 完成而自动收敛。
