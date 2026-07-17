## Context

当前一个 format-specific 参数经过以下链路：

```text
adapter-owned parameter facts
  -> core projections and source resolution
  -> adapter-specific value handoff
```

这条链路把“adapter 实现格式策略”和“adapter 定义产品输入”绑在一起。它只有在 adapter 能独立于 core 发布输入契约时才形成有效扩展边界；Docnav 当前使用 core release 内的 static linked adapters，因此 adapter parameter declaration 只增加了 discovery、translation 和 handoff 层。

本 change 的主承诺是删除 parameter-authoring 边界：core 定义并处理输入，adapter 只执行策略。Typed-field 仍是 core 处理输入的基础。当前调用链、具体删除候选和消费者证据集中在 [`type-field-maintenance-report.md`](type-field-maintenance-report.md)，不作为长期架构契约重复展开。

## Scope and Terms

本 design 使用以下术语：

- “产品参数”是调用方可通过 CLI、env、config 或其它 core-owned source 提供的文档操作输入。
- “Closed consumer binding”是每个 catalog entry 指向一个 compile-time consumer target 的 closed binding。Strategy-visible values 使用 shared `StandardInputBinding`；core/navigation-only controls 使用 navigation/core-owned closed variants，不进入 adapter strategy input。该术语不要求具体 Rust 类型必须命名为 `ClosedConsumerBinding`。
- “Standard operation input”是 core 定义、navigation 构造并传给策略函数的 operation-specific closed Rust contract。只有 strategy-visible values 进入该 contract，并由静态字段、typed accessor 或 closed enum variant 表达；它已完成 source resolution 和标准类型 materialization，但不承诺所有 adapter 语义都已验证。类型放在 navigation 与 adapter 共同依赖的现有 shared operation-contract 层不改变 core 对产品字段和 binding 的 ownership。
- “Core validation”是 typed-field pipeline 在 dispatch 前执行的 structural、type 或 catalog-declared static validation。
- “Adapter semantic validation”是策略函数对格式算法、文档内容或参数组合前置条件执行的校验；它可以与 core validation 重复。

“Adapter 只定义策略函数”描述的是调用方参数边界：adapter definition 仍提供 format detection、capability 和 strategy behavior facts，但不存在 parameter declaration、source registration 或自定义 input-schema surface。

本 change 包含两个有先后关系的结果：

1. 必须完成 core-owned input pipeline 与 standard operation input 的主架构切换。
2. 主切换独立通过后，清理维护证据确认冗余的静态表示和 typed-field support surface。

以下事项由独立 change 负责：

1. 为具体产品字段新增或移除 env locator；本 change 只定义 locator 的 activation 语义和启用后的来源顺序。
2. 引入 runtime parameter plugins、外部可安装 adapters 或 independently published parameter SDK。
3. 新增、删除或改变 observable 产品参数。
4. 改变 `Replace`、`Append`、`MapMerge` 或 `DenyConflict` 的语义。
5. 强制所有 adapter-specific 语义校验只能位于 core 或只能位于 adapter。

## Target Architecture

```text
routing facts + core registry ----------> selected adapter/operation
                                                  |
core parameter catalog -- filter by adapter tag -+
                                                  |
explicit/env/config/defaults + applicable catalog fields
                            |
                            v
                 typed-field input pipeline
          extract -> merge -> default -> materialize
                    -> selected core validation
                            |
                            v
              standard operation input
                            |
                            v
registry strategy + standard input -> semantic validation -> result
```

| Owner | Owns | Consumes |
| --- | --- | --- |
| Core parameter catalog | 接受哪些产品参数、source locators、standard value kind、default、merge strategy、operation binding、closed consumer binding、可选的精确 adapter-id 标记和 core validation rules | static adapter ids 与 shared typed-field mechanics |
| Core adapter registry | adapter identity、format descriptors、capabilities、linked strategy implementation、registry-level implementation source | adapter definitions |
| Navigation | source loading、adapter selection、typed-field orchestration、standard input construction、dispatch | catalog、registry 与 resolution result |
| Adapter | strategy functions、format parsing/navigation algorithms、algorithmic validation、refs、result facts | standard operation input |
| Typed-fields/resolution | canonical field facts、extraction、merge、validation、materialization、provenance | consumer-owned definitions and sources |

同一 catalog 提供两个用途明确的 view：

1. Full config-validation projection 校验 namespace、catalog membership、standard type 和 core-declared static rules；它不决定本次调用消费哪些值。
2. Selected-operation field set 在 adapter selection 后按 exact adapter-id tag 与 operation binding 过滤，并且只有这个 view 进入本次 candidate extraction、resolution 和 standard-input construction。

本 change 的 catalog 只包含 `page`、`limit`、`pagination.enabled`、`output` 和 Markdown `max_heading_level`。`page`、`limit`、`max_heading_level` 的 closed consumer target 是 shared `StandardInputBinding`；`pagination.enabled` 使用 navigation-owned closed binding，与 `limit` 归一化为 effective limit；`output` 使用 core-owned closed binding，只进入 `PreparedNavigationRequest` / core output projection。Adapter routing、document path/ref/query、`invocation_log` 与 config-path selection flags 保持在 catalog 外。

Catalog entry 的 adapter 标记只有两种含义：

1. 没有标记：这是所有 adapter 共用的参数。
2. 标记一个精确的 static adapter id：只有 selected adapter id 与标记相同时，该参数才进入当前 operation field set。

Navigation 使用这个静态标记做一次直接过滤；operation binding 继续决定字段是否属于当前 operation。Adapter 对某个值执行校验，不会使它成为该参数的 declaration owner。

## Runtime Flow

1. Core 提供 static registry、closed parameter catalog 和调用方 raw sources。
2. Navigation 使用 full catalog projection 校验 config source；其它已知 adapter 的合法值可以保留为 source facts，但不因此进入本次 operation input。
3. Navigation 根据 routing facts 选择 adapter strategy，保留未标记参数和 adapter-id 标记等于 selected adapter id 的参数，再按 operation binding 得到当前 field set。
4. Navigation 调用 typed-field pipeline 提取 selected candidates、按来源合并、应用默认值、materialize 标准类型，并执行 catalog 选择放在 core 的校验。
5. Navigation 对同一 resolution result 应用各 entry 的 closed consumer binding：strategy-visible values 构造 closed standard operation input，`pagination.enabled` 与 `limit` 归一化为 effective limit，`output` 只构造 `PreparedNavigationRequest` / core output projection；protocol projection 保持独立。
6. Selected adapter strategy 只以 standard input 取得调用数据，执行必要或重复的语义校验，再运行格式算法并返回 result 或 diagnostic。

Adapter 不参与步骤 1–5 的参数声明、source validation/extraction、priority、merge、default 或 binding。

## Validation Model

校验位置不是 parameter ownership 的判据。本 change 使用分层而非互斥的校验模型：

1. Core 必须完成能构造 standard input 的检查：参数必须存在于 catalog、适用于 selected adapter/operation、能从 source value 解码并 materialize 为标准类型，且 merge/default 过程有效。失败时不 dispatch。
2. Core 可以为某个字段不执行 adapter 语义校验、只执行极简静态校验，或执行完整的 context-independent validation。这个选择属于 core catalog/pipeline，不由 adapter declaration 注入。
3. Adapter strategy 可以校验或重复校验 standard values。凡是算法安全或正确性依赖、而 core 没有保证的 adapter-specific 或 context-dependent 前置条件，adapter 必须在使用前检查。
4. 同一规则在 core 与 adapter 两处执行时，两处必须接受同一 value domain，并映射到兼容的 observable diagnostic；重复校验用于边界防御，不建立第二套参数定义。

这里的“core 不校验”专指 core 不执行 adapter 语义校验。Core 不会把无法解码、无法完成 source resolution 或无法 materialize 为 standard input 的 raw value 直接交给 adapter。

## Runtime Invariants

1. Core catalog 是“本 release 接受哪些产品参数”的唯一 runtime source；registry、adapter definition、manifest、config 和 protocol payload 不能增加参数。
2. Catalog entry 提供 canonical identity、适用的 CLI/env/config locators、standard value kind、default、merge strategy、operation binding、closed compile-time consumer binding、可选的精确 adapter-id 标记，以及 core 选择执行的 validation facts。每项必须有一个 closed consumer target；只有 strategy-visible values target shared `StandardInputBinding`。
3. Catalog construction 拒绝 duplicate identity/locator、指向 unknown adapter id 的标记、missing or incompatible consumer binding、invalid operation binding 与 incompatible definition。
4. Full config-validation projection 与 selected-operation field set 必须从同一 catalog facts 派生；完整校验不会让其它 adapter 的值参与本次 resolution。
5. Navigation 先选择 adapter，再以 `adapter_id is absent || adapter_id == selected_adapter_id` 过滤 catalog，并从 ordered sources 完成 selected extraction、merge、default fallback、typed materialization 与 configured core validation。
6. Protocol `OperationArguments.options`、standard operation input 与 `PreparedNavigationRequest` / core output projection 是同一 resolution result 的 consumer-specific projections；strategy input 不从 protocol JSON 反向构造，`output` 不进入 strategy input。
7. Standard operation input 只暴露 strategy-visible compile-time typed fields/accessors 或 closed variants，并包含策略所需的 normalized operation facts；strategy 不接收 core/navigation-only controls、通用 parameter bag、protocol envelope、source-priority metadata 或 parameter declarations。
8. Env locator 的存在表示该字段启用 env source；缺少 locator 时不产生 env candidate。启用字段使用 `explicit > env > project > user > built_in`，未启用字段保持 `explicit > project > user > built_in`。
9. `Replace`、`Append`、`MapMerge`、`DenyConflict`、equal-priority registration order、invalid candidate handling、fallback 与 provenance 保持现有契约。
10. Schema、processing、extraction、validation、resolution 与 materialization 复用同一 canonical field facts；processing projection 只增加 processing id、input kind 与 locator context。
11. Contract-validation boundary 继续为 protocol、manifest、probe 等 JSON surfaces 直接构造 `FieldDefSet`；字段语义仍归对应 owner，且不进入产品参数 catalog。
12. Adapter strategy 可以返回 adapter semantic validation diagnostic，也可以重复 catalog rule；它不能因此增加、重命名或重新声明产品参数。
13. 既有 adapter-scoped diagnostics 的 code、owner labels、field/source、expected/received、guidance 与 readable projection 保持兼容；label 不再表示参数 authoring ownership。
14. Full scalar catalog projection 与现有 owner-specific compound validator 组合完成 outline full validation；`outline.mode_rules` / `outline.thresholds` 的 compound algorithm 不必改写为普通 scalar catalog fields。

## Decisions

### 1. Core owns a closed product parameter catalog

接受哪些产品输入由发布 `docnav` binary 的 core 定义。新增 adapter-scoped 参数需要在同一 release 中更新 catalog 与 adapter consumer。

### 2. Exact adapter-id tags control inclusion

Format-specific 参数继续使用 `options.<adapter-id>.<key>` 等既有输入路径，并在 core catalog 中带一个精确的 static adapter-id 标记。未标记参数是通用参数；标记参数只在该 adapter 被选中时参与 resolution。Core 拥有 public input surface 与 standard-input binding；adapter 只通过策略函数拥有该值如何影响格式算法。

### 3. Catalog and adapter registry remain separate

Catalog 提供参数 facts 和可选的 static adapter-id 标记；registry 提供 adapter behavior facts。Navigation 只用 selected adapter id 与 catalog 标记做相等比较，不从 registry 或 adapter definition 聚合参数。

### 4. Closed consumer bindings keep projections separate

Navigation 从 typed resolution result 按每项 closed consumer binding 构造 consumer-specific projections。Strategy-visible values 使用 shared `StandardInputBinding`；`pagination.enabled` 与 `output` 使用 navigation/core-owned closed variants，分别参与 effective-limit normalization 与 `PreparedNavigationRequest` / core output projection，且不进入 adapter input。Standard input 类型位于现有 shared operation contract，以便 navigation 构造并由 adapter strategy 消费；这种物理依赖位置不转移 core 对字段、binding 和 accepted input surface 的 ownership。

### 5. Validation is layered, not exclusively owned

Core 保证 source resolution 和 standard type materialization，并按 catalog 执行选定的 pre-dispatch validation。Adapter 可执行或重复执行策略所需语义校验；这种执行权不包含参数声明权。

### 6. Runtime converges on one parameter path

等价测试通过后，runtime 只通过 core catalog 和 closed consumer bindings 构造各产品参数 consumer projection；strategy-visible subset 构造 standard input。实现与验证不需要维护第二套 parameter definition source。

### 7. Full validation and selected resolution are catalog views

Config inspect 和 runtime config source validation 使用 full catalog projection；selected-operation resolution 在 adapter selection 后使用 exact adapter-id 与 operation binding 过滤后的 field set。两个 view 从同一 canonical facts 派生，且只有 selected view 构造本次 standard input。

### 8. Env locator presence enables the source

Catalog entry 只有在该产品字段启用 env 时才包含 env locator。Locator 存在时 env candidate 位于 explicit 之后、project/user 之前；locator 不存在时该字段没有 env source。为具体字段新增或移除 locator 属于独立的 observable product change。

### 9. Secondary cleanup remains subordinate

Checkpoint A 独立达到产品等价后，才执行维护证据确认的静态表示和 support-surface 清理。具体类型与 package 删除项只由 [`tasks.md`](tasks.md) 和 [`type-field-maintenance-report.md`](type-field-maintenance-report.md) 维护；它们不得改变 catalog、standard input、adapter strategy contract、source precedence、merge semantics 或 observable behavior。

## Implementation Plan

### Checkpoint A: Core-owned input pipeline

1. 先固定 `max_heading_level`、source precedence、四种 merge strategy、env extraction、diagnostics、protocol validation 与当前 adapter-side checks。
2. 建立并验证 core catalog、每项 closed consumer binding、可选的精确 adapter-id 标记、strategy-visible closed standard operation input、env activation rule 和 layered validation boundary。
3. 切换 CLI、config inspect 与 navigation，使 full config validation 和 selected-operation resolution 从同一 catalog 派生，再让 typed-field pipeline 只消费 selected field set。
4. 让 adapter definition 只提供 routing、capability 与 strategy facts，并让 Markdown strategy 消费 standard input。
5. 从同一 resolution result 按 closed consumer binding 直接构造 protocol、standard operation input 与 `PreparedNavigationRequest` / core output projections。
6. 清除被 core catalog 路径替代的 parameter declaration、discovery、registration 与 handoff。
7. 保留必要或防御性的 adapter semantic validation，并完成主路径产品等价验证。

退出条件：product parity tests 通过；runtime 只有一个 parameter definition source；full/selected views 不形成第二套 facts；adapter 只通过 closed standard input 执行策略，不声明参数或处理 sources；core-only、adapter-semantic 和重复校验路径都有边界证明。

### Checkpoint B: Secondary infrastructure cleanup

前置条件：Checkpoint A 已通过且 diff 可独立审查。该 checkpoint 是随主切换进行的次要维护工作，不能反向改变 standard input 或 adapter contract。

1. 删除主路径切换后只重复静态 facts 的 construction、wrapper 与 registry surfaces。
2. 让 processing metadata 组合 canonical field facts。
3. 将 direct input 收窄为 bounded source representation。
4. 收窄 foreign declaration 和 framework-specific projection surfaces，保留有 production consumer 的 direct builders、Serde companion 与 env extraction。

退出条件：每个删除项都有 consumer search 和 focused test 证据；四种 merge strategy、Serde、env extraction、protocol validation、routing 与 capability behavior 仍可用；Checkpoint A 的契约和 observable behavior 不变。

### Owner sync and closeout

同步主规范、delta specs、schemas/examples、crate docs、case ledger 与 manifests，再运行 focused、workspace、OpenSpec、docs 和 diff validation。具体命令与完成顺序由 [`tasks.md`](tasks.md) 负责。

## Risks and Controls

| Risk | Control |
| --- | --- |
| Catalog 与 adapter 再次复制参数 facts | Adapter tests 只断言接收和使用 typed value；catalog tests 断言完整 public definition。 |
| Ownership 切换改变 observable input 或 diagnostics | 用现有 CLI/config/protocol/error goldens 做 equivalence tests。 |
| Core 与 adapter 重复校验产生不同 value domain | 对重复规则使用共享常量或等价测试，并验证 diagnostic mapping。 |
| “允许 adapter 校验”重新演变成 adapter declaration | Strategy input contract 只提供 standard input 和 diagnostic/result；限定搜索证明没有 declaration/discovery surface。 |
| Full validation 与 selected filtering 混用，导致其它 namespace 被消费或漏校验 | 两个 view 只从同一 catalog 派生；覆盖 known-valid other adapter、unknown path、invalid value、matching/non-matching tag 与 unsupported operation。 |
| Standard input 重新演变成通用 parameter handoff，或 sibling projection 改变 wire shape | 使用 closed typed input contract；对 protocol JSON、CLI/config errors 与 adapter semantic errors 做等价 golden。 |
| Env locator 成为未启用的 dormant metadata | 以 locator presence 作为 activation gate，并覆盖 absent/present locator 与来源顺序。 |
| 次要清理误删真实 routing/strategy/capability seam | 每个删除项先证明无独立行为；保留 multi-adapter routing、capability validation 与 operation dispatch。 |
| 主路径与次要清理相互掩盖回归 | Checkpoint A 先独立通过；Checkpoint B 使用单独 exit gate。 |
| 删除 package 遗漏隐藏消费者 | 先做 workspace dependency/search gate，再执行 root build/test。 |
| Core 私有 Clap mapping 重新长成通用框架 | 只实现当前 catalog entry 所需 projections；出现通用 array/object/custom matrix 时停止并重新审计。 |

## Verification Outcomes

完成本 change 时必须证明：

1. Core catalog 是 CLI、env/config extraction、config inspect、navigation resolution 与 closed consumer binding 的唯一产品参数 definition source；每项有 compatible compile-time consumer target，只有 strategy-visible values 使用 shared `StandardInputBinding`。Full validation 与 selected-operation view 从同一 facts 派生，且 untagged、matching tag 和 non-matching tag 的结果符合静态规则。
2. `--max-heading-level`、namespaced config、default、range、operation binding、diagnostics、protocol options 与 Markdown effect 保持等价。
3. Adapter definition 只提供 routing、capability 与 strategy facts；strategy 只消费 closed operation-specific standard input，且仍可返回 semantic validation diagnostic。
4. Core structural/materialization failure、adapter-only semantic failure 和 core/adapter duplicate validation 都符合分层边界，且不会产生第二套参数定义。
5. `Replace`、`Append`、`MapMerge`、`DenyConflict`、ordered sources、env extraction、locator activation、validation、defaults、materialization 与 provenance 仍有可执行证明。
6. Checkpoint B 的每个删除项都有 consumer search 和 focused test 证据，且未改变 Checkpoint A 的契约或 observable behavior。
7. Protocol validation、schema/examples、raw/readable output、refs 与 pagination 没有非授权变化。
8. Workspace 验证不新增失败；变更前已存在的失败在 closeout 中单独报告。

## Open Questions

已收敛：参数 catalog 范围、closed standard input、full/selected catalog views、env activation、core/adapter ownership、分层校验、次要清理边界、保留的 typed-field 基础和两个实现 checkpoint 均由本 design 定义；无待实现者自行选择的架构分支。
