# 工程工具链

本文定义 Docnav 仓库内开发脚本、临时工具和本地验证命令的工具链目标。它只拥有“如何运行和检查工程工具”的规则，不定义产品契约、CLI 行为、schema 字段或 smoke case 覆盖目标。

## 脚本语言与包管理

工具链按生态分成包依赖和运行时前置条件：包依赖由项目声明和 lockfile 管理，运行时由本地环境或 CI setup 提供。

1. TypeScript 脚本包依赖使用 `pnpm`；项目脚本通过 `bun run <script>` 执行。
2. TypeScript 脚本运行时使用 Bun；执行这些入口前，环境中必须能解析到 `bun` 可执行文件。
3. Python 工具使用 `uv`。
4. Rust 工具使用 Cargo workspace 命令或验证脚本封装的 Cargo 调用。

根目录 `rust-toolchain.toml` 固定本地开发、验证和发布使用的 Rust 工具链与必要组件；GitHub Actions 的 Rust setup 使用同一精确版本，不跟随浮动 `stable`。验证工具链包含 `rust-src`，确保 trybuild 涉及标准库类型或 trait 时，本地与 CI 都能渲染相同的标准库诊断片段。该固定版本用于保证工程验证可复现，不声明 crate 的最低支持 Rust 版本；升级时应同步工具链文件与全部 workflow setup，并重跑 workspace verification 和 release package 验证。

GitHub Actions 依赖固定到可追溯的完整 commit SHA；有正式 release 的 action 在同一行保留 release 注释，`dtolnay/rust-toolchain` 按其使用约定固定 `master` commit，并通过 workflow input 声明精确 Rust 版本。其他工具和运行时版本也继续显式声明。`.github/dependabot.yml` 每周检查 GitHub Actions 更新并通过 PR 提交，避免 workflow 依赖随可移动 tag 静默变化。

质量观测的 duplicate-code scanner 使用当前仓库 devDependency 中的 `jscpd`，wrapper 通过当前仓库 `node_modules/.bin/jscpd`（Windows 为 `jscpd.cmd`）运行扫描；不要求 baseline materialized repo 安装 `jscpd`，也不要求全局 `jscpd`、`cpd`、Java 或 PMD 安装。当前 `jscpd` 5.x launcher 委托仓库依赖中的 Rust binary，`--version` 实际输出可以使用 `cpd <version>` 前缀；wrapper 接受该版本文本不表示支持全局 `cpd` 命令。CI 可以保留 `pnpm exec jscpd --version` 作为依赖安装 smoke，但扫描 wrapper 不通过 baseline cwd 解析依赖。

## TypeScript 脚本

`scripts/` 和 `test/` 下的手写脚本以 TypeScript 源码为目标形态。脚本源码负责表达模块 contract、输入输出边界和共享状态类型；生成产物、分发产物或外部工具兼容层不拥有这些类型。

运行时目标：

1. 常规项目入口通过 `bun run <script>` 调用；入口内部或直接调试时由 Bun 运行 `.ts` 源码，例如 `bun scripts/foo.ts`。
2. 脚本测试由 Bun test runner 运行，例如 `bun test path/to/foo.test.ts`。
3. 手写源码文件使用 `.ts`；只有包含 JSX 的源码使用 `.tsx`。
4. 脚本保持 erasable TypeScript：不使用需要编译转换的语法。需要枚举语义时，使用 `as const` 对象、string union 或职责内常量模块。
5. 相对 import 使用运行时真实扩展名，例如 `./config.ts`；类型专用符号使用 `import type` 或 inline `type` modifier。

类型检查目标：

1. 项目提供脚本专用 `tsconfig`。
2. 类型检查通过 `bun run typecheck:scripts` 执行；该入口使用 TypeScript native preview 的 `tsgo` 读取脚本 `tsconfig`。
3. 代码 lint 通过 `bun run lint:scripts` 执行，覆盖未使用变量、未使用函数、显式 `any` 和常见脚本错误。
4. 脚本 `tsconfig` 以 `noEmit`、`module: "nodenext"`、`target: "esnext"`、`strict`、`erasableSyntaxOnly`、`verbatimModuleSyntax`、`rewriteRelativeImportExtensions`、`allowImportingTsExtensions` 和 Node.js-compatible API types 为基线。
5. 质量扫描、测试入口、验证脚本和文档引用覆盖 TypeScript 脚本源码。

运行时约束以 [Bun runtime](https://bun.sh/docs/runtime) 和 [Bun test runner](https://bun.sh/docs/test) 文档为准；native typecheck 入口以 [TypeScript Go native preview](https://github.com/microsoft/typescript-go#preview) 为准；类型检查配置以 TypeScript 的 [`erasableSyntaxOnly`](https://www.typescriptlang.org/tsconfig/#erasableSyntaxOnly)、[`verbatimModuleSyntax`](https://www.typescriptlang.org/tsconfig/#verbatimModuleSyntax)、[`rewriteRelativeImportExtensions`](https://www.typescriptlang.org/tsconfig/#rewriteRelativeImportExtensions) 和 [`allowImportingTsExtensions`](https://www.typescriptlang.org/tsconfig/#allowImportingTsExtensions) 选项为准。

## 验证入口集成

`typecheck:scripts`、`lint:scripts` 和 `quality:check` 是脚本模块与质量观测的快速验证入口。前者证明脚本类型、模块边界和共享状态一致；`lint:scripts` 证明脚本源码没有未使用变量/函数、显式 `any` 等静态质量问题；`quality:check` 运行 quick quality profile 并在出现 warning records 时输出前几个 warning、报告路径和“当前不是全量质检”的提示。质量扫描配置可以给已知可接受 warning 填充 `acceptedReason`；单独运行质量扫描时这些 warning 仍保持可见，并在对应 warning 旁展示原因。GitHub annotation 只投影未带 `acceptedReason` 的非 info warning，完整 warning records 和报告仍保留已接受记录。它们不替代真实 CLI、schema、进程 smoke、Rust tests、release package 验证或 `quality:full-check`。

required profile 包含 `typecheck:scripts`、`lint:scripts` 和 quick quality check。full profile 使用 full quality check 替代 quick quality check，并追加更宽验证；full profile 的 quality check 使用 verifier 输出，只在存在未带 `acceptedReason` 的 warning 时把 workspace verification 标记为 warning。profile 组成、质量观测边界和交付前取舍由 [测试策略](testing.md#统一验证入口) 维护。

Workspace verifier 的运行并发预算可由 `--concurrency <n>` 或环境变量 `DOCNAV_VERIFY_CONCURRENCY` 提供，CLI 参数优先；值必须是正整数。两者都省略或环境变量为空时不设置并发上限，由 task runner 使用默认调度行为。

验收标准：手写脚本可以通过 Bun 执行、被 `tsgo -p tsconfig.json` 覆盖，并且不依赖 Bun 运行时不会读取的 `tsconfig` 行为。

## 子进程输出环境

脚本启动子进程时默认使用 `scripts/tools/foundation` 的 process wrapper。该 wrapper 统一注入 plain-text output environment，覆盖 caller-provided color env，例如 `NO_COLOR=1`、`FORCE_COLOR=0`、`CLICOLOR=0`、`CLICOLOR_FORCE=0`、`TERM=dumb`、`CARGO_TERM_COLOR=never`、`PY_COLORS=0`、`UV_NO_COLOR=1`、`npm_config_color=false` 和 `PNPM_CONFIG_COLOR=false`。需要自定义 `spawn` 行为的工具必须复用同一 helper 生成 child env，避免验证日志、测试断言和命令记录依赖终端颜色探测。

## 共享脚本子仓库

`scripts/tools/foundation/`、`scripts/tools/parallel-task-runner/` 和 `scripts/tools/quality-core/` 是私有 Git 子仓库形态的共享脚本工具边界。每个子仓库只保留一份极简 README 作为文档 owner，用于说明用途、public source entrypoint 和本地检查；private `package.json` 与 `tsconfig.json` 只服务 Bun、TypeScript、ESLint 和测试配置，不是 npm publish contract。Docnav 侧通过 `.gitmodules`、submodule revision 和父仓库提交记录持有 revision/pin 集成状态。

Docnav 通过源码 import 和当前 Git revision 或等价 pin 集成这些子仓库。Docnav-owned command entrypoints、callers、quality defaults、workspace profiles、release product config、validators 和 package scripts 仍留在 Docnav 侧，并通过 typed config、task definitions 或 explicit options 直接导入共享 source entrypoint，不保留旧 wrapper/re-export 层。
