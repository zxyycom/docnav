# Docnav Tokenizer Backend 替代品调查

## 调查信息

- 核心问题: 在产品始终只使用一个 production token calculator 的前提下，是否存在语义、分发、平台和性能证据都足以替换当前 `tiktoken-rs` backend 的可靠实现？
- 状态: 已结束
- 最新报告时间: 2026-08-12T08:01:20+00:00

## 调查报告

### 形成时结论：没有可无条件替换的候选；保留 baseline 并解除公共能力耦合

- 形成时间: 2026-08-12T08:01:20+00:00

#### 形成时背景

形成本报告时，仓库 `HEAD` 为 `8fe968ce100da068974cf4a8b0f5d499d768fb9b`。

Docnav 当前通过 [`crates/shared/text-cost`](../../../crates/shared/text-cost/src/lib.rs) 集中计算文本成本。
当前依赖为 `tiktoken-rs 0.12.0`，使用 `o200k_base_singleton()` 与 `count_ordinary()`。
本文把这套实现称为 baseline。

当前 Markdown [`cost_for`](../../../crates/adapters/markdown/src/markdown/text.rs) 和 JSON
[`selection_cost`](../../../crates/adapters/json/src/content.rs) 都会形成 `lines`、`bytes`、`tokens` 三项成本；
两个 adapter 的 full-read measurement hook 也先形成全部单位，再按 `requested_units` 过滤结果。
因此，现有路径的部分成本来自框架同时执行多个 unit，
不能全部归因于 token calculator 本身。

目标框架应在一次请求中只运行实际选择的一个 unit。若选择 token，则只运行 token 计算；
若选择 lines 或 bytes，则不应触发 tokenizer。

产品约束也已澄清：

- production 中永远只有一个 token calculator；
- 不提供 public token profiles；
- 不支持多个 backend 的 runtime 选择；
- 候选比较和迁移机制只服务开发期准入，不构成公共协议；
- 低常数 tokenizer 可以是优化，但不是产品身份。

因此，本调查不以“必须替换”为前提，而是判断替代品是否足够可靠，
以及不替换时新框架是否仍具有可接受的成本边界。

#### 调查目的

本轮调查回答四个问题：

1. Rust 生态中哪些实现能够承担 `o200k_base` 的 production full-count？
2. 候选的热度、维护活跃度和反向依赖能否支持可靠性判断？
3. 候选在冷启动、常驻内存、二进制体积和热路径性能上是否优于 baseline？
4. 是否存在能直接支持 UTF-8 安全 bounded-prefix 的实现？

调查还要给出明确的产品与架构结论：

- 现在是否切换；
- 若不切换，输出限制是否仍可先落地；
- 哪些门槛应由 tokenizer change 独立承担；
- 什么新证据会触发复查。

本文不修改 production 依赖、实现、协议或 Change Plan；它只记录形成时的事实、测量、推断与建议。

#### 调查范围与依据

生态快照形成于 2026-08-12。下载量、recent downloads 与 reverse dependencies 来自 crates.io crate API；
仓库 stars 和维护时间来自对应项目仓库。
这些数字是时点快照，不是永久属性。

主要一手入口包括：

- OpenAI 官方 [`tiktoken` repository](https://github.com/openai/tiktoken)；
- baseline [`tiktoken-rs` crate API](https://crates.io/api/v1/crates/tiktoken-rs)；
- baseline [`tiktoken-rs` repository](https://github.com/zurawiki/tiktoken-rs)；
- 候选 [`tiktoken` crate API](https://crates.io/api/v1/crates/tiktoken)；
- 候选 [`rust-tiktoken` repository](https://github.com/goliajp/rust-tiktoken)；
- 候选 3.8.3 对应源码的 [`CHANGELOG.md`](https://github.com/goliajp/rust-tiktoken/blob/982a2238ce71a72e42339107ebbfe116d38e84b9/tiktoken/CHANGELOG.md)；
- 候选 3.8.3 对应源码的 [`LICENSE-3RD-PARTY`](https://github.com/goliajp/rust-tiktoken/blob/982a2238ce71a72e42339107ebbfe116d38e84b9/tiktoken/LICENSE-3RD-PARTY)；
- [`bpe-openai` documentation](https://docs.rs/bpe-openai/0.3.0/bpe_openai/)；
- [`bpe-openai` crate API](https://crates.io/api/v1/crates/bpe-openai)；
- `bpe-openai 0.3.0` 对应的 [`github/rust-gems` source](https://github.com/github/rust-gems/tree/bcb4204d82e15e83ff446d00301e239b0e09764f/crates/bpe-openai)；
- Hugging Face [`tokenizers` repository](https://github.com/huggingface/tokenizers)；
- Hugging Face [`tokenizers` crate API](https://crates.io/api/v1/crates/tokenizers)；
- [`riptoken` crate API](https://crates.io/api/v1/crates/riptoken)；
- [`riptoken` repository](https://github.com/daechoi/riptoken)；
- [`wordchipper` crate API](https://crates.io/api/v1/crates/wordchipper)；
- [`wordchipper` repository](https://github.com/zspacelabs/wordchipper)；
- [`kitoken` crate API](https://crates.io/api/v1/crates/kitoken)；
- [`kitoken` repository](https://github.com/Systemcluster/kitoken)。

本地 benchmark 环境为：

- Linux x86_64，WSL2；
- AMD Ryzen AI 7 H 450；
- 3 cores / 6 threads；
- 7.8 GiB memory；
- Rust 1.96.0；
- release profile；
- fat LTO；
- codegen units 1；
- panic abort；
- stripped binary。

baseline、`tiktoken 3.8.3` 和 `bpe-openai 0.3.0` 使用同一 proxy harness 构建和测量。
冷启动在一次不计入结果的 page-cache warmup 后，每项运行 20 个新进程；记录的 wall time
包含进程启动、fixture 读取、tokenizer 初始化和一次 count。
热路径在每个进程中初始化一次、预热三次，再执行固定轮次 count；每项运行 7 个新进程，
下文报告进程内 count 循环的 median。

一致性样本覆盖：

- 10 类短文本；
- 4 KiB 混合文本；
- 256 KiB 文本；
- 1 MiB 重复字符 `a`；
- 4 MiB 混合文本；
- 696,713 bytes 的文档集合。

比较记录 token count 与 token-id 序列的 FNV hash。该方法可以发现数量或整体序列 hash 不同，
但不是逐 token 元素断言，也不是 collision-free proof。

bounded 测量只覆盖 `bpe-openai` 的当前接口与选定样本。
没有测量 tail scaling，也没有接入真实 Docnav pipeline。

本轮明确未覆盖：

- Windows build、运行和 RSS；
- canonical `docnav` package 的最终二进制体积；
- 真实 `docnav` CLI 端到端延迟；
- 完整 Cargo advisory dependency graph；
- 法律审阅或许可证兼容性结论；
- 固定 CPU 频率、隔离核心和硬件 counter；
- 可持久复核的 raw benchmark artifact；
- `/tmp` 中间数据的版本化保存。

#### 调查结果与边界

结论是：截至 2026-08-12，当前没有可无条件投入 production 的可靠替代品，
所以现在应保留 baseline。

`tiktoken 3.8.3` 是唯一值得优先推进准入的 full-count 候选。它在本地样本中保持 count 与序列 hash 一致，
且在热路径、RSS 和 proxy 二进制体积上优于 baseline；其 4 KiB cold wall 中位数只改善约 8.6%，
两者观测范围重叠，不能据此声称稳定的大幅冷启动提升。
但其资产分发、近期 parity 修复、Windows 证据和 MSRV 声明仍有 blocker。

`bpe-openai 0.3.0` 展示了很强的 bounded counting 潜力，但不是可以直接替换 Docnav full backend 的完整答案。
其 prefix 返回语义、worst-case buffer、二进制体积、RSS、unsafe、MSRV
与 Windows 证据都需要额外处理。

因此，建议将两件事解耦：

1. 公共 output-limit 可以先评估由 output-window owner 包装现有 backend，形成 exact、full-scan 的 bounded fallback；
2. low-constant 与 early-stop 作为独立性能优化门，不再作为公共协议落地的硬前置。

第一点只是架构可行性推断：wrapper 仍需固定保守 prefix policy，并证明 UTF-8 boundary、
reported count 与重新计数一致。当前 backend 没有现成 bounded-prefix API，仓库也尚未实现或端到端验证该 wrapper。
它不改变“production 只有一个 token calculator”的约束。

在本报告形成时，这项建议与当时 active、尚未对齐的
[`use-low-constant-reference-tokenizer-for-output-cost`](../../decisions/product-direction/use-low-constant-reference-tokenizer-for-output-cost.md)
长期决定，以及
[`replace-pagination-with-unit-output-limits`](../../../changes/replace-pagination-with-unit-output-limits/design.md)
中把低常数 tokenizer capability 作为 public cutover 硬门的计划表述存在方向差异。
本报告只保存该形成时差异，不改写决定或 Change Plan。当前方向由
[保留当前 reference tokenizer，直到可靠替代已具备](../../decisions/product-direction/retain-current-reference-tokenizer-until-qualified-replacement.md)
和活动 Change 拥有，不能把本报告的建议直接当作实施依据。

在正常大小、热运行的 full-count 场景中，
baseline 不太可能显著拖慢只执行一个 unit 的新框架。
但其约 100 ms 冷启动、约 52 MiB cold RSS，
以及 1 MiB spaces 输入触发 stack overflow 的事实，
仍应作为独立风险处理，不能被“无需立即替换”掩盖。

#### 生态与热度快照

以下数字只用于判断采用面与维护风险，
不直接证明语义正确、许可证可分发或性能适合 Docnav。

| 项目 | 版本/快照 | total downloads | recent downloads | reverse dependencies | GitHub stars |
| --- | --- | ---: | ---: | ---: | ---: |
| `tiktoken-rs` | 0.12.0 | 13,735,836 | 6,759,136 | 403 | 405 |
| `tiktoken` | 3.8.3 | 32,003 | 23,708 | 9 | 7 |
| `bpe-openai` | 0.3.0 | 142,252 | 53,779 | 5 | monorepo 126 |
| `riptoken` | 0.3.0 | 192,072 | 191,999 | 0 | 6 |
| `wordchipper` | 0.9.2 | 1,711 | 1,061 | 5 | 33 |
| HF `tokenizers` | 0.23.1 | 26,411,078 | 10,402,198 | 777 | 10,958 |
| `kitoken` | 0.11.0 | 44,501 | 6,090 | 3 | 58 |

`tiktoken-rs` 的采用面远高于 shortlist 候选，这是迁移风险判断中的真实优势。

`tiktoken` 的 repository 创建于 2026-04，仍属于非常年轻的实现。
它的近期下载增长和性能结果值得关注，
但不能替代稳定观察期。

Hugging Face `tokenizers` 具有最成熟的通用生态，
但成熟的是通用 tokenizer framework，
不是 Docnav 所需的内置 `o200k_base` 兼容契约。

热度不能替代可靠性。
高下载量可能来自不同使用场景；
低反向依赖也不代表实现不正确。
准入仍要以语义、资产、平台、依赖面和 Docnav 实测共同判断。

#### 候选筛选

##### `tiktoken 3.8.3`

这是 full-count 技术上最强的候选。
其纯 Rust 实现、内嵌词表和 count-only 路径与 Docnav 的需求接近。
本地结果显示它比 baseline 更快、更小，并降低冷启动 RSS。

它目前仍不能无条件准入，原因包括：

- 3.6 与 3.8 系列近期连续修复 token-id parity；
- package 捆绑 17 套 vocab；
- `LICENSE-3RD-PARTY` 包含 Llama 3 Community License；
- `LICENSE-3RD-PARTY` 包含 DeepSeek Model License；
- 当前默认的 permissive-distribution gate 不能视为已经通过；
- 上述判断是工程分发风险，不是法律结论；
- upstream CI 缺少 Windows 证明；
- Cargo metadata 没有声明 `rust-version`；
- README 声称 MSRV 为 1.94，但尚需自动化验证；
- 没有满足 Docnav 需求的 bounded-prefix API。

建议的准入条件是：

- upstream 提供仅包含 OpenAI permissive assets 的 feature，或项目确认等价处置；
- 完成一个明确的稳定观察期；
- 在 canonical `x86_64-unknown-linux-gnu` 与 `x86_64-pc-windows-msvc` target 上完成 build/link gate；
- 固化并自动验证 MSRV；
- 使用逐 token assertion 与 fuzz 扩展 parity 证明；
- 在真实 Docnav workload 上复测 cold、hot、RSS 和 package size；
- 单独决定 bounded-prefix 是 upstream 能力还是本地算法层责任。

##### `bpe-openai 0.3.0`

该实现适合继续研究为 bounded building block，
但当前不是完整 backend。

其 `count_till_limit` 只返回 fit/none，
不返回 UTF-8 安全的最大 prefix 及其结束位置。
底层存在 `+10` buffer 与 worst-case TODO，
需要在生产采用前给出不变量或修复。

本地 proxy binary 为 34.30 MB，
cold RSS 为 68.23 MiB，
两项都明显高于 baseline 与 `tiktoken` 候选。

此外仍需审计：

- unsafe 边界；
- 明确 MSRV；
- Windows 构建与运行；
- prefix position 语义；
- 超限输入的 tail scaling；
- 与 full-count backend 的职责关系。

##### 其他候选

`riptoken` 的 Rust API 需要外部 ranks 与 pattern，
且没有直接 count-only 契约。
它当前没有 reverse dependency，仓库采用面也很小。

`wordchipper` 带来下载、cache、parallel execution 与较大的依赖面，
这与 CLI 冷启动和可预测分发目标不匹配。

Hugging Face `tokenizers` 需要自行构造配置，
并扩大 native 与通用 framework surface；
它没有直接拥有 Docnav 所需的内置 `o200k_base` contract。

`kitoken` 依赖外部 asset，
且需要 pattern inference；
这会把分发和语义恢复责任转移到 Docnav。

这些淘汰结论适用于本轮“直接 production backend”筛选，不表示这些项目在其他场景没有价值。

#### 本地 full-count 结果

冷启动样本为 4 KiB，20 个独立进程：

| 指标 | baseline | `tiktoken 3.8.3` | `bpe-openai 0.3.0` |
| --- | ---: | ---: | ---: |
| wall median | 103.423 ms | 94.573 ms | 39.793 ms |
| wall range | 96.453–113.209 ms | 86.256–117.261 ms | 38.858–42.432 ms |
| max RSS median | 51.725 MiB | 32.127 MiB | 68.229 MiB |
| proxy binary | 5,717,080 B | 3,301,696 B | 34,295,968 B |

热路径每次 count 的 median：

| 输入 | baseline | `tiktoken 3.8.3` | `bpe-openai 0.3.0` |
| --- | ---: | ---: | ---: |
| 4 KiB | 226.45 µs | 194.87 µs | 33.30 µs |
| 696,713 B docs | 26.516 ms | 18.143 ms | 16.689 ms |
| 4 MiB mixed | 250.466 ms | 200.696 ms | 27.630 ms |
| 1 MiB `a` | 246.500 ms | 149.708 ms | 5.819 ms |

在已列样本上，token count 与 FNV hash 没有 mismatch。这支持候选兼容性的初步判断，
但不等于完整 parity proof。

baseline 在 1 MiB spaces 输入上 3/3 次出现 `SIGABRT StackOverflow`。
两个候选均返回 8192 tokens，且 hash 相同。
这是需要单独修复或规避的 baseline robustness 问题。

热 Cargo registry/source cache 下还做了一次 clean proxy target build 观察：baseline、`tiktoken`、
`bpe-openai` 的 wall 分别为 20.192 s、13.827 s、42.411 s，peak build RSS 分别约为
421 MiB、320 MiB、1,172 MiB。构建顺序没有随机化且每项只有一次，
因此这些数值只说明 `bpe-openai` 的构建面可能较重，不能作为 canonical package build budget。

这些数值不应跨硬件直接外推。
它们也不能证明 canonical package 中的最终差值，
因为 proxy 没有包含完整 Docnav 依赖和链接布局。

#### bounded counting 结果

`bpe-openai` 对 4 KiB 样本给出 full count 666。
limit 665 返回 `None`，
limit 666 和 667 均返回 `Some(666)`。

4 MiB mixed 输入，100 probes：

| limit | 每次 median |
| --- | ---: |
| 100 | 4.92 µs |
| 6,000 | 293.7 µs |
| fit | 28.20 ms |

1 MiB `a` 输入：

| limit | 每次 median |
| --- | ---: |
| 100 | 1.763 ms |
| 6,000 | 1.947 ms |
| fit | 5.936 ms |

这些结果说明 early-stop 对部分超限文档有显著潜力。但 1 MiB 重复字符的低 limit 仍需要约 1.8 ms，
表明成本也受预切分和输入形态影响。

本轮没有测 tail scaling，
也没有得到 UTF-8 安全 prefix end。
因此这些数字不能直接证明 Docnav bounded output 已经可实现。

#### 形成时产品与架构建议

本报告形成时建议保留一个 production token calculator，且该 calculator 继续是 baseline。
不要增加用户可见 profile、backend selector 或协商字段。

本报告形成时建议 output-limit owner 评估以下分层；这不是当前规范或实现状态：

- public contract 只定义 token unit 的精确语义；
- wrapper 可以先用当前 backend full-count，并按 owner 固定的 prefix policy 实现 exact bounded fallback；
- early-stop 只在保持相同 prefix policy 和结果不变量时优化内部工作量；
- 替换 backend 必须保持同一 externally observed token contract；
- 候选双跑只允许出现在开发验证，不进入 runtime 产品模型。

据此，low-constant tokenizer 不应继续作为 public output-limit 的硬门。
它应成为独立性能 change，
用真实 workload 和资源预算决定是否采用。

若 `tiktoken` 满足资产、稳定期、Windows、MSRV、parity 与真实 workload 门槛，
则应直接替换 baseline，
而不是与 baseline 长期并存。

若这些条件不能满足，
新框架仍可通过按需只计算一个 unit 避免无关 tokenizer 成本。
这也是“暂不替换”可接受的主要架构依据。

#### 后续复核条件

出现以下任一变化时，应重新开启本主题并追加新报告：

- `tiktoken` 提供只打包所需 OpenAI assets 的 feature；
- dependency 或 legal owner 明确接受当前资产分发方式；
- upstream 增加 Windows CI 与可验证 MSRV；
- parity 修复进入稳定期且未再出现同类回归；
- 项目取得逐 token、fuzz 与真实 Docnav E2E 证据；
- baseline cold start、RSS 或 stack overflow 成为 release blocker；
- output-limit 的 full-scan fallback 实测超出预算；
- `bpe-openai` 提供 UTF-8 prefix end 与明确 worst-case contract；
- 出现新的、内置 `o200k_base` 且 permissive-distribution 明确的候选。

在本报告形成时且这些证据尚未出现的条件下，“保留 baseline”只是工程选择，
不代表 baseline 已满足所有性能和鲁棒性目标。
