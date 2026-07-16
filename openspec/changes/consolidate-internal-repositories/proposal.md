## Why

`scripts/tools/foundation`、`scripts/tools/parallel-task-runner`、`scripts/tools/quality-core` 和 `subrepos/cli-config-resolution` 的独立仓库边界原本服务于其它项目复用；该消费者需求已取消，而当前修改、测试和验证仍由 Docnav 驱动。继续保留 gitlink、独立 workspace、revision pin 和跨仓库提交顺序只增加初始化、原子变更、CI 与维护成本，因此应在这些 packages 尚未独立发布前完成收敛。

## What Changes

- 仓库拓扑：取消四个目录的 Git submodule、独立 checkout 和 revision pin 集成面；当前 pinned source 作为普通 tracked files 并入 Docnav 主仓库。
- 把 `subrepos/cli-config-resolution` 的五个 Rust packages 移到 `crates/shared/` 并纳入根 Cargo workspace、统一 lockfile 和根验证链路；package 名、crate 边界与 resolution 语义保持不变。
- 保持三个 TypeScript 工具的现有目录、source API 和职责划分，让根工具配置覆盖 typecheck、lint 和 tests；仍服务当前仓库内开发的 focused command 与局部配置可以保留，只删除专门服务 submodule checkout、revision pin、standalone repository readiness 或旧路径转接的间接层。
- 将环境初始化、workspace verifier、quality scan、case catalog、owner docs 和测试路径改为单仓库语义，并用迁移前后等价证据证明现有工程入口与可观察行为保持有效。
- 让 `consolidate-internal-repositories` 先于 `derive-cli-from-field-definitions` 实现；合并后只校准后者的路径、workspace 假设和验证命令，不在本 change 中实施其行为任务。
- 非目标：本 change 不删除、合并或重塑 typed-fields/resolution、脚本 engine、task/result type 或其它领域抽象，不处理原有子仓库的后续生命周期，不发布 crates/npm package，也不改写 archived change。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `shared-script-tooling`：让三个脚本工具成为主仓库内普通源码并纳入根验证链路，删除仅由 submodule checkout、revision pin 和 standalone repository readiness 产生的集成要求；现有通用内核、Docnav 策略和 typed config 边界保持不变。
- `cli-config-resolution`：以 Docnav 根 Cargo workspace packages 替代 independently checkoutable Cargo workspace repository，同时保持 canonical field、resolution 与 framework adapter 语义。

## Impact

- 仓库与依赖：`.gitmodules`、四个 gitlink、根 `Cargo.toml` / lockfile 和 nested workspace metadata。
- TypeScript 工具链：`scripts/tools/**`、根 package/TypeScript/ESLint 配置、项目环境初始化、workspace verifier、quality scan 和 focused tests。
- Rust 代码与测试：`typed-fields`、proc-macro、resolution core、clap/serde companions 的路径、workspace membership、package metadata、examples 和 tests；运行语义保持不变。
- 规范与验证材料：工程工具链、测试策略与 case 账本，以及仍引用 `subrepos/cli-config-resolution` 或共享脚本子仓库路径的当前 OpenSpec artifacts。
- 产品契约：`docnav` CLI、adapter、protocol、ref、readable/raw output、schema 和 examples 不发生行为变化。
