本 change 的 design 说明如何用 jscpd 替换仓库质量观测中的 PMD CPD duplicate-code scanner，并保持现有质量观测 contract、report 和 baseline 语义稳定。

实现前审计阶段曾确认 proposal slice 只在 `openspec/changes/replace-pmd-cpd-with-jscpd/` 下形成临时文档；进入实现后按 tasks 同步主规范、文档、测试、CI、依赖和代码。

## Context

当前 full quality profile 使用 PMD CPD 提供 duplicate-code 信号；quick profile 跳过 baseline comparison 和 duplicate-code detection。PMD CPD 的 wrapper 负责按 code area 传递 minimum tokens、解析 XML、处理 CPD 特殊退出码、写入 duplicate-code cache，并把结果归一化为 `DuplicateCodeFragment` 供 warning、report、baseline comparison 和 verifier 输出使用。

这个 change 不改变质量观测的产品边界：Lizard 继续提供函数级复杂度信号，scc 继续提供文件级体量和 decision-token 信号，Clippy 继续作为 Rust 阻断式 lint gate。变化集中在 duplicate-code scanner 的外部工具和 wrapper 层。

jscpd 官方资料显示，`jscpd@5` 可通过 npm 安装并提供 `jscpd` 命令，支持 JSON reporter、`.jscpd.json` config、`--min-tokens`、多语言格式和自包含预构建二进制。实现时必须以当时 lockfile 中固定的版本再次验证这些行为。来源：

- https://github.com/kucherenko/jscpd
- https://github.com/kucherenko/jscpd/blob/master/docs/rust.md
- https://registry.npmjs.org/jscpd/latest

## Goals / Non-Goals

**Goals:**

- 用项目声明的 npm devDependency 替换 PMD/Java/zip 安装链路。
- 保留当前 duplicate-code normalized model、warning、report、baseline 和 cache 语义。
- 保留 code-area 级 minimum token policy 和 bounded parallel scan planning。
- 让 full quality profile 继续产出 duplicate-code 观测；quick profile 继续跳过 duplicate-code detection。
- 更新测试、文档、OpenSpec delta 和 CI，使失败可本地复现。

**Non-Goals:**

- 不把 duplicate-code warning 改成阻断式 gate。
- 不改变 Docnav CLI、adapter、MCP、protocol、schema、examples 或 release package contract。
- 不替换 Lizard 或 scc。
- 不引入 jscpd 的 Node.js programming API 作为长期 contract；本 change 以 CLI wrapper 和归一化输出为边界。
- 不保留 PMD CPD 作为并行 fallback；回滚应通过 Git revert 或后续 change 恢复旧 wrapper。

## Decisions

### Decision 1: 使用 npm `jscpd` 包和 `jscpd` 命令

实现应通过 `pnpm` 添加并锁定 `jscpd` devDependency，由项目脚本或 wrapper 调用 lockfile 管理的 `jscpd` CLI。不要继续依赖系统 Java、PMD zip、全局 `pmd` 命令或 GitHub Actions 中的 PMD_VERSION 环境变量。

影响：CI 可以删除 Java setup 和 PMD 下载步骤；本地复现路径收敛到 `pnpm install` 后的仓库脚本。实现时仍必须验证当前 npm 版本提供的 `jscpd` 命令、JSON reporter 和需要的 CLI options。

实现记录：本 slice 使用 npm registry latest `jscpd@5.0.11`，`package.json` 和 `pnpm-lock.yaml` 均锁定为 `5.0.11`；本地 smoke `pnpm exec jscpd --version` 输出 `cpd 5.0.11`。

备选方案：

- 使用 npm `cpd` 包：同属 jscpd v5 Rust engine，但命令名和包名与“替换为 jscpd”的维护意图不一致，生态识别度也更弱。
- 使用 cargo install：会把 duplicate-code scanner 从 Java 调性改到 Rust 调性，但仍不属于项目 Node package dependency surface，不能满足用户希望 npm 工具替换的目标。

### Decision 2: 解析 jscpd JSON reporter，而不是兼容 PMD CPD XML

新 wrapper 应运行 jscpd JSON reporter，并把 jscpd raw clone records 映射到现有 `DuplicateCodeFragment`。PMD CPD XML parser、exit 4 特判和 XML root 检查应随迁移删除。

影响：第三方 raw artifact 可以变化，但仓库内部 machine snapshot、warnings、report 和 cache payload 仍由 normalized model 负责。实现必须用 fixture 覆盖 jscpd JSON 中 token count、line count、location path、start/end line 和多 location clone 的映射。

备选方案：

- 使用 jscpd 的 XML reporter 模拟旧 CPD XML：看似减少迁移量，但会把旧 PMD 传输形状继续伪装成长期边界，并可能丢失 JSON 中更直接的 clone 元数据。
- 直接把 jscpd JSON 暴露给 downstream：会破坏 repository-quality-observability 已有“第三方输出不作为稳定 contract”的分层。

### Decision 3: 保留 code-area scan planning 和 cache 语义

现有按 code area 拆分、minimum tokens 配置、fingerprint identity、current/baseline scan kind、cache miss bounded parallelism 和 changed-scope annotation 应继续存在。迁移时应把 cache `toolName`、normalized args 和 config field 从 `pmd-cpd` 迁移到 `jscpd`，使旧 PMD cache 不被误读为 jscpd cache。

影响：首次 jscpd scan 会发生 duplicate-code cache miss，这是预期迁移成本。实现必须证明 jscpd 的文件输入方式能按 code area 限定扫描；如果直接传递 file paths 或生成临时 `.jscpd.json` 的行为不满足边界，必须在实现前调整设计并记录新的决策。

备选方案：

- 用单次全仓库 jscpd 扫描后按 code area 后过滤：实现简单，但会改变 per-area minimum token policy，并可能重新引入 fixtures/generated 噪声。
- 删除 cache：实现简单，但会增加 full profile 和 baseline comparison 的重复扫描成本。

### Decision 4: 迁移 warning identity，但保持 accepted reason 语义

新 duplicate warning 应使用 jscpd identity，例如 `ruleId: "jscpd-duplicate-code"` 和 `sourceTool: "jscpd"`。现有 accepted warnings 的 reason、suggestionIncludes、codeArea、metric/value 匹配语义应迁移到新 identity，但不得假设 PMD token counts 与 jscpd token counts 完全相同。

影响：实现时必须重新跑 jscpd full scan，校准 accepted warning 的 metric value 或改成更稳定的 location/path-based acceptance；任何 token-count 差异都应反映在 tests 和 docs 中。

备选方案：

- 保留 `pmd-cpd-duplicate-code` ruleId：可以减少 report diff，但会在新工具下保留错误来源，降低可审计性。
- 丢弃 accepted warnings：会让已知可接受重复重新污染质量报告。

## Risks / Trade-offs

- [Risk] jscpd JSON output shape 或 CLI option 与预期不一致。 -> Mitigation: 迁移前先用 pinned devDependency 生成 fixture raw output，wrapper parser tests 固定最小受支持 shape。
- [Risk] jscpd tokenization 与 PMD CPD tokenization 不同，导致 token counts 和 duplicate grouping 变化。 -> Mitigation: 把差异作为工具迁移事实处理，只承诺 normalized contract，不承诺 PMD 数值兼容。
- [Risk] jscpd 文件输入和 `.gitignore`/ignore 规则会越过 code-area 边界。 -> Mitigation: 每个 code area 使用显式 path/config invocation，并用 fixture 测试证明 excluded/generated files 不进入 duplicate fragments。
- [Risk] npm 包的预构建二进制在 CI runner 或 Windows 本地不可用。 -> Mitigation: tool availability check 必须区分 dependency missing、binary unavailable、execution failure 和 empty result；CI 运行 full quality check 作为平台 smoke。
- [Risk] accepted warnings 因 token-count 差异无法稳定匹配。 -> Mitigation: 优先用 path/location/suggestionIncludes 匹配，只有稳定时保留 value 匹配。

## Migration Plan

1. 在实现前完成本 change 的阻塞级审计，确认 proposal、design、specs、tasks 范围一致。
2. 添加并锁定 `jscpd` devDependency，确认 `bun`/`pnpm` 脚本可解析本地 `jscpd` CLI。
3. 新增 jscpd wrapper、tool availability 和 parser tests；用 jscpd raw JSON fixture 证明 normalized mapping。
4. 将 current/baseline duplicate-code orchestration 从 PMD CPD wrapper 切到 jscpd wrapper，保留 code-area、cache、changed-scope 和 baseline semantics。
5. 迁移 warnings、accepted warnings、report labels、raw artifact names、docs 和 testing case labels。
6. 删除 PMD CPD wrapper、XML parser、PMD process-result tests、CI Java/PMD setup 和 PMD-specific config。
7. 运行 `bun run quality:test`、`bun run quality:full-check`，并在交付前运行 `bun run verify:docnav-workspace` 或明确记录无法运行原因。

Rollback strategy: 如果迁移后的 full quality profile 无法稳定运行，回滚本 change 的实现 commit 恢复 PMD CPD wrapper 和 CI setup；不要在同一 implementation 中保留双 scanner fallback。

## Open Questions

无未回答开放问题，可以进入实现前审计。
