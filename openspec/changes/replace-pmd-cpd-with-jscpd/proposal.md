本 change 起草用 jscpd 替换仓库质量观测中的 PMD CPD duplicate-code scanner，并保持归一化质量快照、warning、report 和 baseline 语义稳定。

当前 change 只在 `openspec/changes/replace-pmd-cpd-with-jscpd/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

当前 duplicate-code 观测依赖 PMD CPD，引入 Java runtime、PMD zip 下载和 CPD 专用 XML/exit-code 处理；这与 Docnav 的 Bun/pnpm-first 脚本工具链不匹配，也增加 CI setup 和本地复现成本。

jscpd 提供 npm 生态的 copy/paste detector、`jscpd` CLI、JSON reporter 和多语言格式支持。迁移后，duplicate-code 信号仍保持非阻断观测性质，依赖管理则收敛到项目声明的 Node package surface。

## What Changes

- 将 full quality profile 的 duplicate-code scanner 从 PMD CPD 替换为 jscpd。
- 将 PMD/Java CI 安装步骤替换为 `package.json`/lockfile 管理的 `jscpd` devDependency 和 Bun/pnpm 执行路径。
- 保留当前 normalized `DuplicateCodeFragment`、code-area 扫描、cache identity、baseline/current comparison、warning generation、accepted warning、Markdown quality report 和 verifier warning 语义。
- 将 scanner metadata、tool availability、raw artifact、warning `sourceTool`/`ruleId`、测试用例和文档表述从 `pmd-cpd` 迁移到 `jscpd`。
- 不改变 quick profile 的边界：quick quality check 仍跳过 baseline comparison 和 duplicate-code detection。
- 不把 duplicate-code 指标升级为阻断式 gate；质量 warning 仍按现有 repository-quality-observability 语义处理。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `repository-quality-observability`: duplicate-code 观测信号源从 PMD CPD 迁移到 jscpd，同时要求仓库 wrapper 继续拥有稳定归一化输出和非阻断报告语义。

## Impact

- Affected code: `scripts/tools/quality/measurement/scanners/pmd-cpd/**`、tool availability、current/baseline quality scan orchestration、cache identity、warning generation、report labels、quality scanner tests。
- Affected docs/specs: `docs/testing.md`、`docs/tooling.md`、`docs/testing/cases.md`、`openspec/specs/repository-quality-observability/spec.md` 的 duplicate-code 工具表述。
- Affected CI/dependencies: `.github/workflows/ci.yml` 删除 Java/PMD setup；`package.json` 和 lockfile 增加并固定 `jscpd` devDependency。
- Not affected: Docnav CLI 行为、adapter 行为、MCP 行为、protocol/schema/examples、Rust Clippy gate、Lizard 函数复杂度信号和 scc 文件级体量信号。
