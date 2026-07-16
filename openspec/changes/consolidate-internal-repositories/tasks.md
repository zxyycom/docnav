## 1. 阻塞级实现前审计

- [x] 1.1 审计 proposal、design、两个 delta specs 和 tasks 是否都围绕“取消四个子仓库链接、保留现有 crate/module/API 职责与运行语义、只删除子仓库边界直接产生的集成间接层”这一核心句；本组全部完成前不得执行 2.x 及后续实现任务。
- [x] 1.2 确认 proposal 只修改现有 `shared-script-tooling` 与 `cli-config-resolution` capabilities，delta 目录与 capability ID 一致，RENAMED/MODIFIED requirements 对应当前主 spec 的完整 requirements。
- [x] 1.3 确认本轮 artifact 编辑只发生在当前 change 目录，没有修改其它 docs、specs、schemas、examples 或 active change，并确认 `## Open Questions` 没有未回答问题。
- [x] 1.4 审计 `derive-cli-from-field-definitions` 的先后关系、当前 working tree 和四个 submodule 状态，记录实现入口和迁移期间停止写入的旧路径。
  - 顺序与入口（2026-07-16）：`derive-cli-from-field-definitions` 当前为 0/24；本 change 先完成实现与验证，并只在 8.1 校准该 change 的路径和验证假设。正式实现从 `d8f78da` 切出独立 change 分支，先完成 2.1–2.3，再进入 3.1/4.1 的 source import。
  - 状态基线：`main` 比 `origin/main` 超前 1 个已提交变更；根仓库没有 staged 或 tracked working-tree 修改，未跟踪内容只有本 change 目录。四个 submodule 均已初始化、内部干净，且各自 `HEAD` 与父仓库的 `160000` gitlink 一致。
  - 写入冻结：在对应 pinned tree 完成普通源码导入与等价确认前，不向 `scripts/tools/foundation/**`、`scripts/tools/parallel-task-runner/**`、`scripts/tools/quality-core/**` 或 `subrepos/cli-config-resolution/**` 写入源码、提交或 gitlink 更新。三个 TypeScript 同路径目录仅在 3.1 转为 ordinary tracked source 后恢复集成写入；Rust 变更仅写入 4.1 创建的 `crates/shared/**`，旧 `subrepos/cli-config-resolution/**` 保持只读直至移除。

## 2. 来源记录与迁移基线

- [ ] 2.1 记录四个 submodule URL、父仓库 gitlink pin、submodule commit/tree 与 dirty 状态，形成可恢复且不依赖远端可变分支的来源清单。
- [ ] 2.2 记录迁移前的根与 nested Cargo workspace 解析结果、三个脚本模块的 focused checks，以及 `bun run verify:docnav-workspace` 的完成状态，作为迁移后对照基线。
- [ ] 2.3 建立旧路径到目标路径、submodule setup/status、nested workspace/lockfile、双 workspace prefetch、路径转接、局部工具配置和 standalone repository readiness 文件清单；只有能追溯到子仓库边界的项目才纳入删除候选。

## 3. TypeScript 工具并入主仓库

- [ ] 3.1 按记录的 pin 把 `foundation`、`parallel-task-runner` 和 `quality-core` gitlinks 替换为同路径普通 tracked source，并证明导入 tree 与 pinned tree 一致。
- [ ] 3.2 将三个内部模块接入根 package、TypeScript、ESLint 和 Bun test 覆盖；保留仍服务当前仓库内开发的 focused command，不要求每个局部命令建立根入口。
- [ ] 3.3 验证 foundation → scheduler/quality 的依赖方向、现有 source API、Docnav caller imports 和 colocated tests 在普通 clone 中可解析，且没有新增 wrapper、路径别名或循环依赖。

## 4. Rust packages 并入根 Cargo workspace

- [ ] 4.1 按记录的 pin 导入 Rust workspace，并把五个 packages 分别移动到 `crates/shared/typed-fields`、`typed-fields-macros`、`cli-config-resolution`、`cli-config-resolution-clap` 和 `cli-config-resolution-serde`。
- [ ] 4.2 将五个 packages 加入根 Cargo workspace，更新 root workspace dependencies 与 path dependencies，合并 lockfile，并删除 root `exclude` 与 nested workspace/lockfile。
- [ ] 4.3 更新 package metadata、tests、trybuild fixtures 和 examples 路径，保持 package 名、proc-macro 边界、framework dependency isolation 与 public resolution 语义不变。
- [ ] 4.4 运行五个 packages 的范围化 fmt/check/test/doc 和 Docnav protocol、adapter contracts、navigation、core 的受影响 consumer tests。

## 5. 单仓库环境与验证链路

- [ ] 5.1 删除 `.gitmodules` 和四个 gitlinks，更新项目环境 setup/check，移除 submodule init/status、nested lockfile 和双 Cargo workspace dependency prefetch。
- [ ] 5.2 更新 workspace verifier definitions、path resolution 与 tests，使根 workspace 成为唯一仓库级 Rust/TypeScript 验证编排入口，局部 focused checks 由根链路调用或覆盖，并证明普通 clone 无需 recursive submodule 操作。
- [ ] 5.3 更新 quality config、accepted warnings、source discovery 与 case-catalog marker extraction/tests，使自动化覆盖移动后的全部源码。
- [ ] 5.4 运行残留审计，确认环境脚本、运行配置和验证配置不再依赖 `.gitmodules`、gitlink pin、`subrepos/cli-config-resolution`、nested Cargo workspace 或 toolkit-repository readiness。

## 6. 子仓库边界间接层清理

- [ ] 6.1 删除仅服务 TypeScript toolkit submodule checkout、revision pin 或 standalone repository lint/test 且已由根入口覆盖的 metadata/config；保留现有 source entrypoint、调用 API 和具有根调用方或 focused-check 依据的局部配置。
- [ ] 6.2 删除 Rust nested workspace、独立 repository/release readiness 与旧 subrepo path 所需的 metadata/config；不修改 canonical field、resolution 或 framework companion API。
- [ ] 6.3 确认清理项都能追溯到子仓库边界，没有修改现有 source API、typed config、task/result type、quality policy、task scheduling 或领域 owner；其它可疑抽象只记录为后续 change 输入。

## 7. 长期 owner 文档同步

- [ ] 7.1 更新 `docs/tooling.md` 中的普通 clone、根 workspace、内部脚本目录、项目环境和仓库级验证命令接线；只更新 repository role 或本地构建/聚焦检查说明发生变化的 package/module README。
- [ ] 7.2 更新 `docs/testing.md` 与 `docs/testing/cases.md`，保持测试证明目标不变并更新验证入口与源码标记映射；`docs/testing/case-maintenance.md` 与 `docs/architecture.md` 保持不变。

## 8. 在途 change 协调

- [ ] 8.1 更新 `derive-cli-from-field-definitions` 的旧路径、独立 repository/workspace 假设和验证命令，并重新运行其 artifact 审计；不得实施或勾选该 change 的行为任务。

## 9. 等价验证与交付审计

- [ ] 9.1 运行 `bun run verify:docnav-workspace` 和旧路径残留搜索，证明根 workspace 覆盖迁入源码且仓库级验证保持有效。
- [ ] 9.2 对比 2.2 的 focused/full 完成状态与源码覆盖范围，并用局部 diff、`git diff --check`、Git submodule status 和 tracked-file inventory 确认迁移只改变计划内拓扑与路径。
- [ ] 9.3 记录最终来源 pins、tree 等价证据、验证结果、保留的局部配置、聚焦的主仓库提交边界及分阶段 rollback 点。
