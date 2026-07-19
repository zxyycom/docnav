**一句话核心：为现有 canonical release package 增加可公开下载、可校验、可按 Quick Start 复现的 Markdown CLI Beta 发布边界；本文件是仅位于本 change 目录下的未审核临时 delta，不影响现有主规范或其它 change。**

## ADDED Requirements

### Requirement: Beta 公共下载文件必须派生自已验证 package
Docnav Beta prerelease MUST 为每个支持 target 发布一个 target-qualified `docnav-v<version>-<target>[.exe]` 可执行文件及对应的 `<binary-name>.sha256`。公共可执行文件 MUST 是 canonical `package/` 中已通过 manifest、checksum 和 package smoke 验证的 `docnav` 文件的逐字节副本；`.sha256` MUST 使用公共文件名和小写十六进制 SHA-256。公共下载命名 MUST NOT 改变 canonical `package/` 目录、release artifact manifest 或 `SHA256SUMS.txt` 的既有语义。

#### Scenario: 从 Linux package 准备公共文件
- **WHEN** CI 已验证 `x86_64-unknown-linux-gnu/package/` 中的 `docnav`
- **THEN** 发布阶段生成 `docnav-v<version>-x86_64-unknown-linux-gnu`
- **THEN** 公共文件内容与 package 中的 `docnav` 逐字节相同
- **THEN** 同名 `.sha256` 文件能够校验该公共文件

#### Scenario: 从 Windows package 准备公共文件
- **WHEN** CI 已验证 `x86_64-pc-windows-msvc/package/` 中的 `docnav.exe`
- **THEN** 发布阶段生成 `docnav-v<version>-x86_64-pc-windows-msvc.exe`
- **THEN** 公共文件内容与 package 中的 `docnav.exe` 逐字节相同
- **THEN** 同名 `.sha256` 文件能够校验该公共文件

### Requirement: Beta prerelease 发布必须经过显式版本和干净 CI 门禁
公开 Beta MUST 作为 prerelease 发布，并 MUST 由显式 Beta tag 触发。Tag MUST 等于 `v<workspace-version>`，workspace version MUST 包含 SemVer prerelease 部分，首个发布候选为 `0.1.0-beta.1`。发布 job MUST 只消费同一次 CI run 在干净 checkout 中生成且已通过 package smoke 的文件；任一 target 缺失、`source_dirty` 不为 `false`、`producer.kind` 不为 `github-actions`、tag/version 不一致或 acceptance 失败时 MUST NOT 创建或更新公开 prerelease。

#### Scenario: 发布首个 Beta
- **WHEN** tag 为 `v0.1.0-beta.1`
- **AND** workspace version 为 `0.1.0-beta.1`
- **AND** Linux 与 Windows package 均来自干净 GitHub Actions checkout 并通过验证和 acceptance
- **THEN** workflow 创建或更新标记为 prerelease 的 `v0.1.0-beta.1`
- **THEN** prerelease 只附加对应 target-qualified binary 和 checksum

#### Scenario: 拒绝不一致或未验证发布
- **WHEN** tag 与 workspace version 不一致
- **OR** 任一目标 package 未通过验证
- **OR** manifest 表明源码 dirty 或 producer 不是 GitHub Actions
- **THEN** workflow 失败且不发布新的公共文件

### Requirement: Beta Quick Start 必须由发布候选可执行验证
README MUST 提供一个边界明确的 Beta Quick Start，覆盖支持平台、下载、checksum 校验、执行权限或文件命名、`docnav version`，以及针对同一最小 Markdown 样例的 `outline -> ref -> read` 和 `find -> ref -> read` 路径。仓库 MUST 提供 acceptance，使同一示例内容和代表命令直接运行 staged public binary，而不是 Cargo output 或开发期 wrapper；acceptance MUST 同时验证默认 readable output 可读，并至少用一个 `protocol-json` 调用提取实际 ref 后继续 read。

#### Scenario: 新用户走通 Markdown 导航
- **WHEN** 用户按 README 在支持平台准备 Beta binary 和示例 Markdown
- **THEN** `docnav version` 可运行
- **THEN** outline 返回至少一个可读 ref
- **THEN** 用户把该 ref 原样传给 read 后得到对应 Markdown 区域
- **THEN** find 返回的 ref 也能继续 read 到相关区域

#### Scenario: CI 验证公开路径而非开发构建
- **WHEN** Beta acceptance 在发布前运行
- **THEN** acceptance 执行 target-qualified staged public binary
- **THEN** acceptance 不回退到 `target/`、`bun --silent run dnm` 或其它开发期入口
- **THEN** README 示例使用的文档内容和命令语义与 acceptance 一致

### Requirement: Beta 发布说明必须诚实标注范围和证据
README 和 Beta release notes MUST 明确标注 Beta 状态、Markdown-only 格式范围、首期 Linux/Windows target、已知限制、反馈入口和无默认自动遥测边界。它们 MUST NOT 宣称 Docnav 已普遍降低总 token、总延迟或提升答案正确率，除非同一声明链接到可复现的实测证据；当前结构化读取的适用与不适用场景 MUST 保持可见。

#### Scenario: 用户判断 Beta 是否适合自己
- **WHEN** 用户阅读 Quick Start 或 Beta release notes
- **THEN** 用户能在下载前看到支持格式和平台
- **THEN** 用户能看到结构化读取更适合长文档、稀疏相关内容和多轮回溯
- **THEN** 用户也能看到短文档、一次性问题或需要整体理解时全文读取可能更合适
- **THEN** 文档不把理论估算写成已验证产品效果
