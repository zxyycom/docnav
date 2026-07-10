# create-universal-cli-config-crate

从 Docnav typed-fields 和 parameter-resolution 抽象出可作为子仓库复用的 Rust CLI/config 解析底层 crate，统一 flag、env、config、默认值合并和来源追踪。

本 change 先在 Docnav workspace 内形成可验证的 core、`clap` companion 和 `serde_json` companion package 边界。Docnav 仍拥有 command、config layout、adapter、operation、protocol、diagnostic code 和 output 语义；独立 repository 创建和外部发布不在本记录的执行范围内。

2026-07-09 的 prompt-optimize artifact 审阅结论：Docnav 集成采用 hard cutover，不做渐进运行时迁移。工作区实现使用 `cli-config-resolution` 作为 capability 与 crate/package 工作名；外部 package 名默认沿用 `cli-config-resolution`，子仓库化默认迁移到独立 repository。release-readiness 审计发现外部包名不可用、仓库策略冲突、发布渠道风险任一问题时，执行者必须主动向用户确认后再继续。

## 8.1 Release readiness

### Package name evidence

在 2026-07-10T01:07:29Z，crates.io 官方 API 对以下 package 名均返回 HTTP 404：

- `cli-config-resolution`
- `cli-config-resolution-clap`
- `cli-config-resolution-serde`

该结果只证明检查时没有对应 crate 记录，不构成名称预留或后续可用性承诺。任何外部发布动作前必须重新检查名称。

### License evidence

三个 package manifest 均通过 `license.workspace = true` 继承 workspace 的 MIT SPDX metadata。该 metadata 是本 change 的 license 证据；本 change 不增加额外 license abstraction。

### Package matrix

| Package | Consumer role | Normal dependencies | Consumer material |
| --- | --- | --- | --- |
| `cli-config-resolution` | Framework-independent field/source contracts、ordered resolution、merge、diagnostics、materialization 和 provenance | 无 | package README 的 default-source 最小示例 |
| `cli-config-resolution-clap` | 从 CLI projection 构造 `clap` arguments，并将 matches 映射为 core candidates | `cli-config-resolution`、`clap` | package README；`examples/resolution_flow.rs` |
| `cli-config-resolution-serde` | 将 `serde_json::Value` 的 config path 映射为 core candidates | `cli-config-resolution`、`serde_json` | package README；由 `resolution_flow` 联合覆盖 |

三个 README 都只描述 package 消费者需要的用途、入口和稳定性边界。`resolution_flow` 是跨 package 的 runnable example，覆盖 CLI、env、JSON config、default、list/map merge、conflict diagnostic 和 provenance explain；运行入口为 `cargo run -p cli-config-resolution-clap --example resolution_flow`。

当前 workspace version 为 `0.1.0`。三个 README 均明确记录 pre-1.0 public API 尚无兼容性保证；本记录不承诺发布日期、发布节奏或后续 API 稳定级别。derive macro 继续留在后续独立 change。

## 8.2 Independent repository migration record

### Status and repository metadata

本节是迁移准备记录，不执行 repository 创建、代码搬迁或外部发布。真实独立 repository 尚未存在，因此三个 package manifest 有意不设置 `repository` URL；repository 建立并确认 canonical URL 后，才在目标 repository 中为三个 package 设置一致、真实的 metadata。

### Repository boundary

迁入独立 repository 的边界：

- 三个 package 的 source、manifest、package README、tests，以及 `cli-config-resolution-clap/examples/resolution_flow.rs`。
- 三个 package 所需的 workspace-level version、edition、MIT SPDX 和 shared dependency metadata，由目标 repository 的 workspace manifest 承接。
- core 保持不依赖 Docnav protocol、adapter contracts、navigation、output 和 Markdown adapter crates；framework dependency 继续只存在于 companion package。

保留在 Docnav repository 的边界：

- Docnav command/config layout、adapter/operation applicability、protocol/output projection 和 diagnostic code mapping。
- Docnav hard-cutover integration、等价测试、workspace 验证链和本 OpenSpec change 的审计历史。

### Migration impact

1. Package 和 Rust crate 名保持不变，消费者 import path 不因 repository 迁移而重命名。
2. 独立 repository 必须先提供可解析的 core dependency，companion package 才能脱离当前 workspace path dependency；迁移验证不得依赖 Docnav 本地路径。
3. Docnav 随后把三个 workspace dependency 从本地 path 切换到经批准的外部 source，并更新 lockfile；该 source 的类型和真实 URL 由迁移执行时确认，本记录不预设。
4. 依赖 source 切换不得改变 Docnav 的 hard-cutover runtime behavior。切换后需要重跑 package tests、Docnav config/native-option 等价测试和 workspace verification。
5. crates.io 名称必须在任何发布动作前重新检查；本次 404 证据不能替代执行时检查。

### Rollback path

1. 在 Docnav dependency source 切换前，迁移可以停止，三个 package 继续作为当前 workspace member 构建和测试。
2. 若独立 repository 验证失败，修复在独立边界完成；需要撤回迁移时，恢复当前 package directories 和 path dependency metadata，不引入临时兼容 wrapper。
3. 若 Docnav 已切换 dependency source但验证失败，回滚 dependency metadata 和 lockfile 到已验证的 workspace path source，并重跑 hard-cutover 等价测试。
4. repository 迁移的回滚只回退 package location 和 dependency source，不恢复旧 fixed source resolver、runtime feature flag 或 fallback path。
5. 外部发布属于单独审批边界；本记录不创建发布 artifact，也不把外部 artifact 的处置作为可由代码 revert 完成的回滚步骤。

### Migration gates

- 独立 repository owner、canonical URL 和 repository policy 已真实确认。
- 三个 package 在独立 checkout 中通过 metadata、package listing、tests 和 example verification。
- package 名在发布动作前重新检查，version 和发布顺序另行批准。
- Docnav dependency source 切换后通过 hard-cutover 等价测试和 workspace verification。

## Verification evidence

### Final acceptance at 2026-07-10T13:18:57+08:00

本节以当前 worktree 的新鲜验证结果取代此前暂时失效的记录；未复用历史命令结果。

- 格式与 lint：`cargo fmt --all -- --check` 通过；三个新 package 的 `cargo clippy --all-targets -- -D warnings` 通过。
- Package tests：core 18 passed；clap companion 5 passed；serde companion 4 passed；各自 doc-tests 均 0 failed。
- Docnav tests：`docnav-typed-fields` 45 unit tests 加 1 compile test passed；`docnav-typed-fields-macros` package 存在且 unit/doc-tests 0 failed；`docnav-navigation --lib` 42 passed；`docnav --lib` 92 passed。
- Case ledger：`bun run validate:docs -- cases` 通过，101 implemented、1 planned、101 unique source markers。
- Package boundary：`cargo metadata --no-deps --format-version 1` 显示三个新 package 均存在并已被各自测试命令独立选择，旧 `parameter-resolution` package 缺席；core 的 dependency 列表为空。`cargo tree` 进一步显示 core 仅包含自身，clap/serde companion 仅引入各自 framework dependency 与 core。
- Workspace gate：`bun run verify:docnav-workspace` 退出码为 0，14 项检查中 13 passed、1 warning、0 failed。唯一 warning 是非阻断 `quality full check`：24 条未接受原因记录（20 changed、20 regressions），细分为 10 条 file code-lines、9 条 function code-density、4 条 cyclomatic complexity、1 条 parameter count；完整报告位于 `artifacts/docnav-quality/report.md`。同次 gate 的 clippy、tests、format、docs、OpenSpec、smoke 和 duplication checks 均通过。
- Scope audit：`git diff --check` 退出码为 0，仅输出 `Cargo.lock` 的 LF-to-CRLF working-copy advisory。status/diff 范围限定在 workspace manifests/lockfile、三个新 package、Docnav/navigation hard cutover、owner docs/case ledger 和本 change artifacts；core source/manifest 未出现 Docnav、adapter、protocol、output 或 Markdown 专属语义。旧 resolver package、运行时 import、feature/fallback path 均缺席；原目录下文件数为 0。
- OpenSpec：记账前与记账后的 `openspec validate create-universal-cli-config-crate --type change --strict --no-interactive` 均退出 0；记账后 `openspec instructions apply --change create-universal-cli-config-crate --json` 返回 `state=all_done`、46/46 complete、0 remaining。
- 发布边界未改变：三个 crates.io package 名的可用性仍只是 2026-07-10T01:07:29Z 的时间点证据，不构成预留；真实独立 repository 尚未创建，因此 canonical repository URL 继续延后到 repository 确认后设置。
