**一句话核心：实现正式 adapter 管理流程，让 `docnav` 能安全注册、更新、移除和审计外部 adapter。**

## Why

v0 文档明确 `docnav adapter list/install/update/remove` 是正式管理能力，不允许占位式安装更新算法。核心 CLI 路由可跑通后，需要补齐 adapter 来源解析、manifest 校验、协议兼容、安装记录和本地 exe hash 管理。

## What Changes

- 实现 `docnav adapter list`，展示已安装 adapter、manifest 身份、格式、协议范围、来源和可用状态。
- 实现 `docnav adapter install <source>`，首期支持 GitHub 链接和本地可执行文件。
- 安装和更新时执行 adapter `manifest`，校验 manifest schema 与协议兼容性。
- 本地可执行文件来源记录并验证 SHA-256 hash；hash 失配时不得静默继续调用。
- 实现 `docnav adapter update [adapter-id]`，校验失败保留旧版本并返回结构化错误。
- 实现 `docnav adapter remove <adapter-id>`，处理仍被项目配置引用时的失败或 guidance。
- 非目标：本 change 不实现新的格式 adapter，不改变 Markdown adapter 行为，不实现 MCP bridge。

## Capabilities

### New Capabilities

- `docnav-adapter-management-implementation`: 实现 adapter 安装、更新、移除、列表、安装记录、来源解析和本地可执行文件 hash 校验。

### Modified Capabilities

- 无。

## Impact

- 影响核心可执行文件：`docnav adapter ...` 管理命令。
- 影响配置和状态存储：adapter 安装记录、manifest 快照、来源 URL/路径、hash 和健康状态。
- 影响安全与可审计性：本地 exe hash 检查、协议兼容检查、更新失败回滚策略。
