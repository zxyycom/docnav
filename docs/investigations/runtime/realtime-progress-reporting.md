# Docnav 实时进度与心跳调查

## 调查信息
- 核心问题: Docnav 在保持单一最终 stdout 契约时，能否以可接受影响提供执行中进度或心跳？
- 状态: 已结束
- 最新报告时间: 2026-07-30T07:35:21+00:00

## 调查报告

### 保持最终 stdout 契约的进度方案比较
- 形成时间: 2026-07-30T07:35:21+00:00

#### 形成时背景

Docnav 的调用方在处理较大文档时可能在最终结果出现前经历一段无输出时间。用户担心：
如果当前架构没有可靠的执行中输出手段，为 progress 引入线程、异步运行时、新协议或
adapter 重构的代价可能远高于所得收益。因此，本轮先回答“现有边界下能做到什么、各层
保证到哪里、代价落在哪些 owner”，而不是预设必须新增 progress。

形成本报告时，仓库 `HEAD` 为
`45c53876f42c38e1294936fac4b40461ea3acb49`。当前已知基线是：

- document operation 的 stdout 是 completion-only；执行完成并形成一个
  `ProtocolResponse` 后，才进入最终 document output write。
- `protocol-json` stdout 是单一完整 JSON value；`readable-view` renderer 在第一次
  stdout write 前返回完整 UTF-8 `String`。
- 当前 document pipeline、navigation resolution、adapter library dispatch 和可选
  auto-read composition 都是同步调用。
- runtime invocation log 是显式启用的独立审计 sink，不是 document output，也没有
  execution-start、milestone 或 heartbeat event。
- 当前没有已批准的 progress public surface、事件 schema、线程模型、取消模型或实时
  可见性承诺。

本轮授权仅为调查和报告沉淀，不创建 OpenSpec change，不修改产品规范、代码、测试或
release artifact，也不形成实现决定。

#### 调查目的

本轮准备回答：

1. 在不改变最终 stdout shape 和单一 response 契约的前提下，哪些执行中信号可以低成本
   提供，哪些需要线程、machine side channel、adapter contract 或 output/protocol 重构。
2. 顶层 start、同步 milestone、周期 heartbeat、invocation-log tail、JSONL stderr 和
   stdout partial streaming 分别能提供何种可观察保证。
3. 各方案对 TTY、非 TTY、AI subprocess caller 和 linked caller 的兼容性如何。
4. flush、跨 stream ordering、writer failure、隐私、取消、依赖、线程和测试应怎样进入
   成本判断。
5. 哪个层级可以作为后续讨论的最小候选，以及哪些方案当前不值得推进。

预定边界是方案调查和 blast-radius 识别；不选择 CLI flag、默认值、事件字段、依赖或
线程实现，不以本报告代替 proposal、decision、owner 规范或验收任务。

#### 调查范围与依据

调查以仓库 `HEAD` `45c53876f42c38e1294936fac4b40461ea3acb49` 的相关 tracked
内容为主要事实基线，检查了以下 owner 和实现面：

- 架构、CLI、protocol、output、navigation input resolution、测试与 release 规范：
  `docs/architecture.md`、`docs/cli.md`、`docs/protocol.md`、`docs/output.md`、
  `docs/navigation-input-resolution.md`、`docs/testing.md`、
  `docs/testing/release.md`。
- core invocation 和 document pipeline：
  `crates/docnav/src/main.rs`、`crates/docnav/src/lib.rs`、
  `crates/docnav/src/pipeline.rs`、`crates/docnav/src/pipeline/document.rs`、
  `crates/docnav/src/runtime.rs`、`crates/docnav/src/output.rs`。
- navigation 阶段、adapter selection 和 auto-read：
  `crates/shared/navigation/src/execution.rs`、
  `crates/shared/navigation/src/routing.rs`、
  `crates/shared/navigation/src/auto_read.rs`。
- adapter 和 shared output contract：
  `crates/shared/adapter-contracts/src/lib.rs`、
  `crates/shared/adapter-contracts/src/definition.rs`、
  `crates/shared/output/src/lib.rs`、`crates/shared/output/src/writer.rs`、
  `crates/shared/json-io/src/lib.rs`。
- invocation logging 的设置、event、writer、schema、examples 和测试：
  `crates/docnav/src/invocation_log.rs`、`crates/docnav/src/invocation_log/`、
  `docs/schemas/invocation-log-event.schema.json`、
  `docs/examples/json/invocation-log-*.json`、
  `crates/docnav/src/runtime/tests/invocation_logging/`。
- stdout/stderr、writer failure、linked renderer 和真实进程 capture 证据：
  `crates/docnav/src/output/tests.rs`、`crates/shared/output/src/tests.rs`、
  `test/tools/smoke-harness/process.ts`、`test/smoke/core/` 和当前语义 Case。
- 工具链与发布范围：`rust-toolchain.toml`、`crates/docnav/Cargo.toml` 以及 Linux
  `x86_64-unknown-linux-gnu`、Windows `x86_64-pc-windows-msvc` release baseline。

代码结构和调用关系通过 CodeGraph 与局部 source read 交叉恢复；规范按
`docs/navigation.md` 指向的 owner 文档读取。Rust 线程和 I/O 约束参考了 2026-07-30
访问的官方标准库文档：

- [`std::io::IsTerminal`](https://doc.rust-lang.org/stable/std/io/trait.IsTerminal.html)
- [`std::io::Write::flush`](https://doc.rust-lang.org/stable/std/io/trait.Write.html#tymethod.flush)
- [`std::thread::spawn`](https://doc.rust-lang.org/stable/std/thread/fn.spawn.html)
- [`std::thread::scope`](https://doc.rust-lang.org/stable/std/thread/fn.scope.html)
- [`std::sync::mpsc`](https://doc.rust-lang.org/stable/std/sync/mpsc/)

本轮没有：

- 对代表性大文档运行 performance benchmark、stage timing 或静默时长测量。
- 实现 prototype、注入 blocking adapter、验证 heartbeat 调度误差或线程开销。
- 使用真实 TTY/PTY 验证 auto policy。
- 验证 Codex desktop、其它 AI UI 或外部 orchestrator 是否逐 chunk 展示 stderr；现有
  smoke harness 虽逐 chunk 收集 pipe 数据，但只在 child close 后向调用方返回 snapshot。
- 验证 invocation log 在多个进程并发追加时的 event 原子性和 tail 行为。
- 调查或使用工作区内与本主题无关的未跟踪 OpenSpec change。

因此，本报告能判断结构可行性、兼容性方向和相对 blast radius，不能声称某一方案已满足
实际延迟预算、真实 UI 可见性或 production heartbeat SLA。

#### 调查结果与边界

##### 核心答案

当前证据只支持较窄的结构判断：**A+B 的静态影响相对较小且可以枚举**。顶层 start
不需要引入并发或重构 runtime execution model，但仍需修改 core 的 progress policy 与
stderr writer 编排；在现有同步 navigation 调用链加入 observer，也可以不引入 thread、
async runtime 或 adapter trait 变化。这个比较只说明 A+B 的结构成本低于周期 heartbeat、
adapter 内进度和 stdout streaming，不证明其影响已经可接受；是否可接受仍需 proposal、
prototype、代表性 workload 和真实消费者逐 chunk 展示实验。

这不等于低成本获得“每隔固定时间一定可见”的 heartbeat。只要耗时集中在一个同步
adapter 调用内部，同步 milestone 就会在该调用期间静默。周期 heartbeat 需要独立执行
单元和 stop/channel 生命周期；adapter 内细粒度进度则需要改变 adapter contract 或其
内部实现。父进程或 UI 如果缓存 stderr 到进程退出，Docnav 即使及时 flush 也无法强迫
它实时展示。因此，“可写入进度”“最大静默时间近似受控”“机器可稳定解析”和“用户界面
实时展示”是四个不同保证，不能由一个低成本改动同时证明。

##### 已确认事实

1. **最终 stdout 是封闭契约。** `ProtocolJson` 只写一个完整 protocol response；
   `Rendered` 只在 renderer 完整成功后写完整文本。当前 output writer 不提供 result
   chunk、partial response 或流式 renderer contract。
2. **执行链是同步且已有可命名阶段。** navigation 依次进行 adapter intent、adapter
   selection、operation input resolution、request preparation、adapter dispatch、result
   validation 和 auto-read composition。这些边界可以同步调用 observer。
3. **一条 start 不需要并发或 runtime execution-model 重构。** core 在 parse 成功、
   进入 pipeline 前已经持有 stderr writer；候选仍需修改 progress policy 与 writer
   编排，才能在此处发出“document invocation started”。它只表示开始处理，不表示 input、
   config、selection 或 operation 已成功。
4. **同步 navigation milestone 不需要线程、async 或 adapter trait 改动。** 代价主要是
   observer/event 在 core 和 navigation 调用链中的传递，以及 CLI/output policy、测试和
   文档同步。若要报告 auto-read 是否实际尝试，observer 还需进入当前封装该判断的
   `auto_read::compose_response`。
5. **周期 heartbeat 需要并发和生命周期审计。** 如果 reporter thread 拥有 stderr，
   writer 需要满足相应 `Send` 和生命周期约束；如果主线程保留 stderr 而把 operation
   放到 worker thread，runtime/result 需要跨线程承诺。当前 `docnav::run` 只要求
   `W/E: Write`，`DocnavRuntime` 没有 `Send`/`Sync` bound。
6. **adapter 内细粒度 progress 是另一层 contract。** 当前 `Adapter` 方法同步返回完整
   typed result 或 error，没有 progress callback、iterator、channel 或 cancellation
   token。修改这层会影响所有 adapter、definition、navigation dispatch 和测试。
7. **invocation log 当前只有终态/捕获事件。** success event 在 stdout write 成功后记录；
   event append failure 被降级且不改变 operation result。它不能在当前形态下作为实时
   progress 或可靠 delivery channel。
8. **默认成功 stderr 当前为空。** Rust tests、CLI smoke 和语义 Case 观察这个边界。
   默认向所有 stderr 写 progress 会改变现有 caller 可见行为，即使 stdout 不变。
9. **flush 是实时可观察的必要条件但不是充分条件。** 对 generic `Write`，每个 progress
   line 都必须完整写入并显式 flush；这只能把 buffered bytes 推向 sink，不能建立
   stdout/stderr 跨流总序，也不能控制父调用方何时展示。
10. **当前没有 cooperative cancellation。** Progress 或 heartbeat 的内部 stop signal
    只负责结束 reporter，不会自动让正在执行的 adapter operation 可取消。

##### 基于事实的推断

- 对“不要让用户误以为完全没有启动”的需求，单一 start 是最低代码成本方案，但信息量
  很低。
- 对“知道执行在推进而不要求固定 cadence”的需求，start 加同步 milestones 是成本和
  价值最平衡的候选。
- 对“每 N 秒至少一个事件”的需求，必须接受线程/worker、调度、writer ownership 和
  shutdown 复杂度；即使如此也只能给 best-effort cadence，OS 调度、blocked writer 和
  caller buffering 仍可延迟可见性。
- 对 streaming AI caller，versioned JSONL stderr 比 human text 更合适，但它是新的
  machine contract，不应伪装成现有 diagnostic 或 invocation log event。
- invocation log 可以复用 correlation、bounded metadata、privacy summary 和 JSONL
  serialization 思路，但 audit sink 和 progress sink 的启用、失败和保留语义不同，不应
  因代码复用而合并 owner。
- stdout partial streaming 对 progress 目标明显过度：如果只在 stdout 前加提示，会破坏
  protocol；如果真正流式返回结果，则需要重构 response、renderer、adapter 和 failure
  semantics。

##### 候选推荐，不是实现决定

如果后续确认只需要 early/stage visibility，候选最小层级是 **A+B：start + 同步
milestone observer**，并满足以下前提：

- 最终 stdout 和两个 document output mode 完全不变。
- progress 是 core-owned 独立 surface，不进入 `RequestEnvelope`、`ProtocolResponse`、
  adapter input 或 readable renderer。
- 第一版默认关闭或只通过明确 policy 启用；是否允许 direct CLI 的 TTY `auto` 需要另行
  决定和验证。
- 每个 event 完整写入、换行并 flush；stderr writer failure 默认只禁用 progress，不改变
  document result 或退出码。
- 事件只使用 coarse product stages，不暴露内部函数名，不提供虚假百分比。
- 第一版不输出 document path、ref、query 或 content。

这是一个供 proposal 或实验继续评估的**候选推荐**，不是已批准方案、长期决策、实现
授权或测试义务。

##### 明确不推荐

- 不推荐默认向所有 non-TTY stderr 无条件输出 start/progress；它会破坏当前成功 stderr
  边界并给不消费 progress 的机器 caller 增加噪声。
- 不推荐把周期 heartbeat thread 纳入首个最小实现；目前没有 stage latency 数据证明
  它的成本必要。
- 不推荐把 invocation-log tail 作为主要用户 progress UX；当前事件时间、失败降级和
  外部 tail orchestration 都不支持该承诺。
- 不推荐为了 heartbeat 引入第三方 async/logging framework；当前候选用标准库即可，
  且项目规范要求外部日志框架先审计初始化、feature、sink isolation 和 stdout purity。
- 不推荐任何 progress 写入最终 stdout，也不推荐为 progress 建立 partial stdout
  protocol/output mode。

##### 当前边界

本轮已经形成足以比较方案和确定实验顺序的认识，故主题状态为“已结束”。“已结束”只表示
当前没有继续调查动作，不表示已经选择或实现 progress。若后续出现实测静默预算、具体
AI consumer contract、TTY policy 要求、cooperative cancellation 需求或 output/adapter
架构变化，应按复查触发条件追加新的完整报告。

#### 实时保证的分解

| 保证 | 低成本可达程度 | 主要限制 |
| --- | --- | --- |
| 命令已开始 | start 可达 | 不表示后续仍在推进 |
| 到达新的同步阶段 | milestone observer 可达 | 单个长 stage 内仍静默 |
| 固定时间间隔心跳 | 需独立 reporter，近似可达 | OS 调度、writer block、shutdown 和 caller buffering |
| 稳定机器解析 | 需独立 versioned JSONL contract | 新 public surface、schema 和兼容义务 |
| 最终结果纯净 | stderr/独立 sink 可保持 | 默认 stderr 行为仍是 observable contract |
| UI 实时展示 | Docnav 单方不可保证 | 父进程可能缓存或忽略 stderr |
| 操作可取消 | 当前不可达 | adapter 同步 contract 无 cancellation token |

#### 方案矩阵

影响等级使用：

- **低**：core 局部插桩，默认行为和公开 schema 不变。
- **中**：跨 core/navigation、CLI policy、owner docs 和测试，但不改变 adapter/protocol。
- **高**：新增 public machine contract、并发 ownership 或广泛 caller 迁移。
- **极高**：改变 protocol/output/adapter 的基本模型。

| 方案 | 能提供的信号 | 主要代价 | 影响等级 | 当前判断 |
| --- | --- | --- | --- | --- |
| A. 顶层 start | 命令已进入处理 | success stderr policy、flush、文案与 writer failure | 低至中 | 可作为 B 的第一个 event；单独价值有限 |
| B. 同步 milestone observer | 阶段边界进展 | observer/event 穿过 core/navigation，补齐 early failure 和 auto-read | 中 | 低成本候选的主体 |
| C. 周期 heartbeat reporter | 与 stage 无关的周期活性 | thread/channel、Send/lifetime、stop/join、竞态与跨平台测试 | 高 | 无测量证据前不推进 |
| D. invocation-log/tail | 独立文件中的审计事件 | 增加 start/milestone schema、tail orchestration、并发 append 与保留成本 | 中至高 | 只作审计增强，不作主要 UX |
| E. JSONL stderr / machine side channel | 可流式解析的 progress event | 新 CLI/API、schema/version、sequence、failure 和 privacy contract | 高 | 有具体机器消费者时复用 B 事件源 |
| F. stdout partial streaming | stdout 内的提前片段或流式结果 | 破坏单值 stdout，重构 response/renderer/adapter/failure | 极高 | 不推荐 |

##### A. 顶层 start

- **TTY / 非 TTY**：TTY 直接可见；非 TTY 默认开启会改变现有成功 stderr。当前 generic
  `run` 无 `IsTerminal` contract，TTY 判断更适合由 binary entry 取得并显式传入 policy。
- **AI / linked caller**：AI subprocess 只有逐 chunk 消费 stderr 才能获益；linked caller
  更适合显式 observer 或 options API，不能假设其注入 writer 代表 TTY。
- **flush / ordering**：需要 line-level `write_all` 和 `flush`。只能保证同一 stderr 内顺序，
  不能保证父进程合并 stdout/stderr 后的全序。
- **writer failure**：候选语义是 advisory；首次 write/flush failure 后静默，不递归向坏
  stderr 报错。
- **隐私 / 取消**：只写 operation 和 `started`；不包含输入值；不提供取消。
- **测试**：recording writer 证明 write-before-execution 和 flush；fail-on-write/flush；
  默认关闭时 stdout/stderr 与现有结果完全一致。
- **logging 复用**：不能直接复用当前 logger 作为顶层入口，因为 log path 需要 project
  context 后才解析，且 sink/失败语义不同。

##### B. 同步 milestone observer

- **可行插桩点**：CLI parsed、project/config resolved、adapter selected、request prepared、
  dispatch started/returned、base response validated、auto-read attempted/returned、result
  ready、output written。
- **架构边界**：navigation 发布结构化 coarse event，core 决定是否渲染到 stderr；adapter
  不接收 writer。现有无 observer 入口可保留为 no-op wrapper，以降低 caller 迁移。
- **局限**：adapter probe、document parse 或 read handler 内部如果独占主要时间，observer
  只能在调用前后发 event。
- **writer failure / ordering**：事件 sink 失败不改变 navigation；输出写入前只能称
  `result_ready`，不能提前称 `completed`。若 `completed` 表示 stdout 已交付，还需明确
  output write/flush 语义。
- **隐私 / 取消**：coarse event 不复制 request、path、ref、query、diagnostic details 或
  result；observer 不等于 cancellation callback。
- **测试**：fake observer 断言成功和各 early failure 的 event 序列；blocking fake adapter
  用 barrier 证明 dispatch 前 event 已可见；auto-read 分支验证不伪报 nested read。
- **依赖 / logging**：不需要线程或新依赖。可以共享 correlation、event vocabulary 和
  bounded metadata helper，但不能让 audit logger 反向拥有 stderr progress。

##### C. 周期 heartbeat reporter

可行 topology 有两类：

1. reporter thread 持有 stderr writer，main thread 同步执行 operation；这要求把 writer
   安全移入或借给 thread。普通 `thread::spawn` 要求 `Send + 'static`，scoped thread 可
   去掉 `'static`，但仍要求 `Send`。
2. main thread 保持 stderr 并用 timeout 等待，把 operation 放到 worker thread；这要求
   runtime、request、outcome 和相关 borrowed state 满足跨线程约束。

两类都需要：

- start/stop channel、normal return、early error、panic 和 sender disconnect 的退出路径。
- stop 后 join reporter，再进入最终 output，避免迟到 heartbeat 与 final stdout 竞态。
- thread create failure 的降级语义；不应因 progress 辅助线程创建失败而 panic。
- writer failure 后 reporter 自行退出；不能占用或污染最终 diagnostic path。
- 用 barrier、受控 ticker 或宽松 deadline 测试，避免依赖精确 sleep。
- Linux 和 Windows package/runtime evidence。

标准库 `thread` 和 `mpsc` 已足够构造候选，不存在必须选择外部依赖的证据。该 reporter
只能管理自己的停止，不能中断一个不合作的 adapter handler。

##### D. invocation-log/tail

现有可复用部分是：

- project-relative path 解析、correlation id、bounded query/ref summary、content hash、
  JSONL event 形状和独立 sink。

现有不满足 realtime progress 的部分是：

- success 的第一条 `operation_completed` 在 stdout write 成功后才记录。
- schema 只接受四种 terminal/content-capture event。
- append/serialization failure 被忽略，并明确不改变 document operation。
- caller 必须预先知道文件路径并运行外部 tail；仓库没有跨平台 follow command。
- 多 invocation 可以共用一个 sink；实时 consumer 需要按 correlation id 过滤。
- 当前 JSON serialization 和尾随换行是分开的写操作；若要求多进程可靠 tail，还需审计
  partial line 和跨进程交错。

增加 milestones 会改变 audit event volume、schema、examples、Case 和 retention；增加
周期 heartbeat 还会放大磁盘写入。因此它可以作为诊断增强候选，但不能把当前 best-effort
审计 sink 误述为可靠 progress channel。

##### E. JSONL stderr / machine side channel

如果具体 AI/orchestrator 需要机器事件，可在 B 的同一 observer 事件源上增加独立 renderer。
候选最小 event 可包含：

- side-channel schema/version
- correlation id
- 单调 sequence
- operation
- coarse stage
- elapsed time
- 到达对应阶段后才出现的可选 adapter id 或 request id

第一版不应包含 document path、ref、query、content、完整 request/response 或 diagnostic
details，也不应暴露内部函数名或没有可证明分母的百分比。

该方案必须显式 opt-in；每行完整序列化、换行、flush，stderr 内用 sequence 定序，不承诺
与 stdout 的跨 stream 总序。linked caller 若已经在同进程内，应优先直接消费 observer，
而不是解析自己提供的 stderr writer。

它需要新的 owner 文档、schema/example、CLI/API surface、default-off stdout purity
证明、实时 process proof、writer failure 语义和 compatibility policy。现有 invocation log
schema 不应直接复用为 machine progress schema。

##### F. stdout partial streaming

两种解释都不合适：

- 在最终 stdout 前写 progress prefix 会使 `protocol-json` 不再是单一 JSON value，并使
  `readable-view` 不再精确等于 renderer 返回文本。
- 真正流式输出 result 则要求 response 从 immutable completed value 变为增量模型。
  `protocol-json` consumer 在完成前不能解析普通 JSON，mid-stream error 会留下不可恢复的
  truncated value；readable block framing 又依赖已知 UTF-8 byte length。

若还要求 adapter 在 parse/navigation 中提前产生 result，需要修改所有相关 adapter 方法、
operation result、pagination/continuation、renderer、linked caller、error mapping、writer
failure 和 cancellation。该 blast radius 与“让调用方知道仍在工作”的目标不成比例。

#### 影响面与验证方法

| Surface | A+B 候选 | C/E/D/F 追加影响 |
| --- | --- | --- |
| Core entry/pipeline | `main.rs`、`lib.rs`、`pipeline.rs`、`pipeline/document.rs` | C 还涉及 worker/reporter ownership；E 涉及 public policy |
| CLI static surface | progress policy、help、parser 和 command model | E 需稳定 machine mode；TTY auto 需 disable/override |
| Navigation | `execution.rs` 与 `auto_read.rs` observer 插桩 | adapter-level progress 扩至 routing/adapter contracts |
| Output | 最终 stdout 保持现状；progress 独立写 stderr | F 重构 `OutputPlan`、renderer 和 JSON writer |
| Invocation log | 可只共享 helper/vocabulary | D 扩 schema、events、examples、retention 和 failure policy |
| Protocol/schema | A+B 不应改 protocol | E 新独立 schema；F 改 protocol/output contract |
| Linked caller | 保留现有 no-progress API 或增加 options/observer 入口 | C 可能增加 `Send` bounds；F 改 shared output API |
| Tests | observer、writer、default-off、stdout purity、early failure | C 加 timing/thread；E 加 JSONL/schema/live pipe；D 加 file tail；F 加 streaming/failure matrix |
| Release | Linux/Windows CLI smoke 保持旧默认 | TTY/thread/machine mode 需跨 target evidence |

核心 blast-radius 文件和测试包括但不限于：

- `crates/docnav/src/lib.rs`
- `crates/docnav/src/main.rs`
- `crates/docnav/src/pipeline/document.rs`
- `crates/docnav/src/runtime.rs`
- `crates/docnav/src/output.rs`
- `crates/docnav/src/cli/command_model.rs`
- `crates/docnav/src/cli/flags.rs`
- `crates/docnav/src/cli/parser/`
- `crates/shared/navigation/src/execution.rs`
- `crates/shared/navigation/src/auto_read.rs`
- `crates/shared/navigation/src/routing.rs`
- `crates/shared/adapter-contracts/src/lib.rs`
- `crates/shared/output/src/writer.rs`
- `crates/docnav/src/output/tests.rs`
- `crates/docnav/src/runtime/tests/invocation_logging/`
- `crates/shared/navigation/src/tests/navigation/`
- `crates/shared/output/src/tests.rs`
- `test/smoke/core/cases/outputs.ts`
- `test/tools/smoke-harness/process.ts`
- `docs/testing/cases/core-cli.md`
- `docs/testing/cases/output-rendering.md`
- `docs/schemas/invocation-log-event.schema.json`
- `docs/examples/json/invocation-log-*.json`

若未来进入变更阶段，验证应至少分层证明：

1. no-op/default-off 时现有 stdout、stderr、exit code、protocol schema 和 linked renderer
   完全不变。
2. recording writer 能在 operation 完成前观察 event，并记录 event 后立即 flush。
3. progress writer failure 不改变 document result；若产品选择 fatal 语义，则需要显式迁移
   contract，而不是隐式改变。
4. observer event 序列覆盖 early config/selection/request/dispatch failure，且不伪报未发生
   的阶段。
5. machine event 每行独立 schema-valid、sequence 单调、无敏感 payload。
6. heartbeat reporter 在 success、failure、panic、writer failure 和 stop 后都没有遗留
   thread 或迟到 event。
7. CLI smoke 证明 pipe/non-TTY 默认兼容；TTY auto 若存在，需要 PTY 或可注入 terminal
   policy 的独立证据。
8. Linux 和 Windows release package 保持 canonical final output，并只在显式支持的
   progress mode 下增加相应 side channel。

#### 未知与复查触发条件

当前未知：

1. 代表性 Markdown/JSON 文档在哪个 stage 产生最长静默，P50/P95/max 是多少。
2. 用户可以接受的最大静默时间，以及“start 即足够”还是必须阶段/周期反馈。
3. 真实 Codex desktop、AI agent runner、shell wrapper 和 linked caller 是否逐 chunk
   消费 stderr。
4. progress delivery failure 应是 advisory 还是影响退出码；后者如何与已经成功产生的
   final result 协调。
5. direct CLI 的默认 policy 应是 off 还是 TTY auto，以及 linked `docnav::run` 是否始终
   off。
6. machine event 是否需要 wall-clock timestamp、elapsed time、sequence、request id 和
   terminal event；这些字段各自的稳定性与隐私承诺尚未决定。
7. `completed` 表示 operation/result ready、stdout write 成功还是 stdout flush 成功。
8. 是否有独立 cooperative cancellation 需求；当前 adapter contract 不能在长调用内部
   响应取消。
9. invocation log 是否允许 pre-output start/milestone events，以及 schema version 和
   multi-process append 语义如何演进。

出现以下任一情况时，应重新调查并追加完整报告：

- 获得代表性 stage timing 或用户可接受的静默预算。
- 有具体 AI/orchestrator 请求稳定 machine progress schema。
- 真实 UI 测试证明 stderr 被缓存或可逐 chunk 展示。
- adapter contract 改为 async、stream、subprocess 或增加 cancellation/progress callback。
- document output 不再要求单一完整 `ProtocolResponse` 或完整 renderer text。
- invocation log schema、失败语义或 sink ownership 改变。
- release target、Rust toolchain/thread 支持范围改变。
- proposal 准备选择 default policy、writer failure、event vocabulary 或 heartbeat topology。

#### 关键证据定位

- 单一 final output 与 renderer-before-write：
  `docs/output.md:5-35`、`docs/output.md:129-135`、
  `crates/shared/output/src/writer.rs:8-22`。
- protocol stdout 单一 response：
  `docs/protocol.md:5-16`、`docs/protocol.md:46-80`。
- completion 后才写 stdout：
  `crates/docnav/src/lib.rs:46-65`、`crates/docnav/src/output.rs:89-129`。
- 同步 navigation stages：
  `crates/shared/navigation/src/execution.rs:18-46`、
  `crates/shared/navigation/src/execution.rs:65-159`。
- auto-read 内部第二次同步 dispatch：
  `crates/shared/navigation/src/auto_read.rs:22-62`。
- adapter 同步 trait：
  `crates/shared/adapter-contracts/src/lib.rs:27-65`、
  `crates/shared/adapter-contracts/src/definition.rs:47-92`。
- 当前 generic writer/runtime bounds：
  `crates/docnav/src/lib.rs:20-44`、`crates/docnav/src/runtime.rs:27-35`。
- invocation log terminal timing和 best-effort write：
  `crates/docnav/src/output.rs:103-125`、
  `crates/docnav/src/invocation_log.rs:67-88`、
  `crates/docnav/src/invocation_log.rs:132-170`、
  `crates/docnav/src/invocation_log.rs:263-313`、
  `crates/docnav/src/invocation_log/writer.rs:7-14`。
- invocation log schema 仅四种 event：
  `docs/schemas/invocation-log-event.schema.json:5-18`、
  `docs/schemas/invocation-log-event.schema.json:265-474`。
- success stderr 与 output failure 证据：
  `crates/docnav/src/output/tests.rs:43-55`、
  `crates/docnav/src/output/tests.rs:111-153`、
  `crates/shared/output/src/tests.rs:158-233`。
- invocation log success/output failure timing：
  `crates/docnav/src/runtime/tests/invocation_logging/output.rs:5-73`。
- invocation log writer failure 不改变结果：
  `crates/docnav/src/runtime/tests/invocation_logging/content.rs:201-215`。
- subprocess pipe 收集但 close 后返回：
  `test/tools/smoke-harness/process.ts:63-113`。
- release Linux/Windows baseline：
  `docs/testing/release.md:5-29`。
