**一句话核心：打包脚本是发布产物的唯一写入者，发布验证脚本是统一打包目录的消费者。**

## Context

Docnav 当前已有 Cargo workspace 和若干验证脚本，但还没有独立的 CLI 发布产物目录契约。现有 smoke 入口可以直接指向 Cargo `target/` 下的调试二进制，这适合开发期快速验证，但不适合承担发布包验收职责。

这个 change 引入的是构建/发布层规则，不改变 `docnav` 文档操作命令的 stdout 输出模式，也不改变 adapter invoke 协议。它的核心目标是让人和脚本都能通过一个稳定路径找到“某版本、某平台”的最终打包结果。

## Goals / Non-Goals

**Goals:**

- 建立统一发布产物根目录和层级：产品名、版本号、平台 target、最终打包目录。
- 让最终 archive、manifest 和 checksum 在目录末端集中出现，方便直接查找和审计。
- 让发布验证脚本从该目录读取打包产物，避免继续从 Cargo `target/`、临时目录或历史路径推断发布包。
- 保留开发期快速 smoke 的空间，但必须和发布包验证入口命名、职责分离。

**Non-Goals:**

- 不改变 `docnav --output`、adapter 直接 CLI 输出或 MCP 输出映射。
- 不定义 JSON、YAML、TOML、INI 等后续 adapter 的解析或导航能力。
- 不实现正式 adapter 管理命令的安装、更新、移除算法。
- 不要求所有 Rust 单元测试都通过发布 archive 运行；本 change 约束的是打包和发布验证脚本。

## Decisions

1. 发布产物根目录使用 `artifacts/docnav/`。
   - 原因：`artifacts` 表达“可审计产物集合”，比 `target/` 更适合人类查找，也避免和 Cargo 输出目录混淆。
   - 备选：`dist/` 更短，但容易只表达前端或 npm 发布语境；本仓库同时包含 Rust CLI、adapter 和 MCP，`artifacts` 更中性。

2. 目录层级固定为 `artifacts/docnav/v<version>/<target>/package/`。
   - `v<version>` 来自 Cargo workspace 版本，例如 `v0.1.0`。
   - `<target>` 使用 Rust target triple，例如 `x86_64-pc-windows-msvc`。
   - `package/` 是最终 archive、manifest 和 checksum 的唯一落点。
   - 这样目录从“产品 -> 版本 -> 平台 -> 打包结果”逐层收敛，平台信息之后才出现最终打包结果。

3. 打包脚本负责生成 manifest 和 checksum。
   - manifest 至少记录产物名、版本、target、archive 文件名、包含的二进制、构建时间、git commit 和 SHA-256。
   - checksum 使用稳定文件名，例如 `SHA256SUMS.txt`，用于人工和脚本校验。

4. 发布验证脚本必须消费统一打包目录。
   - 如果脚本验证“发布包可用”，它必须从 `artifacts/docnav/v<version>/<target>/package/manifest.json` 定位 archive，并在临时目录解包后运行。
   - 不允许发布验证脚本继续硬编码 `target/debug`、`target/release`、`.log` 或其它非 package 路径来寻找被验收的 CLI。
   - 开发期直接跑 Cargo 输出的 smoke 可以保留，但入口名称和文案必须明确是 dev smoke，不能作为发布包验收。

5. 协议契约不受影响。
   - 打包目录只管理文件系统产物；invoke 请求、protocol-json envelope、readable-json 字段和 MCP structuredContent 都不变化。
   - `docnav-mcp` 仍保持格式无关，只依赖可调用的 `docnav` CLI，不直接理解该目录结构。

## Risks / Trade-offs

- [目录层级过早固化] → 先只约束 `docnav` CLI 发布包；adapter 和 MCP 可以后续用相同模式扩展，不在本 change 内强制。
- [测试脚本职责混淆] → 将发布包验证和开发期 smoke 分成不同脚本入口，并在任务中要求更新 `package.json` 命名。
- [交叉编译环境不完整] → 验证脚本至少支持 host target；其它 target 可以由 CI 或本地显式传入 `--target` 后执行。
- [archive 内部路径和外部目录漂移] → manifest 同时记录外部 archive 名和内部二进制路径，测试脚本只信任 manifest。

## Migration Plan

1. 新增打包脚本，先支持当前 host target。
2. 更新发布验证脚本，从统一 package 目录读取 manifest 和 archive。
3. 调整 `package.json`，区分开发期 smoke、打包、发布包验证和 workspace verify。
4. 保留旧 smoke 行为作为开发辅助入口，或重命名为明确的 dev-only 入口。
5. 通过局部验证确认打包产物、manifest、checksum 和发布包验证链路一致。

## Open Questions

- 首期发布包是否只包含 `docnav`，还是同时包含首批 adapter 二进制，需要在实现时结合核心 CLI crate 落地状态确认。
- Windows 首期 archive 使用 `.zip`，Unix 使用 `.tar.gz` 是否足够；如需统一格式，可在实现前补充决策。
