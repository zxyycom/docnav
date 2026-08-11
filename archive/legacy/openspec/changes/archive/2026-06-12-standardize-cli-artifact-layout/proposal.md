**一句话核心：为 `docnav` 核心 CLI 和当前发布适配器建立未压缩、可逐文件审计的统一制品目录，并由 CI/CD 生成和保存正式制品。**

## Why

当前仓库缺少统一的发布制品位置，构建、验证和发布流程可能采用不同的路径约定。仅包含核心 CLI 会遗漏文档导航链路所需的适配器；将可执行文件封装为归档包，也会增加逐文件审计和直接运行的步骤。

项目需要一套本地可复现、以 CI/CD 输出为正式来源的制品流程。最终目录直接保存各可执行文件、制品清单和校验和，便于逐文件保存、比较、校验和运行；生成结果不进入 Git。

## What Changes

- 新增 Docnav 发布制品目录契约，目录层级显式包含版本号和 Rust target triple。
- `package/` 目录直接包含 `docnav` 核心 CLI、当前发布适配器、`manifest.json` 和 `SHA256SUMS.txt`；仓库脚本不生成 `.zip`、`.tar.gz` 或其它归档包。
- 首期发布组件集合固定包含 `docnav` 和 `docnav-markdown`。后续适配器必须显式加入该集合，不从 Cargo workspace 自动收集所有可执行目标。
- 制品生成脚本使用 Cargo release profile 构建发布组件；直接运行 Cargo 构建结果的 smoke 保持开发期入口，不承担发布制品验收。
- 新增或调整仓库内的制品生成与验证脚本。本地执行用于复现和预验收；CI/CD 调用同一套脚本生成、验证并保存正式制品。
- 首期正式制品工作流覆盖 Linux x64 和 Windows x64，并在匹配目标平台的原生 runner 上构建和验证。
- 将 `artifacts/` 作为生成目录忽略，本地和 CI 生成的可执行文件、制品清单与校验和均不提交到 Git。
- 发布制品验证直接运行 `package/` 中的核心 CLI 和适配器，不从 Cargo `target/`、日志、临时目录或归档包中定位验收对象。
- 本 change 不改变 `docnav --output text|readable-json|protocol-json`、adapter invoke 或 MCP 的输出语义。

## Capabilities

### New Capabilities

- `cli-artifact-layout`：约束 Docnav 核心 CLI 与发布适配器的制品目录、逐文件元数据、校验和、仓库脚本职责、CI/CD 正式制品生成方式和验证入口。

### Modified Capabilities

无。

## Impact

- 影响制品生成脚本，例如 `scripts/package-docnav.*` 或同类入口。
- 影响发布制品 smoke、现有开发期 smoke 的命名和 `package.json` 命令。
- 影响 `.github/workflows/`：正式制品由 CI/CD 调用仓库脚本生成、验证并上传保存。
- 影响 `.gitignore`：`artifacts/` 保持为未跟踪的生成目录。
- 影响测试策略或相关主文档中的发布制品验证说明。
- 新增的 package `manifest.json` 是 release artifact manifest，必须和 adapter manifest schema、示例及语义明确区分。
- 不影响原始协议、阅读输出 schema、Markdown adapter 解析逻辑、adapter 管理的用户级托管制品目录或 MCP tool 输出映射。
