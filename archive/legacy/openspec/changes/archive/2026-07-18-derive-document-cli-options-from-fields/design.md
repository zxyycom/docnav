## Context

`FieldDef` 已拥有 identity、value kind、constraints、default、merge strategy 和 processing locators；`cli-config-resolution-clap` 已能从 `FieldDefSet` 注册基础 arguments 并提取 candidates；`AdapterOptionSpec` 已包装 adapter-owned field declaration、operation applicability 与 handler binding。

当前 core document CLI 仍重写 common arguments、native option catalog、arg id、help/default/accepted values 和 argv decoding。Native input 随后以 raw string 进入 navigation，再按 flag 恢复 declaration 并猜测 JSON value。该路径重复 authoring field facts，也让 candidate 在进入 canonical resolution 前丢失 identity。

目标链路为：

```text
owning field declaration
  -> operation-scoped registry CLI FieldDefSet
  -> cli-config-resolution-clap arguments + candidates
  -> adapter selection
  -> selected adapter/current-operation FieldDefSet
  -> canonical priority / merge / validation / materialization
  -> request construction and typed handler handoff
```

## Ownership

| Concern | Owner | 本 change 的作用 |
| --- | --- | --- |
| Field semantics 与 CLI input facts | Common 或 adapter field owner；typed-fields 负责承载 | 增加并保存 CLI metadata |
| Clap argument/help/candidate projection | `cli-config-resolution-clap` | 直接消费 `FieldDefSet` |
| Root、subcommand、positionals 与 core-only flags | Core CLI | 组合 static shape 与 generated options |
| Operation applicability、adapter selection 与 request construction | Navigation + adapter declarations | 构造 registry/selected field sets |
| Config、output、diagnostic、protocol、ref 与 adapter format behavior | 现有 owner | 保持契约，只提供或消费 canonical field facts |

## Decisions

### Decision 1: Field declaration 承载 CLI processing metadata

CLI processing strategy 在 flag locator 之外保存 owner-authored help、value name 和 Boolean input encoding。Boolean encoding 支持 valueless presence switch 与 explicit token-to-Boolean mapping。Value kind、accepted values、constraints、default、merge 和 identity 继续来自 canonical field facts。

Metadata 必须经过 builder clone、declaration type erasure、field build 和 `FieldDefSet` aggregation 后仍可投影。Non-CLI attachment、duplicate metadata、incompatible Boolean value kind 和 incomplete token mapping 在 field build/projection 时确定性失败。Typed-fields 只承载和验证 facts，不依赖 Clap 或 consumer policy。

### Decision 2: Clap companion 直接投影 FieldDefSet

`cli-config-resolution-clap` 以 `FieldDefSet` 为唯一 field input。Argument augmentation 从同一 projection 取得 flag、identity、help/value name、capture strategy 和 canonical accepted/default display；candidate extraction 返回一个保留 field identity、locator、raw input 与 failure reason 的 CLI `Source`。

Clap 处理 unknown flag、duplicate single-value input、missing value 和 token boundary。Structural parse 成功后的 value-kind decode failure 成为 invalid candidate；enum/range/pattern、required/default 和 merge 留给 selected canonical resolution。Omitted input 不形成 explicit candidate，static default 由 resolver fallback 提供。

### Decision 3: Navigation 提供 registry CLI field set

Navigation 为每个 document operation 聚合 applicable common declarations 与所有 registry adapter 的 applicable native declarations。顺序为 common declaration order，其后是 registry order 与 adapter declaration order。

同一 operation 的 public flags 全局唯一。Generated-to-generated、generated-to-static 和 incompatible declaration conflicts 在 argv parsing 前报告，并包含 owner/field attribution。语义相同的共享 option 应提升为 common declaration；adapter-specific 同名 flag 支持留给后续 change。

### Decision 4: Selected field set 决定 candidate applicability

Structural parse 后，core 将 normalized CLI candidates、fixed positional facts、config source descriptors/paths 和 registry 交给 navigation。Navigation 先从 routing declarations 解析 adapter intent并完成 selection，再从 common declarations 与 selected adapter/current-operation declarations 构造 selected `FieldDefSet`。

Selected-set member 进入 existing priority、merge、validation 和 materialization。不属于 selected set 的 explicit candidate 返回 strict unsupported/unused diagnostic，并停止 request construction 与 dispatch。Config 中其它已知 adapter namespaces 继续遵循现有 config-source contract。

### Decision 5: Core 只组合 static 与 generated command shape

Core 继续拥有 command topology、fixed positionals、core-only flags、help/version side-effect boundary 和 output/process mapping。Document named options 通过 registry CLI field set augmentation 加入各 subcommand。

Generated help 使用 owner-authored help/value name 与 canonical accepted/default facts。`output` 在此只是普通 canonical field，projection 不枚举 mode 数量或名称。若 lexical preflight 继续存在，其 document flag/cardinality facts 也从 static shape 与同一 projection 派生。

### Decision 6: Runtime hard cutover，package 边界保持现状

Field projection、candidate extraction、selected resolution 和 regression evidence 完成后，document command 一次性切换到新路径。删除 native option catalog、derived arg-id table、raw native strings、post-parse flag lookup、JSON guessing、parallel decoder 和 runtime fallback。

共享 helper 仅在调用方清零时删除。Root-workspace package boundary 保持现状；rollback 通过 revert change 完成。

## Acceptance

| Check | Pass condition | Evidence |
| --- | --- | --- |
| Authoring | Common/native option facts 只在 owning declaration author | Declaration + repository search tests |
| Projection | Arguments、help 与 typed/invalid candidates 来自同一 `FieldDefSet` | Typed-fields + Clap companion tests |
| Applicability | Selected candidates resolve；unselected explicit candidates strictly fail | Navigation + process tests |
| Static boundary | Non-document commands、positionals、core-only flags 与 help/version side effects保持 | Core CLI tests |
| Cutover | Runtime 不再经过 catalog、raw remapping、JSON guessing 或 fallback | Repository search + workspace verification |
| Compatibility | Owner-documented source priority、config、diagnostic、output、protocol、ref 与 adapter behavior保持 | Existing owner regression suites |

## Risks / Trade-offs

- CLI metadata 增加 source-specific facts：metadata 只保存 locator 无法表达的 presentation/capture facts，consumer policy 保持在外层 owner。
- Registry set 与 selected set 分阶段构造：两者复用同一 declarations，并用 identity/applicability contract tests 防止漂移。
- 同名 adapter flag 延后支持：registry validation 提供 owner/field attribution；出现明确场景时单独设计 selected projection。
- Internal failure stage 或 Clap help formatting 可能变化：验收保持 owner-documented facts、diagnostic attribution 和 exit class，不建立 byte-for-byte layout contract。

## Open Questions

无。
