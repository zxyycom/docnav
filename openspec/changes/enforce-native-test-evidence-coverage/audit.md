# 原生测试证据完整性阻塞审计

审计日期：2026-07-27

审计基线：`a645ba1575d94374fe49d863225a7bff8a7d3f9f`（`plan：规划原生测试证据完整性迁移`）

审计时工作树：基线提交上无未提交改动
Gate 结论：**Proceed**

本文只保存 `enforce-native-test-evidence-coverage` 的实施输入、当前树观测和 gate
结论，不是长期测试事实或产品契约。归档前的当前行为仍由 v7 skill、
`docs/test-evidence/` 和现有 validator 证明。

## 1. Artifact 一致性

逐项比较 proposal、design、delta spec 和 tasks 后，核心目标、breaking 范围、
非目标、术语和产品边界一致：

| 项目 | 一致结论 |
| --- | --- |
| 目标 | 完整当前树中的 supported runner 原生入口、runner 报告和 machine inventory 必须一一闭合。 |
| 语义分层 | Machine case 只保存可恢复入口事实；长期 owner 判断只进入 Evidence Claim；普通 case 可以没有 Claim。 |
| 完整性 | Git diff、旧 marker、旧账本和人工抽样都不能缩小 required check 的当前树范围。 |
| Breaking 范围 | 删除 v7 一入口一 Markdown 语义、必填 Contract/Proves 和旧查询链，原子切换到 v8。 |
| 非目标 | 不改变 Docnav CLI、adapter、protocol、ref、输出、配置或 canonical release 产品行为；不从 AST、测试名或通过结果自动生成 Claim。 |
| Owner | 项目 wrapper 拥有发现和 runner 归一；通用 skill 拥有 case/Claim 模型、查询契约和 AI 审查流程。 |
| 回滚 | skill、工具、inventory、Claims、validator、文档和旧 v7 数据必须作为一个完整单位恢复。 |

审计发现并已修正一处实施输入假设：固定的上游
`zxyycom/skills` release 只包含 `test-evidence-review` v7，不存在可直接接入的
上游 v8。proposal、design 和 tasks 现已明确：

1. 固定的上游 v7 是 v8 的内容基线，不冒充 v8。
2. Docnav 在项目内拥有并演进 v8。
3. v8 完成后固定自己的完整文件清单和 fingerprint。
4. v7 self-updater 不得继续作为 v8 更新入口，否则它可以把项目 v8 覆盖或
   降级成上游 v7；未来上游同步只能做显式三方审查。

该修正只明确 skill 的维护 owner，没有改变产品责任边界。修正后的 change
通过：

```text
openspec validate enforce-native-test-evidence-coverage \
  --type change --json --strict --no-interactive
```

结果为 1 个 change 通过、0 issue。design 的 Open Questions 仍为零。

## 2. 当前 v7 与入口缺口

### 2.1 Fingerprint 算法

本节目录 fingerprint 统一计算：

```text
sha256(
  concat(
    sort(files).map(
      relativePath + NUL + sha256(fileBytes) + LF
    )
  )
)
```

单文件 fingerprint 为原始 bytes 的 SHA-256。派生 index 不进入权威 source
fingerprint。

### 2.2 v7 目录

现有严格检查通过：

```text
node .codex/skills/test-evidence-review/scripts/test-evidence-catalog.mjs \
  check --root . --json
```

但该命令只证明已登记目录内部自洽，不扫描源码或 runner。

| 观测 | 当前值 |
| --- | ---: |
| Case Markdown | 431 |
| Topic | 11 |
| Entry 行 | 431 |
| 唯一 Entry | 431 |
| Rust ledger entry | 310 |
| Bun ledger entry | 104 |
| Core smoke ledger entry | 17 |
| 通用 Contract 模板 | 426 |
| 通用 Proves 模板 | 396 |
| Contract/Proves 同时为通用模板 | 396 |
| 至少一项不是通用模板的旧 case 候选 | 35 |

Topic 分布：

| Topic | Case |
| --- | ---: |
| `adapter-contracts` | 6 |
| `core-cli` | 110 |
| `diagnostics` | 4 |
| `markdown-adapter` | 50 |
| `navigation` | 43 |
| `output-rendering` | 34 |
| `protocol` | 29 |
| `quality-tooling` | 33 |
| `release` | 26 |
| `shared-foundations` | 51 |
| `test-infrastructure` | 45 |

内容固定值：

| 输入 | Fingerprint |
| --- | --- |
| 431 个 case Markdown manifest | `sha256:16323dac67692c02d180b427d64f60842dd727e86281abdf05642c1d78fdcea8` |
| 431 case + `test-evidence-topics.json` source manifest | `sha256:0848219fcbfd69c2a55aae145637e5f5c7b469fea12962689800a43fda43c549` |
| `test-evidence-topics.json` | `sha256:8a5906c3a23a2b89fdc0d53c29240a52cb2b2e60c696d8b5147741decf34835f` |
| 派生 `test-evidence-index.json` | `sha256:395a484f5053e05f1efb23b0d875bb21d50956a779e2f624c42faa055cdbb9b0` |
| 本地 v7 skill 25 文件 manifest | `sha256:b81148de47a5dece145853226c60627b452c54c5f500d42de9543f5dafbb451e` |

模板分类使用精确结构，不用相似度猜测：

```text
Contract:
- `<owner>` 定义或约束“<title>”所涉及的稳定行为边界。

Proves:
- 原生入口 `<entry>` 直接验证“<title>”所描述的结果。
```

因此 396 个双模板不会通过换词迁移成 Claim。其余 35 个旧 case 只是 Claim
审查候选，也不能自动变成 35 个 Claim。

### 2.3 当前完整入口集合

| Runner | 当前 runtime | v7 ledger | 匹配 | 漏项 | 悬空 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rust | 391 | 310 | 310 | 81 | 0 |
| Bun | 111 | 104 | 104 | 7 | 0 |
| Core smoke leaf | 17 | 17 | 17 | 0 | 0 |
| **合计** | **519** | **431** | **431** | **88** | **0** |

因此当前树切换后的 machine inventory 基线应为 519，而不是继续维持 431。
增加的 88 项只增加机器事实，不要求增加 88 篇 Contract/Proves 文章。

相对建 change 时观测：

- 431 case、11 topic、426 个通用 Contract、396 个双模板均未变化。
- Rust 仍是 391 个 runtime、310 个 ledger、81 个漏项。
- 建 change 时引用的“3 个 Bun supporting test”来自 v7 迁移前只覆盖 18 个
  ledger 相关文件的审计；它们是
  `scripts/tools/foundation/test/foundation.test.ts` 中前三个测试。
- 完整当前树有 20 个 Bun test 文件。v7 最终目录从未覆盖
  `scripts/quality/annotate/warnings.test.ts` 和
  `scripts/tools/quality-core/test/quality-core.test.ts`；两文件再贡献 1 + 3
  个漏项，所以当前正确值为 7。不是本 change 之后新增测试造成的漂移，而是
  旧审计范围不完整。

7 个 Bun 漏项：

```text
scripts/quality/annotate/warnings.test.ts > quality warning annotations > keeps accepted warnings in machine records but selects only unaccepted warnings
scripts/tools/foundation/test/foundation.test.ts > script foundation > parses strict positive integers
scripts/tools/foundation/test/foundation.test.ts > script foundation > parses JSON values and normalizes slash paths
scripts/tools/foundation/test/foundation.test.ts > script foundation > detects failed process results
scripts/tools/quality-core/test/quality-core.test.ts > script quality core > classifies files using caller-provided code areas
scripts/tools/quality-core/test/quality-core.test.ts > script quality core > rejects a metrics envelope without metadata
scripts/tools/quality-core/test/quality-core.test.ts > script quality core > generates warning channels from caller-provided thresholds
```

<details>
<summary>81 个 Rust 漏项</summary>

```text
facade::canonical_parameter_set_drives_env_resolution
facade::primary_facade_builds_constrained_canonical_parameters
config::model::tests::defaults_auto_read_preserves_raw_modes
config::model::tests::native_options_config_accepts_generic_raw_map
parameter_catalog::tests::catalog_fields_do_not_enable_an_env_source
parameter_catalog::tests::catalog_fields_preserve_current_locator_type_default_merge_and_range_facts
parameter_catalog::tests::core_catalog_contains_the_auto_read_orchestration_parameter
parameter_catalog::tests::operation_projection_filters_only_by_closed_bindings
project_context::tests::explicit_config_paths_are_resolved_relative_to_invocation_cwd
project_context::tests::explicit_user_config_path_does_not_require_platform_default
project_context::tests::user_config_path_prefers_docnav_config_dir_then_platform_default
project_context::tests::user_config_path_uses_dot_docnav_under_platform_user_root
runtime::tests::linked_adapter::core_linked_markdown_consumes_project_native_max_heading_level
runtime::tests::linked_adapter::core_linked_markdown_delegates_native_option_range_to_adapter
runtime::tests::linked_adapter::core_linked_markdown_reports_project_native_option_source
runtime::tests::linked_adapter::core_linked_markdown_reports_user_native_option_source
runtime::tests::linked_adapter::linked_adapter_uses_absolute_document_path_from_project_subdir
runtime::tests::linked_adapter::missing_adapter_routing_precedes_invalid_native_option
code::rules::tests::diagnostic_rule_tables_follow_enum_order
tests::details::detail_rule_rejects_one_missing_and_extra_field
tests::details::detail_rule_validates_each_supported_field_type_once
tests::details::invalid_request_details_accept_known_optional_context_fields
auto_read::tests::invalid_composed_response_falls_back_to_the_original_base
auto_read::tests::invalid_nested_success_is_not_accepted_as_a_read_result
auto_read::tests::unique_ref_ignores_empty_refs_and_uses_string_exact_deduplication
parameters::catalog::tests::associations::catalog_retains_known_adapter_ids_for_config_validation
parameters::catalog::tests::associations::entry_adapter_must_be_known
parameters::catalog::tests::associations::field_and_entry_associations_are_total_and_unique
parameters::catalog::tests::associations::field_set_identity_and_locator_errors_are_preserved
parameters::catalog::tests::associations::standalone_config_validation_consumes_catalog_scalar_fields
parameters::catalog::tests::bindings::binding_value_kind_must_match_the_field_definition
parameters::catalog::tests::bindings::every_entry_requires_one_unambiguous_binding_per_operation
parameters::catalog::tests::bindings::navigation_and_core_only_bindings_enforce_their_target_value_kinds
parameters::catalog::tests::bindings::standard_input_targets_are_unique_for_overlapping_adapter_scopes
parameters::catalog::tests::projection::operation_applicability_is_derived_from_closed_bindings
parameters::catalog::tests::projection::operation_projection_borrows_the_canonical_field_facts
parameters::catalog::tests::projection::selected_operation_projection_includes_common_and_exact_adapter_fields_only
parameters::fields::definitions::tests::common_named_fields_author_cli_processing_metadata
parameters::tests::auto_read_replace_trace_keeps_selected_overridden_and_builtin_provenance
tests::navigation::auto_read_composition::find::find_eligibility_keeps_empty_or_multiple_ref_base_results
tests::navigation::auto_read_composition::find::repeated_find_refs_dispatch_one_read_on_later_pages_with_continuation
tests::navigation::auto_read_composition::outline::adapter_base_result_with_auto_read_is_rejected_before_composition
tests::navigation::auto_read_composition::outline::disabled_mode_does_not_dispatch_nested_read
tests::navigation::auto_read_composition::outline::nested_read_diagnostic_silently_keeps_the_validated_base_result
tests::navigation::auto_read_composition::outline::nested_read_reuses_the_effective_limit_when_pagination_is_disabled
tests::navigation::auto_read_composition::outline::outline_eligibility_keeps_non_unique_or_unstructured_base_results
tests::navigation::auto_read_composition::outline::unique_outline_ref_composes_read_with_the_selected_document_context
tests::navigation::auto_read_config::auto_read_mode_resolves_with_cli_project_user_and_builtin_precedence
tests::navigation::auto_read_config::invalid_auto_read_cli_value_reports_the_canonical_flag_and_tokens
tests::navigation::auto_read_config::invalid_auto_read_config_is_attributed_to_its_source
tests::navigation::auto_read_config::read_and_info_recognize_valid_config_without_projecting_auto_read
tests::navigation::native_options::adapter_scopes::navigation_accepts_config_option_applicable_to_operation
tests::navigation::native_options::adapter_scopes::navigation_does_not_forward_other_known_adapter_namespace
tests::navigation::native_options::adapter_scopes::navigation_keeps_same_option_key_distinct_by_adapter_namespace
tests::navigation::native_options::adapter_scopes::navigation_rejects_option_missing_from_core_catalog
tests::navigation::native_options::defaults::navigation_accepts_max_heading_level_range_boundaries
tests::navigation::native_options::defaults::navigation_includes_adapter_native_option_default
tests::navigation::native_options::defaults::optional_non_json_config_null_suppresses_default_projections
tests::protocol::protocol_dispatch_rejects_request_and_standard_input_operation_mismatch
tests::protocol::protocol_request_maps_core_inputs_to_operation_arguments
tests::protocol::protocol_request_maps_read_and_find_operation_shapes
tests::protocol::protocol_request_rejects_missing_read_ref
tests::protocol::response_validation_failure_carries_result_validation_layer
tests::auto_read::built_in_renderer_maps_find_auto_read_response
tests::auto_read::protocol_json_and_readable_view_share_outline_auto_read_facts
tests::options::options_preserve_the_plain_json_object_wire_shape
conformance_01_no_block_outline
conformance_04_single_block
conformance_07_chinese
conformance_10_crlf_payload
conformance_11_no_trailing_newline
conformance_12_block_marker_in_body
conformance_14_readable_error
conformance_15_error_guidance_array
conformance_16_undeclared_extension_fields
conformance_17_order_independent_assertions
conformance_18_renderer_failure_missing_pointer
conformance_19_renderer_failure_non_string
conformance_20_outline_unstructured_content_block
conformance_21_outline_auto_read_nested_content_block
processing_id_has_no_unchecked_from_conversion
```

</details>

## 3. Skill 与 CLI 固定输入

### 3.1 上游 release

两个 skill 都固定到：

| 字段 | 值 |
| --- | --- |
| Repository | `https://github.com/zxyycom/skills.git` |
| Commit | `090c1bc4db13b23e2241530f610363f87aee53a1` |
| Commit time | `2026-07-27T14:24:37+08:00` |
| Exact release | `20260727T071732Z-1b2af5d1c8f9` |
| Release time | `2026-07-27T07:17:37Z` |
| Moving tag at audit time | `skills-latest` 指向同一 commit；实现不依赖该 moving tag |
| Release manifest | `sha256:9b400f39a3d73602eae08f02f25f8c907bb0c7f7158fa397c40d4b974194ddce` |

`ast-grep.zip`：

- Release digest：
  `sha256:40ff4dbb246353d0fbccc0f710c34063cb2db42b2516e5b9b451475d9b6fee96`
- 解包后 6 文件 manifest：
  `sha256:8957af003ca667e987db9e42e7f76e8f6813a0fe9f7e87a09ce4454424de0d44`
- Release manifest version：1

完整文件：

| SHA-256 | Path |
| --- | --- |
| `609710f7ccc1577b39fe1772732ad20767805e882f7575633d0a1aa599feff57` | `SKILL.md` |
| `7686caaa1a1f637b1e8d9b407fdcdd1f9a7c4810a56180407ac2cc3ff658893a` | `agents/openai.yaml` |
| `b2a823d6c6da5157d121490051eaf04b8b3244a4e11411d6dc0fff93cfb3a6e2` | `references/rules-and-recipes.md` |
| `382413dc7791ac72a84788fa24c30f79746674e48d0dbf93a80c92ba904f2fd1` | `scripts/update-skill.d.mts` |
| `9c9e71a6ddccbe8920376262210428dc0428235fbe65bc5bfcfa865aae7b2d38` | `scripts/update-skill.mjs` |
| `f7ac4b1cd618e903cddb153c7c065bc2f0d03eb48a969fea6e98a4b7e1d667a9` | `scripts/update-skill.mjs.map` |

`test-evidence-review.zip`：

- Release digest：
  `sha256:78f15cd159cfdf410cf91cc5b883343f3a61db314c529a80986f1710cf65f61a`
- 解包后 25 文件 manifest：
  `sha256:b81148de47a5dece145853226c60627b452c54c5f500d42de9543f5dafbb451e`
- Release manifest version：7
- 本地 `.codex/skills/test-evidence-review/` 与该 25 文件分发逐字节一致。
- 上游 HEAD 和 exact release 均没有 v8。

25 个 v7 基线文件：

```text
SKILL.md
agents/openai.yaml
references/catalog-contract.md
references/migrate-from-verification-implementation-review.md
references/schemas/test-evidence-case-show-result.schema.json
references/schemas/test-evidence-index-sync-result.schema.json
references/schemas/test-evidence-query-result.schema.json
references/schemas/test-evidence-report.schema.json
references/schemas/test-evidence-state-index.schema.json
references/schemas/test-evidence-topic-catalog.schema.json
references/schemas/test-evidence-topics-result.schema.json
references/upgrade-from-single-file-catalog.md
scripts/test-evidence-case-show-result.types.d.mts
scripts/test-evidence-catalog.d.mts
scripts/test-evidence-catalog.mjs
scripts/test-evidence-catalog.mjs.map
scripts/test-evidence-index-sync-result.types.d.mts
scripts/test-evidence-query-result.types.d.mts
scripts/test-evidence-report.types.d.mts
scripts/test-evidence-state-index.types.d.mts
scripts/test-evidence-topic-catalog.types.d.mts
scripts/test-evidence-topics-result.types.d.mts
scripts/update-skill.d.mts
scripts/update-skill.mjs
scripts/update-skill.mjs.map
```

Ownership 与更新方式：

- `.codex/skills/ast-grep/` 保持该 release 的完整上游分发；它的 updater 只供未来
  显式更新审查，required check 不运行 updater。
- `.codex/skills/test-evidence-review/` v8 由 Docnav 拥有。v7 updater 明确绑定
  `zxyycom/skills` 的 `test-evidence-review.zip`，并且选择旧 release 后仍可进入
  replace 流程，因此 v8 必须移除或替换该 updater。
- v8 完成时必须把最终文件清单和 fingerprint 写入 `verification.md`；此前只把
  上述 v7 manifest 当作可恢复基线。

许可证事实：

- `zxyycom/skills` 根 `package.json` 没有 `license` 字段，仓库根也没有
  `LICENSE` / `COPYING` / `NOTICE`。不能推断公共再分发许可证。
- 用户在本任务中把该仓库明确称为自己的 skill 仓库，并明确要求接入这些
  skill；本次复制以该任务范围内的 owner 授权为依据。若未来把 skill 内容作为
  独立第三方制品再发布，必须另行补充许可证声明。
- 下面的 `@ast-grep/cli` npm package 自身声明 MIT；这不替代 skill 仓库内容的
  许可证。

### 3.2 开发期 ast-grep CLI

| 字段 | 固定值 |
| --- | --- |
| Package | `@ast-grep/cli` |
| Version | `0.45.0`，精确版本，不使用 range |
| Repository | `https://github.com/ast-grep/ast-grep` |
| License | MIT |
| Package integrity | `sha512-OQ4pcktMtg1hcQat/iCpX9r8HJ7mU/2SZVoGHA9id2gEfosvDw5m5RINQXsSRZXQW8bl45FW6FhdK0O2FiKjsw==` |
| Linux x64 artifact integrity | `sha512-rAMZJzAiBuXMViuJgdPeMZXI9HnqwMCh3ybIoj8dfWBPsAywKgU8vyH4kd/R5fFr/oB4lKVhTJ2/mEBsOQTHaQ==` |
| Package manager | `pnpm@11.1.3` |
| Runtime pins | Node 24、Bun 1.3.14、pnpm 11.1.3，由 `mise.toml` / `mise.lock` 拥有 |
| Lock owner | 根 `package.json`、`pnpm-lock.yaml`、`pnpm-workspace.yaml` |

`@ast-grep/cli` 的 postinstall 必须被 pnpm 11 显式允许。实现时只在现有
`pnpm-workspace.yaml` 的 `allowBuilds` 中加入：

```yaml
'@ast-grep/cli': true
```

审计在仓库外临时目录完成以下证明：

1. 用 pnpm 11.1.3 在线 bootstrap 精确版本并生成 lockfile。
2. 删除 `node_modules`，保留 pnpm store 和同一 lockfile。
3. `pnpm install --offline` 成功。
4. `pnpm exec ast-grep --version` 返回 `ast-grep 0.45.0`。

这证明“依赖已经准备后”的干净离线安装和 required 调用不访问网络；不声称一个
没有 pnpm store/cache 的全新机器可以凭空离线获取 tarball。常规依赖 bootstrap
仍由 pnpm、lockfile 和 CI cache/registry 负责。

项目 wrapper 必须通过仓库脚本解析本地 executable，不依赖个人 PATH，不运行
skill updater。当前 canonical release 配置只打包 `docnav` core binary，不复制
`node_modules`、开发规则或 skill；2.5 和 6.5 仍必须增加并运行 file-set 断言。

## 4. Supported runner profile

Profile v1 固定三类入口：

1. Cargo workspace 的 lib/bin/integration test harness。
2. 下列 20 个 Bun `.test.ts` surface。
3. `test/docnav-core-smoke.ts` 的 9 个 root 经真实 `prepareSmokeTasks` 展开的
   17 个 leaf。

lint、typecheck、schema、生成物检查、CI job、fixture、helper、assertion 和 task
内部步骤不形成 case。

### 4.1 Rust

固定枚举路径：

```text
mise exec -- cargo test --locked --workspace --no-run --message-format=json
<每个 compiler-artifact executable> --list --format terse
```

`packageId + target kind + target name` 是稳定 target 身份；带 hash 的 executable
路径只用于本次运行，不进入 `entryKey`。原生 selector 是 libtest 的完整名称，
单项重放为：

```text
<本次 compiler-artifact executable> --exact <selector> --format terse
```

审计实际用
`tests::compact_json_writes_value_and_newline` 证明只运行 1 个测试、其余 3 个被
过滤。

20 个 test executable target：

| Package | Kind | Target | Test |
| --- | --- | --- | ---: |
| `cli-config-resolution-serde` | lib | `cli_config_resolution_serde` | 4 |
| `cli-config-resolution` | lib | `cli_config_resolution` | 0 |
| `cli-config-resolution` | test | `canonical_core` | 20 |
| `docnav-adapter-contracts` | lib | `docnav_adapter_contracts` | 6 |
| `docnav-cli-args` | lib | `docnav_cli_args` | 7 |
| `docnav-diagnostics` | lib | `docnav_diagnostics` | 8 |
| `docnav-json-io` | lib | `docnav_json_io` | 4 |
| `docnav-markdown` | lib | `docnav_markdown` | 21 |
| `docnav-markdown` | test | `adapter` | 29 |
| `docnav-navigation` | lib | `docnav_navigation` | 84 |
| `docnav-output` | lib | `docnav_output` | 13 |
| `docnav-protocol` | lib | `docnav_protocol` | 30 |
| `docnav-readable` | lib | `docnav_readable` | 23 |
| `docnav-readable` | test | `conformance_tests` | 14 |
| `docnav-text-cost` | lib | `docnav_text_cost` | 3 |
| `docnav-typed-fields` | lib | `docnav_typed_fields` | 9 |
| `docnav-typed-fields` | test | `canonical_parameters` | 6 |
| `docnav-typed-fields` | test | `processing_id_compile` | 1 |
| `docnav` | bin | `docnav` | 0 |
| `docnav` | lib | `docnav` | 109 |
| **合计** |  |  | **391** |

`cargo test --locked --workspace -- --list` 另外经过 14 个 doc-test report block；
当前均为 0。Profile 不静默忽略未来新增的 doctest：一旦 runtime list 非零而没有
受支持静态声明，就必须以 `runtime-only` /
`unsupported-entry-shape` 阻断，直到扩展 profile 和规则。

当前 391 个 runtime leaf 名称无重复；静态 `#[test]` 规则也得到 391 个 function。
v1 可以用 target + 完整 selector 归一。未来出现歧义名称时必须失败，不回退到
“任选一个”。

### 4.2 Bun

固定 surface：

```text
scripts/docnav-workspace/verify.test.ts
scripts/docs/test-evidence-validation.test.ts
scripts/quality/annotate/warnings.test.ts
scripts/quality/args.test.ts
scripts/quality/config.test.ts
scripts/tools/foundation/test/foundation.test.ts
scripts/tools/parallel-task-runner/test/index.test.ts
scripts/tools/quality-core/src/input/files.test.ts
scripts/tools/quality-core/src/measurement/cache.test.ts
scripts/tools/quality-core/src/measurement/scanners.test.ts
scripts/tools/quality-core/src/measurement/scanners/jscpd/area-scans.test.ts
scripts/tools/quality-core/src/output/report/markdown-report.test.ts
scripts/tools/quality-core/src/output/warnings/generator.test.ts
scripts/tools/quality-core/test/quality-core.test.ts
scripts/tools/release-package/args.test.ts
scripts/tools/release-package/candidate.test.ts
scripts/tools/release-package/public.test.ts
scripts/tools/release-package/workflow.test.ts
test/smoke/core/fixtures/project.test.ts
test/tools/smoke-harness.test.ts
```

固定 report：

```text
mise exec -- bun test <以上排序后的 20 个文件> \
  --reporter=junit --reporter-outfile=<owned-temp-file>
```

JUnit 当前报告 111 tests、0 failure。静态身份由 `sourcePath + suite chain +
literal test name` 组成；source line/range 用于本次静态/runtime 关联和诊断，不单独
充当长期 selector。单项选择使用文件加转义并锚定的完整名称：

```text
mise exec -- bun test <sourcePath> --test-name-pattern '^<escaped full name>$'
```

审计对
`quality warning annotations > keeps accepted warnings in machine records but selects only unaccepted warnings`
实际得到 1 pass、0 fail、1 test。

当前直接形态为：

- 二参数 `test("literal", fn)` / `it("literal", fn)`。
- 一个三参数 `it("literal", options, fn)`。
- 无 `.each`、import alias 或动态名称。

runner 非零、JUnit failure 或 malformed report 属于 runner failure，不得改写成
`missing-case`。静态/runtime 集合差异只在 runner report 合法后判断。

### 4.3 Core smoke

固定 root 是 `test/docnav-core-smoke.ts` 传给 `runSmokeTasks` 的 9 项：

```text
real-markdown-link-chain
real-markdown-ref-error
auto-read
document-output-boundary
adapter-selection
cli-argument-failure
config-context
registry-contract-failures
tool-commands
```

固定 list/report adapter 必须导入真实 factory 并调用 `prepareSmokeTasks`，不得
复制一份 ID 清单。当前展开 17 个唯一 leaf：

```text
CORE-LINK-001
CORE-MD-OPTIONS-001
CORE-MD-DOCHEAD-001
CORE-REF-001
CORE-AUTO-READ-001
CORE-OUTPUT-001
CORE-SELECT-001
CORE-ARGS-001
CORE-CONFIG-001
CORE-CONFIG-002
CORE-CONFIG-003
CORE-CONFIG-004
CORE-CONFIG-PATH-001
CORE-FAIL-001
CORE-SOURCE-001
CORE-TOOLS-001
CORE-ADAPTER-MGMT-001
```

聚合 root 和含 `tasks` 的 group 不形成 case；最终 leaf object 才形成 case。
`entryKey` 使用 profile id、root/report id 和 leaf id，项目 wrapper 提供按 leaf id
的精确 list/selection。factory import、task 展开、重复 ID 或 report 失败保持独立
runner/smoke 诊断。

## 5. 仓库外原型

原型使用固定的 `@ast-grep/cli@0.45.0`，所有 fixture 和 rule 都位于仓库外临时
目录，没有修改产品源码。

### 5.1 当前树闭合

| 原型 | 静态 | Runtime | 匹配 | 仅静态 | 仅 Runtime |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rust `#[test]` function | 391 | 391 | 391 | 0 | 0 |
| Bun literal direct registration，含二/三参数 | 111 | 111 JUnit | 111 | 0 | 0 |
| Smoke literal leaf object | 17 | 17 expanded leaf | 17 | 0 | 0 |

Rust rule 以 `function_item` 紧邻精确 `#[test]` attribute 为条件；普通 helper 不
匹配，`#[ignore]` 加 `#[test]` 仍匹配。Bun 规则只接受 literal name 和已声明的
二/三参数形态。Smoke 规则只接受具有 literal `id`、literal `label` 和 `run` 的
leaf object，嵌套 group 自身不匹配，group 内 leaf 匹配。

### 5.2 Unsupported 形态

最小正反例证明以下形态不会静默进入正常 case：

| 形态 | 静态结果 |
| --- | --- |
| Rust `#[tokio::test] async fn` 等 test attribute macro | `unsupported-entry-shape` 候选 |
| Rust `mod tests` 顶层 `generated_cases!()` | `unsupported-entry-shape` 候选；普通函数体内 assertion macro 不匹配 |
| Bun `import { test as spec }` | alias diagnostic |
| Bun `test.each(...)` / `it.each(...)` | parameterized diagnostic |
| Bun `test(dynamicName, fn)` | dynamic-name diagnostic |
| Bun wrapper 内 `test(name, run)` | dynamic registration diagnostic |
| Smoke factory `return [makeTask(dynamicId)]` | dynamic-task diagnostic |
| Smoke `tasks: [...]` group | 合法组合容器，不是 leaf case |

对当前树运行这些 unsupported rules 得到 0 个 Rust test attribute macro、0 个
Rust test-module 顶层生成 macro、0 个 Bun alias/parameterized/dynamic registration
和 0 个 smoke dynamic returned task。

宏展开、任意 wrapper 和任意动态 task 无法只靠 AST 证明。集合闭合为此保留第二道
门：任何 runner entry 没有受支持静态声明时，稳定报告
`runtime-only`，并以 `unsupported-entry-shape` 阻断；任何静态声明没有 runtime
entry 时报告 `static-only`。因此未知形态可以导致明确失败，但不能静默漏登。

## 6. 当前 owner 与 active change 审计

当前源码没有 `@case`、`test-evidence:`、`test_case_id` 或 `case-id:` marker。
v7 活跃 owner/caller 只有：

| 当前 surface | 切换 owner |
| --- | --- |
| `.codex/skills/test-evidence-review/**` | project-owned v8 通用模型与模块；移除 v7 self-updater 路径 |
| `docs/test-evidence/<topic>/*.md` | 机器 Entry 迁入 inventory；非模板内容只进入 Claim 审查；旧 Markdown 在原子切换时删除 |
| `test-evidence-topics.json` / `test-evidence-index.json` | Claim topic 权威源和统一可重建 query index |
| `scripts/docs/validate.ts` | 只调用项目 v8 wrapper |
| `scripts/docs/test-evidence-validation.test.ts` | 替换为 discovery、closure、Claim、index 集成证明 |
| `docs/navigation.md`、`docs/testing.md`、`docs/testing/case-maintenance.md`、`docs/testing/coverage.md`、`AGENTS.md` | 同步 owner、读取时机、状态语义和命令 |
| `package.json`、`pnpm-lock.yaml`、`pnpm-workspace.yaml` | 固定 CLI、项目脚本和 build allowlist |
| workspace verifier | 保留 `validate:docs -- cases` 入口，切换其内部 owner 并补充回归测试 |

全部 active changes 中，只有 `add-ast-grep-code-adapter` 与 ast-grep 有交叉：

- 该 change 固定产品期进程内 Rust crates，并明确禁止发现或启动外部
  `ast-grep` executable。
- 本 change 固定开发期 npm CLI，只由验证 wrapper 使用。
- 两者版本不要求相同，也不继承彼此 dependency、release 或 protocol 责任。
- 切换时必须在 `add-ast-grep-code-adapter` 留下该开发/产品边界说明；其产品 spec
  和 release file-set 要求不修改。

其它 6 个 active changes 没有 v7 case、marker、topic/index 或 ast-grep CLI
依赖，不需要协调修改。所有 archived changes 保持历史原文。

本 change 不触及 Docnav CLI、adapter、protocol、ref、输出、schema、example 或
产品配置。canonical release 仍只包含 `docnav` executable；开发 CLI、rules、
inventory tooling 和 skills 不进入产品 file set。

## 7. 单轨 changeset、回滚和验收

### 7.1 原子切换单位

实现可以分小步 shadow 开发，但 required owner 只能在同一最终 changeset 中切换：

1. `.codex/skills/ast-grep/**` 完整上游分发。
2. `.codex/skills/test-evidence-review/**` project-owned v8。
3. 根 package/pnpm 声明和本地 CLI 调用脚本。
4. `scripts/test-evidence/**` 的 profile、rules、rule tests、runner adapters、
   closure、inventory/Claim/query wrapper 和测试。
5. `docs/test-evidence/**` 的 Claim topic、Claims、machine inventory 和 query index。
6. docs validator、workspace verifier、稳定文档、AGENTS 和相关 active change。
7. 删除 431 个 v7 case Markdown、v7 topic/index 形态和旧 validator 测试入口。
8. 完整 `migration-map.json` 和最终 `verification.md`。

不得保留 v7/v8 双读、第二份 runner profile、第二个 Claim owner 或 marker fallback。

### 7.2 回滚单位

完整回滚必须同时：

1. 按本审计 fingerprint 恢复 v7 25 文件 skill。
2. 恢复 431 case Markdown、11 topic 和派生 index。
3. 恢复当前 `scripts/docs/validate.ts`、catalog integration tests、稳定文档、
   AGENTS 和 active change 文本。
4. 删除 project-owned v8、ast-grep skill、npm CLI 声明、rules、wrapper、
   inventory、Claims 和 v8 index。
5. 重新运行 v7 `sync-index --write`、`check`、docs validation 和 workspace
   verification。

`migration-map.json` 必须覆盖全部 431 个旧 ID，才能证明该恢复不是“只恢复数据”
或“只切 validator”。Git 层面可回滚整个最终 cutover changeset；不得用部分文件
checkout 制造混合状态。

### 7.3 性能基线

同一已构建工作树的单次 warm 观测：

| 操作 | 耗时 |
| --- | ---: |
| v7 catalog check | 1276 ms |
| Rust ast-grep scan，391 match | 74 ms |
| Bun ast-grep scan，111 match | 40 ms |
| Smoke ast-grep scan，17 match | 20 ms |
| Cargo workspace list，391 test | 1821 ms |
| Bun 20 文件 JUnit report，111 test | 2996 ms |
| Smoke factory expand/list，17 leaf | 313 ms |
| 新闭合链关键步骤简单合计 | 5264 ms |

另一次 warm Cargo `--no-run --message-format=json` 为 666 ms。墙钟时间受机器和
编译 cache 影响，不作为跨机器硬编码测试值；实现后在同一环境超过约 10.5 秒
（本地基线 2 倍）必须调查并记录原因。required check 不得为追求耗时而跳过 Bun
report 或缩小当前树。

### 7.4 固定验收入口

实现必须建立并通过：

```text
mise exec -- pnpm install --offline --frozen-lockfile
bun run test:test-evidence-rules
bun run test:test-evidence
bun run validate:docs -- cases
bun run test:validators
bun run test:release-package-scripts
mise exec -- cargo test --locked --workspace
mise exec -- bun test <profile 中排序后的 20 个 Bun test 文件>
mise exec -- bun test/docnav-core-smoke.ts
openspec validate enforce-native-test-evidence-coverage \
  --type change --json --strict --no-interactive
bun run verify:docnav-workspace
git diff --check
```

其中离线 install 证明要求 pnpm store/cache 已由 bootstrap 准备；最终交付还必须
保存 rule mutation、static/runtime/inventory 等价、Claim/index、release file-set
和 431 项 migration map 的机器结果。

### 7.5 停工条件

出现以下任一条件，停止 cutover 并回到 change 设计层：

1. 固定 release、v7 基线或 ast-grep CLI integrity 无法复现。
2. 当前或新增 runner entry 既不能归一，也不能稳定形成
   `unsupported-entry-shape` 阻断。
3. 任一 profile 需要 Git diff、旧 marker、人工 allowlist 或个人 PATH 才能闭合。
4. required check 在依赖准备后仍访问网络或运行 updater。
5. external ast-grep executable、规则或 Node dependency 进入 canonical release
   或产品运行时。
6. migration map 不是 431 项完全函数，或模板叙述被自动改写成 Claim。
7. v7 与 v8 必须双读才能通过验证。
8. 回滚不能同时恢复 skill、数据、validator、文档和 active change。
9. 实现需要改变 Docnav CLI、adapter、protocol、ref、输出或产品 release 行为。

## 8. Gate

阻塞审计结论：

- **输入已固定**：基线 commit、v7 数据、两个 skill release、CLI/lock owner、
  runner profile 和当前 519 个入口均有可重放观测。
- **原型可行**：Rust 391/391、Bun 111/111、smoke 17/17 静态/runtime 闭合；
  宏、alias、wrapper、参数化、动态注册和 task 组合都有明确支持或阻断路径。
- **冲突可处理**：唯一 active change 交叉是产品期 Rust crates 与开发期 CLI，
  责任和 release 边界可分离。
- **可原子回滚**：v7 完整 fingerprint、431 个旧 case、validator 和文档恢复单位
  已固定。

**Proceed**：允许从任务 2.1 开始 shadow 实现；在任务 5.18 的单轨切换和全部
验收完成前，v7 仍是 required 当前行为。
