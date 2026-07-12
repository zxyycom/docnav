本 change 的核心目标是让 Clap 与 `cli-config-resolution-clap` 成为 Docnav CLI 结构解析和动态字段解码的唯一业务实现路径；本文是仅位于 `openspec/changes/refactor-cli-parsing-through-clap/` 的未审核临时 design，不影响现有主规范或其它文档。

## Context

当前 CLI 参数链路有四个并行事实源：core Clap builders、`docnav-cli-args` scanner、native option catalog，以及 navigation 中的字符串/JSON 解码。动态参数虽然来自 adapter declaration，却在 core 被重建为 string-valued argument，导致 short flag、boolean、collection 和 string semantics 与 canonical metadata 脱节。

`cli-config-resolution` 子仓库已经包含 canonical `FieldDefSet`、resolution core 和 Clap companion。本 change 要完成两件事：让 companion 真正通过 Clap typed value parsers 解码动态值；让 Docnav 完整切换到该路径并删除旧桥接。

## Goals / Non-Goals

**Goals:**

- 一棵 authoritative Clap command tree 完成所有 CLI 业务结构解析。
- Core-owned arguments 和 adapter native arguments 都在进入 command model 前完成 Clap lexical decoding。
- Dynamic canonical projection、typed extraction 和 generic conflict detection 由子仓库拥有。
- Navigation 只接收 typed source facts，并在 adapter 选择后完成 owner/applicability、canonical validation 和 handler binding。
- 删除旧 scanner、字符串 bridge、JSON guess 和 runtime fallback，质量记录回到零豁免。

**Non-Goals:**

- 不改变产品命令、默认值、source priority、adapter selection、protocol/readable shape、ref 或格式语义。
- 不把 Docnav command policy、adapter registry、diagnostic/output policy 放进子仓库。
- 不新增 positional/alias DSL、custom parser registry、plugin framework 或跨 adapter flag-equivalence layer。
- 不处理 Markdown 性能。

## Target Flow

```text
raw argv
  ├─ presentation-only probe -> operation/output hint（仅用于 parse failure 投影）
  └─ authoritative Clap command tree
       ├─ core static args -> typed core command facts
       └─ registry native fields
            -> cli-config-resolution-clap
            -> typed CLI Source candidates

typed command facts + CLI Source + config descriptors + registry
  -> navigation loads config and selects adapter
  -> navigation filters selected adapter/operation candidates
  -> resolver merges explicit > project > user > built_in
  -> OperationArguments + NativeOptionHandoff
  -> selected adapter dispatch
```

## Ownership Boundaries

| Owner | Owns | Does not own |
| --- | --- | --- |
| `docnav` core | command tree、static args、help/version、presentation probe、Clap error 到 diagnostic/exit 的映射 | dynamic field semantics、adapter selection、canonical merge |
| `cli-config-resolution-clap` | canonical CLI metadata 到 Clap `Arg`、typed parser、typed `ArgMatches` read、`SourceCandidate` | Docnav command policy、adapter id、operation applicability、public diagnostic |
| `docnav-navigation` | registry projection、config loading、adapter selection、candidate filtering、source priority、request construction | Clap/`ArgMatches`、raw native string、format semantics |
| Adapter | native option declaration、operation applicability、constraints、default、handler binding 和业务语义 | CLI parsing、source priority、public output |

## Decisions

### Decision 1: 使用一棵 authoritative Clap command tree

Core 在 parse 前为每个 document subcommand 加入 static arguments 和 operation-scoped native projection，然后只执行一次 authoritative `try_get_matches_from`。成功结果直接从 nested `ArgMatches` 构造 typed command facts；不再先扫描 token、再按 subcommand 二次解析。

Core static arguments 也必须注册对应 typed parser：string/path、output enum、pagination enum-to-bool、positive page/limit 等均在 Clap boundary 完成 lexical parsing。Help/version 使用 `DisplayHelp` / `DisplayVersion` outcome，且不触发项目解析、config loading、adapter selection 或 dispatch。

Broad `allow_hyphen_values(true)` 会让下一个 known flag 被前一个 option 吞为值。实现统一使用无歧义规则：hyphen-leading string/path 通过 `--flag=<value>` 传入；negative number 只使用 numeric parser 的负数支持。

### Decision 2: Dynamic value decoding 完整归属 Clap companion

Companion 按 canonical `ValueKind` 生成固定投影：

| Value kind | Clap representation |
| --- | --- |
| `String` | `StringValueParser` -> `String` |
| `Integer` | `value_parser!(i64)` -> `i64` |
| `Number` | finite `f64` `TypedValueParser` -> `f64` |
| `Boolean` | `ArgAction::SetTrue` -> `bool` |
| `Array` | repeated string parser -> `Vec<String>` |
| `Object` | repeated bounded `key=value` parser -> typed entries |
| `Json` | `UnsupportedValueKind`，不注册 |

`extract_cli` 只读取 typed values 并机械转换为 canonical JSON values；不再调用 `str::parse` 或 `serde_json::from_str`。Clap 负责 CLI lexical type 和 `key=value` shape；typed-fields/resolution 继续负责 enum、range、requiredness、default、merge 和 materialization。

### Decision 3: Parse 前聚合 registry projection，selection 后解释语义

Navigation parameter aggregation 按 operation 收集 static registry 中所有适用 native declarations，形成一个 canonical CLI `FieldDefSet`。Core 只把该 set 交给 companion，不解释 adapter owner 或 field constraints。

同一 operation 的 CLI locator 必须全局唯一，且不得与 core static argument 冲突。所有 document subcommand projections 在接受 authoritative invocation 前完成 validation；冲突属于 release-local internal declaration failure。即使两个 adapter 的 flag 和 value kind 相同，也不自动合并，因为 constraints、default 和 handler binding 仍可能不同。

该规则解决当前真实的 pre-selection registration 需求，同时避免新增 profile/catalog/equivalence framework。

### Decision 4: 跨 crate 只交接 typed source，并固定错误优先级

Document command 保存 companion 生成的 CLI `Source`，不再保存 `NativeOptionCliInput { flag, value }`。Navigation 选择 adapter 后：

1. 取得 selected adapter 当前 operation 的 `OperationFieldSet`。
2. 核对每个 explicit native candidate 的 identity、owner 和 operation applicability。
3. 对 unknown、unselected-adapter 或 operation-inapplicable candidate 返回 strict diagnostic。
4. 只把 selected candidates 与 core direct、project、user、built-in sources 交给 resolver。
5. 从 canonical result 构造 `OperationArguments` 和 `NativeOptionHandoff`。

错误优先级固定为：

1. Registry/command-model declaration failure。
2. Authoritative Clap structure/lexical failure。
3. Navigation selected-owner/applicability failure。
4. Canonical field validation failure。

因此，属于未选 adapter 且词法值非法的 flag 先返回 Clap value diagnostic；只有成功形成 typed candidate 后，navigation 才能返回 unselected-adapter diagnostic。Core 不把 `ArgMatches` 传给 navigation，navigation 也不回读 raw argv。

### Decision 5: Failure presentation 保留一个严格限界的 raw argv probe

Authoritative parse 失败时，Docnav 仍要识别 document operation 和有效 `--output`，以保持现有 failure envelope。针对 Clap 4.6.1 的 focused proof 已确认：positional passthrough 和 `ignore_errors(true)` 都不能可靠恢复 earlier parse error 之后的 `--output`。因此这里保留一个明确例外，不虚构“纯 Clap preflight”。

Probe 只能识别：

- root operation token；
- `--output <value>`；
- `--output=<value>`。

它复用同一个 `OutputMode` parser；多个 occurrence 时，最后一个 syntactically valid mode 仅作为 presentation hint，没有 valid mode 时使用 `readable-view`。Authoritative Clap parse 仍负责 duplicate/invalid failure。Probe 不生成 command、field、native candidate 或 parse-success 结论，也不得增加其它参数。

## Risks / Trade-offs

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Operation-global flag uniqueness | Adapter author 不能无声明地复用常见 flag | 出现真实共享需求后先建立 shared owner；本 change 不推断兼容 |
| Clap lexical failure 更早 | Diagnostic 文案可能变化 | 稳定 code/details/channel/exit；使用 structured error context 和 smoke 验证 |
| Presentation probe 接触 raw argv | 可能重新增长为第二 parser | Spec 限制为 operation/output；测试禁止返回其它 facts |
| Hyphen-leading separated value 不再接受 | 少量 argv spelling 需要迁移 | 保留 `--flag=<value>`，在 help/guidance/tests 中明确 |
| 跨主仓库与子仓库迁移 | 中间提交可能不可运行 | 子仓库能力先独立完成，最终只接受单一路径和全量验证 |

## Migration Plan

1. 完成 tasks 中的阻塞级 artifact/API 审计，固化 Clap 4.6.1 focused proof。
2. 在子仓库实现 typed parsers/extraction，补齐独立 tests、README 和 example。
3. 在 navigation 增加 operation-scoped registry projection 和 selected candidate filtering。
4. 在 core 接入 authoritative command tree、typed command model、structured error mapping 和 bounded presentation probe。
5. 删除 `docnav-cli-args`、native string bridge、JSON guess、compatibility path 和对应 accepted warnings。
6. 更新 owner docs/cases，完成子仓库验证、core smoke、workspace full verifier 和 OpenSpec strict validation。

Rollback 通过整体 revert 实现；不提供 runtime feature flag、双跑或旧 parser fallback。

## Verification

- 子仓库：覆盖 short/long、string-looking-JSON、integer、finite number、`SetTrue`、repeated array/object、malformed value、unsupported JSON 和 conflicts；运行独立 metadata/fmt/build/clippy/tests/docs/example。
- Navigation：覆盖 registry conflict、selected/unselected candidates、source priority、typed handoff 和 pre-dispatch failure。
- Core：覆盖 command families、help/version、strict argv、hyphen-leading inline value、structured diagnostics 和 presentation probe。
- Process boundary：用 CLI smoke 锁定 stdout/stderr/exit code 与 `readable-view`、`readable-json`、`protocol-json`。
- Delivery：运行 `bun run verify:docnav-workspace:full`，要求 0 warning、0 failed，并严格校验 OpenSpec。

## Open Questions

无未回答开放问题，可以进入实现前审计。实现阶段若要改变 owner、错误优先级、flag uniqueness 或 presentation probe 范围，必须先更新本 design 和对应 delta spec。

## References

- [Clap `Arg::value_parser`](https://docs.rs/clap/4.6.1/clap/struct.Arg.html#method.value_parser)
- [Clap `value_parser!`](https://docs.rs/clap/4.6.1/clap/macro.value_parser.html)
- [Clap `TypedValueParser`](https://docs.rs/clap/4.6.1/clap/builder/trait.TypedValueParser.html)
- [Clap `Command::try_get_matches_from`](https://docs.rs/clap/4.6.1/clap/struct.Command.html#method.try_get_matches_from)
- [Clap structured error context](https://docs.rs/clap/4.6.1/clap/error/struct.Error.html#method.context)
- [Clap `ArgMatches::value_source`](https://docs.rs/clap/4.6.1/clap/parser/struct.ArgMatches.html#method.value_source)
