# Docnav 格式路由推断依赖调查

## 调查信息

- 核心问题: Docnav 应以什么权威信号替代 probe traversal，以及现有 Rust 格式识别依赖中是否有功能充分、重量可接受、生态可信且比项目已有 metadata 更合适的候选？
- 状态: 已结束
- 最新报告时间: 2026-08-03T03:20:21+00:00

## 调查报告

### JSON/Markdown 候选依赖、路由语义与最小维护面审计

- 形成时间: 2026-07-30T12:59:29+00:00

#### 形成时背景

`replace-probe-traversal-with-inferred-routing` OpenSpec change 触发了本轮调查：它拟用一次格式推断和精确 registry lookup 替代 automatic adapter probe traversal，并完整删除 probe surface。用户已经确认这是破坏性更新，不需要 backward-compatibility 或 inspection fallback；尚未批准任何依赖或 production 实现。该 change 是本报告的首个消费者，不拥有本调查主题；后续其它 routing、adapter 或 dependency 决策也可以独立读取本主题。

形成本报告时，仓库 `HEAD` 为 `ebca55a8564e1ae478e96a3c90645ca3bd7cf2db`。

调查开始时，design 已规定 inference 只是 routing hint，selected adapter 继续拥有真实读取、parse、semantic validation 和 operation diagnostic，但尚未明确“hint”的权威输入究竟是 path extension 还是 document content。这个差异直接决定候选是否“功能充分”：同一个库在 extension-led 语义下可能完全正确，在 content-led 语义下则可能不具备所需能力。

Current built-in manifests 已分别声明 JSON 的 `.json` / `application/json` 和 Markdown 的 `.md`、`.markdown` / `text/markdown`。因此 dependency audit 还必须回答一个最小性问题：若 path extension 是权威信号，通用 MIME extension table 是否比项目已经拥有的 `FormatDescriptor.extensions` 更能降低总维护成本。

#### 调查目的

本轮调查回答以下问题：

1. 恢复 Current automatic/explicit routing、probe、diagnostic 和 manifest ownership 基线，定位后续 breaking removal 的当前消费者。
2. 用同一组 JSON/Markdown cases 比较成熟候选、替代库和 no-new-dependency baseline 的功能覆盖、失败分类和第三方值封装边界。
3. 比较 adoption、热度、维护、license、MSRV、advisory/unsafe/supply-chain、依赖图、构建、binary/package 和 startup 重量。
4. 给出有条件 recommendation，并把仍会改变 public behavior 或长期维护面的选择留给用户或指定 architecture/product owner。

本轮不选择依赖，不修改 production/Cargo/lockfile，不完成 probe removal，不替代 Windows/release package 的最终验证。

#### 调查范围与依据

- **Current source baseline**：用 CodeGraph 检查 `AdapterDefinition`、`select_adapter`、built-in Markdown/JSON definitions 和 `FormatDescriptor`；用限定路径的 token/reference search 补充 Rust tests、owner docs、schemas、examples、validators 和 release-adjacent material。
- **候选版本**：`mime_guess 2.0.5`、`magika 1.1.0`、`infer 0.22.0`、`file-format 0.29.0`、`tree_magic_mini 3.2.2`；`tft 0.1.1` 在 license/成熟度筛选阶段淘汰；另比较 `std::path::Path::extension()` baseline。
- **Spike feature sets**：`mime_guess`、`infer`、`tree_magic_mini` 使用 `default-features=false`；`file-format` 使用 `default-features=false, features=["reader-txt"]`。Magika 先以 `default-features=false` 检查最小图，但 release link 缺少 ONNX Runtime；功能 spike 随后显式提供 `ort 2.0.0-rc.12` 的 `std, ndarray, download-binaries, tls-rustls, copy-dylibs, api-24` features。
- **生态快照**：2026-07-30 查询 crates.io crate/reverse-dependency API、对应 GitHub repositories、docs.rs API 文档和 crate source。下载、reverse dependencies、stars、issues 和最近 push 都只是 adoption/maintenance 信号，不单独证明正确性。
- **功能 spike**：27 个实际文件 cases，加 1 个 missing-path failure-taxonomy check；覆盖正常/大写 extension、extensionless、empty、UTF-8 BOM、non-UTF-8、误导 extension、content/extension conflict、malformed、polyglot、binary payload 和约 3.2–3.6 MB representative large inputs。两次运行结果逐字相同。
- **重量 spike**：临时目录 `/tmp/docnav-format-inference-weight-20260730`，Linux x86_64 WSL2 host，workspace pinned `rustc/cargo 1.96.0`；release proxy 使用 `opt-level=3`、fat LTO、单 codegen unit、abort panic 和 symbol stripping。clean build 每项 3 次，warm build 7 次，cold build 1 次；process warm startup 50 次，cold proxy 15 次。
- **安全筛查**：对 RustSec advisory-db commit `7c7ccac53056b87f69ac677f15ea2d9a98a6f8e2` 做 direct crate-name search，并检查候选 source 中的 unsafe sites。环境没有 `cargo-audit`，所以没有形成完整 resolved-graph advisory verdict。
- **未覆盖**：本环境未安装 `x86_64-pc-windows-msvc` standard library，Windows check 在缺少 target 时失败；没有形成真实 Docnav integration binary/package delta；cold build 只有一次，cold startup 只是 cache-eviction proxy；`tree_magic_mini` 的 system MIME database 无法可靠 eviction；临时 spike/raw TSV 不是受版本控制的 release evidence。

#### 调查结果与边界

以下结论区分已核实事实、条件性事实、建议和仍需 owner 决定的事项；“推荐”不表示已经批准：

1. **[决策缺口] 当前不能批准 dependency。** 先要决定 automatic routing 的权威信号是 path extension 还是 content；否则“功能充分”的判定没有稳定 oracle。
2. **[条件性事实] 若 path extension 是权威 routing hint，`mime_guess 2.0.5` 功能充分且不是重量级依赖。** 它在 27/27 routing cases 上符合 extension-led oracle，最小图为 3 个 external packages、约 285 KB unpacked source；proxy clean build 中位数比 baseline 增加约 1.20 s，warm startup 中位数增加约 0.074 ms。
3. **[建议] `mime_guess` 不是 extension-led 方案的最小维护面。** `std` baseline 同样为 27/27，且 built-in manifests 已拥有全部当前 extension mappings。`mime_guess` 只按 extension 查静态 MIME 表、不读取文件，仍需要 Docnav-owned MIME→format mapping，并引入 patch-level mapping churn。若选择 extension-led 语义，首选建议是修订 `replace-probe-traversal-with-inferred-routing` design 的 Decision 6，评估由 project/manifest-owned extension mapping 直接产出 normalized format identity，而不是为了“使用推断库”引入第二张 extension 表。
4. **[条件性候选] 若 architecture 明确要求一个独立于 loaded registry 的成熟外部 MIME knowledge base，唯一建议候选是**：

   ```toml
   mime_guess = { version = "=2.0.5", default-features = false }
   ```

   Exact private normalization 为 `application/json → json`、`text/markdown | text/x-markdown → markdown`、其它值 → `Unknown`，映射后排序去重；多个 project identities 才形成 `Ambiguous`。必须 exact pin，因为 upstream 明确说明同一 extension 的 MIME values 可在 patch release 改变。
5. **[淘汰结论] 若 content 必须决定格式，没有候选通过。** Magika 是唯一能直接产出 JSON/Markdown identities 的 content-based 候选，但短 Markdown、BOM JSON、empty/malformed 和 conflict cases 不满足当前需求；其 ONNX runtime/model/package 成本也显著更高。此语义下应修订或结束 change，而不是临时写 custom detector。
6. **[可达性边界] `FORMAT_AMBIGUOUS` 没有在当前候选的真实 JSON/Markdown corpus 中自然出现。** `.md` 的两个 MIME values 都归一为同一个 `markdown` identity。若保留该 public branch，只能以 project-owned synthetic normalization case 验证，不能声称它是 `mime_guess` 当前可达行为。
7. **[未验证] Windows target、真实 Docnav binary/package comparison 和部分 cold measurement 尚未形成。** 依赖比较和功能 corpus 已形成 bounded evidence，但不足以替代最终跨 target 和 release-package validation。

#### 术语、所有权与使用规则

- **extension-led**：path extension 产生 routing identity；selected adapter 随后读取并验证实际内容。Extensionless 文档不因内容本身获得格式 identity。
- **content-led**：document bytes 参与或决定 routing identity；extensionless 内容可以被识别，content/extension conflict 可能由 content 覆盖。
- **manifest-native**：不引入外部格式识别 crate，由 Docnav-owned closed mapping 或 `FormatDescriptor.extensions` 支持 routing identity；它不是已经批准的实现。
- **external MIME table**：由 `mime_guess` 提供 extension→MIME facts，再由 Docnav-owned finite mapping 转成 format identity。

本主题拥有调查时点的 source facts、候选比较、测量结果、适用条件和 recommendation。用户或指定 architecture/product owner 拥有 authoritative signal、routing mechanism 和 dependency approval；消费本报告的 OpenSpec change 在批准后拥有目标 architecture/spec/tasks。后续 change 不得把本报告中的建议当作已批准 decision，也不得把自己的进度状态回写成调查事实。出现会改变候选结论的新证据时，应追加新的完整报告并同步调查索引。

#### 当前实现事实：路由、manifest owner 与删除面

Current source-backed call chain 是：

```text
automatic:
select_adapter
  -> registry.adapters() in static order
  -> AdapterDefinition::probe(path) for each candidate
  -> first supported candidate wins
  -> all fail: FORMAT_UNKNOWN / NO_SUPPORTED_ADAPTER
               + candidates + candidate_failures

explicit:
select_adapter
  -> find_adapter(declared id)
  -> selected definition probe(path)
  -> missing/rejected/invalid candidate: ADAPTER_UNAVAILABLE
  -> successful probe: selected operation
```

当前 automatic route 的结果依赖 registry order；每个 probe 可以读取和 parse 文件。Explicit adapter id 也不会直接表达最终 caller intent，因为 lookup 后仍以 probe 作为 selection gate。被选中后的 operation 再次读取/parse，并由 adapter owner 报告其 operation diagnostic。

Current exact selection diagnostic baseline 是：

| Current outcome | Code | Current canonical details |
| --- | --- | --- |
| automatic 全部候选失败 | `FORMAT_UNKNOWN` | `path`、`reason="NO_SUPPORTED_ADAPTER"`、`candidates[]`，并把相同 subordinate evidence 再投影到 `candidate_failures[]` |
| explicit id 不存在 | `ADAPTER_UNAVAILABLE` | `adapter_id`、`reason="ADAPTER_NOT_FOUND"`、`selection_source`、`stage="resolve"` |
| explicit definition 的 probe unsupported/invalid | `ADAPTER_UNAVAILABLE` | `adapter_id`、`reason="PROBE_UNSUPPORTED"` 或 `PROBE_INVALID`、`selection_source`、`stage="probe"` |
| selected operation 失败 | adapter/document owner code | operation owner 的既有 details；selection 不尝试后续 adapter |

Current candidate objects 包含 `adapter_id`、`stage` 和 `reason`；automatic candidate stage 可为 `resolve` 或 `probe`。Target artifacts 已计划删除 `candidate_failures`、probe stage/reasons 和第三方 facts，但此处只记录当前实现。

Manifest ownership 已经是明确的单一项目 metadata surface：

| Owner | Current format metadata | 当前 probe gate |
| --- | --- | --- |
| JSON definition | `id=json`；`.json`；`application/json` | extension 命中后读取并完整 load JSON |
| Markdown definition | `id=markdown`；`.md`、`.markdown`；`text/markdown` | extension 命中且内容为 UTF-8 |

限定在 Current 非 OpenSpec surface 的 direct token/reference inventory 包括：

- **Production Rust**：`crates/shared/adapter-contracts` definition/exports，`crates/shared/navigation` routing/context/exports，`crates/shared/protocol` constants/probe/decode/schema/contract-validation，`crates/shared/diagnostics` candidate details/codes，built-in JSON/Markdown adapters。
- **Rust tests**：上述 shared crates 的 unit tests、navigation auto/explicit/adapter-source support、core registry tests、runtime invocation-log recording adapter、JSON adapter tests、Markdown metadata tests。
- **Owner docs**：`docs/adapter-contract.md`、`docs/architecture.md`、`docs/navigation.md`、`docs/navigation-input-resolution.md`、`docs/protocol.md`、`docs/adapters/json.md`、`docs/adapters/markdown.md`、`docs/testing.md` 和 `docs/testing/coverage.md`。
- **Schema/examples/Case ledgers**：`docs/schemas/probe-result.schema.json`、`protocol-response.schema.json`、schema index；probe result、format unknown/ambiguous examples；core CLI、navigation、protocol、JSON 和 Markdown Case ledgers。
- **Validation/release-adjacent**：`scripts/tools/validators/config.ts`、schema validator index 和 accepted-warning test description。`.github/workflows/release-package.yml` 没有 direct probe token；其影响是通过 workspace/schema/example/package smoke 间接进入最终 release verification。
- **历史材料边界**：`docs/decisions/**` 中的 probe 文字是形成时历史，不自动改写为 Current contract；任何实施 probe removal 的 change 都必须逐项判断保留历史、迁移当前引用或记录 breaking impact。普通 “env probe” 和进度 observer 中的英文 probe 不是 adapter routing consumer。

该 inventory 证明迁移不是只改 `routing.rs`；但它还不是逐 consumer deletion/migration disposition，因此任何消费本报告的 removal change 都必须另行闭合 removal inventory。

#### 功能证据：统一 corpus 与第三方封装

Extension-led oracle 的精确定义是：

- `.json`、`.md`、`.markdown` 按 ASCII case-insensitive path extension 路由，不因 bytes 内容改变 selection。
- extensionless 和未声明 extension 返回 `Unknown`。
- content/extension conflict 由 extension 选择 adapter；selected adapter 随后决定 parse/encoding/semantic diagnostic。
- known extension 的 empty、malformed、non-UTF-8 和 missing path 仍先选择 adapter；真实 document I/O/parse error 由 selected adapter owner 报告。

| 候选 | 27 个 routing cases | 决定性结果 |
| --- | --- | --- |
| `std` extension baseline | 27/27；FP/FN/wrong = 0/0/0 | 零依赖；与当前 manifest extension metadata 同语义，但当前 artifacts 尚未授权此实现 |
| `mime_guess 2.0.5` | 27/27；0/0/0 | 大小写、known/unknown extension 全部符合；`.md` 返回两个 raw MIME values，均去重为 `markdown`；不读取 path |
| `magika 1.1.0` | 8/27；3/14/2 | 正常短 Markdown→plain text，BOM JSON→Snap，`.md` polyglot→JSON |
| `infer 0.22.0` | 5/27；0/22/0 | generic JSON/Markdown text 不在 magic-byte catalog；只在 oracle 本来也为 unknown 的 cases 偶然一致 |
| `file-format 0.29.0` | 5/27；0/22/0 | reader 把普通输入归为 plain text；extension table 的 `.json` 是 JSON Feed、`.md` 是 Mega Drive ROM，不能承担本 contract |
| `tree_magic_mini 3.2.2` | 5/27；0/22/0 | 普通 JSON/Markdown 归为 `text/plain`；依赖 host freedesktop MIME database，跨环境结果不够封闭 |

Missing-path check 单独记录：`mime_guess` 和 `std` 仅按 `.json` 先返回 `json`，没有发生 I/O；content candidates 返回 document failure。两种行为哪个正确取决于权威信号决策。

所有 spike 都用 private normalizer 把 raw MIME、enum、score 和 errors 收敛为 project-owned `Recognized` / `Unknown` / `Ambiguous` / `DocumentFailure` / `InternalFailure`；没有技术障碍迫使第三方 enum、message、confidence 或 debug data进入 public details/logs。

Future code-format fit：

- `mime_guess` 已覆盖大量 code extensions，但每个可观察 Docnav format 仍须进入有限 normalization mapping；不得自动公开 upstream 新格式。
- manifest-native extension index 会随 adapter descriptor 增长，owner 更直接，但需要决定如何保留“recognized but registry unsupported”的 `NO_SUPPORTED_ADAPTER + format` branch。
- Magika 的 code-format breadth 最强，但当前核心 Markdown/JSON corpus 已未过门槛。
- magic-byte/system MIME 候选对多数 source code/plain text 仍只能返回 generic text，不能形成稳定 code adapter identity。

#### 生态证据：adoption、维护、license 与 supply-chain

下列数值是 2026-07-30 的查询快照：

| 候选 | Adoption / activity signals | License、MSRV 与 risk |
| --- | --- | --- |
| `mime_guess 2.0.5` | 240,531,346 downloads；1,349 reverse dependencies；GitHub 220 stars / 16 open issue-or-PR records；latest crate 2024-06-29，repo last push 2025-09-12 | MIT；transitives 为 MIT 或 MIT/Apache-2.0；README states Rust 1.33，manifest 未声明 `rust-version`，Rust 1.96 实测通过；crate source 无 unsafe block |
| `magika 1.1.0` | 502,429 downloads；5 reverse dependencies；Google monorepo 17,836 stars / 151 open issue-or-PR records、2026-07-29 active push；stars/production claims 不是 Rust crate 单独 adoption | Apache-2.0；crate 未声明 MSRV，`ort` declares 1.88；3 个 crate-local unsafe sites，加 ONNX Runtime FFI/native packaging boundary |
| `infer 0.22.0` | 104,741,616 downloads；330 reverse dependencies；GitHub 392 stars / 33 open issue-or-PR records；2026-07-15 release/push | MIT；MSRV 1.74；zero transitive dependency，crate forbids unsafe；功能不合格 |
| `file-format 0.29.0` | 1,560,097 downloads；34 reverse dependencies；GitHub 136 stars / 2 open issue-or-PR records；2026-03-27 release/push | MIT/Apache-2.0；MSRV 1.85；minimal reader graph 无 transitive dependency；功能不合格 |
| `tree_magic_mini 3.2.2` | 12,146,043 downloads；49 reverse dependencies；GitHub 48 stars / 11 open issue-or-PR records；2025-11-14 release/push | MIT；MSRV 1.85；runtime DB 带环境差异，embedded DB feature 为 GPL，不适合作为默认分发路径 |
| `tft 0.1.1` | 15,570 downloads；3 reverse dependencies；GitHub 1 star / 0 open issue-or-PR records；last release/push 2023-09-23 | GPL-3.0-only，且 content detection 尚不足；在 corpus 前筛掉 |

RustSec snapshot 中没有发现这些 direct crate names、`mime`、`unicase`、`ort` 或 `ort-sys` 的直接 advisory record。这个结果只表示 direct-name snapshot search 无命中：没有 `cargo-audit` full resolved graph、OSV/GitHub advisory 交叉检查或长期维护者风险审计，不能解释为“供应链已认证安全”。

License 初筛显示 `mime_guess` 最小图与 Docnav MIT 分发兼容；若最终批准，仍应由 release/license material 对 resolved lock graph 做一次正式核对。Magika 的 Apache-2.0 本身可用，但 native runtime/download-binary 和 notice/package obligations 显著扩大。

Upgrade/rollback cost 也不同：`mime_guess` upgrade 必须重跑完整 mapping corpus、复核 `.md` 的多 MIME ordering 和 resolved license graph；exact pin 避免无意中吸收 patch mapping change。其 rollback 只涉及 navigation-owned helper、Cargo/lockfile、license material 和相应 tests，没有数据迁移。Manifest-native baseline 没有第三方升级面，但每次新增 adapter format 必须验证 descriptor collision 和 routing cases。Magika upgrade/rollback 还涉及 model、`ort` prerelease/API、download/link behavior、native artifacts 和 package verification，明显更昂贵。

#### 重量证据：依赖图、构建、binary 与 startup

最小 proxy 的可比较结果如下；source graph size 是下载 crate 解包大小，不是最终 package size：

| 候选 | Resolved graph / proxy binary | Build / startup evidence |
| --- | --- | --- |
| baseline | 0 external packages；310,272 B | clean median 2.713 s；warm process median 0.878 ms；cold proxy 1.828 ms |
| `mime_guess` minimal | 3 packages；67,246 B compressed / 284,925 B unpacked；498,456 B，较 baseline +188,184 B | clean median 3.915 s（+1.202 s，1.44×）；warm Cargo build median 109 ms；warm process 0.952 ms（+0.074 ms）；cold proxy 2.271 ms（+0.443 ms） |
| `infer` minimal | 1 package；95,139 B unpacked；338,096 B | clean median 3.315 s；warm process 0.961 ms；功能不合格 |
| `file-format` minimal | 1 package；257,585 B unpacked；357,912 B | clean median 8.320 s；warm process 1.018 ms；功能不合格 |
| `tree_magic_mini` runtime DB | 9 unique package names / 10 resolved nodes；约 9.03 MB unpacked；392,272 B | clean median 10.925 s；warm process 0.941 ms；system DB cold eviction 不可靠；功能不合格 |
| `magika` minimal/runtime | 20 active unique package-name entries，Cargo metadata 23 external nodes；约 14.95 MB unpacked source，其中 embedded ONNX model 3,163,737 B | dependency-only check 约 14.7 s；minimal release link 因缺少 ONNX Runtime 失败；启用 `ort` download/rustls/copy-dylibs 后 functional executable 28,339,448 B；与其它 proxy profile 不同，不作精确 delta |

`mime_guess` 应分类为**轻量依赖**，不是重量级依赖；其真实成本主要是多维护一张 upstream MIME table和 upgrade retest，而不是 runtime。Magika 应分类为**重量级依赖**：不仅图和 binary 大，还引入 model、ONNX Runtime、FFI、下载/链接和跨 target package 风险。

Linux host build 已用 pinned Rust 1.96 对 baseline 和 `mime_guess` exact-minimal manifest 重跑成功。Windows preflight：

```text
cargo +1.96.0 check --locked --target x86_64-pc-windows-msvc
error[E0463]: can't find crate for `core` / `std`
```

这证明当前环境缺少 target，不证明候选在 Windows 失败。跨 target 结论需要在 project-supported Windows runner 或安装完整 target/linker 的环境中继续形成。

#### 供 architecture/product owner 决策的选项

本报告不作出 architecture decision。Owner 应按以下顺序选择；compatibility 已由用户关闭，不再作为选项：

1. **权威信号**
   - **A — path extension 是 routing hint（推荐）**：extensionless valid JSON/Markdown → `Unknown`；conflict → extension 选择 adapter；known extension 的 empty/malformed/non-UTF-8/missing path → 先选 adapter，再由 adapter 报真实 diagnostic。
   - **B — content 决定 format**：extensionless 内容可以识别，content 可以覆盖 extension。当前没有通过 JSON/Markdown corpus 的候选；选择 B 等于修订或结束当前 change，而不是开始实现。
2. **A 路径下的 mechanism**
   - **A1 — project/manifest-owned extension mapping（最小维护面推荐）**：不新增依赖，复用 `FormatDescriptor.extensions` 或一个 project-owned closed mapping；批准时必须确保 change 不再把 “no dependency” 等同于 “no implementation”，并精确解决 `NO_SUPPORTED_ADAPTER + format` 的可达语义。
   - **A2 — 独立外部 MIME table**：批准 exact `mime_guess = "=2.0.5"`、`default-features=false` 和上述三项 normalization；接受约 184 KiB proxy binary delta、3-package graph、patch mapping churn、尚待 Windows/真实 package 验证。

若用户选择 A2，生态、功能和当前 bounded weight 证据支持进入剩余 gate；若选择 A1，需要由消费 change 的 artifact 收敛任务把它写成 approved Target，而不是把它误记为已批准 dependency；若选择 B，当前全部候选都应拒绝。

#### 证据来源、持久性与复核入口

下列临时原始材料只证明本轮运行；它们不受版本控制、可能被环境清理，不能作为长期 release artifact：

- `/tmp/docnav-format-routing-spike.BhDZiD/results.tsv`
- `/tmp/docnav-magika-routing-spike/results.tsv`
- `/tmp/docnav-format-inference-weight-20260730/measure/build-times.csv`
- `/tmp/docnav-format-inference-weight-20260730/measure/graph-sizes.json`
- `/tmp/docnav-format-inference-weight-20260730/measure/startup-times.json`

主线程已用 release binaries 重跑两份 corpus，`cmp` 均为 byte-identical；已用 `cargo +1.96.0 build --release --locked` 重跑 baseline 和 `mime_guess` host builds。临时路径可被环境清理，本文保存的是可审计摘要；最终 implementation/release evidence 必须由对应 change 的 tracked validation 重新产生。

官方/第一方来源：

- [`mime_guess 2.0.5` API](https://docs.rs/mime_guess/2.0.5/mime_guess/struct.MimeGuess.html)
- [`mime_guess` repository / README](https://github.com/abonander/mime_guess)
- [`magika 1.1.0` `ContentType`](https://docs.rs/magika/1.1.0/magika/enum.ContentType.html)
- [Magika repository](https://github.com/google/magika)
- [ONNX Runtime `ort` linking guide](https://ort.pyke.io/setup/linking)
- [`infer 0.22.0`](https://docs.rs/infer/0.22.0/infer/)
- [`file-format 0.29.0`](https://docs.rs/file-format/0.29.0/file_format/)
- [`tree_magic_mini 3.2.2`](https://docs.rs/tree_magic_mini/3.2.2/tree_magic_mini/)
- [crates.io API](https://crates.io/data-access)
- [RustSec advisory database snapshot](https://github.com/RustSec/advisory-db/commit/7c7ccac53056b87f69ac677f15ea2d9a98a6f8e2)

### Manifest 路径提示决策、常见配置别名与 JSON-family 边界复查

- 形成时间: 2026-07-31T03:17:33+00:00

#### 形成时背景

上一份报告把 automatic routing 的权威信号、mechanism 和 dependency 保留为决策缺口。形成本报告时，用户已经明确批准：

- pathname 是 automatic routing 的唯一信号；
- adapter manifest 继续拥有 `formats[].extensions[]`，并新增 exact-basename `formats[].filenames[]`；
- routing 不新增依赖；
- pathname hint 只用于快速选择 adapter，不证明内容真实性；
- explicit adapter id 跳过 automatic routing并强制选中的 adapter 解析；
- selected parse、semantic 或 operation failure 不 fallback；
- 本次 breaking change 完整删除 probe surface，不保留 compatibility path。

这些是用户批准的决策事实，不是调查建议。对应跨 change 方向已经写入活动决策 `docs/decisions/adapter-evolution/route-by-manifest-pathname-hints.md`；由于形成本报告时主规范和 production code 仍是 probe traversal，该决策状态为 `active + unaligned`。

形成本报告时，仓库 `HEAD` 仍为 `ebca55a8564e1ae478e96a3c90645ca3bd7cf2db`。`replace-probe-traversal-with-inferred-routing` 已回写批准后的 Target，但 blocking inventory、cross-change handoff 和 final artifact audit 尚未完成；没有 routing code、Cargo manifest、lockfile、owner 主规范、schema、test 或 release artifact 因该 change 被应用。

用户同时要求让 JSON adapter 支持 JSONC，并要求讨论其它“泛 JSON”文件格式。独立 OpenSpec change `support-jsonc-in-json-adapter` 已形成 proposal、design、delta spec 与 tasks。经对 routing handoff 和 closed operation input 的审查，它已经选择一个 JSONC-capable grammar、一个 `json` identity、确定的 normalized/source output 与 stable diagnostic；尚未选择 parser dependency，也未获 implementation authorization。

#### 调查目的

本轮调查回答以下问题：

1. 在权威信号已经批准后，确认零依赖 manifest-native 方案是否仍是证据支持的最小正确实现。
2. 确定 ordinary extension 与 exact filename 的职责、匹配顺序、大小写和冲突边界。
3. 核对 `.prettierrc`、`.code-workspace`、`.watchmanconfig` 等常见配置文件能否由该模型表达，以及“可路由”与“可解析”的差异。
4. 把 JSON-family 按 grammar/encoding、document model、validation/canonicalization profile 和 semantic profile 分类，避免把所有 JSON-like 格式错误地塞进一个 parser 或 adapter contract。
5. 定位 JSONC change 与已批准 routing contract 之间的 handoff 缺口，并给出后续优先级，不在本报告中实施 parser 或 semantic adapter。

#### 调查范围与依据

- **批准事实**：本轮用户对 manifest-owned extension/exact-filename、零依赖、hint-only、explicit force-selection、no fallback 和 breaking probe removal 的直接确认。
- **仓库证据**：已复核 routing change 的 proposal、design、tasks 与九份 delta specs，Current `FormatDescriptor`/manifest schema，Current adapter definition、selection/dispatch 和 closed standard operation input。CodeGraph 显示 Current `AdapterSelection` 只保留 selected `AdapterDefinition` 与 candidate evidence，`StandardOperationInput` 只向 adapter 提供 normalized `document_path` 和 operation fields，不提供 matched format identity。
- **alias spike**：延续上一报告的相同 Rust 1.96/Linux 基线，补查 `std::path::Path::extension()` 与 `mime_guess 2.0.5` 对 `.prettierrc`、`.code-workspace` 和普通 `.json` 的行为。Rust pathname API 不把 `.prettierrc` 视作 ordinary extension；`mime_guess` 对 `.prettierrc` 无结果，对 `.code-workspace` 没有可用 MIME mapping。
- **配置文件 owner 证据**：[Prettier configuration](https://prettier.io/docs/configuration) 明确 `.prettierrc` 可写成 JSON 或 YAML；[VS Code workspace documentation](https://code.visualstudio.com/docs/editing/workspaces/multi-root-workspaces) 的 `.code-workspace` 示例和说明允许 comments；[Watchman configuration](https://facebook.github.io/watchman/docs/config) 要求 `.watchmanconfig` 是 valid JSON。
- **扩展配置名复核**：[VS Code snippets](https://code.visualstudio.com/docs/editing/userdefinedsnippets) 把 `.code-snippets` 定义为支持 comments 的 JSON；[Babel config files](https://babeljs.io/docs/config-files/) 把 `.babelrc` 作为 `.babelrc.json` alias，但用 JSON5 parser；[Stylelint configuration](https://stylelint.io/user-guide/configure/) 允许 extensionless `.stylelintrc` 为 YAML 或 JSON；[ESLint current configuration](https://eslint.org/docs/latest/use/configure/configuration-files) 已转向 JavaScript/TypeScript flat config，而旧 `.eslintrc` family 还包含 JSON、YAML 和 JavaScript 变体。
- **JSON exact-name 候选证据**：[Pipenv documentation](https://pipenv.pypa.io/en/stable/pipfile.html) 把 `Pipfile.lock` 定义为 JSON；[Deno lockfile documentation](https://docs.deno.com/examples/dependency_lockfile_tutorial/) 展示 `deno.lock` 的 JSON document shape，Deno config 本身则明确支持 `deno.json` 与 `deno.jsonc`。
- **JSON-family 上游证据**：
  - [RFC 6839](https://www.rfc-editor.org/rfc/rfc6839.html) 的 `+json` structured suffix 区分底层 JSON generic processing 与上层媒体类型语义。
  - [JSONC specification](https://jsonc.org/) 把 JavaScript-style comments 定义为 JSONC 扩展，推荐 `.jsonc` / `application/jsonc`，并把 trailing comma 定义为 parser 可选能力而非默认必需能力。
  - [JSON5 specification](https://spec.json5.org/) 还允许 unquoted keys、single quotes、hex、leading `+`、`Infinity` 和 `NaN` 等更广语法。
  - [NDJSON specification](https://github.com/ndjson/ndjson-spec) 和 [RFC 7464 JSON Text Sequences](https://www.rfc-editor.org/rfc/rfc7464.html) 都承载多个 JSON text，但分别以 newline 与 RS frame 划分记录，且错误恢复规则不同。
  - [RFC 8949 CBOR](https://www.rfc-editor.org/rfc/rfc8949.html) 与 [BSON specification](https://bsonspec.org/spec.html) 具有普通 JSON 无法无损表达的二进制类型和 key/value semantics。
  - [JSON-LD 1.1](https://www.w3.org/TR/json-ld11/)、[RFC 7946 GeoJSON](https://www.rfc-editor.org/rfc/rfc7946.html) 与 [Jupyter notebook format](https://nbformat.readthedocs.io/en/latest/format_description.html) 用于核对“strict JSON representation + domain semantics”的 profile 层。
- **JSONC contract review**：审查了子代理草案与 Current closed operation input，确认 routing 不会把 matched extension/format identity 传给 strategy；据此排除 routing-selected strict/JSONC 双 mode，并在 JSONC change 中固定单 grammar 行为。
- **未覆盖**：没有在本轮选择或 benchmark JSONC parser；没有为 profile-specific navigation 建立真实 user-demand corpus；没有实现或验证 Windows/release package；没有把候选 JSON-family suffix 全部批准进当前 routing change。

#### 调查结果与边界

##### 1. 已批准的 routing 机制与 exact behavior

`replace-probe-traversal-with-inferred-routing` 的 Target 已收敛为：

```text
explicit adapter id
  -> exact adapter-id lookup
  -> selected adapter parses

automatic
  -> normalized pathname
  -> case-sensitive exact basename lookup in filenames[]
  -> otherwise ASCII case-insensitive terminal extension lookup in extensions[]
  -> manifest-owned format identity
  -> exact registry definition lookup
  -> selected adapter parses
```

Automatic lookup 不打开文件，不读取 bytes，不运行 adapter code。Exact filename 优先于 extension；它们是不同 hint kinds，所以一个 basename 可以用 filename mapping 覆盖自己的 generic extension mapping。

Format identity、ASCII-case-fold 后的 extension 和 exact filename 在 validated registry 中分别唯一。Construction、doctor 和 release validation 阻断 duplicate format 或 same-kind hint；若冲突逃到 runtime，分别使用 global `registry-format-identity-conflict` 与 `registry-path-hint-conflict`。无 hint match 使用 `FORMAT_UNKNOWN / FORMAT_NOT_RECOGNIZED`。同一 validated manifest registry 派生 hint 与 format indexes，因此该 automatic route 不产生 routing-only `NO_SUPPORTED_ADAPTER + format` 或 `FORMAT_AMBIGUOUS`。

##### 2. Probe 删除要求 selected parse 拥有稳定诊断

Current JSON adapter 在 probe 成功后若第二次解析遇到 invalid syntax、trailing input、duplicate decoded member 或 depth overflow，会投影 `INTERNAL_ERROR / json-document-changed-after-probe`。完整删除 probe 后，这个 stage-specific identity 不再真实，也没有其它可复用的 selected JSON parse diagnostic。

Routing Target 因而新增 project-owned `DOCUMENT_CONTENT_INVALID`，由 JSON adapter 映射四个 stable reasons：

- `JSON_SYNTAX_INVALID`
- `JSON_TRAILING_INPUT`
- `JSON_DUPLICATE_MEMBER`
- `JSON_MAXIMUM_DEPTH_EXCEEDED`

Canonical details 只包含 normalized `path` 与 stable `reason`。Parser type、raw message、unstable offset、duplicate member name 和 dependency trace 保持 private；invalid UTF-8 继续使用 `DOCUMENT_ENCODING_UNSUPPORTED`。这是 probe 删除所需的 public-contract replacement，不是 routing 对 adapter parse 的接管。

##### 3. 零依赖选择有充分证据，不是“依赖太重才勉强不用”

`mime_guess 2.0.5` 仍应分类为轻量而非重量级：上一报告测得 minimal graph 为 3 个 external packages、约 285 KB unpacked source，proxy binary delta 约 188 KiB，runtime/startup delta 很小。拒绝它的决定性理由是功能与 ownership 重复：

- 它不覆盖 `.prettierrc` exact dotfile；
- 它没有 `.code-workspace` 的可用 MIME mapping；
- 即使返回 MIME，Docnav 仍需维护有限 MIME→format mapping；
- manifest 已经拥有当前和未来 adapter 的 format/extension facts；
- 引入通用 MIME table 会形成第二份 mapping 与独立升级复测面。

因此批准的 manifest-native 零依赖方案同时满足功能和最小维护面，不需要保留可选 `mime_guess` flag。Routing library 只选择一种的要求也自然闭合：本 change 选择“不引入 routing library”。

##### 4. `filenames[]` 解决的是 exact basename，不是任意 path pattern

`filenames[]` 适合没有 ordinary extension 的 dotfile，或需要覆盖 generic extension route 的少量 known basename。它不是 glob、relative-path、directory-aware 或 content-profile 规则：

- `.prettierrc` 需要 exact filename，因为 Rust ordinary extension extraction不会把它视作 extension；
- `.watchmanconfig` 同理，且 owner 明确要求 valid JSON；
- `.code-workspace` 可以作为 terminal `.code-workspace` extension，不需要塞进 `filenames[]`；
- `package.json`、`tsconfig.json`、`devcontainer.json` 已由 `.json` extension 路由 JSON，不需要为每个 basename 重复声明；
- `.vscode/settings.json` 之类 directory-sensitive dialect 不能由 basename `settings.json` 精确表达；当前不为此增加 glob/path-pattern surface，因为 pathname route 只选择 adapter，不负责 parser dialect。

Approved initial hints 保持小集合：JSON 的 `.json`、`.code-workspace`、exact `.prettierrc`、exact `.watchmanconfig`，以及 Markdown 的 `.md`、`.markdown`。`.prettierrc` 的 YAML 分支和含 comments 的 `.code-workspace` 可能先选中 JSON 再被 then-current strict parser 拒绝；这符合 hint-only/no-fallback 契约，不是 routing bug。

##### 5. 常见配置名应按 grammar certainty 分组，不按“看起来像 JSON”收集

| Pathname 类别 | 结论 | 当前处理 |
| --- | --- | --- |
| `package.json`、`tsconfig.json`、`jsconfig.json`、`deno.json`、`devcontainer.json` 等 `.json` basename | 不需要逐个 alias；terminal `.json` 已足够选择 JSON adapter | 保持 generic `.json` route |
| `.jsonc`、`.code-workspace`、`.code-snippets` | 强 JSONC-family hints；suffix 足够选 adapter，但不证明内容有效 | `.jsonc` 由 JSONC change 增加；`.code-workspace` 已在 initial set；`.code-snippets` 是下一批候选 |
| exact `.watchmanconfig` | Owner 要求 JSON，且没有 ordinary extension | 已在 initial `filenames[]` |
| exact `.prettierrc` | JSON/YAML 歧义真实存在 | 作为用户批准的 best-effort 例外保留；YAML parse failure 不 fallback |
| exact `.babelrc` | 名称虽是 JSON config alias，实际 grammar 是更宽的 JSON5 | JSONC change 不加入；等 JSON5 contract |
| exact `.stylelintrc` 与旧 `.eslintrc` family | 可能是 JSON、YAML 或 executable JavaScript，生态本身也在迁移 | 不加入默认 JSON hints |
| exact `Pipfile.lock`、`deno.lock` | 上游定义为 JSON，但 terminal `.lock` 不能泛化 | 记录为下一批 exact-filename 候选，不扩大通用 `.lock` |

这套分类刻意不创建“常见工具配置大全”。Manifest allowlist 应只收录 owner 证据稳定、能由同一 adapter grammar 合理处理的 pathname；弱 basename 和多语法 basename 保持显式 adapter 入口。`.prettierrc` 是已知且已接受的例外，而不是放宽其它 ambiguous dotfile 的先例。

##### 6. JSON-family 必须分层，不能用一个“JSON-ish”布尔值管理

| 层级 | 代表格式 | Parser/model 结论 | Routing 与 contract 结论 |
| --- | --- | --- | --- |
| Strict JSON representation + semantic profile | JSON-LD、GeoJSON、HAR、Web App Manifest、ipynb、JSON Feed、JSON Patch/Merge Patch、JSON Schema/OpenAPI JSON、SARIF、MongoDB Extended JSON | 现有 strict JSON parser 可做通用结构导航；profile-specific semantics 可按需求另加 strategy/adapter | `.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif` 是下一批强 extension-hint 候选，但 hint 不能宣称 profile validity |
| JSONC grammar family | `.jsonc`、`.code-workspace`、VS Code/TypeScript/Deno/devcontainer configs | 可复用 JSON logical tree，但 comments、trailing comma、source spans、normalized/source output 必须有 exact dialect contract | 一部分 JSONC 文件仍以 `.json` 结尾，extension 不能独立证明或完整选择 dialect |
| JSON5 grammar | `.json5` | 需要独立 grammar/mode；`Infinity`、`NaN` 等不能无损投影为 strict JSON | 不应作为 JSONC parser 的隐藏宽松开关；native/normalized/source output 要另行决定 |
| Multi-document JSON | `.ndjson`、`.jsonl`、`application/json-seq` | 多个 top-level records/frames 改变 error recovery、outline root、ref、pagination 和 find 边界 | 应有独立 document-model change；顶层 JSON array 仍是一个普通 JSON document |
| Validation/canonicalization profile | I-JSON、JCS | 复用 JSON parser，增加约束校验或 canonical serializer | 通常不是新 parser，也不应只靠 suffix 宣称通过 |
| Binary JSON-like encoding | CBOR/CBOR Sequence、BSON/document sequence | 任意 map keys、tags、binary、int64/decimal128 等不能无损塞入普通 JSON model | 需要独立 adapter、typed ref 与 machine/readable projection；不是 JSON adapter 的 lexer variant |

##### 7. Strict JSON profile 可以先 generic-read，但 suffix 不是语义真实性

RFC 6839 支持 generic JSON processor 处理 `+json` representation，同时把 domain semantics 留给更具体的处理器。Docnav 可以据此分两步：

1. 先让 generic JSON adapter 对 `.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif` 等做结构导航；
2. 只有实际用户场景证明 generic outline 不足时，再增加 profile-specific navigation。

即使加入这些 extension hints，generic JSON operation 仍只应声称返回其真实 payload 的 `application/json`。仅因 path 为 `.geojson` 或 `.ipynb`，不能把任意选中子树标成整个 profile 的 media type。JSON-LD remote contexts、Schema/OpenAPI `$ref`、feed links 也不得在普通 `read` 中自动取远端资源。

本报告把这些 suffix 记录为 **P0 候选**，没有把它们追加到当前 routing change 的 approved initial set。原因不是 parser 不支持，而是用户要求先讨论 JSON-family policy；后续可一次批准一组 strong hints，不应把弱/歧义 basename（例如 `manifest.json`、`feed.json`、`.map`）误写成 profile guarantee。

##### 8. JSONC 独立 change 已关闭 mode ownership 缺口

JSONC 上游只固定 comments 基线；trailing comma 是可选 dialect 能力。`support-jsonc-in-json-adapter` 因而明确拥有 exact grammar、normalized/source output 与 diagnostics，而不是直接继承某个 crate default。

子代理的初稿曾同时倾向“strict `.json` mode 保持严格”和“JSONC 是同一个 `docnav-json` 的另一 mode”。Current code 与 routing Target 的 closed operation input 都只把 `document_path` 和 operation fields 交给 strategy；matched format identity 不进入 adapter input，所以该双 mode 无法由批准后的 routing contract 实现。审查后已选择以下单一责任边界：

- `docnav-json` 对所有被选输入采用一个经过批准的 JSONC-capable grammar，strict JSON 是其子集；
- `.json`、`.jsonc`、`.code-workspace`、exact JSON filename hint 和 explicit `--adapter docnav-json` 都进入同一 grammar；pathname 不选择 dialect；
- grammar 接受 `//`、非嵌套 `/* ... */`、EOF line comment，以及 object/array 最后一个成员或元素后的单个 trailing comma；
- grammar 继续拒绝 nested/unterminated block comment、`#` comment、single quote、unquoted key、hex、leading `+`、leading/trailing decimal point、`NaN`、`Infinity`、multiple roots 和其它 JSON5 syntax；
- structured `read` 输出 deterministic strict JSON；source full-read 保留 BOM-stripped 原文；source 只有在实际出现 JSONC-only syntax 时使用 `application/jsonc`，否则使用 `application/json`；
- `find` 搜索原始 BOM-stripped source；comment/trivia occurrence 映射到最深 enclosing object/array，无 child container 时映射 root；
- invalid content 复用本报告第 2 节的 `DOCUMENT_CONTENT_INVALID` mapping，不存在 mode/identity conflict，也不 fallback。

该选择同时覆盖许多仍以 `.json` 结尾的实际 JSONC 配置，并保持 explicit adapter 真正“强制由所选 adapter 解析”。JSONC change 剩余的设计 gate 是只选择一个 parser implementation/dependency，并用 resolved graph、feature、size、runtime、维护、生态、license 与 contract-fit 证据决定是否引入；不得同时保留两套 parser。

##### 9. 推荐的后续优先级

- **P0**：完成本 routing change 的小型初始 hints；在 JSONC change 中增加 `.jsonc`；下一批可一次批准 `.code-snippets`、`Pipfile.lock`、`deno.lock` 和一组 strict-JSON profile extensions，只增加 generic route，不承诺 domain semantics。
- **P1**：完成 JSONC parser dependency audit；另起 NDJSON/JSON Lines document-model change；若有明确用户需求，再优先 notebook、HAR 或 GeoJSON 的 semantic navigation。
- **P2**：按需求考虑 JSON5、JSON Patch、SARIF、OpenAPI/JSON Schema profile。
- **P3**：JSON Text Sequences、完整 JSON-LD semantic processing、CBOR/BSON 及其 sequences；这些需要更大的 ref、partial-error、binary output 或安全契约。

##### 10. 当前状态与使用边界

- 已应用到 planning/decision records：routing proposal/design/tasks/delta specs、活动决策，以及独立 JSONC change 的单-grammar Target。
- 尚未应用到 Current 产品：owner 主规范、code、Cargo/lockfile、schema/examples、tests 和 release package。
- 本报告保存形成时证据和边界，不替代 routing change 的 Target owner，也不替代未来 JSONC/NDJSON/profile change 的批准。
- 若后续批准额外 JSON-family hints，或形成 parser dependency 的统一重量证据，应追加新的完整报告；不得回写本报告制造“当时已经决定”的历史。

### 纯 pathname 前置路由与完整 basename suffix 语义复查

- 形成时间: 2026-08-03T03:20:21+00:00

#### 形成时背景

前一份报告形成后，用户进一步纠正了基础格式识别 Target 的执行顺序与匹配粒度：automatic routing 应是纯文件名/pathname 路由，在确认能够选择 adapter 前不需要也不应访问目标文件；`extensions[]` 不应先被压成“最后一段 extension token”，而应以类似锚定文件名 pattern 的方式匹配完整 basename；suffix 匹配需要大小写不敏感或等价的大小写归一化；matched format 不应传给 `StandardOperationInput`，因为 pathname hint 只是快速、方便但可能错误的选择事实。用户同时指出，adapter id 的唯一性已经由既有设计拥有，adapter 内部 parser mapping 由其它 change 负责，本 change 不应为这些边界增加新设计。

形成本报告时，仓库 `HEAD` 为 `ebca55a8564e1ae478e96a3c90645ca3bd7cf2db`。Current production code 尚未应用该 Target：`crates/docnav/src/runtime.rs` 在进入 navigation 前调用 `normalize_document_path`；`crates/docnav/src/project_paths.rs` 中该 helper 会读取 metadata、打开文件并执行 filesystem canonicalization。Current `docs/cli.md` 与 `openspec/specs/core-cli/spec.md` 也把 filesystem-backed document path normalization 放在 downstream navigation 之前。因此，“route 前无 document I/O”不只是 routing helper 内部限制，还需要改变 core/CLI 与 navigation 之间的执行顺序。

对应长期方向已演进为活动决策 `docs/decisions/adapter-evolution/route-by-manifest-basename-hints.md`；较窄的 terminal-extension 记录 `route-by-manifest-pathname-hints.md` 已通过 `修订` 关系归档。两者的演进保存决策历史，不表示 Current 代码已经迁移。

#### 调查目的

本轮复查回答以下问题：

1. 纯 pathname routing 是否能在不访问目标文件的前提下获得足够稳定的 adapter-selection signal。
2. “完整文件名正则这类”需求应落为通用 regex surface，还是更小的 full-basename anchored-suffix contract。
3. 大小写归一化、compound suffix overlap、exact filename precedence 和 no-match diagnostic 需要哪些确定性规则。
4. Matched hint/format 是否需要进入 adapter input，以及该 choice 对真实性与 parser ownership 的影响。
5. 哪些先前讨论属于本 change，哪些应明确留给既有 identity contract、JSONC 或其它 adapter/parser change。

本轮不实施 production code、owner 主规范、schema、tests 或 release artifacts，不重新调查已经关闭的 routing dependency 候选，也不为 JSON/JSONC/JSON5/NDJSON 选择 parser。

#### 调查范围与依据

- **用户确认的目标与边界**：纯文件名/pathname 先路由、route 命中后才考虑 I/O；完整 basename 级匹配；suffix 大小写不敏感或归一化；matched format 不传入 `StandardOperationInput`；不新增 adapter-id 唯一性设计；adapter 内部 parser mapping 留给其它 change。
- **Current source sequencing**：复核 `crates/docnav/src/runtime.rs` 对 `normalize_document_path` 的调用，以及 `crates/docnav/src/project_paths.rs` 中 metadata、`File::open`、`fs::canonicalize` 的真实执行面；这证明 Current 入口与获批 Target 的顺序不同。
- **Current/Target contract**：复核 `docs/cli.md`、`openspec/specs/core-cli/spec.md`、`docs/navigation-input-resolution.md`、routing change proposal/design/tasks 与 capability deltas，区分 Current filesystem-backed normalized document path 和 Target route 阶段尚未访问文件的 lexical pathname。
- **输入边界**：复核 closed `StandardOperationInput` 与 typed-field/routing deltas；没有发现 adapter 为完成真实 parse 而必须接收 matched suffix 或 matched format 的义务。
- **前序依赖证据**：沿用本主题前两份报告对 `mime_guess`、content detectors、manifest metadata、常见配置 aliases 与 JSON-family 的功能和重量结论。本轮没有出现能推翻 manifest-native、零新增 routing dependency 选择的新证据。
- **未覆盖**：没有实现跨 Windows/Unix separator 的 helper，也没有运行路由 benchmark 或真实 release package；这些属于 apply 阶段测试与 verification，而不是新的产品选择。

#### 调查结果与边界

##### 1. 获批 Target 必须是两阶段路径流程

Automatic path 的目标顺序应为：

```text
caller path + cwd
  -> lexical routing pathname
  -> complete basename match
  -> exact registry definition selection
  -> filesystem-backed path/access normalization
  -> closed StandardOperationInput
  -> selected adapter reads/parses
```

Route miss 在 complete-basename lookup 后直接结束，不应为了证明文件存在、是普通文件或能 canonicalize 而执行 metadata、open、read 或 filesystem canonicalization。Route hit 或 explicit adapter selection 后，既有 path/access normalization 仍有价值：它为真实 operation 提供稳定 document path，并负责正常的 missing/path/access diagnostic。该顺序同时满足“先确定能路由，再考虑 I/O”和 selected adapter 对实际文档处理的所有权。

##### 2. 不需要通用 regex；完整 basename anchored suffix 已足够

用户的核心诉求是不要先提取最后一段 extension，而不是要求 manifest 作者维护任意正则。最小且足够的契约是：

1. `filenames[]` 先对完整 basename 做大小写敏感 exact match；
2. 未命中时，把每个 `extensions[]` 值视为带前导点、可包含多个点的 anchored basename suffix；
3. basename 与 suffix 做 ASCII 大小写归一化后比较；
4. 多个不同长度 suffix 同时命中时选择最长者；
5. 归一化后完全相同的 suffix 才是重复 hint 冲突。

因此 `model.schema.JSON` 同时符合 `.json` 与 `.schema.json` 时选择 `.schema.json`，`settings.json.backup` 不符合 `.json`。这种语义在完整 basename 上工作，支持 compound suffix，却可以用普通 suffix comparison 实现；不需要 regex crate、regex DSL、glob、目录感知 pattern 或新的 detector extension point。

Exact filename 继续优先于 suffix。`.prettierrc`、`.watchmanconfig` 之类 dotfile 由 `filenames[]` 表达；`.code-workspace`、`.code-snippets`、`.schema.json` 之类可由 suffix 表达。普通 `package.json`、`tsconfig.json` 和 `.code-workspace` 是否能被真实解析仍取决于 selected JSON adapter 当时的 grammar，不由 route 保证。

##### 3. Routing match 必须保持私有

Matched filename/suffix、matched format identity 和 derived-index key 都只是选择过程的低成本中间事实。Pathname 可能故意或意外写错；把 matched format 传给 `StandardOperationInput` 不会增加真实性，反而让 adapter 可能把错误 hint 当成 parser mode 或 validity evidence。

因此 Target 保持 closed operation input 不变：selected adapter 只接收正常 document path 与 operation fields，并对真实 bytes 执行 acquisition、decode、parse 和 semantic validation。Routing state 不进入 adapter input、typed fields、protocol、readable output、ref、continuation或 invocation log；selected failure 不 fallback。

##### 4. No-match 与 selected failure 使用不同路径事实

Route miss 发生在 filesystem-backed normalization 之前，所以 `FORMAT_UNKNOWN / FORMAT_NOT_RECOGNIZED` 的 canonical `path` 只能是 lexical `<routing-pathname>`，不能声称是已经 canonicalize 的 `<normalized-path>`。Route hit 后的 missing/path/encoding/parse/operation failure 仍使用既有 post-selection normalized document path。这一区分是执行顺序的可观察后果，不是新增 public routing evidence。

##### 5. Adapter id 与 parser mapping 不属于本轮新增设计

本 change 继续消费既有 adapter identity lookup，不新增另一套 adapter-id 唯一性、冲突或 fallback policy。它只需要 manifest format/path-hint lookup 能确定一个 definition。JSON、JSONC、code adapter 或未来 adapter 如何在内部共享 parser mapping、选择 grammar/dialect 或复用事实源，由对应 adapter/change 拥有；基础 routing 只保证不把低置信度 pathname match 作为 parser input。

##### 6. 依赖结论没有变化

完整-basename suffix matching 进一步降低了外部 format/MIME table 的价值：它直接覆盖 compound suffix 和项目专用 suffix，同时 exact filename 处理 dotfile。现有调查仍支持“不引入 routing library”；`mime_guess` 仍是轻量但重复 ownership 且 alias 覆盖不足的候选，不需要增加 optional feature flag。通用 regex 也不是功能所需，因此没有新增 regex dependency 的理由。

##### 7. 当前没有新的产品决策缺口

基础格式识别 change 现在已有确定的 signal、执行顺序、匹配 precedence、大小写规则、compound suffix 规则、no-match path fact、explicit override、private handoff、no-fallback、zero-dependency 与 probe-removal scope。剩余工作是该 change 已列出的 removal inventory、cross-change handoff、十个 capability delta 的 artifact audit，以及 apply 阶段的 tests/implementation/verification；这些是执行门禁，不是需要用户再次选择的产品问题。

本报告只记录本次复查形成时的证据、结论与边界。若未来要求任意 regex/glob、目录感知 routing、Unicode case folding、content detection 或 routing-selected parser mode，必须作为新的 contract 选择调查，不能从当前 suffix 语义自动推导。
