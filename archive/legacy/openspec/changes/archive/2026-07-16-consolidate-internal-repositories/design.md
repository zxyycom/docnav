## Context

Docnav 当前通过四个 Git submodule 集成三组 TypeScript 工具和一个包含五个 packages 的 Rust Cargo workspace。它们尚未独立发布，当前修改、测试和验证均由 Docnav 驱动，但仓库仍需处理首次 clone 初始化、gitlink pin、nested Cargo workspace、两套 lockfile 预取，以及 quality、case catalog 和 workspace verifier 对 nested repository path 的特判。

`derive-cli-from-field-definitions` 尚无已完成任务，但其 artifacts 已同时引用主仓库与 `subrepos/cli-config-resolution`。本 change 先收敛物理边界和路径，避免同一批字段与 Clap 改动经历两次迁移。

本迁移只改变源码归属、构建拓扑和仓库级维护承诺；现有产品与工程可观察行为保持不变。

## Goals / Non-Goals

**Goals:**

- 普通 clone 直接包含四个目录的源码，不再需要 submodule 初始化、gitlink pin 或跨仓库提交协调。
- 五个 Rust packages 使用根 Cargo workspace、统一 dependency metadata、根 lockfile 和根验证入口。
- 三个 TypeScript 工具使用根 package、TypeScript、ESLint 和 test 配置，同时保留现有领域目录与 focused tests。
- 删除仅用于独立 checkout、revision pin、nested workspace 或 standalone readiness 的配置和路径特判。
- 记录来源 revision、tree 等价证据和可执行回滚点。

**Non-Goals:**

- 不删除、合并或重塑 typed-fields、resolution、foundation、scheduler、quality engine、task/result type 或其它领域抽象；相关简化由后续独立 change 承接。
- 不合并现有 Rust crates 或 TypeScript 领域目录。
- 不改变 Docnav 产品、进程、artifact 或验证结果的可观察契约。
- 不发布 crate/npm package，也不重写 archived OpenSpec change。
- 不实施 `derive-cli-from-field-definitions` 的字段、Clap 或 stage-scoped processing 行为。

## Ownership Boundaries

- `shared-script-tooling` delta 拥有三个 TypeScript 模块的主仓库源码边界与根验证接入要求；`cli-config-resolution` delta 拥有五个 Rust packages 的根 Cargo workspace membership 要求。
- `docs/tooling.md` 在实现完成后拥有普通 clone、项目环境、包管理、脚本执行方式和仓库级验证命令接线的长期说明；模块或 package README 只保留本地用途、入口和当前仓库内仍使用的 focused checks。
- `docs/architecture.md` 与现有 capability requirements 继续拥有 crate/module 的语义职责。本 change 不修改这些职责，因此不为物理路径迁移改写架构规则。
- `docs/testing.md` 拥有验证 profile 用途、测试层级与证明边界，`docs/testing/cases.md` 拥有 case 证明目标和源码标记映射；`docs/testing/case-maintenance.md` 的维护流程不因路径迁移改变。
- `derive-cli-from-field-definitions` 继续拥有字段、Clap projection 与 processing 行为；本 change 只更新其路径、workspace 假设和验证入口。

## Decisions

### Decision 1: 以 pinned source 快照建立明确的单仓库路径

实现先记录四个 submodule 的 URL、commit 与 dirty 状态，再把 pinned tree 作为普通 tracked files 导入。TypeScript 路径保持不变以避免无收益的 import churn；Rust packages 移出 `subrepos/` 并进入现有 shared-crate 目录约定。

| 当前路径 | 目标路径 |
| --- | --- |
| `scripts/tools/foundation/` | `scripts/tools/foundation/` |
| `scripts/tools/parallel-task-runner/` | `scripts/tools/parallel-task-runner/` |
| `scripts/tools/quality-core/` | `scripts/tools/quality-core/` |
| `subrepos/cli-config-resolution/crates/<package>/` | `crates/shared/<package>/` |

主仓库历史按迁移阶段记录来源导入和集成清理。来源导入提交保存 URL、pinned commit 与 tree 等价证据；这些证据足以支持追溯和恢复，不要求在主仓库重放子仓库的完整提交历史。原有子仓库的归档、可见性或删除由独立维护决定，不影响本迁移完成。

### Decision 2: 保留 package 名与现有 crate/module 职责

根 workspace 继续使用 `docnav-typed-fields`、`docnav-typed-fields-macros`、`cli-config-resolution`、`cli-config-resolution-clap` 和 `cli-config-resolution-serde` package 名。`docnav-typed-fields-macros` 的 proc-macro 约束、`docnav-typed-fields` 的多个真实消费者，以及 resolution/framework packages 的依赖隔离都构成当前边界依据。

本 change 不以单仓库为理由合并 crate、移动产品策略或修改语义 API。后续若要合并模块或调整接口，必须由调用关系、维护成本和行为验收单独证明。

### Decision 3: 根 workspace 统一集成与验证编排

根 `Cargo.toml` 与根 lockfile 拥有 Rust workspace membership、依赖集成和锁定结果；根 `package.json`、TypeScript/ESLint 配置与 workspace verifier 拥有仓库级命令和覆盖范围。nested Cargo workspace、nested lockfile 和 submodule setup/status 逻辑被删除。

Package manifest 继续拥有 package-specific metadata。局部 tsconfig、README 或 focused command 在服务当前仓库内调用方或聚焦维护时可以保留；根验证编排覆盖适用源码和 tests，但不要求直接调用每个局部命令。保留项只承接模块内信息或聚焦检查，不形成独立 checkout、版本、release 或 public entrypoint contract。

### Decision 4: 清理只覆盖子仓库边界产生的间接层

可以删除的内容限于其唯一用途是 submodule checkout、gitlink/revision pin、nested workspace/lockfile、双 workspace dependency prefetch、旧 subrepo path 转接或 standalone repository readiness 的 metadata、配置、setup/status logic 和 path adapter。

当前 in-repo callers 使用的 source API、显式 typed configuration、task/result type、framework dependency boundary、quality policy 和 task scheduling 保持不变。本 change 不以源码进入主仓库为理由删除或重塑领域抽象；后续抽象简化单独规划和验收。

### Decision 5: 使用两个可独立验证的迁移阶段

第一阶段完成 pinned source 导入、Rust 路径移动、根 workspace 接入和 gitlink 删除，并运行受影响的 focused checks。第二阶段删除 submodule/nested-workspace 特判、冗余独立工具配置并同步 owner docs，随后运行根 workspace 全量验证。

分阶段可以把路径或 dependency resolution 问题与配置清理问题分开定位。来源导入和集成清理分别形成聚焦、可独立 revert 的主仓库提交，不使用运行时 feature flag 或新旧路径兼容层，也不为保留完整子仓库历史打散迁移提交。

### Decision 6: Consolidation 先于相关 CLI field change

在本 change 的阻塞审计、实现和验证完成前，`derive-cli-from-field-definitions` 不进入实现。合并后只更新其旧路径、独立 workspace/repository 假设和验证命令，再由该 change 自己的审计门确认行为设计；本 change 不代替或预先完成其任务。

## Risks / Trade-offs

- [Risk] Rust 路径移动使 case 账本、accepted warnings、quality globs 和 active artifacts 失效 → Mitigation: 使用固定路径映射机械更新，并运行旧路径残留搜索。
- [Risk] 把仍有仓库内维护用途的局部配置误判为子仓库残留 → Mitigation: 审计当前调用方与 focused-check 用途，只删除唯一用途属于子仓库边界的文件。
- [Risk] “内部化”演变为一般性抽象重构 → Mitigation: 删除候选必须能追溯到子仓库边界；领域 API、type 和职责调整移入独立 change。
- [Risk] 在途 change 继续写入旧路径 → Mitigation: consolidation 作为前置 change，并在实现前审计 active artifacts 与 working tree。
- [Trade-off] 本轮保留五个 Rust crates 和三个 TypeScript 领域目录 → Mitigation: 先获得单仓库、单 lockfile 和原子提交收益，模块合并留给有独立证据的后续决策。

## Migration Plan

1. 完成阻塞审计，记录四个 URL、pin、dirty 状态、旧路径写入边界和迁移前验证基线。
2. 导入三个 TypeScript pinned trees，保留目录和 source imports，接入根 TypeScript/ESLint/test 配置。
3. 导入 Rust pinned tree，把五个 packages 移到 `crates/shared/`，接入根 Cargo workspace 和 lockfile。
4. 删除 `.gitmodules`、gitlinks、nested workspace/lockfile、submodule setup/status 和双 workspace 预取逻辑。
5. 更新 workspace verifier、quality、case catalog、accepted warnings、owner docs 和测试路径；删除只服务独立仓库集成且已被根入口覆盖的配置。
6. 校准 `derive-cli-from-field-definitions` 的路径与验证假设，不执行其行为任务。
7. 运行根 workspace 全量验证、旧路径残留审计和基线对比，记录来源、结果和回滚点。

回滚按阶段提交执行：配置清理可单独 revert；若根 workspace 集成失败，则 revert source import/path migration，并使用记录的 pin 恢复 `.gitmodules` 与 gitlinks。

## Open Questions

无未回答开放问题；阻塞级实现前审计已完成，可以进入来源记录与迁移基线。
