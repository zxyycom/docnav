## Context

`docnav-typed-fields::FieldDef` 已拥有 identity、value kind、constraints、default、merge strategy 和 processing locator。`cli-config-resolution-clap` 已能从 `FieldDefSet` 注册 arguments，并能把 value decoding failure 保存为 invalid `SourceCandidate`。Framework-independent resolver 已定义 selected/merge-contributing invalid candidate 阻断、被 `Replace` 覆盖的 invalid candidate 只进入 trace 的语义。

Docnav 仍在 core Clap builders、`docnav-cli-args`、native option catalog 和 navigation string/JSON bridge 中重复维护 CLI facts。当前 typed-fields API 只提供固定 schema/processing projections，不能让 consumer 在字段声明处附加并随后 typed retrieval 项目专属元信息。现有 change 因此需要同时补齐通用 extension boundary、Docnav 声明式 builder、single Clap path 和 selected-field resolution。

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

## Goals / Non-Goals

**Goals:**

- Canonical field facts 与 Docnav CLI presentation 在同一 declaration authoring flow 中声明。
- Typed-fields 提供不理解 payload 含义的通用 extension metadata 保存与 typed retrieval 能力。
- Command-shape failure、candidate decode facts 与 selected-field validation 分层处理。
- Resolver 只处理 current invocation 实际选择的 fields。
- 普通 execution 与 `config inspect` 使用适合各自用途的 field projection。
- 完成单路径切换并删除重复 CLI models、scanners 和 fallback。

**Non-Goals:**

- 不让 typed-fields core 依赖 Clap、Docnav、operation 或 adapter 类型。
- 不规定 extraction lifecycle、缓存策略或通用 extension plugin framework。
- 不改变 config source loading、source priority、materialization 和 adapter dispatch 语义。
- 不扩展 protocol payload、adapter format 或 Markdown 性能范围。

## Decisions

### Decision 1: Typed-fields 提供 opaque extension，Docnav 提供项目 builder 与 extractor

`FieldDefBuilder` 提供通用 consumer extension metadata 的声明接口，`FieldDef` / `FieldDefSet` 在 build、clone、type erasure 和 aggregation 后保留这些值，并提供按 extension key/type 的只读 retrieval。底层不解释 payload；重复声明必须确定性失败，显式更新使用独立 replace/set API。

主仓库新增 `docnav-field-authoring` shared crate，位于 `docnav-typed-fields` 之上、adapter contracts/navigation/core 之下。它拥有 Docnav extension payload、`FieldDefBuilder` 项目扩展和 framework-neutral projection view，不依赖 Clap。Adapter contracts 与 navigation 在声明处调用同一 builder 扩展；core/Clap companion 只消费派生 projection，依赖方向不反转。

Docnav extension 用于 help prose、value name、display order 与 Boolean switch/token-map encoding。Projection 函数遍历 `FieldDefSet`，从同一 field 读取 canonical metadata、processing locator 与项目 extension，生成 companion 可消费的 view。带 public CLI locator 的 field 缺少 Docnav CLI extension，或 extension 与 locator 不兼容时，projection 必须确定性失败；没有 CLI locator 的 config-only field 可以不携带该 extension。

### Decision 2: Registry parse 产生 candidate facts，不跟踪最终是否应用

Navigation 为每个 document operation 聚合 applicable common fields 和 registry adapter native fields。Core 使用派生 projection 扩展 authoritative Clap tree；Clap 处理 command topology、flag recognition、cardinality、duplicate single-value input、missing value 和 token boundary。

Companion 按 canonical value kind 与 Docnav Boolean extension 对已注册输入进行初步 decode。成功值成为 typed candidate；无法 decode 的值成为保留 field identity、locator、raw input 和 reason 的 invalid candidate。Projection mismatch、extension mismatch、match storage mismatch、source construction failure 和 declaration conflict 是 structural/internal failure。该阶段只产出 candidate facts，不判断其最终适用性。

### Decision 3: Adapter selection 后重组 selected FieldDefSet

Adapter selection 前只解析 routing-required declarations。Selection 后，navigation 从 current-operation common declarations 与 selected adapter/current-operation declarations 重新组成 `FieldDefSet`，并只把 identities 存在于该 set 的 candidates 交给 resolver。

Selected `Replace` winner 和实际 merge contributor invalid 时，现有 canonical resolution 返回 field failure；未进入 selected set 的 candidate 无论内容如何都不参与 validation、request 或 dispatch，并在该边界丢弃。系统不为这些 candidates 建立 usage state 或诊断分支。

### Decision 4: Normal execution 与 config inspect 使用不同 field scope

Config loader 继续按现有规则处理 absent default path、missing explicit path、unreadable、invalid JSON 和 top-level non-object。

成功形成 JSON object 后，普通 document invocation 只按 routing fields 与 selected operation `FieldDefSet` 的 config locators 提取 candidates。未被该 projection 声明的其它 key、operation field、adapter namespace 或 option 不读取、不校验、不报告。`docnav config inspect` 使用 registry-wide config projection，报告完整 source 的 unknown key、unknown adapter、shape 和 typed-value facts，但不计算 source winner、effective request 或 dispatch input。

### Decision 5: Core 只拥有 authoritative command tree 和 output/process mapping

Core 组合 root/subcommand topology、fixed positionals、routing/static arguments 和 registry projection，然后只执行一次 authoritative Clap parse。Raw native option scanner 和第二套业务 argv parser 被删除；normalized candidates 和 owner correspondence 交给 navigation。

Explicit `output` 必须成功 extraction 并通过 canonical validation 后，才能选择后续 failure renderer。Command-shape failure、duplicate output、invalid output，或尚未确定 valid output 时的 failure 使用 PlainText。该 breaking behavior 同步修改 `output-contract` 与 owner docs。String/path value 以 `-` 开头时使用 `--flag=<value>`。

### Decision 6: 通过 hard cutover 删除重复路径

先完成 extension、project authoring、companion projection、selected-set 重组和 authoritative core tree，再一次性切换 command path。旧路径的具体删除项与验收搜索由 tasks 维护；运行时不保留双路径，rollback 通过 revert change 完成。

## Risks / Trade-offs

- 未进入 selected field set 的 CLI/config facts 不影响普通 invocation；`config inspect` 是完整 source 检查入口。
- Opaque extension 的 clone、type erasure、duplicate/replace 与 retrieval 语义需要 typed-fields contract tests，避免 metadata 在 declaration aggregation 中丢失。
- Registry CLI projection 仍需确定性拒绝 locator/static-command conflict；不同 operation 可以复用 locator。
- Config source 无法形成 JSON object 时仍会提前失败；更细粒度的 lazy source loading 需要独立 change。
- PlainText early failure 与 hyphen-leading value spelling 是可观察 CLI 变化，需要同步 output/CLI docs 和 process tests。

## Migration Plan

1. 固化当前 CLI surface、source priority、candidate resolution 和 output mapping 证据。
2. 在 typed-fields 中增加 opaque extension metadata，并建立 Docnav-specific builder/extractor contract tests。
3. 由 canonical fields 与项目 extensions 生成 Clap projection 和 typed/invalid candidates。
4. 生成 operation registry projection，并在 selection 后重组 selected `FieldDefSet`。
5. Core 切换到 authoritative Clap tree 和 normalized candidate handoff；normal config extraction 收窄为 selected fields。
6. 删除 legacy path，更新 owner docs/tests/cases，并运行子仓库与 workspace 验证。

## Open Questions

无。
