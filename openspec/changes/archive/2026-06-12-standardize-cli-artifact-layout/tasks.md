一句话核心：实现包含核心 CLI 与发布适配器的未压缩制品目录，并由 CI/CD 生成、验证和保存正式制品。

## 0. 修订后审计门禁

- [x] 0.1 用户审计确认：用户已审计本次修订后的 proposal、design、spec 和 tasks，并明确允许开始实现；未完成本项前，1.x 及后续任务全部处于阻塞状态。

## 1. 未压缩制品目录与脚本

- [x] 1.1 确定制品生成脚本的入口和参数，支持默认 host target 与显式 `--target <triple>`。
- [x] 1.2 通过 Cargo metadata 读取 workspace version，并通过 Rust/Cargo 信息确定 target triple。
- [x] 1.3 建立显式发布组件集合，首期固定包含 `docnav` 与 `docnav-markdown`。
- [x] 1.4 使用 Cargo release profile 构建发布组件集合，显式 `--target <triple>` 时传给 Cargo，并将目标平台的可执行文件复制到 `artifacts/docnav/v<version>/<target>/package/`。
- [x] 1.5 拒绝缺少任一发布组件的不完整 `package/`；仓库脚本不生成 `.zip`、`.tar.gz` 或其它 Docnav 归档包。
- [x] 1.6 生成 release artifact manifest `manifest.json`，记录固定 schema version、product、version、target、generated_at、git commit、源码工作树状态、生成来源，以及逐文件的 component、path、size 和 hash；`docnav-markdown` 条目记录 adapter id，且不复用 adapter manifest schema 或语义。
- [x] 1.7 生成 `SHA256SUMS.txt`，按相对路径升序使用 `<lowercase-sha256>  <relative-path>` 格式覆盖全部可执行文件和 `manifest.json`，不包含自身，并保证可执行文件 hash 与 `manifest.json` 一致。

## 2. 验证脚本与 smoke 分层

- [x] 2.1 审计 `scripts/`、`package.json` 和验证入口中对 Cargo 构建结果、日志、临时目录、归档包或旧制品路径的引用。
- [x] 2.2 新增发布制品验证脚本，从统一的 `package/manifest.json` 读取 release artifact manifest，并校验文件集合、组件类型、大小和校验和。
- [x] 2.3 让发布制品 smoke 直接运行 `package/` 中的 `docnav` 与 `docnav-markdown`，不执行归档包解压。
- [x] 2.4 将保留的 Cargo 构建结果 smoke 在脚本名称、输出文案和 `package.json` 命令中明确标识为开发期 smoke。
- [x] 2.5 更新 workspace verify，明确区分开发期 smoke 与发布制品验证，且不将本地生成结果标记为正式制品。

## 3. CI/CD 与版本库边界

- [x] 3.1 将根级 `artifacts/` 加入 `.gitignore`，确认其中生成的可执行文件、制品清单和校验和不被 Git 跟踪。
- [x] 3.2 新增正式制品工作流，在干净 checkout 中调用仓库制品生成脚本；工作流不复制脚本内的生成逻辑。
- [x] 3.3 为工作流配置手动触发入口和首期 target matrix：`x86_64-unknown-linux-gnu`/`ubuntu-latest`、`x86_64-pc-windows-msvc`/`windows-latest`；每个 target 在匹配的原生 runner 上构建并验证。
- [x] 3.4 正式制品验证确认 `source_dirty: false`、`producer.kind: "github-actions"` 和 CI 运行来源；`source_dirty` 只受 Git 修改、暂存和未被 ignore 的未跟踪文件影响，不受被 ignore 的生成物影响；再按 version 与 target 命名 CI artifact，并上传对应的 `package/` 文件集合。
- [x] 3.5 确认 CI artifact 服务的传输包装不改变 Docnav `package/` 的逻辑文件集合，且仓库脚本不生成归档包。

## 4. 命令入口与文档同步

- [x] 4.1 更新 `package.json` scripts，区分本地制品生成、发布制品验证、开发期 smoke 和 workspace verify。
- [x] 4.2 更新测试策略或相关主文档，说明发布组件集合、Cargo release profile、release artifact manifest、未压缩 `package/`、本地预验收和 CI/CD 正式制品的职责边界。
- [x] 4.3 确认本 change 不修改协议 schema、readable JSON schema、MCP tool schema、adapter invoke 契约或 adapter 托管制品目录。

## 5. 验证与审计

- [x] 5.1 运行 host target 制品生成命令，验证目录结构、`docnav`、`docnav-markdown`、`manifest.json` 和 `SHA256SUMS.txt` 均存在，且未生成 Docnav 归档包。
- [x] 5.2 运行发布制品 smoke，确认脚本通过 `manifest.json` 定位并直接运行核心 CLI 与适配器文件。
- [x] 5.3 运行相关开发期 smoke，确认直接使用 Cargo 构建结果的验证仍可用且命名明确。
- [x] 5.4 用搜索确认发布制品验证只从 `package/` 定位验收对象，不以 `target/debug`、`target/release`、归档包或其它旧路径中的文件替代。
- [x] 5.5 运行 OpenSpec 严格校验和工作流语法及行为验证。
- [x] 5.6 运行 `pnpm run verify:docnav-workspace`，并用局部 diff 确认只修改目标范围。
