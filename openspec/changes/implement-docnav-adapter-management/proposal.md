**一句话核心：实现正式 adapter 管理流程，维护用户级安装记录和项目级 id/version 策略记录，供核心 resolver 选择可调用 adapter。**

## Why

v0 文档明确 `docnav adapter list/install/update/remove` 是正式管理能力。核心 CLI 路由跑通后，需要补齐内置 adapter 下载简写、本地 adapter 登记、manifest 当前契约校验、用户级安装 registry、项目级 adapter 策略 registry 和低频 SHA-256 fingerprint 管理。

该 change 同时划清两类状态的所有权：用户机器保存真实命令路径和来源信息；项目只保存可共享的 id/version 策略，避免把本机路径、用户名目录或来源细节写入项目配置。管理命令负责维护这些记录；运行期 adapter 选择由核心 resolver 消费这些记录完成。

## What Changes

- 实现 `docnav adapter list`，展示已安装 adapter、版本、manifest 身份、格式、来源、项目策略命中和可用状态。
- 实现 `docnav adapter install <source> [--mode managed|path]`，首期只支持内置 adapter 下载简写和本地可执行文件；内置下载简写固定使用 `managed`，本地可执行文件可选择托管安装或路径登记。
- 实现 `docnav adapter register <local-exe>` 作为 `docnav adapter install <local-exe> --mode path` 的语义别名，用于本地 adapter 开发调试。
- 不支持任意 URL 或 GitHub 链接动态下载；非内置来源一律走本地可执行文件方案。
- 安装和更新时执行 adapter `manifest`，校验 manifest 当前 schema、必需字段、字段类型和语义。
- 用户级安装 registry 使用 adapter id 作为一级键、adapter version 作为二级键；版本记录保存命令路径、install mode、来源、manifest 快照、SHA-256 fingerprint 和健康状态。
- 托管安装的 adapter 可执行文件存放在 `docnav` 管理的用户级 artifact 目录，不存放在配置文件目录；registry 只记录路径和元数据。
- 项目级 adapter 策略 registry 只保存 id/version 级 allowlist、denylist 和当前版本选择；不保存本机命令路径、用户名目录、来源 URL 或 fingerprint。
- adapter 可执行文件记录 SHA-256 fingerprint；fingerprint 在 install/register/update 和显式健康检查中验证，普通文档操作不为 fingerprint 读取整个可执行文件。
- 实现 `docnav adapter update [adapter-id] [--version <version>]`，校验失败保留旧记录并返回结构化错误。
- 实现 `docnav adapter remove <adapter-id> [--version <version>]`，处理仍被项目策略 registry 引用时的失败或 guidance。

## Capabilities

### New Capabilities

- `adapter-management`: 定义 adapter 安装、更新、移除、列表、用户级安装 registry、项目级 adapter 策略 registry、内置下载简写、本地可执行文件来源和低频 SHA-256 fingerprint 校验。

### Modified Capabilities

- 无。

## Impact

- 影响核心可执行文件：`docnav adapter ...` 管理命令。
- 影响配置、状态和本机制品存储：用户级 adapter 安装 registry、项目级 adapter 策略 registry、用户级 managed adapter artifact 目录、manifest 快照、install mode、来源信息、SHA-256 fingerprint 和健康状态。
- 影响完整性与可审计性：低频 fingerprint 检查、manifest 当前契约校验、项目策略解析和更新失败回滚策略。
