本 tasks 定义 `adopt-strict-input-boundaries` 的实现分派方式：用一个已完成的启动清单和多个并行 owner 轨道推进 Docnav strict public input boundary。

## Parallelization Contract

`## 1. Implementation Kickoff` 记录已完成的启动条件。Track A-E 按依赖并行推进；Track A 是与 active changes 重叠的实现和测试工作的前置输入，Track B 是 schema、代码和测试字段决策的 owner 输入，`## 7. Integration Verification` 只在相关轨道完成后执行。

每个并行任务必须有明确的 owner、输入、写入范围、输出和依赖。同一文件由一个 worker 拥有；需要跨轨道合并时，由先完成者在 `coordination_items` 中声明后续合并要求。

每个 worker 交付时必须使用这个 JSON 形状：

```json
{
  "track": "<track id>",
  "changed": ["<file paths>"],
  "validated": ["<commands or checks>"],
  "blocked": ["<blockers or empty>"],
  "coordination_items": ["<cross-track merge notes or empty>"]
}
```

公共契约以本 change 的 proposal、design 和 spec deltas 为输入；长期 owner docs 是实现后同步目标。`DiagnosticRecord`、protocol/readable failure projection、success payload 和 candidate failure list 的最终字段形状以 diagnostics/output/protocol/adapter-selection owner 决策为准；测试断言以 owner docs、schema/examples 和实现行为三者一致为准。

## 1. Implementation Kickoff

负责人：main integrator。

输出：已确认的实现分派清单；后续进入主 docs、schema、examples、代码或测试执行。

- [x] 1.1 自洽确认：proposal、design、specs 和 tasks 都围绕 strict public input boundaries、single primary `DiagnosticRecord`、success payload shape、owner-scoped native options 和 retained clap parser/help。
- [x] 1.2 范围确认：本 change 的提案材料集中在 `openspec/changes/adopt-strict-input-boundaries/` 下，后续实现按 owner 轨道写入主 docs、schema/examples、代码和测试。
- [x] 1.3 active change 协调范围确认：必须协调的 OpenSpec artifacts 至少覆盖 `replace-clap-with-bpaf-frontend`、`separate-entry-pipeline-from-parameter-resolution`、`implement-docnav-mcp-bridge`、`outline-unstructured-full-read`、`enable-local-core-adapter-service-mode` 和 `markdown-document-head-outline-mode`；Track A 同时扫描其它 active changes 中的 diagnostics、protocol/readable output、config、native option、CLI parser/help 和 adapter selection 语言。
- [x] 1.4 docs-first owner 范围确认：按 `docs/navigation.md` owner 表归并主 docs、schema/examples 和 testing docs 中需要更新的 strict input、primary `DiagnosticRecord`、native options、internal discovery 和 output projection 条目。
- [x] 1.5 public/internal 边界确认：自动 adapter discovery、Markdown `doc:full` navigation fallback、quality scan report、verifier status 和 tooling report 均保留各自 owner 归属，并与 invalid public input failure 区分。
- [x] 1.6 分派确认：Track A-E 的可并行文件范围、依赖顺序和最终集成检查点已确定。

## 2. Track A: Active OpenSpec Coordination

负责人：OpenSpec coordination。

输入：1.x 启动确认结果、本 change 的 proposal/design/spec deltas。

写入范围：`openspec/changes/replace-clap-with-bpaf-frontend/`, `openspec/changes/separate-entry-pipeline-from-parameter-resolution/`, `openspec/changes/implement-docnav-mcp-bridge/`, `openspec/changes/outline-unstructured-full-read/`, `openspec/changes/enable-local-core-adapter-service-mode/`, `openspec/changes/markdown-document-head-outline-mode/`，以及 task 1.3 记录的其它 active change 目录。

输出：task 1.3 确认的每个 active change 的协调说明、改动文件、OpenSpec strict validation 结果和剩余阻塞点。

- [x] 2.1 更新 `replace-clap-with-bpaf-frontend`，使 active artifacts 在 strict direct CLI contract 下保留 `clap` parser/help 决策。
- [x] 2.2 更新 `separate-entry-pipeline-from-parameter-resolution`，使 source resolution 使用 explicit adapter native option sources，并在 owner 边界处理 unmapped public input。
- [x] 2.3 更新 `implement-docnav-mcp-bridge`，使 MCP 映射 successful payloads 和 primary `DiagnosticRecord` failures。
- [x] 2.4 更新 `outline-unstructured-full-read`，使 `doc:full` 保持 Markdown navigation behavior，readable success output 保持 documented payload contract。
- [x] 2.5 更新 `enable-local-core-adapter-service-mode`，使 service fallback events 表达为 internal fast-path diagnostics 或 owner-scoped status，document success output 保持 documented payload contract。
- [x] 2.6 更新 `markdown-document-head-outline-mode`，使新增 Markdown adapter-owned options 通过 owner-scoped native option sources 声明、校验或拒绝。
- [x] 2.7 更新 task 1.3 识别的其它 active changes，使其使用 strict input、primary `DiagnosticRecord`、output projection 和 owner-scoped option rules。

验证：对本轨道触及的每个 active change 运行 `openspec validate <change> --type change --strict --no-interactive`。

## 3. Track B: Owner Docs

负责人：`docs/navigation.md` 中的 docs owners。

输入：1.x owner mapping、存在 active change 重叠时的 Track A coordination notes，以及本 change design。

写入范围：只更新 main docs；同一文件只由一个 owner 负责。

输出：updated owner docs，以及从既有行为语言到当前 strict contract language 的简短映射。

- [x] 3.1 Navigation and architecture owner: update `docs/navigation.md` and `docs/architecture.md` so owner tables and architecture flow describe strict public input validation, internal adapter discovery failure lists and primary `DiagnosticRecord` ownership.
- [x] 3.2 CLI owner: update `docs/cli.md` so core CLI uses strict `clap` parser/mapper semantics, fails unknown/extra/operation-inapplicable argv before execution and routes invalid input to primary `DiagnosticRecord` projection.
- [x] 3.3 Parameter and adapter owner: update `docs/navigation-input-resolution.md`, `docs/adapter-contract.md`, `docs/protocol.md` and `docs/adapters/markdown.md` so unmapped public input fails by default, config absence/invalid states are distinct, Markdown config-source failures use blocking diagnostics and adapter native options are explicit owner-scoped sources.
- [x] 3.4 Ref owner: update `docs/ref-contract.md` so explicit ref input is strict at the entry boundary while ref generation and semantic interpretation remain adapter-owned.
- [x] 3.5 Diagnostics and output owner: update `docs/diagnostics.md` and `docs/output.md` so public failures use one primary `DiagnosticRecord`, structured readable errors are written to the owning structured output channel and document success output follows the owning success payload schema.
- [x] 3.6 Testing owner: update `docs/testing.md`, `docs/testing/cases.md`, `docs/testing/case-maintenance.md` and `docs/testing/coverage.md` so coverage targets strict failure, primary `DiagnosticRecord` projection and success-output shape.

验证：docs 保持 Current/Planned 状态清晰；只有存在实现证据时才把目标行为标为已实现。

## 4. Track C: Schema, Examples and Fixtures

负责人：schema/examples/fixtures。

输入：3.5 diagnostics/output decisions、3.3 native option/config decisions、3.4 ref decisions，以及 protocol/readable owner docs。

写入范围：`docs/schemas/`, `docs/examples/`, smoke fixtures and schema index docs。

输出：schema/example/fixture diff，以及 success payloads 和 failure diagnostics 的 validation results。

- [x] 4.1 Update readable schemas, including `readable-common.schema.json`, `readable-outline.schema.json`, `readable-read.schema.json`, `readable-find.schema.json`, `readable-info.schema.json` and `readable-error.schema.json`, so successful document payloads follow the documented success shape and readable failure payloads project the primary `DiagnosticRecord`.
- [x] 4.2 Update `protocol-response.schema.json` so protocol failure envelopes validate primary `DiagnosticRecord` projection, candidate failure lists and strict input/config/ref failure details.
- [x] 4.3 Update `protocol-request.schema.json` and protocol request examples where needed so strict protocol input validation has schema/example coverage.
- [x] 4.4 Update `docs/schemas/json-schema.md` so schema ownership points to primary `DiagnosticRecord` failure projection and success payload field ownership.
- [x] 4.5 Add or update protocol/readable error examples for unknown argv, extra positional input, operation-inapplicable flag, explicit adapter failure, explicit config failure, explicit ref failure and unknown config field.
- [x] 4.6 Update adapter contract examples/proofs so invalid core CLI/protocol/config input fails before linked adapter execution, and valid inputs share the document operation semantic pipeline through linked handler dispatch.
- [x] 4.7 Update smoke fixture expectations so `protocol-json` invalid input emits a failure envelope and successful stdout contains only the expected success payload shape.
- [x] 4.8 Update readable-view conformance vectors so they prove success payload/header consistency, readable error projection and block payload restoration through the current readable field set.

验证：schema/examples/fixtures 覆盖新的 public shape；generated readable success fixtures 匹配 documented success payload fields；protocol failure fixtures 覆盖 primary `DiagnosticRecord` projection 和 candidate failure list shape。

## 5. Track D: Rust Implementation Slices

负责人：涉及实现面的 crate owners。

输入：重叠 active changes 的 Track A completed coordination、触及行为的 Track B owner docs、涉及 output/schema 时的 Track C field decisions，以及 `docs/coding-style.md`。

写入范围：implementation crates 和每个 slice 的 focused tests；只有文件 owner 不重叠时才能并行。

输出：per-slice code diff、focused tests、public behavior notes 和 cross-slice API coordination items。

- [x] 5.1 Core CLI slice: use strict `clap` parser/mapper behavior for document commands; unknown argv, extra positional values and operation-inapplicable flags return input diagnostics before navigation input resolution.
- [x] 5.2 Adapter selection slice: explicit CLI/config adapter failure returns adapter selection diagnostic; automatic discovery may continue internally and returns a candidate failure list when all candidates fail.
- [x] 5.3 Parameter/config/native option slice: unmapped direct/config input returns blocking diagnostics; default config absence is silent absence; explicit or present invalid config fails; adapter native options are explicit owner-scoped sources.
- [x] 5.4 Diagnostics/output slice: project one primary `DiagnosticRecord` for public failures; successful document output carries only the owning success payload; protocol-json stdout remains schema-valid.
- [x] 5.5 Linked adapter contract slice: linked adapter handlers receive prepared operation input and typed native option values, return structured adapter diagnostics, while selected adapter typed-field declarations validate option support/type/range before business handling.
- [x] 5.6 Cleanup slice: align implementation APIs/tests with strict argv diagnostics, primary `DiagnosticRecord` projection and success payload construction.

验证：每个 slice 包含其 public boundary 的 focused tests，并在 integration 前报告 cross-slice API changes。

## 6. Track E: Tests and Validation Materials

负责人：test and verification owners。

输入：coordinated active changes 的 Track A validation output、Track B owner docs、Track C schema/examples/fixtures 和 Track D implementation behavior。

写入范围：unit tests、smoke tests、case ledger、validators，以及必要的 verification docs/scripts。

输出：updated proof targets、changed tests/fixtures、validation commands 和 residual coverage gaps。

- [x] 6.1 Unit tests: update core CLI parser, navigation adapter selection, navigation input resolution, linked adapter contract, diagnostics and output tests.
- [x] 6.2 Smoke tests and case ledger: migrate unknown argv, extra positional, unused flag, explicit adapter failure and config invalid cases to strict failure assertions; prioritize `BB-CORE-OUTPUT-001`, `BB-CORE-SELECT-001`, `BB-MD-BOUNDARY-001` and `BB-MD-CONFIG-001`，并把 Markdown 成功路径预期改为 success payload 或 failure diagnostic proofs。
- [x] 6.3 Schema/example validation: update validators and fixtures so readable success output has the documented success payload shape and protocol/readable failure examples include primary `DiagnosticRecord` guidance.
- [x] 6.4 Active-change validation: run strict OpenSpec validation for this change and every coordinated active change touched in Track A.

验证：tests 证明 valid input 通过 shared document operation pipeline 成功，invalid public input 返回 actionable diagnostics。

## 7. Integration Verification

负责人：main integrator。

输入：completed Track A-E worker JSON outputs 和 local diff。

- [x] 7.1 跨轨道 diff review：确认每个 changed file 都属于声明的 owner，且每个 track 都使用 strict input diagnostics、primary `DiagnosticRecord` projection、owner-scoped native options 和 documented success payload shape。
- [x] 7.2 契约一致性 review：确认 docs、specs、schema/examples、tests 和 implementation 对 strict input、internal discovery、config absence/invalid state、native options、ref ownership 和 primary `DiagnosticRecord` 使用同一组规则。
- [x] 7.3 运行 core CLI、navigation input resolution、diagnostics/output 和 adapter selection 的相关 Rust test targets。
- [x] 7.4 运行 protocol/readable/manifest/probe 和 diagnostic fixtures 的 schema/example/docs validation。
- [x] 7.5 跨边界改动集成后运行 `bun run verify:docnav-workspace`。
- [x] 7.6 最终 local diff review：确认变更范围只覆盖 strict input boundary、diagnostic guidance、success output shape、owner-scoped native options 和对应 validation materials。
