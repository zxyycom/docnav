## 1. 阻塞级实现前审计

- [x] 1.1 审计 proposal、design、两个 delta specs 和 tasks 是否都围绕“取消四个子仓库链接、保留现有 crate/module/API 职责与运行语义、只删除子仓库边界直接产生的集成间接层”这一核心句；本组全部完成前不得执行 2.x 及后续实现任务。
- [x] 1.2 确认 proposal 只修改现有 `shared-script-tooling` 与 `cli-config-resolution` capabilities，delta 目录与 capability ID 一致，RENAMED/MODIFIED requirements 对应当前主 spec 的完整 requirements。
- [x] 1.3 确认本轮 artifact 编辑只发生在当前 change 目录，没有修改其它 docs、specs、schemas、examples 或 active change，并确认 `## Open Questions` 没有未回答问题。
- [x] 1.4 审计 `derive-cli-from-field-definitions` 的先后关系、当前 working tree 和四个 submodule 状态，记录实现入口和迁移期间停止写入的旧路径。
  - 顺序与入口（2026-07-16）：`derive-cli-from-field-definitions` 当前为 0/24；本 change 先完成实现与验证，并只在 8.1 校准该 change 的路径和验证假设。正式实现从 `d8f78da` 切出独立 change 分支，先完成 2.1–2.3，再进入 3.1/4.1 的 source import。
  - 状态基线：`main` 比 `origin/main` 超前 1 个已提交变更；根仓库没有 staged 或 tracked working-tree 修改，未跟踪内容只有本 change 目录。四个 submodule 均已初始化、内部干净，且各自 `HEAD` 与父仓库的 `160000` gitlink 一致。
  - 写入冻结：在对应 pinned tree 完成普通源码导入与等价确认前，不向 `scripts/tools/foundation/**`、`scripts/tools/parallel-task-runner/**`、`scripts/tools/quality-core/**` 或 `subrepos/cli-config-resolution/**` 写入源码、提交或 gitlink 更新。三个 TypeScript 同路径目录仅在 3.1 转为 ordinary tracked source 后恢复集成写入；Rust 变更仅写入 4.1 创建的 `crates/shared/**`，旧 `subrepos/cli-config-resolution/**` 保持只读直至移除。

## 2. 来源记录与迁移基线

- [x] 2.1 记录四个 submodule URL、父仓库 gitlink pin、submodule commit/tree 与 dirty 状态，形成可恢复且不依赖远端可变分支的来源清单。
  - 来源清单（2026-07-16，父仓库基线 `0f5c96c831577e84ae018f908b91375875f1d9f8`）：

    | 路径 | URL | gitlink pin / checkout HEAD | pinned tree | dirty |
    | --- | --- | --- | --- | --- |
    | `scripts/tools/foundation` | `https://github.com/zxyycom/script-tool-foundation.git` | `f593edbf55fd03be7db54ef44a38d0a9feda4dbd` | `c9dd5b6231cbd9eac27af677c423546cf2c204fb` | clean |
    | `scripts/tools/parallel-task-runner` | `https://github.com/zxyycom/script-tool-parallel-task-runner.git` | `025af7350e2d624eeded23784f411bec5f4a1473` | `9e646fa6b442edc0c75ee783cfa29639cdd16240` | clean |
    | `scripts/tools/quality-core` | `https://github.com/zxyycom/script-tool-quality-core.git` | `3acea8c2f643ea86f7a1e8f2a6db716b7e320c76` | `0b78bd41de78e47687f170899326a6b061d14cc8` | clean |
    | `subrepos/cli-config-resolution` | `https://github.com/zxyycom/cli-config-resolution` | `7a45be6cf47c5431656f980ddebc991b55c56748` | `8799bd9c4271c58e2b98f3eff4ba7a8aa60baf09` | clean |

  - 四个 checkout 的 `HEAD` 都与父仓库 `160000` entry 一致；恢复时按表中 URL 获取对象并直接 checkout 对应 pin，不依赖 `main` 或其它可变分支。
- [x] 2.2 记录迁移前的根与 nested Cargo workspace 解析结果、三个脚本模块的 focused checks，以及 `bun run verify:docnav-workspace` 的完成状态，作为迁移后对照基线。
  - `cargo metadata --locked --offline --format-version 1 --no-deps`：根 workspace 解析为 11 个成员并使用根 `target/`；nested workspace 解析为 5 个成员并使用 `subrepos/cli-config-resolution/target/`。迁移前 lockfile SHA-256 分别为根 `6a514c78d645fee1ad191db092b813e8bceaeb11f13d2b5fbcbe734e5cc00e90`、nested `409c56db471ab41099fd94f7d944bccb68d01758bff89e941040147f442ddb9a`。
  - 三个模块的局部 `typecheck`、`lint`、`test` 全部通过：foundation 4 tests、parallel-task-runner 10 tests、quality-core 30 tests，均 0 failure。
  - `bun run verify:docnav-workspace` full profile 通过 19/19 checks、0 warning、0 failure，耗时 25s；其中根与 nested Cargo checks、resolution flow、脚本 checks、quality、smoke、docs 和 OpenSpec 均通过。
- [x] 2.3 建立旧路径到目标路径、submodule setup/status、nested workspace/lockfile、双 workspace prefetch、路径转接、局部工具配置和 standalone repository readiness 文件清单；只有能追溯到子仓库边界的项目才纳入删除候选。
  - 路径映射：三个 `scripts/tools/*` 目录保持原路径；`subrepos/cli-config-resolution/crates/{typed-fields,typed-fields-macros,cli-config-resolution,cli-config-resolution-clap,cli-config-resolution-serde}` 分别迁到同名 `crates/shared/*`。
  - 删除候选：`.gitmodules`、release checkout 的 `submodules: recursive`、`scripts/project-environment/{index.ts,workspaces.test.ts}` 中的 init/status 与双 workspace loop、`scripts/docnav-workspace/checks/definitions.ts` 中的 nested-workspace checks、根 `Cargo.toml` 的 `exclude`/旧 path、nested 根级 `.gitignore`/`Cargo.toml`/`Cargo.lock`/`README.md`，以及 quality-core 中只为 gitlink discovery/materialization 服务的 `src/input/{revision-tree.ts,revision-materialization.ts}` 与其在 `files.test.ts`/`revisions.ts` 的接线。
  - 路径更新而非删除：`scripts/quality/{config.ts,config.test.ts,accepted-warnings.ts}`、case-catalog source roots/tests、`docs/testing/cases.md`、`docs/tooling.md`、`docs/testing.md`、crate/module README 和 `derive-cli-from-field-definitions` artifacts。
  - 保留项：三个 TypeScript 模块的 source entrypoint、领域 API、tests、private `package.json`、`tsconfig.json` 和 README；这些文件仍有根调用方与局部 typecheck/lint/test 依据。五个 crate 边界、package 名、package manifests、crate README、tests、trybuild fixtures 和 example 同样保留并只更新 workspace/path metadata。

## 3. TypeScript 工具并入主仓库

- [x] 3.1 按记录的 pin 把 `foundation`、`parallel-task-runner` 和 `quality-core` gitlinks 替换为同路径普通 tracked source，并证明导入 tree 与 pinned tree 一致。
  - staged subtree 分别为 `c9dd5b6231cbd9eac27af677c423546cf2c204fb`、`9e646fa6b442edc0c75ee783cfa29639cdd16240`、`0b78bd41de78e47687f170899326a6b061d14cc8`，与 2.1 的三个 pinned tree 逐项相等。
- [x] 3.2 将三个内部模块接入根 package、TypeScript、ESLint 和 Bun test 覆盖；保留仍服务当前仓库内开发的 focused command，不要求每个局部命令建立根入口。
  - 根 `tsconfig.json` 与 ESLint `scripts/**/*.ts` glob 已直接覆盖三个目录；根 `test:workspace-verifier` 覆盖 foundation/scheduler tests，`quality:test` 覆盖 quality-core。导入后根 typecheck、lint 与 workspace tests（44 tests）通过；三个局部 typecheck/lint/test 仍分别通过 4、10、30 tests。
- [x] 3.3 验证 foundation → scheduler/quality 的依赖方向、现有 source API、Docnav caller imports 和 colocated tests 在普通 clone 中可解析，且没有新增 wrapper、路径别名或循环依赖。
  - source imports 保持真实相对路径；foundation 不反向依赖 scheduler/quality，scheduler/quality 继续复用 foundation，Docnav callers 直接导入原 entrypoint。CodeGraph blast-radius 审计与根 typecheck/test 均未发现循环、wrapper 或 alias。

## 4. Rust packages 并入根 Cargo workspace

- [x] 4.1 按记录的 pin 导入 Rust workspace，并把五个 packages 分别移动到 `crates/shared/typed-fields`、`typed-fields-macros`、`cli-config-resolution`、`cli-config-resolution-clap` 和 `cli-config-resolution-serde`。
  - 五个目标 subtree 依次为 `5de721cb03f89b38d82b9cb8b10d790f7b52b1be`、`69cda2a38740885798551b4aeb1c1d6976edca3e`、`481027f3dcf341cd41b3f2fe68aa6c7b0ea72120`、`b16ac8887efcd42fc8177d2965f568e2191f4dfe`、`382337b87e90460ff79c62483b068009a57e73bf`，与 pinned source 对应 package tree 逐项相等。
- [x] 4.2 将五个 packages 加入根 Cargo workspace，更新 root workspace dependencies 与 path dependencies，合并 lockfile，并删除 root `exclude` 与 nested workspace/lockfile。
  - 根 metadata 现解析为 16 个成员、唯一 `/workspace/docnav/target`；五个 dependency path 均指向 `crates/shared/*`，根 `exclude`、nested manifest/lockfile/target 已移除。根 lockfile 离线加入 moved package tests 所需的 14 个锁定条目，并可由 `--locked` 重放。
- [x] 4.3 更新 package metadata、tests、trybuild fixtures 和 examples 路径，保持 package 名、proc-macro 边界、framework dependency isolation 与 public resolution 语义不变。
  - 五个 manifest 复用根 workspace license/repository metadata，package 与 crate 名、proc-macro 类型及依赖方向未变；README/example 链接改为根 workspace 语义，moved crates 内旧 subrepo/workspace 路径搜索为空。
- [x] 4.4 运行五个 packages 的范围化 fmt/check/test/doc 和 Docnav protocol、adapter contracts、navigation、core 的受影响 consumer tests。
  - `cargo fmt --all --check`、五个 package 的 locked check/test/doc、`resolution_flow` example，以及 protocol、adapter-contracts、navigation、docnav consumer tests 均通过；trybuild 在仓库 mise Rust 1.96 + rust-src 工具链下通过。

## 5. 单仓库环境与验证链路

- [x] 5.1 删除 `.gitmodules` 和四个 gitlinks，更新项目环境 setup/check，移除 submodule init/status、nested lockfile 和双 Cargo workspace dependency prefetch。
  - 环境 setup/check 现只安装根依赖、执行一次根 `cargo fetch/metadata` 并同步 CodeGraph；release checkout 不再请求 recursive submodules。
- [x] 5.2 更新 workspace verifier definitions、path resolution 与 tests，使根 workspace 成为唯一仓库级 Rust/TypeScript 验证编排入口，局部 focused checks 由根链路调用或覆盖，并证明普通 clone 无需 recursive submodule 操作。
  - verifier 删除 nested manifest checks，保留从根运行的 resolution example；`bun run env:check` 报告根 workspace 16 members，workspace verifier 42 个测试通过。
- [x] 5.3 更新 quality config、accepted warnings、source discovery 与 case-catalog marker extraction/tests，使自动化覆盖移动后的全部源码。
  - quality source discovery、area fixtures、accepted-warning paths 与 case marker roots 均改为根相对路径；quality 37 个测试与 validator focused tests 通过。
- [x] 5.4 运行残留审计，确认环境脚本、运行配置和验证配置不再依赖 `.gitmodules`、gitlink pin、`subrepos/cli-config-resolution`、nested Cargo workspace 或 toolkit-repository readiness。
  - 排除本 change 的迁移记录后，运行配置、验证配置、owner docs 与在途 change 的旧路径/边界关键词搜索为空；`git submodule status`、mode `160000` inventory、`git ls-files subrepos` 与 nested `.git` 搜索均为空。

## 6. 子仓库边界间接层清理

- [x] 6.1 删除仅服务 TypeScript toolkit submodule checkout、revision pin 或 standalone repository lint/test 且已由根入口覆盖的 metadata/config；保留现有 source entrypoint、调用 API 和具有根调用方或 focused-check 依据的局部配置。
  - 删除 gitlink-aware revision traversal/materialization 与 submodule workspace verifier；保留三个 private manifest/tsconfig 作为根 full profile 调用的 focused checks，source entrypoint 与调用 API 未变。
- [x] 6.2 删除 Rust nested workspace、独立 repository/release readiness 与旧 subrepo path 所需的 metadata/config；不修改 canonical field、resolution 或 framework companion API。
  - 删除 nested root manifest/lockfile/README/target 并由根 workspace metadata 接管；五个 crate 的 Rust source、tests、fixtures 与 public API 保持导入内容，仅路径和 workspace metadata 改变。
- [x] 6.3 确认清理项都能追溯到子仓库边界，没有修改现有 source API、typed config、task/result type、quality policy、task scheduling 或领域 owner；其它可疑抽象只记录为后续 change 输入。
  - 局部 diff 显示 Rust `src/**` 保持导入内容，foundation/parallel 只改 README；quality-core 只删除 gitlink traversal/materialization 并以原有 Git helper 处理单仓库路径，threshold、typed config、task/result 与调度语义未变。项目环境的 plain-text child env 因依赖安装前 bootstrap 仍有根调用依据，未误删。

## 7. 长期 owner 文档同步

- [x] 7.1 更新 `docs/tooling.md` 中的普通 clone、根 workspace、内部脚本目录、项目环境和仓库级验证命令接线；只更新 repository role 或本地构建/聚焦检查说明发生变化的 package/module README。
  - tooling owner 与八个迁入模块/package README 已改为普通跟踪源码、根 workspace 和 focused-check 语义；文档 outline 可由 `dnm` 正常读取。
- [x] 7.2 更新 `docs/testing.md` 与 `docs/testing/cases.md`，保持测试证明目标不变并更新验证入口与源码标记映射；`docs/testing/case-maintenance.md` 与 `docs/architecture.md` 保持不变。
  - case catalog 仍为 103 implemented / 103 markers，moved Code paths 与三项自动化证明已同步；两份要求保持的 owner 文档 diff 为空。

## 8. 在途 change 协调

- [x] 8.1 更新 `derive-cli-from-field-definitions` 的旧路径、独立 repository/workspace 假设和验证命令，并重新运行其 artifact 审计；不得实施或勾选该 change 的行为任务。
  - proposal/design/spec/tasks 已改为根 workspace 路径与 consumer-neutral crate dependency boundary；旧假设搜索为空，artifact status 完整且 strict validation 通过，原行为任务均未勾选。

## 9. 等价验证与交付审计

- [x] 9.1 运行 `bun run verify:docnav-workspace` 和旧路径残留搜索，证明根 workspace 覆盖迁入源码且仓库级验证保持有效。
  - full profile 15/15 passed、0 warning、0 failed；根 cargo fmt/clippy/test 覆盖 16-member workspace，TypeScript typecheck/lint、full quality、docs/OpenSpec、CLI smoke 与 resolution example 全部通过，运行/验证范围旧路径搜索为空。
- [x] 9.2 对比 2.2 的 focused/full 完成状态与源码覆盖范围，并用局部 diff、`git diff --check`、Git submodule status 和 tracked-file inventory 确认迁移只改变计划内拓扑与路径。
  - 三个 TypeScript focused tests 仍为 4/10/30 passed，五个 Rust package 与四个 consumer test 集保持通过；baseline 19 checks 中四个 nested-workspace 重复 checks 收敛进根 cargo checks，最终为 15/15。working/index diff check、submodule status、gitlink mode、subrepos inventory 与 nested `.git` 审计均通过。
- [x] 9.3 记录最终来源 pins、tree 等价证据、验证结果、保留的局部配置、聚焦的主仓库提交边界及分阶段 rollback 点。
  - 2.1/4.1 保留四个 pins 与逐树等价哈希；保留三个 TypeScript private manifest/tsconfig 及 bootstrap child-env 配置。建议提交/回滚边界为：（1）普通源码导入、Cargo workspace/lockfile 与路径接线；（2）subrepo-aware 自动化删除、质量/案例/owner docs/OpenSpec 同步。任一阶段均可按该边界回退，不需要兼容 wrapper。
