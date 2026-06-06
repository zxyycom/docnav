一句话核心：实现统一 CLI 发布产物目录，并让发布验证脚本只从该目录消费产物。

## 0. 审计门禁

- [ ] 0.1 用户审计确认：用户已审计本 change 的 proposal、design、spec 和 tasks，并明确允许开始实现；未完成本项前，1.x 及后续任务全部处于阻塞状态。

## 1. 打包产物目录与脚本

- [ ] 1.1 （阻塞：等待 0.1 用户审计确认）确定打包脚本入口和参数，支持默认 host target 与显式 `--target <triple>`。
- [ ] 1.2 （阻塞：等待 0.1 用户审计确认）实现版本读取，版本号必须来自 Cargo workspace package version，并写入 `v<version>` 目录层级。
- [ ] 1.3 （阻塞：等待 0.1 用户审计确认）实现统一输出目录 `artifacts/docnav/v<version>/<target>/package/`，并确保最终 archive 只写入该 `package/` 目录。
- [ ] 1.4 （阻塞：等待 0.1 用户审计确认）实现平台 archive 命名，文件名必须包含产品名、版本号和 target。
- [ ] 1.5 （阻塞：等待 0.1 用户审计确认）生成 `manifest.json`，记录版本、target、archive 文件名、包含的二进制、构建时间、git commit 和 SHA-256。
- [ ] 1.6 （阻塞：等待 0.1 用户审计确认）生成 `SHA256SUMS.txt`，并保证 checksum 与 manifest 中的 archive hash 一致。

## 2. 测试脚本迁移

- [ ] 2.1 （阻塞：等待 0.1 用户审计确认）审计 `scripts/`、`package.json` 和相关验证入口中直接引用 `target/debug`、`target/release` 或其它旧产物路径的位置。
- [ ] 2.2 （阻塞：等待 0.1 用户审计确认）新增或调整发布包 smoke 脚本，使其从统一 `package/manifest.json` 定位 archive，解包后运行 CLI。
- [ ] 2.3 （阻塞：等待 0.1 用户审计确认）更新 workspace 验证入口，使发布包验收链路先打包，再从统一 package 目录读取产物。
- [ ] 2.4 （阻塞：等待 0.1 用户审计确认）保留需要直接运行 Cargo 输出的开发期 smoke 时，必须将入口名称和文案标识为 dev smoke，避免被误认为发布包验收。
- [ ] 2.5 （阻塞：等待 0.1 用户审计确认）移除或改写发布验证脚本中的旧路径假设，确保发布包验证不再从 Cargo `target/` 查找被验收 CLI。

## 3. 命令入口与文档同步

- [ ] 3.1 （阻塞：等待 0.1 用户审计确认）更新 `package.json` scripts，区分打包、发布包 smoke、开发期 smoke 和 workspace verify。
- [ ] 3.2 （阻塞：等待 0.1 用户审计确认）更新测试策略或相关主文档，说明发布包验证必须消费统一打包目录。
- [ ] 3.3 （阻塞：等待 0.1 用户审计确认）确认该 change 不修改协议 schema、readable JSON schema、MCP tool schema 或 adapter 解析契约。

## 4. 验证与审计

- [ ] 4.1 （阻塞：等待 0.1 用户审计确认）运行 host target 打包命令，验证目录结构、archive、manifest 和 checksum 全部存在。
- [ ] 4.2 （阻塞：等待 0.1 用户审计确认）运行发布包 smoke，确认脚本通过 manifest 解包并运行 CLI。
- [ ] 4.3 （阻塞：等待 0.1 用户审计确认）运行现有相关 dev smoke，确认开发期直接二进制验证仍可用或已被明确替换。
- [ ] 4.4 （阻塞：等待 0.1 用户审计确认）用搜索确认发布验证脚本不再硬编码 `target/debug` 或 `target/release` 作为发布包来源。
- [ ] 4.5 （阻塞：等待 0.1 用户审计确认）运行 `pnpm run verify:docnav-workspace`，并用局部 diff 确认只修改目标范围。
