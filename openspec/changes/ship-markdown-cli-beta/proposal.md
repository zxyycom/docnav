**一句话核心：冻结推测性扩展，把当前 Markdown 能力交付为可公开安装、可按真实路径体验、限制透明的 Docnav CLI Beta；本文是仅位于本 change 目录下的未审核临时提案，不影响现有主规范或其它 change。**

## Why

Docnav 已经具备经过验证的核心 CLI、Markdown adapter 和发布包链路，但当前发布产物仍偏向工程验收，用户缺少从获取二进制到完成一次 `outline -> ref -> read` 的公开路径。继续扩展 MCP、交互能力或预判式体验优化，无法回答当前产品是否已经对真实用户有价值；现在应先交付一个范围诚实的 Markdown CLI Beta，并让后续工作由真实使用证据驱动。

## What Changes

- 将当前 CLI 定位为 Markdown-only Beta，并发布一个版本化的公开 prerelease；首期继续使用现有 `x86_64-unknown-linux-gnu` 与 `x86_64-pc-windows-msvc` 支持范围。
- 从已通过 package verification 和 smoke 的 canonical `package/` 生成 target-qualified 公共下载文件，确保多个 target 可以同时附加到同一 prerelease，且不改变内部 package 布局。
- 在 README 提供最小 Quick Start：下载与校验、运行 `version`、对一个真实 Markdown 文档执行 `outline`、把返回 ref 传给 `read`，并展示 `find` 的继续读取路径。
- 明确 Beta 的支持范围、已知限制和反馈入口；不增加自动遥测，也不把产品验证包装成未经观测的效果结论。
- 增加面向已发布文件的 Beta acceptance：从公共下载形状安装或准备二进制，并逐字执行 Quick Start 的代表命令，证明文档与真实制品一致。
- 非目标：本 change 不实现 MCP bridge、local service mode、interactive outline selection、outline preview/skim、更多 operation composition、新格式 adapter、全面 UX 重做或新的产品评测平台；这些工作不作为 Beta 发布依赖。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `release-artifacts`: 在现有 canonical package 和 CI 验证契约之上，增加可公开下载的 target-qualified Beta prerelease 文件、发布前验收和 Quick Start 可执行性要求。

## Impact

- 影响 `README.md`、`docs/testing/release.md` 以及对应发布测试资料。
- 影响 `.github/workflows/release-package.yml` 和 release-package 脚本的公开 prerelease 发布阶段，但不改变 Cargo release profile、canonical `package/` 内容或 manifest/hash 语义。
- 影响发布 smoke/acceptance fixture 与 CI 权限边界；正式发布动作仍只在干净 CI checkout 和显式 release 触发条件下发生。
- 不改变 CLI command/flag、adapter contract、protocol envelope、schema、ref grammar、配置语义或 Markdown 解析行为。
