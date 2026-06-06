**一句话核心：为 `docnav` CLI 建立带版本号和平台信息的统一发布产物目录，并让测试脚本统一从该目录查找打包结果。**

## Why

当前仓库还没有统一的 CLI 打包产物位置，构建、验证和后续发布流程容易各自假设不同路径。随着 `docnav`、adapter 和 MCP 逐步进入端到端链路，需要一个稳定、可审计、方便人工直接查找的产物目录结构。

## What Changes

- 新增 `docnav` CLI 发布产物目录契约，目录必须显式包含版本号和平台 target 信息。
- 定义最终打包结果只放在平台目录末端的专用打包目录中，避免二进制、临时文件和最终 archive 混放。
- 新增或调整打包脚本，使其按统一目录生成 archive、manifest 和校验材料。
- 调整相关测试和验证脚本，使它们从统一打包目录读取产物，不再继续从 `target/`、临时目录或其它旧路径查找发布包。
- 明确该 change 不改变 `docnav --output text|readable-json|protocol-json` 的 stdout 语义。

## Capabilities

### New Capabilities

- `cli-artifact-layout`: 约束 `docnav` CLI 发布产物的目录结构、命名、元数据、校验材料，以及测试脚本查找产物的位置。

### Modified Capabilities

无。

## Impact

- 影响构建和打包脚本，例如后续新增的 `scripts/package-docnav.*` 或同类入口。
- 影响测试和验证脚本，尤其是当前直接引用 Cargo `target/` 输出的 smoke 或 workspace verify 入口。
- 影响 `package.json` 中与打包、smoke、verify 相关的脚本命令。
- 影响发布审计材料：打包产物目录、manifest、checksum 和验证日志。
- 不影响原始协议、阅读输出 schema、Markdown adapter 解析逻辑或 MCP tool 输出映射。
