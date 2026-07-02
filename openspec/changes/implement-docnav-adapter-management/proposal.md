**一句话核心：该 change 已被 `adopt-core-linked-adapter-libraries` 收敛为 core release 内置 adapter inspection；不再实现动态 adapter 安装、注册、更新或移除。**

## Why

`adopt-core-linked-adapter-libraries` 决定默认文档操作的 adapter implementation source 只来自当前 core release 编译进来的 adapter-layer workspace crates 和 static registry。历史动态 adapter 管理会重新引入外部 executable、command path、用户级 artifact record 和运行时 registry，和新的默认边界冲突。

## What Changes

- 保留 `docnav adapter list`，但它只展示 core release static registry 中的内置 adapter metadata。
- 删除 `docnav adapter install/register/update/remove` 作为默认 CLI surface 的目标；这些命令应作为无效子命令失败。
- 不再设计或实现用户级 adapter 安装 registry、项目级 adapter 策略 registry、managed artifact 目录、fingerprint、download source key 或 path registration。
- `docnav doctor` 检查项目/用户配置、static registry 和 adapter layer 可用性，不检查历史安装记录。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `adapter-management`: 收敛为 static adapter inspection 和 dynamic management removal，不再定义运行时 adapter artifact 管理。

## Impact

- 影响 CLI 文档和测试：只保留 `adapter list`，并覆盖 `install/register/update/remove` 不再有效。
- 影响历史 OpenSpec 审计：本 change 不应再作为动态制品管理的实现依据；相关需求以 `adopt-core-linked-adapter-libraries` 的 static registry 边界为准。
