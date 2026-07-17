# Typed Fields 维护成本评估

## 文档角色

- 用途：为保留、收缩或回退 field-derived document option 路径提供架构决策证据。
- 读者：准备维护 typed-fields、CLI/config resolution、navigation 或 adapter option contract 的工程师。
- 证据 owner：本 OpenSpec change；当前产品契约仍由 `docs/` 中的 owner 文档和已实现代码、测试拥有。
- 状态：实现后评估。本文推荐 B′，但不记录已确认 Decision，也不授权应用代码迁移。
- 调查日期：2026-07-17。

阅读架构结论时先看“决策摘要”“关键发现”和“推荐边界”；复核实验或来源时再看附录。

## 决策摘要

当前证据支持以下处理方向：

1. 暂停扩大完整 type-field：不把 root flags、schema、outline compound config 或 invocation logging 纳入同一通用字段系统。
2. 保留已经证明有价值的共享语义：adapter-owned option facts、typed validation、selected applicability、source resolution 和 provenance。
3. 将候选目标定义为 B′“薄共享契约 + 受控分散”：继续从 owner spec 投影 CLI help 和 config facts，同时按真实消费者收缩通用 derive、env、复杂 merge 和 companion 能力。
4. 不直接 revert `95281ae`。若确认采用 B′，新建 OpenSpec change，明确保留边界、迁移顺序和回滚验证。

核心依据如下：

| 判断面 | 已确认事实 | 对决策的影响 |
| --- | --- | --- |
| 边际维护 | A/B 新增同一整数 option 都净增 80 行生产代码；A 少一个 core 文件和一个 help 定义点。 | 集中式有边界收益，但本实验没有证明 LOC 收益。 |
| 前置投入 | `95281ae` 相对父提交涉及 67 个文件、`+3147/-1226`；当前只有一个生产 native option 完整摊销链路。 | 评估回本必须同时计算基础设施和边际收益。 |
| 正确性 | typed validation、selected applicability、source attribution 和 protocol validation 已有生产用途。 | 完全分散会重新引入关键 resolver 与验证责任。 |
| 未使用能力 | env extraction、三种复杂 merge、derive/materialization 等没有生产消费者。 | 收缩应从无消费者能力开始，而不是拆除共享语义。 |
| 外部实践 | 10 个成熟 CLI 都按稳定领域集中 schema、resolver 或 provider，没有全局描述符统一所有 surface。 | B′ 比“全量集中”或“自由散落”更接近成熟边界。 |

## 结论强度与边界

- 已确认：相关 OpenSpec 21/21 完成；当前复杂度不是剩余任务导致；A/B 的边际指标、验证结果和 owner 差异已由独立 reviewer 复核。
- 推荐：B′ 是当前证据下维护面更小、且能保留关键 contract 的候选方向。
- 尚未确认：Boolean presence/reset、array/object、keyed merge 和多个 adapter 大规模增长时，完整 processing abstraction 是否会产生更高收益。
- 不可外推：本实验不能证明实现速度、runtime 性能、完整回本周期或可直接删除的净代码量。

## 证据与方法

### 外部项目调查

调查覆盖 Cargo、uv、ripgrep、Starship、Zellij、GitHub CLI、kubectl、Terraform、AWS CLI 和 Docker Compose。每个结论均以官方文档或固定 commit 的官方源码核对；样本用于比较成熟实现模式，不作流行度排名。

### Docnav 内部审计

审计基于当前提交 `95281aec632d3d36618409d92eefc62fc37df294`、其直接父提交 `9892a925d24a97cd8f5011b560509ce511c49f0e`、相关 OpenSpec artifacts、代码、测试和调用点搜索。

### A/B 实验

两个实现代理在隔离 detached worktree 中完成相同的 `min_heading_level` 行为和 RED→GREEN 证明；第三个只读 reviewer 对称复核 diff、真实 CLI 输出和比较口径。实验 diff 不提交，结论固化后清理。

B 已有 adapter spec、config projection 和 resolver，只保留旧 CLI candidate bridge。因此本实验比较 current centralized 与 legacy hybrid 的边际成本，不代表 centralized 与完全手写实现的差异。

## 关键发现

### 成熟 CLI 集中稳定领域，而非所有 surface

| 模式 | 项目 | 可复用判断 |
| --- | --- | --- |
| 集中 merge、provider 或 provenance | [Cargo](https://doc.rust-lang.org/cargo/reference/config.html)、[uv](https://docs.astral.sh/uv/concepts/configuration-files/)、[AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-configure.html#configuration-and-credentials-precedence) | CLI、config、env 结构分开；解析后的语义、来源和优先级集中。 |
| 稳定领域内局部集中 | [GitHub CLI](https://github.com/cli/cli/blob/2af8c115be240a8018add33bf5c7a9ba5070a62c/internal/config/config.go#L587-L700)、[kubectl](https://kubernetes.io/docs/concepts/configuration/organize-cluster-access-kubeconfig/) | 持久配置项、kubeconfig model 或共享 flags 有 owner；命令参数仍由命令维护。 |
| 复用 argv 或有序 collector | [ripgrep](https://github.com/BurntSushi/ripgrep/blob/0d7054d8e466d6aa0a6bb6cf121e87225d26df44/GUIDE.md#L540-L624)、[Terraform](https://developer.hashicorp.com/terraform/cli/config/environment-variables#tf_cli_args-and-tf_cli_args_name) | 通过收缩配置语言复用 parser，适合平坦参数面。 |
| Typed model + 专用 merge | [Docker Compose](https://docs.docker.com/reference/compose-file/merge/) | mapping、sequence、unique resource 和 reset 使用不同规则。 |
| 收缩输入源 | [Starship](https://starship.rs/config/#config-file-location) | 从产品层减少逐字段多源覆盖。 |
| 共享 struct + 显式边界代码 | [Zellij](https://github.com/zellij-org/zellij/blob/0871f5d2b6f2fed36c6818bc9871bbbb5703dabe/zellij-utils/src/input/options.rs#L447-L578) | derive Clap/Serde 后仍手工维护 merge 与 KDL parse/serialize。 |

这些项目共同支持三条规则：

1. 集中 canonical model、provider chain、precedence、provenance 或一个 owner 的 field facts；不同 surface 在明确边界归一化。
2. 把 absent、empty、false 和 reset 视为独立 public contract；复杂列表、互斥字段和结构 merge 使用专用 resolver 或语法。
3. 用 owner-local generation、schema validation、precedence/reset tests 和 effective-config inspection 控制漂移，不依赖全局 DSL 或单纯文档提醒。

### Docnav 已获得局部集中收益，但尚未充分摊销

| 观察面 | 已确认状态 | 含义 |
| --- | --- | --- |
| Change 状态 | `derive-document-cli-options-from-fields` 21/21 完成。 | 当前复杂度主要来自既定边界和抽象，不是最后几项未实现。 |
| 已用收益 | owner declaration 投影 CLI/help/config/default/constraint/applicability；resolver 保留 typed candidate、selection 与 attribution。 | document option projection 的重复确实减少。 |
| 当前消费者 | 只有 Markdown `max_heading_level` 一个 native option 完整使用链路。 | 通用基础设施的经济性尚未由规模证明。 |
| 未使用能力 | `env_var`/`extract_env`、`Append`、`MapMerge`、`DenyConflict`、derive/materialization 等无生产消费者。 | 可优先做 consumer-driven 收缩审计。 |
| 维护面 | 5 个基础包约 5,408 行生产代码、3,874 行测试；另有 27 个生产文件直接依赖相关契约。 | 删除量不能视为净收益，但理解和验证成本已经存在。 |
| 事实源边界 | Serde config model、合法路径、JSON Schema、runtime/output 映射仍有独立 owner。 | 当前路径不是“所有产品事实只写一次”。 |

原 proposal 只承诺 document named options。Config/source priority、diagnostic projection、protocol/readable shape、ref、pagination 和 adapter format behavior 本就保留原 owner；不应再用“统一所有东西”作为当前实现的验收标准。

### A/B 实验只证明了有限但真实的边际收益

实验新增一个非 no-op 的 Markdown `min_heading_level` integer option。完整契约见附录。

| 指标 | A：centralized | B：legacy hybrid |
| --- | ---: | ---: |
| Production files | 4 | 5 |
| Production numstat | `+127/-47`，net `+80` | `+124/-44`，net `+80` |
| Test files | 6 | 6 |
| Test numstat | `+247/-1`，net `+246` | `+235/-9`，net `+226` |
| Production owners | 1：Markdown adapter | 2：Markdown adapter + core CLI parser |
| Field fact authoring sites | 1：adapter declaration | 2：adapter declaration + core help mapping |
| Common domain diff | `+64/-20` | `+64/-20` |
| Help default | 自动显示 `[default: 1]` | 不显示 |
| Golden/docs/schema sync | 手工 | 手工 |

实验支持以下判断：

- A 不修改 core production；B 需要一处 arg-id 到 help 的手工映射。
- A 从 owning declaration 投影 flag、help、config、range、default、applicability、candidate identity 和单字段 diagnostic。
- A/B 的生产净增相同，adapter handoff、整数转换、outline/find 领域行为和长期材料同步也相同。
- A 的基础设施已经由 `95281ae` 预付；本次实验只能衡量新增一个简单 integer/Replace option 的边际成本。

两边满足冻结的最低契约，但并非 byte-for-byte 等价：A 自动展示 default；两名实现代理采用不同 declaration order，导致 sibling projection 顺序和多错误时的 primary diagnostic 不同。后者不在冻结契约内，属于实验混杂，不能用作架构优劣证据。

## 推荐边界：B′ 薄共享契约 + 受控分散

### 责任分配

| 责任 | 保留方式 | Owner |
| --- | --- | --- |
| Adapter option facts | Plain option spec 保存 identity、type、constraint、default、operations、CLI help/value name 和 config path。 | Adapter |
| Source resolution | 统一处理 precedence、presence、candidate selection、typed validation 和 provenance。 | Navigation |
| 共享验证原语 | 保留 protocol、navigation 和 adapter contract 实际复用的 value/constraint primitives。 | Shared validation core |
| CLI projection | 直接消费 owner spec，生成 flag、help 和 candidate。 | Core CLI |
| Config projection | 直接消费 owner spec，生成 config path facts 和 candidates。 | Navigation config-source boundary |
| Handoff 与领域效果 | 转换为 adapter 私有类型并实现 format-specific behavior。 | Adapter |
| 特殊 merge/reset | 使用命名明确的 field-specific resolver；真实复用出现后再抽象。 | Owning domain |

### 收缩候选

按消费者证据依次评估：

1. derive/materialization 和无独立 contract 的 passthrough API；
2. 尚未进入产品输入契约的 env extraction；
3. 无生产消费者的复杂 merge strategies；
4. 为未来 value kinds 预建、但当前 owner surface 不需要的 companion 分支。

每次收缩都必须先核对 protocol、adapter-contracts、navigation、CLI/config 和测试调用方。本报告中的 LOC 只表示审计范围，不表示可删除净代码量。

### 必须保留的防漂移机制

- CLI/config/default/operation applicability 从同一个 adapter option spec 投影；
- precedence 只由 navigation input resolution 拥有；
- absent、explicit false/empty 和 reset 有明确类型或 resolver contract；
- source attribution、selected applicability、protocol validation 有可执行证明；
- docs、schema、examples、case materials 标明 owner 和验证命令；
- 对最终值与来源保留可观察 inspection/trace。

## 方案比较

| 方案 | 适用条件 | 当前判断 |
| --- | --- | --- |
| A：继续扩大完整 type-field | 近期会出现大量同构 adapter options，且更多 value kinds/merge 已有真实消费者。 | 当前证据不足，不扩大范围。 |
| B′：薄共享契约 + 受控分散 | 需要保留 single-site projection、typed resolution 和 provenance，同时减少未使用通用能力。 | 当前推荐，尚未成为 Decision。 |
| C：完全分散 | 调用方愿意分别重建 validation、selection、provenance 和 protocol mapping。 | 当前收益不足以覆盖 contract 漂移风险。 |

## 进入新 OpenSpec change 的门槛

只有以下条件同时满足，才把 B′ 推进为实施 change：

1. 完成 macro、env、merge、companions 和 passthrough API 的生产消费者清单，删除候选均有直接证据。
2. 薄 option spec 能继续单点 author flag、help/default、config path、constraint 和 operation applicability；core 不新增逐字段表。
3. 明确保留 protocol validation、selected applicability、source attribution 和现有 error mapping 的实现边界与测试。
4. 若未来需求包含 Boolean presence/reset 或 repeated/list input，先用一个最小 spike 判断是否需要通用 processing abstraction。
5. Proposal 写清迁移 slices、rollback、owner docs、schema/examples/case materials 和 `bun run verify:docnav-workspace` 验收。

满足门槛后创建新的 OpenSpec change，以分阶段迁移和明确 rollback 保持可逆；当前已完成 change 的 Decisions 继续作为历史记录。

## 附录 A：A/B 冻结契约

实验 option：Markdown `min_heading_level`。

- identity：`docnav.adapters.docnav-markdown.options.min_heading_level`；
- CLI：`--min-heading-level <value>`；
- config：`options.docnav-markdown.min_heading_level`；
- operations：`outline`、`find`；
- range/default：`1..=6`、built-in `1`；
- priority：`explicit > project > user > built_in`；
- effect：只有 `min <= heading.level <= max` 的 heading 可见；
- `min > max`：不增加 cross-field diagnostic，沿用 `doc:full` fallback；
- `read`、`info`：不声明、不展示、不接受；
- explicit `0`：`INVALID_REQUEST`，field `arguments.options.min_heading_level`，reason `range_invalid`，source `explicit`。

显式 `2` 会过滤 H1；实验不改变 ref、pagination、protocol 或 output shape。

## 附录 B：实验验证

两边均完成真实 RED，并通过：

- `cargo test -p docnav-markdown --all-targets`：各 49 passed；
- core parser、linked runtime、navigation focused tests；
- `cargo check -p docnav-markdown -p docnav-navigation -p docnav`；
- `cargo fmt --all --check`、`git diff --check`；
- built-in/user/project/explicit、outline/find、invalid values、duplicate flag、`min > max`、read/info 的真实 CLI replay。

`cargo test -p docnav --lib`：

- A：105 passed，2 failed；
- B：92 passed，2 failed。

两边失败均为同两份 config-inspect golden 未包含新 option 的 config projection 与 built-in fact。实验刻意不更新长期材料，因此两个 diff 都不可合并；该失败不是 A/B 差异。

## 附录 C：测量与来源快照

内部审计和基线核对使用：

```bash
git show --stat --oneline 95281ae
cargo metadata --format-version 1
rg -n 'docnav_typed_fields|cli_config_resolution|FieldDefSet|SourceCandidate|AdapterOptionSpec' crates
```

外部源码固定 commit：

- Cargo `31476f8bc7633ae05fde9f8ea7c40155f3d47d29`
- uv `d046cdd10625bd0cb95549b5d47ff376ef160056`
- ripgrep `0d7054d8e466d6aa0a6bb6cf121e87225d26df44`
- Starship `8eb25b8130d1b7bf0c98c71d6f978224814b5208`
- Zellij `0871f5d2b6f2fed36c6818bc9871bbbb5703dabe`
- GitHub CLI `2af8c115be240a8018add33bf5c7a9ba5070a62c`
- Kubernetes `6d5610685c55faf1eab630ed7f6cd9f6d4accd13`
- Terraform `c07e79c1c88935d75bfbff52c0eab9ac9f84f688`
- AWS CLI `699c16c7377dcdb9bbb62c0fd7ab58c8a7ddece1`、Botocore `91a4146fe194857c4f8cf91c16838644f0fa4ce7`
- Docker Compose `5bf5a21687107138629baa30be97f0bd9a0c55b2`、compose-go `0670a1c375c19defea19147bd82eb126a7c33b29`

## 附录 D：实验清理

2026-07-17，在证据固化并通过验证后，已删除两个 detached worktree 及其未提交实验 diff：

- `/tmp/docnav-type-field-ab-iLaWCt/A-centralized`
- `/tmp/docnav-type-field-ab-iLaWCt/B-legacy`

随后运行 `git worktree prune` 并删除空父目录 `/tmp/docnav-type-field-ab-iLaWCt`。实验 working diff 不作为长期 artifact 保留。
