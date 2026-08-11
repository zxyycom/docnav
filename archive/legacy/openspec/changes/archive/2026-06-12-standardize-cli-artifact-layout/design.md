**一句话核心：仓库脚本生成并验证未压缩的核心 CLI 与适配器文件，CI/CD 是正式发布制品的唯一生成和保存边界。**

## Context

Docnav 当前已有 Cargo workspace、`docnav` 核心 CLI、`docnav-markdown` 适配器，以及若干直接使用 Cargo 构建结果的 smoke 脚本，但尚未定义独立的发布制品契约。开发期 smoke 适合快速验证源码构建结果，不承担正式制品验收职责。

本 change 引入构建与发布层规则，不改变文档操作命令、adapter invoke 协议或 MCP 映射。人和自动化可以通过稳定路径直接检查指定版本与 target 的核心 CLI 和适配器文件，并追溯生成这些文件的 Git commit 与 CI 运行。

## Goals / Non-Goals

**Goals:**

- 建立包含产品名、版本号、Rust target 和最终 `package/` 目录的统一制品层级。
- 直接保存核心 CLI 和当前发布适配器，不生成仓库自有归档包。
- 使用 Cargo release profile 构建发布组件，避免开发期 debug build 被当作发布制品。
- 为每个可执行文件记录组件角色、路径、大小和 SHA-256，并生成独立的校验和文件。
- 将 package `manifest.json` 明确作为 release artifact manifest，和 adapter manifest 契约分离。
- 让本地与 CI/CD 使用同一套仓库脚本，避免制品生成逻辑漂移。
- 由 CI/CD 生成、验证和保存正式制品；本地结果仅用于复现和预验收，不进入 Git。
- 将发布制品 smoke 与直接运行 Cargo 构建结果的开发期 smoke 明确分离。

**Non-Goals:**

- `artifacts/` 下的生成内容不提交到仓库。
- 本 change 不发布 npm 包、容器镜像或 GitHub Release，也不定义长期的发布晋级策略。
- 不改变 `docnav --output`、adapter 直接 CLI 输出或 MCP 输出映射。
- 不实现 adapter 安装、更新、移除或用户级托管制品存储。
- 不要求 Rust 单元测试从发布制品目录运行。

## Decisions

1. 制品根目录使用 `artifacts/docnav/`。
   - 固定层级为 `artifacts/docnav/v<version>/<target>/package/`。
   - `<version>` 来自 Cargo workspace package version，目录名带 `v` 前缀。
   - `<target>` 使用 Rust target triple。
   - `artifacts/` 是生成目录，必须由 `.gitignore` 排除。

2. `package/` 直接保存未压缩文件。
   - 首期目录直接包含平台对应的 `docnav[.exe]` 和 `docnav-markdown[.exe]`。
   - 同目录包含 `manifest.json` 和 `SHA256SUMS.txt`。
   - 制品生成脚本使用 Cargo release profile 构建发布组件；显式 `--target <triple>` 必须传给 Cargo 构建和制品目录。
   - 仓库脚本不生成 `.zip`、`.tar.gz` 或其它归档包。
   - GitHub Actions 等 CI artifact 服务可以使用自身的传输格式；Docnav 验收的对象始终是上传前和下载后具有相同文件集合的 `package/` 目录。

3. 发布组件集合必须显式维护。
   - 首期核心组件是 `docnav`，适配器组件是 `docnav-markdown`。
   - 制品生成脚本只构建和复制显式声明的发布组件，不把 workspace 中的所有可执行目标自动视为发布内容。
   - 新增发布适配器时，同步更新发布组件集合、制品清单断言和发布制品 smoke。

4. 制品清单与校验和支持逐文件审计。
   - 该 `manifest.json` 的契约名称是 release artifact manifest；它不同于 adapter manifest，不能复用或混淆 `docs/schemas/manifest.schema.json` 的 adapter manifest 语义。
   - `manifest.json` 记录 `schema_version: 1`、`product: "docnav"`、`version`、`target`、`generated_at`、`git_commit`、`source_dirty`、`producer` 和 `files`。
   - `producer.kind` 取 `local` 或 `github-actions`。CI 生成时同时记录 workflow、run id 和 run attempt；本地生成时这些 CI 字段为 `null`。
   - `files` 中每项记录相对 `package/` 的 `path`、`component`、`size_bytes` 和小写十六进制 `sha256`。
   - `component` 只取 `core` 或 `adapter`；`docnav-markdown` 条目使用 `component: "adapter"` 和 `adapter_id: "docnav-markdown"`，core 条目不带 `adapter_id`。
   - `SHA256SUMS.txt` 至少覆盖所有可执行文件和 `manifest.json`。
   - 制品清单中每个可执行文件的 hash 与 `SHA256SUMS.txt` 保持一致。
   - 校验和条目按相对路径升序写成 `<lowercase-sha256>  <relative-path>`；`SHA256SUMS.txt` 不包含自身，避免自引用。

5. 仓库脚本与 CI/CD 职责分离。
   - 仓库提交制品生成脚本、验证脚本、`package.json` 命令和工作流定义。
   - 本地可以运行同一套脚本生成 `artifacts/`，用于复现目录、制品清单、校验和和 smoke；这些结果属于本地预验收制品。
   - 正式制品由 CI/CD 工作流在干净 checkout 中调用同一制品生成脚本创建，通过制品 smoke 后按 target 上传保存。
   - 正式制品验证必须确认 `source_dirty: false` 且 `producer.kind: "github-actions"`；本地预验收允许 `producer.kind: "local"`。
   - `source_dirty` 通过 Git 状态计算：修改、暂存或未被 ignore 的未跟踪文件会使其为 `true`；`target/`、`node_modules/`、`artifacts/`、`.log/` 等被 ignore 的生成物不得单独导致 dirty。
   - 初始工作流支持手动触发，首期矩阵固定为 `x86_64-unknown-linux-gnu`/`ubuntu-latest` 和 `x86_64-pc-windows-msvc`/`windows-latest`。
   - 工作流在匹配 target 的原生 runner 上构建并执行制品 smoke；只有可在当前 runner 上运行并通过验证的文件才视为已验收制品。
   - version tag 触发和发布晋级策略由后续 change 定义。

6. 发布制品验证直接消费原文件。
   - 验证脚本从统一的 `package/manifest.json` 读取组件列表和相对路径。
   - 验证脚本先校验文件集合、大小和校验和，再直接运行 `package/` 中的 `docnav` 与 `docnav-markdown`。
   - 验收对象只能来自 `package/`；验证脚本不以 Cargo `target/` 下的可执行文件替代制品，也不执行归档包解压。
   - 可以保留直接运行 Cargo 构建结果的 smoke，但入口名称和文案必须明确标识为开发期 smoke。

7. 协议契约不受影响。
   - 制品目录只管理文件系统结果；protocol-json envelope、readable-json、adapter invoke 和 MCP structuredContent 均不变化。
   - `docnav-mcp` 不理解该目录结构；它仍只依赖可调用的 `docnav` CLI。

## Risks / Trade-offs

- [CI artifact 服务使用平台自有传输格式] -> 验收上传前和下载后的逻辑文件集合，不将传输容器视为 Docnav 制品格式。
- [核心 CLI 与适配器集合发生漂移] -> 显式维护发布组件集合，并由制品清单和 smoke 同时断言 `docnav` 与 `docnav-markdown`。
- [本地与 CI 结果发生漂移] -> CI 只调用仓库内同一套制品生成与验证脚本，不在工作流中复制实现逻辑。
- [本地结果被误认为正式制品] -> 制品清单记录生成来源和源码工作树状态；正式工作流只上传由 CI 基于干净源码生成的结果。
- [生成物被误提交] -> 根级忽略 `artifacts/`，并验证其中的生成物未被 Git 跟踪。
- [逐文件分发缺少单文件下载体验] -> 本 change 优先保证可审计性和直接运行；如需发布归档包，由后续 change 单独定义。

## Migration Plan

1. 更新 change artifacts，明确未压缩制品和首期发布组件集合。
2. 新增未压缩制品生成脚本，以及制品清单和校验和生成逻辑。
3. 新增发布制品验证脚本，直接使用 `package/` 中的核心 CLI 与适配器文件。
4. 重命名或调整现有 smoke，使 Cargo 构建结果仅用于开发期 smoke。
5. 更新 `.gitignore`、`package.json`、测试策略和 CI/CD workflow。
6. 在本地验证脚本的可复现性，再由 CI/CD 生成并保存正式制品。

## Open Questions

无。首期发布组件集合固定为 `docnav` 与 `docnav-markdown`；发布晋级策略和更多平台矩阵由后续 change 单独定义。
