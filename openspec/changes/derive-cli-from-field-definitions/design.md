## Context

`docnav-typed-fields::FieldDef` 已拥有 identity、value kind、constraints、default、merge strategy 和 processing locator。`cli-config-resolution-clap` 已能从 `FieldDefSet` 注册 arguments，并能把 value decoding failure 保存为 invalid `SourceCandidate`。Framework-independent resolver 已定义 selected/merge-contributing invalid candidate 阻断、被 `Replace` 覆盖的 invalid candidate 只进入 trace 的语义。

Docnav 仍在 core Clap builders、`docnav-cli-args`、native option catalog 和 navigation string/JSON bridge 中重复维护 CLI facts。当前 typed-fields API 只提供固定 schema/processing projections，不能让 consumer 在字段声明处附加并随后 typed retrieval 项目专属元信息。普通 document execution 还会在单个阶段不需要某段配置时提前暴露其它 section、operation 或 adapter 的校验问题。现有 change 因此需要同时补齐最小 extension boundary、Docnav 声明式 builder、single Clap path、selected-field resolution 和 stage-scoped config processing。

目标链路：

```text
canonical field declaration
  + opaque consumer extension metadata
  -> Docnav-specific field builder
  -> operation registry projection
  -> authoritative Clap command shape
  -> normalized typed/invalid candidates
  -> routing fields and adapter selection
  -> selected operation FieldDefSet
  -> selected candidates only
  -> existing priority / merge / validation / materialization
  -> request construction and dispatch
```

配置处理与上述 CLI 链路共享 canonical declarations，但使用按阶段选择的 projection：

```text
config source bytes
  -> origin-aware source loading and JSON object formation
  -> current stage projection
  -> only current-stage candidates and diagnostics become observable
  -> later stages select their own projection when reached
```

## Goals / Non-Goals

**Goals:**

- Canonical field facts 与 Docnav CLI presentation 在同一 declaration authoring flow 中声明。
- Typed-fields 提供不理解 payload 含义的 immutable type-indexed extension metadata 保存与 typed retrieval 能力。
- Command-shape failure、candidate decode facts 与 selected-field validation 分层处理。
- Resolver 只处理 current invocation 实际选择的 fields。
- 普通 execution 的 logging、routing、outline policy 和 selected-operation 阶段各自只暴露当前 projection 的配置结果；`config inspect` 使用完整 source projection。
- Main workspace 与独立 `cli-config-resolution` workspace 通过有明确 owner 的 projection input 单向衔接。
- 完成单路径切换并删除重复 CLI models、scanners 和 fallback。

**Non-Goals:**

- 不让 typed-fields core 依赖 Clap、Docnav、operation 或 adapter 类型。
- 不提供 extension replacement、mutation、lifecycle、缓存策略或通用 plugin framework。
- 不改变 config source loading、source priority、materialization 和 adapter dispatch 语义。
- 不要求 config source 按 section 物理 lazy parse；允许共享实现提前计算内部 facts，但 projection 外结果不得进入本阶段 outcome 或 diagnostic。
- 不迁移 config-only outline selectors 或 core-owned invocation logging config。
- 不扩展 protocol payload、adapter format 或 Markdown 性能范围。

## Decisions

### Decision 1: Typed-fields 只提供 immutable type-indexed extension

`FieldDefBuilder` 提供 `extension<E>(value)`，built `FieldDef` 提供 `extension<E>() -> Option<&E>`；`E` 必须满足 `Send + Sync + 'static`。每个 field 对同一 Rust extension type 只允许一个 immutable payload，重复声明在 field build 时确定性失败。Clone 共享同一 immutable payload，metadata 在 builder clone、declaration type erasure、field build 和 definition-set aggregation 后保持可取回。

本 change 不提供 string extension key、replace/set、mutation 或 `FieldDefSet` 上的第二套 retrieval API。Projection 先从 set 取得 field，再读取对应 extension。需要不同 presentation 时，项目 builder 在 attach 前构造最终 payload；未来只有在出现实际覆盖场景时才单独扩展 API。

主仓库新增 `docnav-field-authoring` shared crate，位于 `docnav-typed-fields` 之上、adapter contracts/navigation/core 之下。它拥有 `DocnavCliPresentation`、`FieldDefBuilder` 项目扩展和 framework-neutral `DocnavFieldProjection`。带 public CLI locator 的 field 缺少该 extension，或 extension 与 locator/value kind 不兼容时，projection 确定性失败；没有 CLI locator 的 config-only field 可以不携带该 extension。

### Decision 2: Clap companion 拥有 framework input，core 拥有机械 bridge

独立 `cli-config-resolution` workspace 不依赖 Docnav 主仓库。`cli-config-resolution-clap` 定义自己的 `ClapFieldSpec` input，承接 argument identity、locator、value kind、cardinality、help/value name/order、canonical accepted/default display facts 和 Boolean encoding。Accepted/default facts 只用于 generated help；它们不得变成 Clap value parser 或 selection 前的 semantic validation。`docnav-field-authoring` 不依赖该 companion。

Docnav core 同时依赖 `docnav-field-authoring` 和 companion，并把 `DocnavFieldProjection` 一对一转换为 `ClapFieldSpec`。该 bridge 只复制已派生 facts，不重新计算 flag、constraints、default、accepted values、owner 或 operation applicability。映射由 contract test 固定，避免主仓库与子仓库反向依赖。

### Decision 3: Command-shape failure 与 field candidate 分层

Navigation 为每个 document operation 聚合 applicable common fields 和 registry adapter native fields。Core 使用派生 projection 扩展 authoritative Clap tree；Clap 处理 command topology、flag recognition、cardinality、duplicate single-value input、missing value 和 token boundary。这些 structural command-shape failures 在 adapter selection 前阻断，不属于 field candidates，也不受 selected-set filtering 影响。

Structural parse 成功后，companion 按 canonical value kind 与 Docnav Boolean extension decode 已注册输入。成功值成为 typed candidate；无法 decode 的值成为保留 field identity、locator、raw input 和 reason 的 invalid candidate。Enum、range、pattern、required/default 和其它 canonical semantic constraints 不在 Clap 中 eager validate，只在 candidate 进入 selected `FieldDefSet` 后由 resolver 判断。Projection mismatch、match storage mismatch、source construction failure 和 declaration conflict 是 structural/internal failure。该阶段只产出 candidate facts，不判断其最终是否被 selected operation 使用。

### Decision 4: Adapter selection 后重组 selected FieldDefSet

Adapter selection 前只解析 routing-required declarations。Selection 后，navigation 从 current-operation common declarations 与 selected adapter/current-operation declarations 重新组成 `FieldDefSet`，并只把 identities 存在于该 set 的 candidates 交给 resolver。

Selected `Replace` winner 和实际 merge contributor invalid 时，现有 canonical resolution 返回 field failure；未进入 selected set 的 typed/invalid candidate 无论内容如何都不参与 validation、request 或 dispatch，并在该边界丢弃。系统不为这些 candidates 建立 usage state 或诊断分支。

### Decision 5: Config diagnostics 由 current-stage projection 限定

Config loader 继续按现有规则处理 absent default path、missing explicit path、unreadable、invalid JSON 和 top-level non-object。这些 source-level failures 发生在任何 field projection 可用之前，继续阻断对应 invocation。

成功形成 JSON object 后，每个阶段只选择自己需要的 projection：core-owned logging 阶段只处理 invocation logging fields；navigation routing 阶段只处理 routing-required fields；outline policy 阶段只在 outline invocation 中处理 config-only selectors；adapter selection 后只处理 current-operation common fields 与 selected adapter declarations。每个 normal-stage projection 是所选 field locators 及读取它们所需结构祖先的正向白名单，不是完整 config object 的 schema。一个阶段不得因为其它 section、operation、adapter namespace、option、field 或 sibling key 的 unknown/shape/typed facts 失败。

实现可以共享已解析 JSON object，也可以为了复用提前生成更宽的内部 validation facts；但在 current-stage boundary 之外的 facts 必须被丢弃，不得进入 primary diagnostic、trace、request、handler input 或 invocation outcome。Later stage 只有在实际到达并选择自己的 projection 后，才能使对应结果可观察。

`docnav config inspect` 明确选择 registry-wide inspection projection，报告完整 source 的 unknown key、unknown adapter、shape 和 typed-value facts，但不计算 source winner、effective request 或 dispatch input。

### Decision 6: Core 只拥有 authoritative command tree 和 output/process mapping

Core 组合 root/subcommand topology、fixed positionals、routing/static arguments 和 registry projection，然后只执行一次 authoritative Clap parse。Raw native option scanner 和第二套业务 argv parser 被删除；normalized candidates 和 owner correspondence 交给 navigation。

Explicit `output` 必须成功 extraction 并通过 canonical validation 后，才能选择后续 failure renderer。Command-shape failure、duplicate output、invalid output，或尚未确定 valid output 时的 failure 使用 PlainText。Valid explicit output 控制其后的 config loading、routing、selection、resolution 和 adapter failures；未提供 explicit output 时，config/default output 只有在 normal navigation resolution 成功后才成为 output context，之前的 failure 使用 PlainText。String/path value 以 `-` 开头时使用 `--flag=<value>`。

### Decision 7: 通过 hard cutover 删除重复路径并拆开 companion 热点

先完成 extension、project authoring、companion projection、selected-set 重组和 authoritative core tree，再一次性切换 command path。旧路径的具体删除项与验收搜索由 tasks 维护；运行时不保留双路径，rollback 通过 revert change 完成。

本 change 会直接修改当前 quality acceptance 指向的 CLI parsing hotspot，因此 `cli-config-resolution-clap` 同步拆分 projection validation、command augmentation 和 candidate extraction 职责，并移除被此次修改触发的 accepted warnings。该拆分只服务本 change 的 touched path，不引入 plugin 或 lifecycle abstraction。

## Observable Behavior Matrix

| ID / Surface | Target behavior | Required evidence |
| --- | --- | --- |
| B1 / CLI command shape | Unknown、duplicate single-value、missing value 在 selection 前失败并使用 PlainText。 | CLI/process tests |
| B2 / Registry field candidate | Structural parse 成功后，decode 或 canonical semantic constraint 不合法的值保留为 field candidate/fact；未进入 selected set 时被丢弃且不产生 diagnostic。 | Companion + navigation tests |
| B3 / Config source formation | Explicit missing、unreadable、invalid JSON、top-level non-object 按 source-level 规则失败。 | Config loading tests |
| B4 / Current-stage config | 当前 logging、routing、outline policy 或 selected-operation field path及其必要结构祖先内的 invalid fact 可以阻断该阶段。 | Stage-focused config tests |
| B5 / Out-of-stage config | Projection 外 facts 即使被内部实现计算，也不产生 diagnostic、trace 或 request effect。 | Negative stage-isolation tests |
| B6 / Config inspect | Registry-wide projection 报告完整 source issues，但不构造 effective request。 | Config inspect tests |
| B7 / Explicit output | Valid explicit output 控制 structural parse 成功后的 later failure；invalid output 使用 PlainText。 | Output/process tests |
| B8 / Config/default output | 只在 normal navigation resolution 成功后成为 output context；此前 failure 使用 PlainText。 | Output stage tests |
| B9 / Hyphen-leading value | String/path value 使用 `--flag=<value>`；separated flag-shaped token 服从 Clap shape。 | CLI spelling tests |

## Risks / Trade-offs

- 未进入 selected field set 的 CLI candidate 和 current-stage projection 外的 config facts 不影响普通 invocation；`config inspect` 是完整 source 检查入口。
- Stage isolation 是 observable contract；共享 parser 或 validator 的内部复用不得把 projection 外 diagnostics 泄漏到当前 outcome。
- Immutable type-indexed extension 的 clone、type erasure、duplicate 与 retrieval 语义需要 typed-fields contract tests，避免 metadata 在 declaration aggregation 中丢失。
- Registry CLI projection 仍需确定性拒绝 locator/static-command conflict；不同 operation 可以复用 locator。
- Config source 无法形成 JSON object 时仍会提前失败；更细粒度的 lazy source loading 需要独立 change。
- PlainText early failure 与 hyphen-leading value spelling 是可观察 CLI 变化，需要同步 output/CLI docs 和 process tests。
- Companion 拆分必须保持 public extraction behavior，不借机扩展通用 framework scope。

## Migration Plan

1. 在本文件维护 B1–B9 observable behavior matrix，并把每行分配给 owner docs 和 tests。
2. 在 typed-fields 中增加 immutable type-indexed extension metadata，并建立 Docnav-specific builder/extractor contract tests。
3. 由 canonical fields 与项目 extensions 生成 Docnav projection，经 core bridge 转为 companion-owned `ClapFieldSpec` 和 typed/invalid candidates。
4. 生成 operation registry projection，并在 selection 后重组 selected `FieldDefSet`。
5. 为 logging、routing、outline policy、selected operation 和 config inspection 建立明确 projection boundary 与 stage-isolation tests。
6. Core 切换到 authoritative Clap tree 和 normalized candidate handoff；同步固定 output context 阶段。
7. 拆分 touched companion hotspot、删除 legacy path、更新 owner docs/tests/cases，并运行子仓库与 workspace 验证。

## Open Questions

无。
